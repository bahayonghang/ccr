// 🧹 Codex 进程清理服务
//
// 跨平台清理残留的 Codex `app-server` 进程：
// - 仅匹配命令行含 "codex" + "app-server" 的进程（等价脚本正则 `codex.*app-server`）；
//   刻意不触碰 `codex` / `codex exec` / `codex resume` 等普通 CLI 任务，也不误杀 `ccr` 自身。
// - Unix：先 SIGTERM 让进程自清理 socket/DB/状态，超时后再 SIGKILL；
//   Windows：无优雅信号语义，`kill_with` 返回 None 时直接 terminate。
// - 不做显式 user 过滤：非特权用户在 OS 层本就只能向自己的进程发信号（他人进程 kill 直接失败）。

use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use sysinfo::{ProcessesToUpdate, Signal, System};

/// 单个被识别的 Codex app-server 进程。
///
/// `cmdline` is a display-only, non-sensitive joined command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexAppServer {
    pub pid: u32,
    pub cmdline: String,
}

/// 终止某进程时实际采用的手段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationKind {
    /// Unix: exited after SIGTERM.
    Term,
    /// Unix: SIGKILL after SIGTERM timeout / Windows: terminate.
    Kill,
    /// Process disappeared before we could signal it.
    AlreadyGone,
}

/// 一次清理动作的结构化结果，供 CLI 渲染。
#[derive(Debug, Clone, Default)]
pub struct CodexAppServerCleanup {
    /// 初次发现的 app-server 进程。
    pub found: Vec<CodexAppServer>,
    /// 每个 PID 的终止手段（dry-run 时为空）。
    pub terminated: Vec<(u32, TerminationKind)>,
    /// 复检时仍存活的 app-server（被客户端重新拉起，或 kill 因权限失败）。
    pub respawned: Vec<CodexAppServer>,
    /// 是否为 dry-run（仅列出、不终止）。
    pub dry_run: bool,
}

/// Unix 下 SIGTERM 后等待进程退出的单次轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(300);
/// 最大轮询次数（约 3 秒），对齐脚本的宽限窗口。
const POLL_ROUNDS: u32 = 10;
/// 终止后到复检 respawn 之间的静默等待。
const RESPAWN_SETTLE: Duration = Duration::from_secs(1);

/// Codex 进程清理服务（无状态）。
#[derive(Debug, Default)]
pub struct CodexProcessService;

impl CodexProcessService {
    pub fn new() -> Self {
        Self
    }

    /// 枚举当前进程表中所有 Codex app-server 进程。
    pub fn find_app_servers(&self) -> Vec<CodexAppServer> {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        collect_app_servers(&sys)
    }

    /// 清理 app-server：Unix SIGTERM→（超时）SIGKILL，Windows terminate，然后复检 respawn。
    ///
    /// `dry_run` 为真时只枚举、不发送任何信号，也不做复检。
    pub fn cleanup(&self, dry_run: bool) -> CodexAppServerCleanup {
        let found = self.find_app_servers();
        let mut result = CodexAppServerCleanup {
            found: found.clone(),
            dry_run,
            ..Default::default()
        };
        if dry_run || found.is_empty() {
            return result;
        }

        let targets: HashSet<u32> = found.iter().map(|a| a.pid).collect();
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // Phase 1: 优雅终止。Unix 发 SIGTERM；Windows 上 kill_with 返回 None → 直接 kill()。
        let mut term_sent: HashSet<u32> = HashSet::new();
        for (pid, process) in sys.processes() {
            let raw = pid.as_u32();
            if !targets.contains(&raw) {
                continue;
            }
            match process.kill_with(Signal::Term) {
                Some(_) => {
                    term_sent.insert(raw);
                }
                None => {
                    // 平台不支持 SIGTERM（Windows）→ 直接终止。
                    process.kill();
                    result.terminated.push((raw, TerminationKind::Kill));
                }
            }
        }

        // 初次就已不在进程表里的目标记为 AlreadyGone。
        let present = alive_targets(&sys, &targets);
        for app in &found {
            let handled = term_sent.contains(&app.pid)
                || result.terminated.iter().any(|(p, _)| *p == app.pid);
            if !handled && !present.contains(&app.pid) {
                result
                    .terminated
                    .push((app.pid, TerminationKind::AlreadyGone));
            }
        }

        // Phase 2: Unix 轮询等待 SIGTERM 目标自行退出。
        if !term_sent.is_empty() {
            for _ in 0..POLL_ROUNDS {
                thread::sleep(POLL_INTERVAL);
                sys.refresh_processes(ProcessesToUpdate::All, true);
                if alive_targets(&sys, &term_sent).is_empty() {
                    break;
                }
            }
        }

        // Phase 3: 对仍存活的 SIGTERM 目标补 SIGKILL；已退出者记为 Term。
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let mut killed: HashSet<u32> = HashSet::new();
        for (pid, process) in sys.processes() {
            let raw = pid.as_u32();
            if term_sent.contains(&raw) {
                process.kill();
                killed.insert(raw);
            }
        }
        for raw in &term_sent {
            let kind = if killed.contains(raw) {
                TerminationKind::Kill
            } else {
                TerminationKind::Term
            };
            result.terminated.push((*raw, kind));
        }

        // Phase 4: 复检 respawn（被 Desktop / VS Code Remote 重新拉起，或 kill 因权限失败）。
        thread::sleep(RESPAWN_SETTLE);
        result.respawned = self.find_app_servers();

        result
    }
}

