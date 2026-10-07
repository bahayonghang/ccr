//! 💾 codex auth save 命令实现
//!
//! 保存当前登录到指定名称。

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
use crate::services::CodexAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;

/// 💾 保存当前登录到指定名称
///
/// 将当前 ~/.codex/auth.json 保存为命名账号。
///
/// # 参数
///
/// * `name` - 账号名称
/// * `description` - 账号描述 (可选)
/// * `force` - 是否强制覆盖已存在的账号
///
/// # 返回
///
/// * `Ok(())` - 保存成功
/// * `Err(CcrError)` - 保存失败
pub async fn save_command(name: &str, description: Option<String>, force: bool) -> Result<()> {
    let service = CodexAuthService::new()?;
    let auth_state = service.get_auth_state();

    if matches!(
        auth_state.status,
        crate::models::AuthStateStatus::Unsupported
    ) {
        ColorOutput::error("当前凭据存储模式暂不支持 CCR 保存账号");
        ColorOutput::key_value("凭据存储", auth_state.store.as_str(), 2);
        ColorOutput::key_value("状态说明", &auth_state.reason, 2);
        ColorOutput::info("多账号管理需要将 cli_auth_credentials_store 切换为 file");
        print_next_steps(&[
            ("使用官方登录", "codex login"),
            ("使用官方登出", "codex logout"),
        ]);
        return Ok(());
    }

    // 检查是否已登录
    if !service.is_logged_in() {
        ColorOutput::error("未登录 Codex");
        print_next_steps(&[("登录 Codex", "codex login")]);
        return Ok(());
    }

    // 执行保存
    match service.save_current(name, description.clone(), force) {
        Ok(()) => {
            ColorOutput::success(&format!("已保存账号 {name}"));

            if let Some(desc) = description
                && !desc.is_empty()
            {
                ColorOutput::key_value("描述", &desc, 2);
            }

            // 显示当前账号信息
            if let Ok(info) = service.get_current_auth_info()
                && let Some(email) = &info.email
                && !email.is_empty()
            {
                ColorOutput::key_value("邮箱", &service.mask_email(email), 2);
            }

            print_next_steps(&[("查看账号", "ccr codex auth list")]);
        }
        Err(e) => {
            ColorOutput::error(&format!("保存失败: {}", e));

            // 如果是因为账号已存在，提示使用 --force
            let err_msg = e.to_string();
            if err_msg.contains("已存在") {
                ColorOutput::info("覆盖会替换该账号已保存的凭据快照");
                print_next_steps(&[(
                    "覆盖已保存账号",
                    &format!("ccr codex auth save --force -- {name}"),
                )]);
            }
        }
    }

    Ok(())
}
