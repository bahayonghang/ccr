# Design — ccr codex fix

## 1. 命令面（clap）

在 `crates/ccr-cli/src/cli/subcommands/codex.rs` 的 `CodexAction` 枚举新增变体：

```rust
/// 清理残留 Codex app-server 进程并诊断实际加载的配置/认证来源
///
/// 修复 SSH / Desktop / VS Code Remote 断开后 app-server 锁定旧登录态的问题。
/// 示例: ccr codex fix
///       ccr codex fix --dry-run
Fix {
    /// 只列出将被清理的 app-server 进程，不实际终止
    #[arg(long)]
    dry_run: bool,
},
```

> 决策已定：`--dry-run` 纳入 MVP；`--json` 本次不做（简单优先）。

`crates/ccr-cli/src/cli/dispatch.rs` 在 `CodexAction` 匹配臂新增：

```rust
Some(CodexAction::Fix { dry_run }) => {
    crate::commands::codex::fix::fix_command(*dry_run).await
}
```

`ccr codex help` 呈现：**已确认帮助为 clap 派生**（`cli/help.rs::print_subcommand_help` → `render_help_text`；`cli/help_config.rs::configure_codex_command` 仅定制 about/template 与 `auth` 子命令）。新增 `Fix` derive 变体后其 doc 注释会自动出现在 `ccr codex help`，**无需改动 help 装配**。

## 2. 分层与边界

| 层 | 位置 | 职责 |
|----|------|------|
| 域服务 | `crates/ccr-codex/src/services/codex_process_service.rs`（新增） | app-server 枚举、分类、信号终止 + 升级、重生复检。纯域逻辑、可单测。 |
| CLI 命令 | `crates/ccr-cli/src/commands/codex/fix.rs`（新增） | 编排：调用域服务 → 渲染进程清理结果 → PATH 校验 → spawn `codex doctor` → 保存报告/高亮字段 → 环境提示 → 退出码。 |

**为何拆分**：进程终止是破坏性且需要跨平台正确性的核心逻辑，放域层便于单测与未来 UI 复用；`codex doctor` 是 Codex 自身诊断的外部编排，属 CLI 关注点，不污染 `ccr-codex`（避免 `ccr-codex` 引入 spawn/PATH 概念）。依赖方向保持 `ccr-cli → ccr-codex`。

## 3. 域服务：`CodexProcessService`

### 3.1 数据结构

```rust
/// 单个被识别的 Codex app-server 进程
#[derive(Debug, Clone)]
pub struct CodexAppServer {
    pub pid: u32,
    pub cmdline: String, // 已 join、用于展示（不含敏感信息）
}

/// 终止某进程时采用的手段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationKind {
    Term,      // Unix: SIGTERM 后正常退出
    Kill,      // Unix: SIGTERM 超时后 SIGKILL / Windows: terminate
    AlreadyGone,
}

/// 清理动作的结构化结果（供 CLI 渲染 / --json）
#[derive(Debug, Clone, Default)]
pub struct CodexAppServerCleanup {
    pub found: Vec<CodexAppServer>,        // 初次发现
    pub terminated: Vec<(u32, TerminationKind)>,
    pub respawned: Vec<CodexAppServer>,    // 复检仍存活 → 被客户端重新拉起
    pub dry_run: bool,
}
```

### 3.2 分类器（纯函数，重点单测）

```rust
/// 判断一条命令行是否为 Codex app-server（等价脚本 `codex.*app-server`）。
/// 规则：小写后同时包含 "codex" 与 "app-server"，且不含 "ccr"（排除自身/CCR 进程）。
pub(crate) fn is_codex_app_server(cmdline_lower: &str) -> bool {
    cmdline_lower.contains("codex")
        && cmdline_lower.contains("app-server")
        && !cmdline_lower.contains("ccr")
}
```

- 命令行来源：`sysinfo::Process::cmd()` → `&[OsString]`，`join(" ")` 后 `to_string_lossy().to_lowercase()`。
- 反面用例（必须为 false）：`codex`、`codex exec ...`、`codex resume ...`、`ccr codex fix`、任何含 `ccr` 的进程。
- 正面用例（必须为 true）：`codex app-server --listen unix://...`、`node .../codex app-server`。

### 3.3 用户范围（依赖 OS 强制，不做显式过滤）

