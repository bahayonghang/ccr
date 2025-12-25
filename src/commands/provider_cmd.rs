//! 🏥 Provider 命令
//!
//! 提供 Provider 测试相关的 CLI 命令。

use crate::core::ColorOutput;
use crate::core::error::Result;
use crate::services::ConfigService;
use crate::services::health_check::{HealthCheckService, HealthStatus};
use clap::{Args, Subcommand};
use comfy_table::{Cell, Color, Table, presets::UTF8_FULL};

/// Provider 命令参数
#[derive(Args, Debug, Clone)]
pub struct ProviderArgs {
    #[command(subcommand)]
    pub command: ProviderCommand,
}

/// Provider 子命令
#[derive(Subcommand, Debug, Clone)]
pub enum ProviderCommand {
    /// 测试 Provider 端点
    Test {
        /// Provider 名称（配置名称）
        name: Option<String>,

        /// 测试所有 Provider
        #[arg(short, long)]
        all: bool,

        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },

    /// 验证 API Key
    Verify {
        /// Provider 名称
        name: String,
    },
}

/// 执行 provider 命令
pub fn execute(args: ProviderArgs) -> Result<()> {
    match args.command {
        ProviderCommand::Test { name, all, verbose } => {
            if all {
                cmd_test_all(verbose)
            } else if let Some(n) = name {
                cmd_test(&n, verbose)
            } else {
                ColorOutput::error("请指定 Provider 名称或使用 --all");
                Ok(())
            }
        }
        ProviderCommand::Verify { name } => cmd_verify(&name),
    }
}

/// 测试单个 Provider
fn cmd_test(name: &str, verbose: bool) -> Result<()> {
    let config_service = ConfigService::with_default()?;
    let config_list = config_service.list_configs()?;

    // 查找配置
    let config = config_list.configs.iter().find(|c| c.name == name);

    match config {
        Some(c) => {
            ColorOutput::info(&format!("测试 Provider: {}", name));

            // 创建运行时执行异步测试
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                crate::core::error::CcrError::ConfigError(format!("无法创建运行时: {}", e))
            })?;

            let service = HealthCheckService::new();

            let result = rt.block_on(async {
                let section = crate::managers::config::ConfigSection {
                    auth_token: c.auth_token.clone(),
                    base_url: c.base_url.clone(),
                    model: c.model.clone(),
                    ..Default::default()
                };
                service.check(name, &section).await
            });

            // 显示结果
            println!();

            let status_color = match result.status {
                HealthStatus::Healthy => Color::Green,
                HealthStatus::Degraded => Color::Yellow,
                HealthStatus::Unhealthy => Color::Red,
                HealthStatus::Unknown => Color::White,
            };

            let mut table = Table::new();
            table.load_preset(UTF8_FULL);

            table.add_row(vec![
                Cell::new("状态").fg(Color::Cyan),
                Cell::new(result.status.display()).fg(status_color),
            ]);
            table.add_row(vec![
                Cell::new("Base URL").fg(Color::Cyan),
                Cell::new(&result.base_url),
            ]);

            if let Some(latency) = result.latency_ms {
                table.add_row(vec![
                    Cell::new("延迟").fg(Color::Cyan),
                    Cell::new(format!("{} ms", latency)),
                ]);
            }

            if let Some(ref error) = result.error {
                table.add_row(vec![Cell::new("错误").fg(Color::Red), Cell::new(error)]);
            }

            if verbose && !result.available_models.is_empty() {
                table.add_row(vec![
                    Cell::new("可用模型").fg(Color::Cyan),
                    Cell::new(result.available_models.join(", ")),
                ]);
            }

            println!("{}", table);
            println!();
        }
        None => {
            ColorOutput::error(&format!("未找到配置: {}", name));
            ColorOutput::info("使用 'ccr list' 查看可用配置");
        }
    }

    Ok(())
}

/// 测试所有 Provider
fn cmd_test_all(verbose: bool) -> Result<()> {
    let config_service = ConfigService::with_default()?;
    let config_list = config_service.list_configs()?;

    if config_list.configs.is_empty() {
        ColorOutput::warning("没有可用的配置");
        return Ok(());
    }

    ColorOutput::info(&format!(
        "测试 {} 个 Provider...",
        config_list.configs.len()
    ));
    println!();

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::core::error::CcrError::ConfigError(format!("无法创建运行时: {}", e)))?;

    let service = HealthCheckService::new();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        Cell::new("名称").fg(Color::Cyan),
        Cell::new("状态").fg(Color::Cyan),
        Cell::new("延迟").fg(Color::Cyan),
        Cell::new("错误").fg(Color::Cyan),
    ]);

    for config in &config_list.configs {
        let result = rt.block_on(async {
            let section = crate::managers::config::ConfigSection {
                auth_token: config.auth_token.clone(),
                base_url: config.base_url.clone(),
                model: config.model.clone(),
                ..Default::default()
            };
            service.check(&config.name, &section).await
        });

        let status_color = match result.status {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Degraded => Color::Yellow,
            HealthStatus::Unhealthy => Color::Red,
            HealthStatus::Unknown => Color::White,
        };

        let latency_str = result
            .latency_ms
            .map(|l| format!("{} ms", l))
            .unwrap_or_else(|| "-".to_string());

        let error_str = result.error.unwrap_or_else(|| "-".to_string());
        let error_short = if error_str.len() > 30 {
            format!("{}...", &error_str[..27])
        } else {
            error_str
        };

        table.add_row(vec![
            Cell::new(&config.name),
            Cell::new(result.status.display()).fg(status_color),
            Cell::new(latency_str),
            Cell::new(error_short),
        ]);
    }

    println!("{}", table);

    if verbose {
        println!();
        ColorOutput::info("提示: 使用 'ccr provider test <name> --verbose' 查看单个 Provider 详情");
    }

    Ok(())
}

/// 验证 API Key
fn cmd_verify(name: &str) -> Result<()> {
    let config_service = ConfigService::with_default()?;
    let config_list = config_service.list_configs()?;

    let config = config_list.configs.iter().find(|c| c.name == name);

    match config {
        Some(c) => {
            let base_url = c
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com".to_string());

            let api_key = c.auth_token.clone().unwrap_or_else(|| {
                tracing::debug!("配置 {} 未设置 API Key", name);
                String::new()
            });

            if api_key.is_empty() {
                ColorOutput::error("API Key 未配置");
                return Ok(());
            }

            ColorOutput::info(&format!("验证 API Key: {}", name));

            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                crate::core::error::CcrError::ConfigError(format!("无法创建运行时: {}", e))
            })?;

            let service = HealthCheckService::new();

            let valid = rt.block_on(async { service.verify_api_key(&base_url, &api_key).await })?;

            println!();
            if valid {
                ColorOutput::success("API Key 有效 ✓");
            } else {
                ColorOutput::error("API Key 无效 ✗");
            }
        }
        None => {
            ColorOutput::error(&format!("未找到配置: {}", name));
        }
    }

    Ok(())
}
