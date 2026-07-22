# Codex App-Server Cleanup And Runtime Diagnosis

> Contracts for `ccr codex fix`: narrow app-server cleanup, read-only CCR/profile/runtime reconciliation, explicit runtime repair, and supplemental `codex doctor` evidence.

## Scenario: app-server cleanup + local runtime diagnosis

### 1. Scope / Trigger

- Trigger: changing `CodexProcessService`, the `ccr codex fix` command, or its `codex doctor` invocation/rendering.
- Purpose: kill残留 Codex `app-server` 进程（SSH / Desktop / VS Code Remote 断开后锁定旧登录态），对调用瞬间的 CCR profile 与 Codex runtime 做只读一致性诊断，再运行 `codex doctor` 补充上游证据。
- Cross-platform: Unix + Windows single build (no `#[cfg]` process code; sysinfo abstracts signals).

### 2. Signatures

- Service: `CodexProcessService::new() -> Self`（无状态，infallible）。
- Service: `CodexProcessService::find_app_servers() -> Vec<CodexAppServer>`。
- Service: `CodexProcessService::cleanup(dry_run: bool) -> CodexAppServerCleanup`。
- Types: `CodexAppServer { pid: u32, cmdline: String }`；`TerminationKind { Term, Kill, AlreadyGone }`；`CodexAppServerCleanup { found, terminated: Vec<(u32, TerminationKind)>, respawned, dry_run }`。
- Domain: `CodexPlatform::inspect_runtime() -> Result<CodexRuntimeDiagnostic>`（只读，不调用会 reconcile pointer 的 `stable_current_profile()`）。
- Domain: `CodexPlatform::repair_runtime(snapshot: &CodexRuntimeDiagnostic) -> Result<()>`（仅重放快照解析出的当前 profile）。
- Diagnostic: `CodexRuntimeDiagnostic::{profile_status, route_status, credential_status, provider_auth_validity, repairable}`；状态值为 `match|missing|mismatch|not_applicable|unsupported`，Provider 有效性当前固定为 `not_checked`。
- CLI: `ccr codex fix [--dry-run] [--repair-runtime]`（clap 派生，自动进入 `ccr codex help`）。

### 3. Contracts

- **窄匹配（防误杀，最高优先）**：进程命中当且仅当其**命令行**（`sysinfo::Process::cmd()` join 后小写）同时包含 `"codex"` 与 `"app-server"`，且不包含 `"ccr"`。
  - `codex` / `codex exec` / `codex resume` / `codex login` 因不含 `"app-server"` **绝不命中**。
  - `ccr ...` 进程因含 `"ccr"` **绝不命中**（含 `ccr codex fix` 自身）。
  - `cmd()` 为空/不可读的进程一律**跳过**（宁漏杀不误杀）。含 `"ccr"` 路径的真实 app-server 会被漏杀——漏杀安全、误杀危险，接受此代价。
- **跨平台终止升级**：Unix 先 `kill_with(Signal::Term)`（SIGTERM），轮询 ~3s（10 × 300ms）等待退出，仍存活者 `kill()`（SIGKILL）。Windows 上 `kill_with(Signal::Term)` 返回 `None`（信号不支持）→ 直接 `kill()`（terminate）。
- **不做显式 user 过滤**：非特权用户在 OS 层只能向自己的进程发信号（他人进程 kill 失败），窄匹配已足够；按 PID 记录终止手段。
- **respawn 复检**：非 dry-run 终止后 `sleep ~1s` 重新 `find_app_servers()`；仍存活者填入 `respawned`（客户端重新拉起或 kill 权限失败）。
- **只读快照**：先原样读取 registry pointer、`profiles.toml.current_config`、实际 `config.toml` / `auth.json` 与当前进程环境；pointer 不一致时保留证据并拒绝猜测 profile。profile secret 只在内存中参与相等比较，不进入诊断结构、Debug、序列化或日志。
- **route / credential 分层**：route 比较必须复用 `build_switch_spec` 的 auth mode、route 与 credential-store 规则。`openai_api_key + file` 比较 profile secret 与 `auth.json.OPENAI_API_KEY`；`provider_env_key` 同时采样实际 runtime 和目标 profile 声明的 env key；keyring/auto 标为 `unsupported`；`no_auth` 标为 `not_applicable`。
- **Provider 结论**：本地 `match` 只证明 profile/runtime 一致，`provider_auth_validity` 始终为 `not_checked`，不得复用结构性 `AuthStateStatus::Valid` 或宣称第三方已接受 key。
- **默认不修 runtime**：裸 `ccr codex fix` 可清理进程，但不得重写 `config.toml` / `auth.json`。只有 `--repair-runtime` 且快照 `repairable=true` 时，才通过既有 `apply_profile` 原子提交路径重放当前 profile，并再次 inspection 证明结果。
- **dry_run**：只枚举 `found`，`terminated` 为空，不发任何信号，不做 respawn 复检；与 `--repair-runtime` 组合时只预览重放，不写 runtime，也不落盘 doctor 临时报告。
- **doctor 调用**：`codex doctor --json`，以「stdout 是否为有效 JSON」判成功（**非退出码**——检查项失败时 doctor 可能返回非 0，但 stdout 仍是有效报告，须照常展示）。旧版无 `--json` → 回退 `codex doctor` 纯文本。外部调用 30s 超时，`stdin` 置 null。
- **doctor 快照归属**：doctor 标题必须携带开始前的 resolved profile；doctor 后再次 inspection。任一 profile/runtime 字段变化时，输出不得归属于旧快照，并以 local-drift 退出码结束。
- **doctor schema v1**：`{ codexVersion, overallStatus, checks: { <id>: { details: { <key>: <value> } } } }`。`details` **嵌套在 `checks.<id>` 之下，非顶层**。高亮 = 顶层 `codexVersion`/`overallStatus` + 遍历 `checks.*.details` 中键名（小写）包含 `codex_home|config|provider|auth|base_url|endpoint|model` 的字段。
- **报告落盘**：非 dry-run 时将 CCR 再脱敏后的完整 doctor stdout 写入 `temp_dir()/codex-doctor-<user>-<ts>.<json|txt>`，打印路径。
- **安全**：`OPENAI_API_KEY`、`CODEX_API_KEY` 与 provider env key 只显「已设置/未设置」；doctor 中敏感 label 的值一律替换为 `<redacted>`，URL userinfo/query 在渲染和保存前移除；绝不打印 `auth.json` 原文、secret 片段、长度、哈希或 fingerprint，也不 log token。

