// dispatch 路由直接测试（对应 07-03-arch-ccr-facade design 决策 2 的缩水范围）
//
// 覆盖：
// 1. 4 个 TUI 入口的注入路由：记录型启动器验证 `ccr` / `ccr codex` /
//    `ccr opencode` / `ccr claude` 各自命中正确的启动器；
// 2. launchers = None 时的降级分支不 panic 且行为符合降级逻辑
//    （tempdir + 环境变量隔离，不读写用户真实配置）；
// 3. 纯输出分支（version / help）返回 Ok；
// 4. 若干只读命令的进程内集成测试（不再依赖 ccr 层子进程黑盒）。
//
// 不覆盖（见 design.md 决策 2 的说明）：110+ 命令分支的全量可注入执行器
// 改造；写路径命令仍由 crates/ccr/tests/commands/ 的黑盒测试兜底。

#![allow(clippy::unwrap_used)]

use ccr_cli::cli::dispatch::TuiLaunchers;
use ccr_cli::cli::{Cli, CommandDispatcher};
use ccr_core::core::error::CcrError;
use clap::Parser;
use std::ffi::{OsStr, OsString};
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 环境隔离（写法对齐 crates/ccr/tests/support/env.rs）
// ---------------------------------------------------------------------------

static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

/// tempdir 隔离环境：接管所有 ccr 路径解析相关的环境变量，
/// 保证降级分支/只读命令不会触碰用户真实配置。
struct IsolatedEnv {
    _guard: MutexGuard<'static, ()>,
    _temp_dir: TempDir,
    previous_vars: Vec<(&'static str, Option<OsString>)>,
}

impl IsolatedEnv {
    fn new() -> Self {
        let guard = TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = TempDir::new().unwrap();
        let home = temp_dir.path().to_path_buf();
        let root = home.join(".ccr");
        let claude_dir = home.join(".claude");
        let lock_dir = home.join(".locks");
        let codex_dir = home.join(".codex");
        for dir in [&root, &claude_dir, &lock_dir, &codex_dir] {
            std::fs::create_dir_all(dir).unwrap();
        }

        let mut previous_vars = Vec::new();
        set_env_var(&mut previous_vars, "HOME", home.as_os_str());
        set_env_var(&mut previous_vars, "USERPROFILE", home.as_os_str());
        set_env_var(&mut previous_vars, "CCR_ROOT", root.as_os_str());
        set_env_var(&mut previous_vars, "CCR_LOCK_DIR", lock_dir.as_os_str());
        set_env_var(
            &mut previous_vars,
            "CLAUDE_CONFIG_DIR",
            claude_dir.as_os_str(),
        );
        set_env_var(
            &mut previous_vars,
            "CLAUDE_JSON_PATH",
            home.join(".claude.json").as_os_str(),
        );
        set_env_var(
            &mut previous_vars,
            "CCR_SETTINGS_PATH",
            claude_dir.join("settings.json").as_os_str(),
        );
        set_env_var(
            &mut previous_vars,
            "CCR_BACKUP_DIR",
            claude_dir.join("backups").as_os_str(),
        );
        set_env_var(&mut previous_vars, "CCR_CODEX_DIR", codex_dir.as_os_str());
        remove_env_var(&mut previous_vars, "CCR_CONFIG_PATH");

        Self {
            _guard: guard,
            _temp_dir: temp_dir,
            previous_vars,
        }
    }
}

impl Drop for IsolatedEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous_vars.drain(..).rev() {
            restore_env_var(key, previous);
        }
    }
}

