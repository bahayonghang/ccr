# Codex App-Server Cleanup

> Contracts for `ccr codex fix`: narrow app-server process cleanup plus `codex doctor` runtime diagnosis.

## Scenario: app-server cleanup + doctor diagnosis

### 1. Scope / Trigger

- Trigger: changing `CodexProcessService`, the `ccr codex fix` command, or its `codex doctor` invocation/rendering.
- Purpose: kill残留 Codex `app-server` 进程（SSH / Desktop / VS Code Remote 断开后锁定旧登录态），再运行 `codex doctor` 展示实际加载的配置/认证来源。等价于外部 `codexfix` bash 脚本。
- Cross-platform: Unix + Windows single build (no `#[cfg]` process code; sysinfo abstracts signals).

### 2. Signatures

- Service: `CodexProcessService::new() -> Self`（无状态，infallible）。
- Service: `CodexProcessService::find_app_servers() -> Vec<CodexAppServer>`。
- Service: `CodexProcessService::cleanup(dry_run: bool) -> CodexAppServerCleanup`。
- Types: `CodexAppServer { pid: u32, cmdline: String }`；`TerminationKind { Term, Kill, AlreadyGone }`；`CodexAppServerCleanup { found, terminated: Vec<(u32, TerminationKind)>, respawned, dry_run }`。
- CLI: `ccr codex fix [--dry-run]`（clap 派生，自动进入 `ccr codex help`）。

### 3. Contracts

- **窄匹配（防误杀，最高优先）**：进程命中当且仅当其**命令行**（`sysinfo::Process::cmd()` join 后小写）同时包含 `"codex"` 与 `"app-server"`，且不包含 `"ccr"`。
  - `codex` / `codex exec` / `codex resume` / `codex login` 因不含 `"app-server"` **绝不命中**。
  - `ccr ...` 进程因含 `"ccr"` **绝不命中**（含 `ccr codex fix` 自身）。
  - `cmd()` 为空/不可读的进程一律**跳过**（宁漏杀不误杀）。含 `"ccr"` 路径的真实 app-server 会被漏杀——漏杀安全、误杀危险，接受此代价。
- **跨平台终止升级**：Unix 先 `kill_with(Signal::Term)`（SIGTERM），轮询 ~3s（10 × 300ms）等待退出，仍存活者 `kill()`（SIGKILL）。Windows 上 `kill_with(Signal::Term)` 返回 `None`（信号不支持）→ 直接 `kill()`（terminate）。
- **不做显式 user 过滤**：非特权用户在 OS 层只能向自己的进程发信号（他人进程 kill 失败），窄匹配已足够；按 PID 记录终止手段。
- **respawn 复检**：非 dry-run 终止后 `sleep ~1s` 重新 `find_app_servers()`；仍存活者填入 `respawned`（客户端重新拉起或 kill 权限失败）。
- **dry_run**：只枚举 `found`，`terminated` 为空，不发任何信号，不做 respawn 复检。
- **doctor 调用**：`codex doctor --json`，以「stdout 是否为有效 JSON」判成功（**非退出码**——检查项失败时 doctor 可能返回非 0，但 stdout 仍是有效报告，须照常展示）。旧版无 `--json` → 回退 `codex doctor` 纯文本。外部调用 30s 超时，`stdin` 置 null。
- **doctor schema v1**：`{ codexVersion, overallStatus, checks: { <id>: { details: { <key>: <value> } } } }`。`details` **嵌套在 `checks.<id>` 之下，非顶层**。高亮 = 顶层 `codexVersion`/`overallStatus` + 遍历 `checks.*.details` 中键名（小写）包含 `codex_home|config|provider|auth|base_url|endpoint|model` 的字段。
- **报告落盘**：完整 stdout 写入 `temp_dir()/codex-doctor-<user>-<ts>.<json|txt>`，打印路径。
- **安全**：`OPENAI_API_KEY` 只显「已设置/未设置」，绝不回显值；`CODEX_HOME`/`OPENAI_BASE_URL` 显值（非密钥）；只转发 codex 自身脱敏输出，绝不打印 `auth.json` 原文；不 log token。

### 4. Validation & Error Matrix

- `codex` 不在 PATH（`which_on_path("codex") == None`）→ 中文 error + `std::process::exit(127)`。
- `cleanup.respawned` 非空 → `std::process::exit(2)`（在 doctor 之后判定，对齐脚本先警告后退出码）。
- doctor 超时 / 无法 spawn → 记为诊断失败并 warning，**不 panic**；命令仍返回（除非 respawn），退出码 0。
- doctor `--json` stdout 非有效 JSON → 回退纯文本；纯文本也失败 → warning，退出码 0。
- 报告写盘失败 → `report_path = None`，不影响主流程。
- 退出码用 `std::process::exit`（先 `io::stdout().flush()`），沿用 `doctor_cmd` 先例。
- 不新增 `CcrError` 变体（变体冻结）。

### 5. Good/Base/Bad Cases

- Good: 改分类逻辑时补 `is_codex_app_server` 正反用例（app-server ✓ / exec ✓不命中 / resume / ccr / 空串）。
- Good: doctor 解析改动时用真实 schema 样例（`checks.<id>.details` 嵌套）断言 highlights。
- Base: 分类器为纯函数，可脱离真实进程表测试；highlights 为纯函数，可脱离真实 codex 测试。
- Bad: 按 `Process::name()` 宽匹配 `contains("codex")`——会误杀 `codex exec`/`resume`，且拿不到 `app-server` 判据。
- Bad: 用 doctor 退出码判定成功——检查项失败（overallStatus=fail）时会丢弃有效报告。
- Bad: 假设 `details` 在 JSON 顶层——真实 schema 嵌套在 `checks.<id>` 下，会导致 highlights 恒空。

### 6. Tests Required

- `cargo test -p ccr-codex codex_process_service -- --test-threads=1`（分类器正反用例）。
- `cargo test -p ccr-cli --lib fix -- --test-threads=1`（highlights 提取 / value_to_display）。
- `cargo test -p ccr-cli --lib codex_fix -- --test-threads=1`（`ccr codex fix` / `--dry-run` 解析）。
- `just lint-strict`（跨平台编译 + 无 unwrap/panic 门）。
- Assertion points: app-server 命中且 exec/resume/ccr 不命中；highlights 命中 CODEX_HOME/config.toml/auth file/stored auth mode/model provider 且忽略 cwd/log dir；dry-run 解析出 `dry_run == true`。

### 7. Wrong vs Correct

#### Wrong

```rust
// 按进程名宽匹配：会误杀 `codex exec` / `codex resume`，且无法定位 app-server
let name = process.name().to_string_lossy().to_lowercase();
name.contains("codex") && !name.contains("ccr")
```

#### Correct

```rust
// 按命令行窄匹配 app-server：exec/resume/plain codex 均不命中
let cmdline = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
is_codex_app_server(&cmdline.to_lowercase()) // contains("codex") && contains("app-server") && !contains("ccr")
```
