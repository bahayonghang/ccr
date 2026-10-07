//! 🔄 codex auth switch 命令实现
//!
//! 切换到指定账号。

use crate::commands::common::print_next_steps;
use crate::services::CodexAuthService;
use ccr_codex::{CodexProcessService, DaemonRestartOutcome, restart_codex_daemon};
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use std::io::IsTerminal;

/// 守护进程重启交互决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaemonRestartAction {
    /// `--restart-daemon`：跳过确认直接重启。
    RestartNow,
    /// 交互式终端：警告并询问。
    Prompt,
    /// 非交互环境：只输出警告与手动命令。
    WarnOnly,
}

fn resolve_daemon_restart_action(restart_daemon: bool, interactive: bool) -> DaemonRestartAction {
    if restart_daemon {
        DaemonRestartAction::RestartNow
    } else if interactive {
        DaemonRestartAction::Prompt
    } else {
        DaemonRestartAction::WarnOnly
    }
}

/// 🔄 切换到指定账号
///
/// 将 ~/.codex/auth.json 切换为指定账号的登录状态。
/// 若检测到托管 app-server 守护进程仍缓存旧账号，按交互模式决定是否重启；
/// 拒绝或重启失败不改变切换结果与退出码。
///
/// # 参数
///
/// * `name` - 要切换到的账号名称
/// * `restart_daemon` - 切换成功后直接重启 app-server 守护进程（跳过确认）
///
/// # 返回
///
/// * `Ok(())` - 切换成功
/// * `Err(CcrError)` - 切换失败
pub async fn switch_command(name: &str, restart_daemon: bool) -> Result<()> {
    let service = CodexAuthService::new()?;
    let auth_state = service.get_auth_state();

    if matches!(
        auth_state.status,
        crate::models::AuthStateStatus::Unsupported
    ) {
        ColorOutput::error("当前凭据存储模式暂不支持 CCR 切换账号");
        ColorOutput::key_value("凭据存储", auth_state.store.as_str(), 2);
        ColorOutput::key_value("状态说明", &auth_state.reason, 2);
        ColorOutput::info("多账号管理需要将 cli_auth_credentials_store 切换为 file");
        print_next_steps(&[
            ("使用官方登录", "codex login"),
            ("使用官方登出", "codex logout"),
        ]);
        return Ok(());
    }

    // 检测 Codex 进程
    let running_processes = service.detect_codex_process();
    if !running_processes.is_empty() {
        ColorOutput::warning("检测到 Codex 进程正在运行");
        ColorOutput::key_value("进程 ID", &format!("{:?}", running_processes), 2);
        println!();
        ColorOutput::warning("切换账号可能导致正在运行的 Codex 会话出现问题");
        ColorOutput::info("建议先关闭所有 Codex 进程后再切换账号");
        println!();
    }

    // 执行切换
    match service.switch_account(name) {
        Ok(()) => {
            ColorOutput::success(&format!("已切换到账号 {name}"));

            // 显示切换后的账号信息
            if let Ok(info) = service.get_current_auth_info()
                && let Some(email) = &info.email
            {
                ColorOutput::key_value("邮箱", &service.mask_email(email), 2);
            }

            handle_daemon_after_switch(restart_daemon).await;

            print_next_steps(&[("查看当前账号", "ccr codex auth current")]);
        }
        Err(e) => {
            ColorOutput::error(&format!("切换失败: {}", e));

            // 如果是账号不存在，显示可用账号
            let err_msg = e.to_string();
            if err_msg.contains("不存在") {
                print_next_steps(&[("查看可用账号", "ccr codex auth list")]);
            }
        }
    }

    Ok(())
}

/// 切换成功后检测托管守护进程，并按交互模式处理重启。
///
/// 无守护进程时零输出；重启失败只给警告与手动命令，不改变切换结果。
async fn handle_daemon_after_switch(restart_daemon: bool) {
    let Some(daemon) = CodexProcessService::new().find_managed_daemon() else {
        return;
    };
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    match resolve_daemon_restart_action(restart_daemon, interactive) {
        DaemonRestartAction::RestartNow => {
            ColorOutput::info(&format!(
                "检测到 app-server 守护进程 (PID {})，正在重启...",
                daemon.pid
            ));
            render_daemon_restart_outcome(restart_codex_daemon().await);
        }
        DaemonRestartAction::Prompt => {
            ColorOutput::warning(&format!(
                "检测到 app-server 守护进程 (PID {}) 仍在使用切换前的账号",
                daemon.pid
            ));
            ColorOutput::info(
                "重启守护进程后 Codex 客户端的新会话会使用新账号；正在运行的会话会被中断",
            );
            if ColorOutput::ask_confirmation("是否立即重启 app-server 守护进程?", false) {
                ColorOutput::info("正在重启 app-server 守护进程...");
                render_daemon_restart_outcome(restart_codex_daemon().await);
            } else {
                print_next_steps(&[("手动重启守护进程", "codex app-server daemon restart")]);
            }
        }
        DaemonRestartAction::WarnOnly => {
            let warning = format!(
                "app-server 守护进程 (PID {}) 仍在使用切换前的账号；运行 codex app-server daemon restart 使切换对 Codex 客户端生效",
                daemon.pid
            );
            eprintln!(
                "{}",
                ColorOutput::format_status(
                    ccr_core::core::logging::OutputStatus::Warning,
                    &warning,
                    std::io::stderr().is_terminal(),
                )
            );
        }
    }
}

/// 渲染守护进程重启结果；失败统一给出手动命令，不影响切换成功语义。
fn render_daemon_restart_outcome(outcome: DaemonRestartOutcome) {
    match outcome {
        DaemonRestartOutcome::Restarted { pid: Some(pid) } => {
            ColorOutput::success(&format!("app-server 守护进程已重启 (新 PID {pid})"));
        }
        DaemonRestartOutcome::Restarted { pid: None } => {
            ColorOutput::success("app-server 守护进程已重启");
        }
        DaemonRestartOutcome::Failed { detail } => {
            ColorOutput::warning(&format!("app-server 守护进程重启失败: {detail}"));
            print_next_steps(&[("手动重启守护进程", "codex app-server daemon restart")]);
        }
        DaemonRestartOutcome::Timeout => {
            ColorOutput::warning("app-server 守护进程重启超时，未能确认结果");
            print_next_steps(&[("手动重启守护进程", "codex app-server daemon restart")]);
        }
        DaemonRestartOutcome::Unavailable => {
            ColorOutput::warning("PATH 中找不到 codex，无法重启 app-server 守护进程");
            print_next_steps(&[("手动重启守护进程", "codex app-server daemon restart")]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DaemonRestartAction, resolve_daemon_restart_action};

    #[test]
    fn restart_flag_and_interactive_terminal_resolve_daemon_action() {
        assert_eq!(
            resolve_daemon_restart_action(true, true),
            DaemonRestartAction::RestartNow
        );
        assert_eq!(
            resolve_daemon_restart_action(true, false),
            DaemonRestartAction::RestartNow
        );
        assert_eq!(
            resolve_daemon_restart_action(false, true),
            DaemonRestartAction::Prompt
        );
        assert_eq!(
            resolve_daemon_restart_action(false, false),
            DaemonRestartAction::WarnOnly
        );
    }
}