### 4. Validation & Error Matrix

- `codex` 不在 PATH（`which_on_path("codex") == None`）→ 中文 error + `std::process::exit(127)`。
- `cleanup.respawned` 非空 → `std::process::exit(2)`（在 doctor 之后判定，对齐脚本先警告后退出码）。
- profile/route/credential 存在 `missing|mismatch` 且未修复、修复后仍漂移，或 doctor 期间快照变化 → `std::process::exit(3)`。
- 固定优先级：PATH missing `127` > app-server respawn `2` > local drift `3` > success `0`。
- doctor 超时 / 无法 spawn → 记为诊断失败并 warning，**不 panic**；若无 respawn/local drift，退出码 0。
- doctor `--json` stdout 非有效 JSON → 回退纯文本；纯文本也失败 → warning；若无 respawn/local drift，退出码 0。
- 报告写盘失败 → `report_path = None`，不影响主流程。
- pointer 冲突、profile/secret 缺失、provider env 缺失或不可读 keyring/auto → `repairable=false`，`--repair-runtime` 不猜测、不写文件。
- 退出码用 `std::process::exit`（先 `io::stdout().flush()`），沿用 `doctor_cmd` 先例。
- 不新增 `CcrError` 变体（变体冻结）。

### 5. Good/Base/Bad Cases

- Good: 改分类逻辑时补 `is_codex_app_server` 正反用例（app-server ✓ / exec ✓不命中 / resume / ccr / 空串）。
- Good: doctor 解析改动时用真实 schema 样例（`checks.<id>.details` 嵌套）断言 highlights。
- Good: route 漂移时同时显示实际 runtime env key 与目标 profile env key 的存在性，值全部隐藏。
- Good: `--dry-run --repair-runtime` 对 registry、profiles、secret store、config 与 auth 做 byte-for-byte 不变断言。
- Base: 分类器为纯函数，可脱离真实进程表测试；highlights 为纯函数，可脱离真实 codex 测试。
- Base: 无当前 profile 时报告 runtime-only 状态，不把某个 pointer 猜成事实。
- Bad: 按 `Process::name()` 宽匹配 `contains("codex")`——会误杀 `codex exec`/`resume`，且拿不到 `app-server` 判据。
- Bad: 用 doctor 退出码判定成功——检查项失败（overallStatus=fail）时会丢弃有效报告。
- Bad: 假设 `details` 在 JSON 顶层——真实 schema 嵌套在 `checks.<id>` 下，会导致 highlights 恒空。
- Bad: 看到 `auth.json` 有非空 key 就输出 Provider 凭据“valid”；这只能判定本地字段存在。
- Bad: 诊断前调用 `stable_current_profile()`；route mismatch 时它会清掉 registry pointer，破坏要报告的现场证据。

### 6. Tests Required

- `cargo test -p ccr-codex codex_process_service -- --test-threads=1`（分类器正反用例）。
- `cargo test -p ccr-codex runtime_diagnostic -- --test-threads=1`（pointer、route、file secret、provider env、keyring、no-auth、无当前 profile、修复后二次验证与 secret-free Debug/JSON）。
- `cargo test -p ccr-cli --lib fix -- --test-threads=1`（highlights 提取 / value_to_display）。
- `cargo test -p ccr-cli --lib codex_fix -- --test-threads=1`（`ccr codex fix` / `--dry-run` / `--repair-runtime` 解析）。
- `cargo test -p ccr --test commands codex_fix -- --test-threads=1`（binary dry-run 无写入、退出码和无 secret 输出）。
- `just lint-strict`（跨平台编译 + 无 unwrap/panic 门）。
- Assertion points: app-server 命中且 exec/resume/ccr 不命中；highlights 命中 CODEX_HOME/config.toml/auth file/stored auth mode/model provider 且忽略 cwd/log dir；doctor URL/query 与敏感字段已脱敏；dry-run 解析出 `dry_run == true` 且受管文件字节不变；local drift=3、respawn=2、PATH missing=127。

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

#### Wrong

```rust
// 会在 route mismatch 时清掉 registry pointer，导致诊断丢失现场证据
let current = platform.get_current_profile()?;
```

#### Correct

```rust
// 只读采样 raw pointers；只有显式 repair 才重放当前 profile
let before = platform.inspect_runtime()?;
if repair_runtime && before.repairable {
    platform.repair_runtime(&before)?;
    let after = platform.inspect_runtime()?;
}
```
