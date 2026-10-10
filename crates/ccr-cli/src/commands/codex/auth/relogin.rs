//! 🔑 codex auth relogin 命令实现
//!
//! 本地移除当前 Codex 登录后重新执行 `codex login`。
//! 全程不调用 `codex logout`，远端 refresh token 不被吊销，已保存账号快照保持可用。

use crate::application::codex_local_auth_off;
use crate::commands::common::print_next_steps;
use crate::services::install_detect::which_on_path;
use ccr_codex::{CodexProcessService, DaemonStopOutcome, stop_codex_daemon};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;

/// `codex login` 的参数；relogin 只拉起登录，不传任何会触发 logout 的参数。
const CODEX_LOGIN_ARGS: &[&str] = &["login"];

/// 🔑 本地重新登录 Codex
///
/// 1. 删除本地 auth.json（删除前回写轮换后的 tokens 到已保存账号），不调用 `codex logout`
/// 2. 停止托管 app-server 守护进程，使其丢弃缓存的旧 tokens
/// 3. 在当前终端运行 `codex login`
///
/// keyring/auto 凭据存储只能经 `codex logout` 清除，因此直接拒绝。
/// 守护进程停止失败时不启动登录：守护进程仍缓存旧 tokens，经它发起的 logout 会吊销远端 token。
pub async fn relogin_command() -> Result<()> {
    let result = codex_local_auth_off()?;
    if result.changed {
        ColorOutput::success("已移除本地 Codex 登录（未调用 codex logout，远端 token 未吊销）");
    } else {
        ColorOutput::info("当前没有本地 Codex 登录文件");
    }
    for warning in &result.warnings {
        ColorOutput::warning(warning);
    }

    if let Some(daemon) = CodexProcessService::new().find_managed_daemon() {
        ColorOutput::step(&format!(
            "正在停止 app-server 守护进程 (PID {})...",
            daemon.pid
        ));
        if let Some(detail) = stop_failure_detail(&stop_codex_daemon().await) {
            print_next_steps(&[
                ("手动停止守护进程", "codex app-server daemon stop"),
                ("再重新登录", "codex login"),
            ]);
            return Err(CcrError::ExternalCommandError(format!(
                "{detail}；本地登录已移除，未启动 codex login"
            )));
        }
        ColorOutput::success("app-server 守护进程已停止");
    }

    let bin = which_on_path("codex").ok_or_else(|| {
        CcrError::ExternalCommandError(
            "PATH 中找不到 codex；本地登录已移除，安装后运行 codex login".into(),
        )
    })?;
    ColorOutput::step("正在启动 codex login...");
    let status = tokio::process::Command::new(&bin)
        .args(CODEX_LOGIN_ARGS)
        .status()
        .await
        .map_err(|error| {
            CcrError::ExternalCommandError(format!("无法启动 codex login: {error}；本地登录已移除"))
        })?;
    if !status.success() {
        return Err(CcrError::ExternalCommandError(format!(
            "codex login 未成功 ({status})；本地登录已移除，可再次运行 codex login"
        )));
    }

    ColorOutput::success("codex login 已完成");
    print_next_steps(&[
        ("确认当前账号", "ccr codex auth current"),
        ("保存为命名账号", "ccr codex auth save --help"),
    ]);
    Ok(())
}

/// 守护进程停止结果 → 失败说明；`None` 表示可以继续登录。
fn stop_failure_detail(outcome: &DaemonStopOutcome) -> Option<String> {
    match outcome {
        DaemonStopOutcome::Stopped => None,
        DaemonStopOutcome::Failed { detail } => {
            Some(format!("app-server 守护进程停止失败: {detail}"))
        }
        DaemonStopOutcome::Timeout => Some("app-server 守护进程停止超时，未能确认结果".into()),
        DaemonStopOutcome::Unavailable => {
            Some("PATH 中找不到 codex，无法停止 app-server 守护进程".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_stopped_daemon_allows_login() {
        assert_eq!(stop_failure_detail(&DaemonStopOutcome::Stopped), None);
        for outcome in [
            DaemonStopOutcome::Failed {
                detail: "exit code 7".into(),
            },
            DaemonStopOutcome::Timeout,
            DaemonStopOutcome::Unavailable,
        ] {
            assert!(stop_failure_detail(&outcome).is_some(), "{outcome:?}");
        }
    }

    #[test]
    fn login_args_never_request_logout() {
        assert_eq!(CODEX_LOGIN_ARGS, ["login"]);
    }
}
