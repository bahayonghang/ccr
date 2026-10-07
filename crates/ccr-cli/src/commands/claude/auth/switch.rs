//! 🔄 claude auth switch 命令实现

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
use crate::models::ClaudeLoginState;
use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;

pub async fn switch_command(name: &str) -> Result<()> {
    let service = ClaudeAuthService::new()?;
    let had_api_key_override = service
        .get_runtime_summary()
        .ok()
        .is_some_and(|summary| matches!(summary.login_state, ClaudeLoginState::ApiKeyActive));

    match service.switch_account(name) {
        Ok(outcome) => {
            ColorOutput::success(&format!("已切换到 Claude 官方账号 {name}"));
            ColorOutput::key_value("凭据文件", "已更新 ~/.claude/.credentials.json", 2);
            if had_api_key_override || !outcome.cleared_managed_sources.is_empty() {
                ColorOutput::key_value(
                    "Profile 托管覆盖",
                    &format!(
                        "已清理 settings.json 中 {} 个",
                        outcome.cleared_managed_sources.len()
                    ),
                    2,
                );
            }
            if outcome.remaining_suppressors.is_empty() && outcome.warnings.is_empty() {
                ColorOutput::info("本进程可见范围内未发现其他订阅压制源");
            } else if !outcome.remaining_suppressors.is_empty() {
                ColorOutput::warning("仍存在 CCR 不会自动清理的认证来源（请按置信度判断）:");
                for source in &outcome.remaining_suppressors {
                    println!(
                        "  • {} @ {} ({}; {}; {})",
                        source.kind.as_str(),
                        source.location.as_str(),
                        source.confidence.as_str(),
                        source.evidence.as_str(),
                        source.ownership.as_str()
                    );
                }
                ColorOutput::info("以上结论仅覆盖本 ccr 进程可见的 env 与已解析用户配置");
            } else {
                ColorOutput::warning("切换后认证来源诊断未完成:");
                for warning in &outcome.warnings {
                    println!("  • {warning}");
                }
            }
            print_next_steps(&[("查看当前认证", "ccr claude auth current")]);
        }
        Err(e) => {
            ColorOutput::error(&format!("切换失败: {}", e));
            if e.to_string().contains("不存在") {
                print_next_steps(&[("查看可用账号", "ccr claude auth list")]);
            }
        }
    }

    Ok(())
}
