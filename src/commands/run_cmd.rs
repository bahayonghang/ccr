// 🏃 run 命令实现 - 临时运行隔离环境
//
// 执行流程(3 个步骤):
// 1. 🔍 在所有平台中查找指定配置
// 2. 🧮 提取配置对应的环境变量
// 3. 🚀 以子进程方式启动对应平台的 CLI 工具（如 claude）

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::utils::Validatable;
use colored::Colorize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::str::FromStr;

/// 🏃 以隔离环境变量方式临时运行 CLI 工具
pub async fn execute(config_name: &str, args: &[String]) -> Result<()> {
    ColorOutput::title(&format!("临时运行配置: {}", config_name));
    println!();

    // 🔍 步骤 1: 查找配置属于哪个平台
    ColorOutput::step("步骤 1/3: 查找配置关联的平台");

    let platform_config_mgr = PlatformConfigManager::with_default()?;
    let unified_config = platform_config_mgr.load()?;

    // 遍历所有可用平台，寻找该配置
    let mut target_platform = None;
    let mut target_profile = None;

    for (platform_name, _) in unified_config.platforms.iter() {
        if let Ok(platform) = crate::models::Platform::from_str(platform_name) {
            if let Ok(platform_impl) = crate::platforms::create_platform(platform.clone()) {
                if let Ok(profiles) = platform_impl.load_profiles() {
                    if let Some(profile) = profiles.get(config_name) {
                        target_platform = Some(platform_name.clone());
                        target_profile = Some((platform, profile.clone()));
                        break;
                    }
                }
            }
        }
    }

    let platform_name = target_platform.ok_or_else(|| {
        ColorOutput::error(&format!("在所有平台中均未找到配置 '{}'", config_name));
        println!();
        ColorOutput::info("💡 提示:");
        println!("  • 运行 'ccr list' 查看可用配置");
        CcrError::ConfigSectionNotFound(config_name.to_string())
    })?;

    let (platform, profile) = target_profile.unwrap();

    ColorOutput::success(&format!(
        "✅ 找到配置 '{}' 属于平台: {}",
        config_name,
        platform_name.bright_yellow()
    ));
    println!();

    // 🧮 步骤 2: 提取环境变量
    ColorOutput::step("步骤 2/3: 准备隔离环境变量");

    // 转换 ProfileConfig 为 ConfigSection 以利用验证逻辑
    let target_section = crate::managers::config::ConfigSection {
        description: profile.description.clone(),
        base_url: profile.base_url.clone(),
        auth_token: profile.auth_token.clone(),
        model: profile.model.clone(),
        small_fast_model: profile.small_fast_model.clone(),
        provider: profile.provider.clone(),
        provider_type: profile.provider_type.as_ref().and_then(|pt: &String| {
            use crate::managers::config::ProviderType;
            match pt.as_str() {
                "official_relay" => Some(ProviderType::OfficialRelay),
                "third_party_model" => Some(ProviderType::ThirdPartyModel),
                _ => None,
            }
        }),
        account: profile.account.clone(),
        tags: profile.tags.clone(),
        usage_count: profile.usage_count,
        enabled: profile.enabled,
        other: indexmap::IndexMap::new(),
    };

    target_section.validate().map_err(|e| {
        ColorOutput::error(&format!("目标配置验证失败: {}", e));
        e
    })?;

    // 获取环境变量列表
    let mut process_envs: HashMap<String, String> = HashMap::new();

    if platform == crate::models::Platform::Claude {
        let env_status = target_section.to_anthropic_env_status();
        for (var_name, val_opt) in env_status {
            if let Some(val) = val_opt {
                process_envs.insert(var_name, val);
            }
        }
    } else if platform == crate::models::Platform::Codex {
        // Codex 主要使用 OPENAI_API_KEY，且 config.toml 优先，临时覆盖可能不完美，
        // 但可以通过注入 OPENAI_API_KEY 配合自定义 proxy URL.
        if let Some(token) = profile.auth_token.as_ref() {
            process_envs.insert("OPENAI_API_KEY".to_string(), token.clone());
        }
        if let Some(url) = profile.base_url.as_ref() {
            process_envs.insert("OPENAI_API_BASE".to_string(), url.clone());
        }
        if let Some(model) = profile.model.as_ref() {
            process_envs.insert("OPENAI_API_MODEL".to_string(), model.clone());
        }
    } else if platform == crate::models::Platform::Gemini {
        if let Some(token) = profile.auth_token.as_ref() {
            process_envs.insert("GEMINI_API_KEY".to_string(), token.clone());
        }
    } else if platform == crate::models::Platform::Qwen
        || platform == crate::models::Platform::Droid
    {
        // 兼容其他平台
        if let Some(token) = profile.auth_token.as_ref() {
            let key = format!("{}_API_KEY", platform_name.to_uppercase());
            process_envs.insert(key, token.clone());
        }
        if let Some(url) = profile.base_url.as_ref() {
            let key = format!("{}_BASE_URL", platform_name.to_uppercase());
            process_envs.insert(key, url.clone());
        }
    }

    // 打印即将注入的环境变量掩码
    if process_envs.is_empty() {
        ColorOutput::info("注：该配置未提取到特定的环境变量，将以当前环境运行");
    } else {
        for (k, v) in &process_envs {
            let k_str = k.as_str(); // Borrow key as string for iteration
            let is_sensitive = k_str.contains("TOKEN") || k_str.contains("KEY");
            let v_display = if is_sensitive {
                ColorOutput::mask_sensitive(v)
            } else {
                v.clone()
            };
            println!("  {} = {}", k.cyan(), v_display.truecolor(100, 100, 100));
        }
    }
    println!();

    // 🚀 步骤 3: 启动子进程
    let cli_tool = platform_name.clone(); // claude, codex, gemini

    let cmd_str = format!("{} {}", cli_tool, args.join(" "));
    ColorOutput::step(&format!(
        "步骤 3/3: 执行命令 `{}' ...",
        cmd_str.bright_green()
    ));
    println!();

    let mut child = Command::new(&cli_tool)
        .args(args)
        .envs(process_envs)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            CcrError::ExternalCommandError(format!("启动子进程 '{}' 失败: {}", cli_tool, e))
        })?;

    let status = child
        .wait()
        .map_err(|e| CcrError::ExternalCommandError(format!("等待子进程完成失败: {}", e)))?;

    if !status.success() {
        // 如果子进程异常退出，转发退出码
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