fn set_env_var(
    previous_vars: &mut Vec<(&'static str, Option<OsString>)>,
    key: &'static str,
    value: &OsStr,
) {
    previous_vars.push((key, std::env::var_os(key)));
    // SAFETY: IsolatedEnv 持有进程级测试环境锁，直到 Drop 恢复该变量。
    unsafe {
        std::env::set_var(key, value);
    }
}

fn remove_env_var(previous_vars: &mut Vec<(&'static str, Option<OsString>)>, key: &'static str) {
    previous_vars.push((key, std::env::var_os(key)));
    // SAFETY: IsolatedEnv 持有进程级测试环境锁，直到 Drop 恢复该变量。
    unsafe {
        std::env::remove_var(key);
    }
}

fn restore_env_var(key: &str, previous: Option<OsString>) {
    // SAFETY: Drop 执行时 IsolatedEnv 仍持有测试环境锁。
    unsafe {
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

// ---------------------------------------------------------------------------
// 记录型 TUI 启动器
//
// fn 指针无法捕获环境，因此用 4 个静态原子 + 顶层 fn 记录调用。
// 4 个入口的断言放在同一个 #[test] 内顺序执行并在每轮 reset，
// 避免测试并行时静态状态互相串扰。
// ---------------------------------------------------------------------------

static MAIN_LAUNCHED: AtomicBool = AtomicBool::new(false);
static CODEX_AUTH_LAUNCHED: AtomicBool = AtomicBool::new(false);
static OPENCODE_AUTH_LAUNCHED: AtomicBool = AtomicBool::new(false);
static CLAUDE_AUTH_LAUNCHED: AtomicBool = AtomicBool::new(false);

fn record_main() -> Result<(), CcrError> {
    MAIN_LAUNCHED.store(true, Ordering::SeqCst);
    Ok(())
}

fn record_codex_auth() -> Result<(), CcrError> {
    CODEX_AUTH_LAUNCHED.store(true, Ordering::SeqCst);
    Ok(())
}

fn record_opencode_auth() -> Result<(), CcrError> {
    OPENCODE_AUTH_LAUNCHED.store(true, Ordering::SeqCst);
    Ok(())
}

fn record_claude_auth() -> Result<(), CcrError> {
    CLAUDE_AUTH_LAUNCHED.store(true, Ordering::SeqCst);
    Ok(())
}

fn recording_launchers() -> TuiLaunchers {
    TuiLaunchers {
        main: record_main,
        codex_auth: record_codex_auth,
        opencode_auth: record_opencode_auth,
        claude_auth: record_claude_auth,
    }
}

fn reset_launch_flags() {
    for flag in [
        &MAIN_LAUNCHED,
        &CODEX_AUTH_LAUNCHED,
        &OPENCODE_AUTH_LAUNCHED,
        &CLAUDE_AUTH_LAUNCHED,
    ] {
        flag.store(false, Ordering::SeqCst);
    }
}

fn launch_flags() -> [bool; 4] {
    [
        MAIN_LAUNCHED.load(Ordering::SeqCst),
        CODEX_AUTH_LAUNCHED.load(Ordering::SeqCst),
        OPENCODE_AUTH_LAUNCHED.load(Ordering::SeqCst),
        CLAUDE_AUTH_LAUNCHED.load(Ordering::SeqCst),
    ]
}

/// 用于"不应触发任何 TUI 启动器"的场景：一旦被调用直接 panic 使测试失败。
fn must_not_launch() -> Result<(), CcrError> {
    panic!("TUI 启动器不应被调用");
}

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

// ---------------------------------------------------------------------------
// 1. TUI 入口注入路由
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn tui_entries_route_to_injected_launchers() {
    let launchers = recording_launchers();
    // (命令行, 期望命中的启动器下标: 0=main 1=codex 2=opencode 3=claude)
    let cases: [(&[&str], usize); 4] = [
        (&["ccr"], 0),
        (&["ccr", "codex"], 1),
        (&["ccr", "opencode"], 2),
        (&["ccr", "claude"], 3),
    ];

    for (args, expected_index) in cases {
        reset_launch_flags();
        let cli = parse(args);
        let result = CommandDispatcher::dispatch(&cli, Some(&launchers)).await;
        assert!(result.is_ok(), "{args:?} 应返回启动器的 Ok: {result:?}");

        let mut expected = [false; 4];
        expected[expected_index] = true;
        assert_eq!(
            launch_flags(),
            expected,
            "{args:?} 应只命中启动器 #{expected_index}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn shortcut_config_name_returns_legacy_error_without_launching_tui() {
    // `ccr <配置名>` 快捷切换优先于 TUI：即便注入了启动器也不应触发，
    // 而是直接返回 legacy shortcut 迁移错误（不触碰文件系统）。
    let launchers = TuiLaunchers {
        main: must_not_launch,
        codex_auth: must_not_launch,
        opencode_auth: must_not_launch,
        claude_auth: must_not_launch,
    };
    let cli = parse(&["ccr", "team"]);
    let result = CommandDispatcher::dispatch(&cli, Some(&launchers)).await;

    match result {
        Err(CcrError::ConfigError(message)) => {
            assert!(
                message.contains("legacy shortcut retired"),
                "错误信息应为 legacy shortcut 迁移提示: {message}"
            );
        }
        other => panic!("应返回 ConfigError(legacy shortcut retired)，实际: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 2. launchers = None 时的降级分支
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn no_subcommand_without_launchers_falls_back_to_current_overview() {
    // 降级逻辑：`ccr`（无子命令）在未注入启动器时执行 current_command(false, false)，
    // 该命令在空环境下也能给出双平台状态总览（只读），返回 Ok。
    let _env = IsolatedEnv::new();
    let cli = parse(&["ccr"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "降级 current 总览应返回 Ok: {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn codex_without_launchers_falls_back_to_auth_list() {
    // 降级逻辑：`ccr codex`（无 action）在未注入启动器时执行 codex auth list，
    // 空环境下打印"未登录 Codex"提示并返回 Ok（只读）。
    let _env = IsolatedEnv::new();
    let cli = parse(&["ccr", "codex"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "降级 codex auth list 应返回 Ok: {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn opencode_without_launchers_falls_back_to_help() {
    // 降级逻辑：`ccr opencode`（无 action）在未注入启动器时打印 opencode 帮助，
    // 纯输出，无需真实环境。
    let cli = parse(&["ccr", "opencode"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "降级 opencode 帮助应返回 Ok: {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn claude_without_launchers_falls_back_to_auth_list() {
    // 降级逻辑：`ccr claude`（无 action）在未注入启动器时执行 claude auth list，
    // 空环境下打印"尚未保存任何官方账号快照"提示并返回 Ok（只读）。
    let _env = IsolatedEnv::new();
    let cli = parse(&["ccr", "claude"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(
        result.is_ok(),
        "降级 claude auth list 应返回 Ok: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// 3. 纯输出分支
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn version_branch_is_pure_output_and_returns_ok() {
    // `ccr version` 走 show_version，纯打印不触碰环境。
    let cli = parse(&["ccr", "version"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "version 分支应返回 Ok: {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn help_branch_is_pure_output_and_returns_ok() {
    // `ccr help` 走 help::print_command_help，纯打印不触碰环境。
    let cli = parse(&["ccr", "help"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "help 分支应返回 Ok: {result:?}");
}

// ---------------------------------------------------------------------------
// 4. 只读命令的进程内集成测试（tempdir 隔离）
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn list_command_routes_and_runs_in_isolated_env() {
    // `ccr list` 只读列出配置；空环境下 ConfigManager 会自动初始化默认配置
    // （写入的是隔离 tempdir 内的 CCR_ROOT），预期 Ok。
    let _env = IsolatedEnv::new();
    let cli = parse(&["ccr", "list"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "list 命令应返回 Ok: {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn current_json_routes_and_runs_in_isolated_env() {
    // `ccr current --json` 只读输出双平台结构化摘要，空环境下预期 Ok。
    let _env = IsolatedEnv::new();
    let cli = parse(&["ccr", "current", "--json"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "current --json 应返回 Ok: {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn platform_list_json_routes_and_runs_in_isolated_env() {
    // `ccr platform list --json` 只读列出支持的平台，预期 Ok。
    let _env = IsolatedEnv::new();
    let cli = parse(&["ccr", "platform", "list", "--json"]);
    let result = CommandDispatcher::dispatch(&cli, None).await;
    assert!(result.is_ok(), "platform list --json 应返回 Ok: {result:?}");
}