- 决策：**不做显式 `user_id` 过滤**。非特权用户在 OS 层本就只能向自己的进程发信号；他人进程 `kill()` 直接失败（Unix EPERM，返回 `false`）。app-server 窄匹配已足够安全，且避免引入 sysinfo `user` feature 的编译不确定性。
- 因此 `cleanup` 按 PID 记录终止成功/失败（`kill`/`kill_with` 的返回值），失败者不视为「已清理」。

### 3.4 枚举 + 终止 + 升级（跨平台）

```rust
pub fn find_app_servers(&self) -> Vec<CodexAppServer> { /* refresh + 分类（无用户过滤） */ }

pub fn cleanup(&self, dry_run: bool) -> CodexAppServerCleanup {
    let found = self.find_app_servers();
    if dry_run || found.is_empty() { return /* 只带 found */; }

    // 1) 尝试 SIGTERM；kill_with 返回 None ⇒ 平台不支持该信号（Windows）⇒ 直接 kill()
    for p in &found {
        match process.kill_with(Signal::Term) {
            None => { process.kill(); /* Windows terminate */ }
            Some(_) => { /* Unix 已发 SIGTERM */ }
        }
    }
    // 2) Unix：轮询最多 ~3s（10 × 300ms），刷新进程表，等 SIGTERM 目标退出
    // 3) 仍存活者 process.kill()（SIGKILL）
    // 4) sleep ~1s 后再 find_app_servers() → 复检 respawned
}
```

- 关键 `sysinfo` 事实（0.38.4，已验证 docs）：
  - `Process::kill()` 发送 `SIGKILL`（唯一全平台支持信号）/ Windows terminate，返回 `bool`。
  - `Process::kill_with(Signal::Term)` 返回 `Option<bool>`；`None` 表示当前平台不支持该信号 → Windows 走此分支回退 `kill()`。
  - `Process::cmd()` 返回 `&[OsString]`；`Process::user_id()` 返回 `Option<&Uid>`。
- 轮询用 `System::refresh_processes(ProcessesToUpdate::Some(&[pid...]), true)` 局部刷新，避免全表重扫。
- 不阻塞 async runtime：`cleanup` 为同步 CPU/系统调用；在 CLI 侧用 `tokio::task::spawn_blocking` 包裹，或直接同步调用（命令本身短时）。倾向直接同步 + `std::thread::sleep`（命令是一次性前台操作，简单优先）。

### 3.5 导出

- `services/mod.rs` 加 `pub mod codex_process_service;` 并 `pub use`。
- `lib.rs` re-export：`CodexProcessService`、`CodexAppServer`、`CodexAppServerCleanup`、`TerminationKind`。

## 4. CLI 编排：`commands/codex/fix.rs`

```rust
pub async fn fix_command(dry_run: bool) -> Result<()> {
    // A. 进程清理
    let service = CodexProcessService::new()?;
    let cleanup = service.cleanup(dry_run);
    render_cleanup(&cleanup);           // 打印发现/终止/PID

    // B. 环境提示（脱敏）
    render_env_hints();                 // CODEX_HOME / OPENAI_BASE_URL 值 + OPENAI_API_KEY 存在性

    // C. codex doctor
    let Some(codex_bin) = which_on_path("codex") else {
        ColorOutput::error("PATH 中找不到 codex 命令");
        flush(); std::process::exit(127);
    };
    let doctor = run_codex_doctor(&codex_bin).await;   // 见 §5
    render_doctor(&doctor);             // 保存报告路径 + 高亮字段（或降级文本）

    // D. 退出码：复检到 respawn ⇒ 2
    if !cleanup.respawned.is_empty() {
        ColorOutput::warning("app-server 已被重新拉起，请关闭 Desktop / VS Code Remote 后重试");
        flush(); std::process::exit(2);
    }
    Ok(())
}
```

- `which_on_path` 复用 `crate::services::install_detect::which_on_path`（同 crate，`pub fn`，已支持 Windows `.exe/.cmd/.bat`）。若非跨模块可见则薄封装一层。
- 退出码用 `std::process::exit`（先 `io::stdout().flush()`），沿用 `commands/doctor_cmd.rs` 既有先例。
- `--dry-run` 时：`render_cleanup` 只打印将清理列表；跳过 D 的 respawn 退出码判定（无终止即无重生复检）。

## 5. `codex doctor` 调用