/// 从进程快照中收集所有 app-server 进程。
fn collect_app_servers(sys: &System) -> Vec<CodexAppServer> {
    sys.processes()
        .iter()
        .filter_map(|(pid, process)| {
            let cmdline = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            if is_codex_app_server(&cmdline.to_lowercase()) {
                Some(CodexAppServer {
                    pid: pid.as_u32(),
                    cmdline,
                })
            } else {
                None
            }
        })
        .collect()
}

/// 目标 PID 中当前仍存活的子集。
fn alive_targets(sys: &System, targets: &HashSet<u32>) -> HashSet<u32> {
    sys.processes()
        .keys()
        .filter_map(|pid| {
            let raw = pid.as_u32();
            targets.contains(&raw).then_some(raw)
        })
        .collect()
}

/// 判断一条（已小写的）命令行是否为 Codex app-server。
///
/// 规则：同时包含 "codex" 与 "app-server"，且不含 "ccr"（排除 CCR 自身进程）。
/// 该规则刻意窄化——`codex` / `codex exec` / `codex resume` 因不含 "app-server" 均不命中，
/// 从而杜绝误杀普通 Codex CLI 任务。含 "ccr" 的路径可能被漏杀，但漏杀是安全的（误杀才危险）。
fn is_codex_app_server(cmdline_lower: &str) -> bool {
    cmdline_lower.contains("codex")
        && cmdline_lower.contains("app-server")
        && !cmdline_lower.contains("ccr")
}

#[cfg(test)]
mod tests {
    use super::is_codex_app_server;

    #[test]
    fn matches_real_app_server_command_lines() {
        assert!(is_codex_app_server(
            "codex app-server --listen unix:///tmp/codex.sock"
        ));
        // 通过 node 启动的封装也应命中。
        assert!(is_codex_app_server(
            "/usr/bin/node /home/u/.npm/lib/codex/bin/codex app-server"
        ));
    }

    #[test]
    fn never_matches_plain_codex_tasks() {
        assert!(!is_codex_app_server("codex"));
        assert!(!is_codex_app_server("codex exec run some task"));
        assert!(!is_codex_app_server("codex resume thread-a"));
        assert!(!is_codex_app_server("/usr/local/bin/codex login"));
    }

    #[test]
    fn never_matches_ccr_or_unrelated_processes() {
        // ccr 自身（即便命令行含 codex）必须被排除。
        assert!(!is_codex_app_server("ccr codex fix"));
        assert!(!is_codex_app_server("ccr codex sessions trash-list"));
        // 路径含 ccr 会被漏杀——这是可接受的安全代价（漏杀而非误杀）。
        assert!(!is_codex_app_server("/home/u/.ccr/bin/codex app-server"));
        // 与 codex 无关的 app-server 不命中。
        assert!(!is_codex_app_server("some-other-app-server --port 1234"));
        assert!(!is_codex_app_server(""));
    }
}