```rust
struct DoctorOutcome {
    report_path: Option<PathBuf>, // 保存的完整 json/文本
    highlights: Vec<(String, String)>, // best-effort 关键字段
    raw_text: Option<String>,     // 降级路径
    failed: bool,
}
```

- 首选：`codex doctor --json`，用 `tokio::process::Command` + `tokio::time::timeout(30s)` 包裹，防网络探活挂死。
- 保存：完整 stdout 写入 `std::env::temp_dir().join(format!("codex-doctor-{user}-{ts}.json"))`，打印路径。
- 解析（best-effort）：`serde_json::from_slice::<Value>`；若含 `details`（对象）则遍历，按「键名/路径包含子串」提取 CODEX_HOME、config、provider、auth mode 等并高亮；解析失败或无 `--json` 支持（stderr 提示 unknown/unrecognized flag，或退出非 0）→ 回退 `codex doctor`（文本）原样打印。
- 安全：只转发 codex 自身（已脱敏）输出；不额外读取或打印 `auth.json`。

## 6. 输出格式（人读默认）

沿用 `ccr_core::core::logging::ColorOutput`（info/success/warning/error），风格对齐既有 codex 命令。区块顺序：
1. 进程检查（发现的 app-server / "未发现残留"）。
2. 终止动作（PID + 手段）或 dry-run 列表。
3. 复检结果（清除干净 / 被重新拉起告警）。
4. 环境提示（脱敏）。
5. doctor 关键字段 + 报告路径。

## 7. 错误与退出码

| 情况 | 处理 | 退出码 |
|------|------|--------|
| 正常完成，无 respawn | `Ok(())` | 0 |
| app-server 被重新拉起 | 告警 + `exit` | 2 |
| PATH 无 codex | 中文错误 + `exit` | 127 |
| `codex doctor` 失败/超时 | 记为诊断失败并提示，不 panic | 0（清理已完成）或沿用既有错误路径 |
| 进程表不可读 / 无权限 | 降级为「未发现」或诊断信息，不 panic | 0 |

- 复用现有 `CcrError` 变体（如 `ConfigError` / 通用 IO 映射），**不新增变体**（变体冻结）。

## 8. 跨平台矩阵

| 行为 | Unix | Windows |
|------|------|---------|
| 枚举 | sysinfo cmd() 匹配 | 同 |
| 优雅终止 | `kill_with(Signal::Term)` = SIGTERM | 返回 None → `kill()` terminate |
| 强杀 | 超时后 `kill()` = SIGKILL | 已在上一步 terminate |
| 用户过滤 | user_id 比较 | best-effort（None 则放宽） |
| `codex` 发现 | PATH 无扩展名 | PATH + `.exe/.cmd/.bat` |

## 9. 测试策略

- **域层单测（`codex_process_service.rs`）**：
  - `is_codex_app_server`：正/反用例矩阵（R1 边界，防误杀）——**必测**。
  - 用户过滤与枚举：对 `cmd()`→cmdline 的构造/join 逻辑抽纯函数测试；真实进程终止不进单测。
- **CLI 解析测试**（`cli/definitions.rs` 既有 `Cli::try_parse_from(["ccr","codex","sessions","trash-list"])` 风格）：新增 `["ccr","codex","fix"]` 与 `--dry-run` 解析断言。
- **doctor 渲染纯函数测试**：给定样例 `{"checks":..,"details":..}` JSON 字符串，断言 highlights 提取；给定非 JSON 断言降级为 raw_text。不 spawn 真实 codex。
- 涉及 env 的测试用 `TestCodexEnv`（若触及 Codex 路径解析）。

## 10. 验证命令

- `just fmt-check`
- `cargo test -p ccr-codex -- --test-threads=1`
- `cargo test -p ccr --test commands -- --test-threads=1`
- `just lint-strict`
- 手动（Unix，可选）：起一个假 `sleep`/伪 app-server 验证匹配窄化；`ccr codex fix --dry-run` 真机自测。

## 11. 影响面 / 兼容性

- 纯新增命令与新增域服务；不改既有 codex 命令行为，不改配置/认证读写路径 → 向后兼容。
- 不新增依赖、不改版本号。
- 回滚：删除新增文件 + 回退 4 处装配点（subcommands/codex.rs、dispatch.rs、services/mod.rs、lib.rs、commands/codex/mod.rs）即可，无数据迁移。
