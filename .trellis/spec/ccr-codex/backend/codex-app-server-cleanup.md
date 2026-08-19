# Codex App-Server Cleanup And Runtime Diagnosis

> Contracts for `ccr codex fix`: narrow app-server cleanup, read-only CCR/profile/runtime reconciliation, explicit runtime repair, and supplemental `codex doctor` evidence.

## Scenario: app-server cleanup + local runtime diagnosis

### 1. Scope / Trigger

- Trigger: changing `CodexProcessService`, the `ccr codex fix` command, or its `codex doctor` invocation/rendering.
- Purpose: kill残留 Codex `app-server` 进程（SSH / Desktop / VS Code Remote 断开后锁定旧登录态），对调用瞬间的 CCR profile 与 Codex runtime 做只读一致性诊断。默认不调用上游 `codex doctor`；`--doctor` 才补充上游证据。
- Cross-platform: Unix + Windows single build (no `#[cfg]` process code; sysinfo abstracts signals).

### 2. Signatures

- Service: `CodexProcessService::new() -> Self`（无状态，infallible）。
- Service: `CodexProcessService::find_app_servers() -> Vec<CodexAppServer>`。
- Service: `CodexProcessService::cleanup(dry_run: bool) -> CodexAppServerCleanup`。
- Service: `CodexProcessService::cleanup_report(dry_run: bool) -> CodexAppServerCleanupReport`；CLI 必须使用此入口，旧 `cleanup()` 仅保留兼容投影。
- Types: `CodexAppServer { pid: u32, cmdline: String }`（`cmdline` 是脱敏摘要，不是原始 argv）；`TerminationKind { Term, Kill, AlreadyGone }`；`CodexAppServerCleanup { found, terminated: Vec<(u32, TerminationKind)>, respawned, dry_run }`。
- Detailed types: `CodexAppServerCleanupReport { cleanup, discovered_during_cleanup, signal_failures, discovery_issue }`；`CodexSignalFailure { pid, stage }`；`CodexProcessDiscoveryIssue::{CurrentProcessUnavailable, CurrentOwnerUnavailable, CommandLineUnavailable}`。
- Domain: `CodexPlatform::inspect_runtime() -> Result<CodexRuntimeDiagnostic>`（只读，不调用会 reconcile pointer 的 `stable_current_profile()`）。
- Domain: `CodexPlatform::repair_runtime(snapshot: &CodexRuntimeDiagnostic) -> Result<()>`（仅重放快照解析出的当前 profile）。
- Diagnostic: `CodexRuntimeDiagnostic::{profile_status, route_status, credential_status, provider_auth_validity, repairable}`；状态值为 `match|missing|mismatch|not_applicable|unsupported`，Provider 有效性当前固定为 `not_checked`。
- CLI: `ccr codex fix [--dry-run] [--repair-runtime] [--doctor]`（clap 派生，自动进入 `ccr codex help`）。`--repair-runtime` 不隐含 `--doctor`。

### 3. Contracts

- **显式进程快照**：只能用 `refresh_processes_specifics(..., ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always).with_user(UpdateKind::Always).without_tasks())`。`System::new()` 后调用默认 `refresh_processes(...)` 不会加载 `cmd()`，会让清理静默退化为 no-op；不得用 `System::new_all()` 隐藏字段依赖或读取无关数据。
- **owner 边界与 fail closed**：匹配只接受与当前 CCR 进程 effective/user ID 相同的进程；每次信号前重新刷新并验证 owner。当前 PID、owner 或当前命令行不可用时设置 `discovery_issue`、停止发送信号，并渲染 `process_state = unavailable`，绝不能投影为 `clean`。
- **argv 窄匹配（防误杀，最高优先）**：argv[0] 必须是 basename `codex` / `codex.exe`，或 argv[0] 是 `node` 且 argv[1] 是 Codex launcher；launcher 之后必须存在精确参数 `app-server`。
  - `codex` / `codex exec` / `codex resume` / `codex login` **绝不命中**。
  - `python tool.py codex app-server`、`ccr codex fix` 等仅在参数中出现关键字的进程**绝不命中**。
  - 安装路径包含 `ccr` 不构成排除条件；自排除使用当前 PID。展示只用 `codex app-server` / `node codex app-server` 摘要，原始 argv 不进入公开结果或日志。
- **进程身份**：内部身份必须是 `pid + start_time`。每次信号前同时重新验证身份、owner 与 argv；相同 PID 的新 `start_time` 必须按新进程重新分类，不能沿用旧目标身份。
- **动态终止升级**：Unix 对初始目标发送 TERM，然后每 300ms 全量重查，最多约 3s。当前匹配目标为空则立即结束宽限循环，不再继续 `wait` / `discover`。窗口内新出现的身份写入 `discovered_during_cleanup`。截止时若仍有匹配身份，对当时身份发送 KILL，不限于初始 PID。Windows 上 TERM 为 `None` 时立即使用 `kill()`，但仍执行截止点复检。只要本轮发送过信号，仍等待约 1s settle 再拍最终快照。dry-run 与初始快照为空：只做一次 discover，零等待。
- **信号事实**：`kill_with(Term)` 仅 `Some(true)` 记为已发送；`Some(false)` 进入 TERM failure；`None` 触发 KILL fallback；`kill() == false` 进入 KILL failure。只有成功信号对应的 `pid + start_time` 在最终全进程身份快照中确认消失，才记录 `Term` / `Kill`；仍存活但不再匹配的身份不记录为已终止。
- **最终复检**：settle 后再读取“当前匹配目标 + 全部存活身份”。仍匹配者填入 `cleanup.respawned`。空快照结束宽限之后、仅在 settle 出现的新 `pid+start_time` 一律进入 `respawned`，不在已结束的宽限里补发 deadline KILL；命令以退出码 2 结束。新身份和无法终止的旧身份都属于进程阶段不干净。公开旧结果按 PID 投影，安全判断始终使用内部完整身份。
- **只读快照**：先原样读取 registry pointer、`profiles.toml.current_config`、实际 `config.toml` / `auth.json` 与当前进程环境；pointer 不一致时保留证据并拒绝猜测 profile。profile secret 只在内存中参与相等比较，不进入诊断结构、Debug、序列化或日志。
- **route / credential 分层**：route 比较必须复用 `build_switch_spec` 的 auth mode、route 与 credential-store 规则。`openai_api_key + file` 比较 profile secret 与 `auth.json.OPENAI_API_KEY`；`provider_env_key` 同时采样实际 runtime 和目标 profile 声明的 env key；keyring/auto 标为 `unsupported`；`no_auth` 标为 `not_applicable`。
- **Provider 结论**：本地 `match` 只证明 profile/runtime 一致，`provider_auth_validity` 始终为 `not_checked`，不得复用结构性 `AuthStateStatus::Valid` 或宣称第三方已接受 key。
- **默认不修 runtime**：裸 `ccr codex fix` 可清理进程，但不得重写 `config.toml` / `auth.json`。只有 `--repair-runtime` 且快照 `repairable=true` 时，才通过既有 `apply_profile` 原子提交路径重放当前 profile，并再次 inspection 证明结果。
- **dry_run**：只枚举 `found`，`terminated` 为空，不发任何信号，不做 respawn 复检；与 `--repair-runtime` 组合时只预览重放，不写 runtime。与 `--doctor` 组合时仍运行 doctor，但不落盘 doctor 临时报告。
- **阶段隔离**：process cleanup、CCR runtime inspection/repair、环境提示和可选 doctor 分别保存结果，不得用 `?` 让 runtime 错误跳过后续独立阶段。阶段错误只输出稳定状态和动作边界，不打印可能含 secret 的底层错误字符串。默认路径不查找 PATH 中的 `codex`，不启动该二进制，不做 doctor 后 inspection。
- **doctor 调用**：仅 `--doctor` 才运行。`codex doctor --json`，以「stdout 是否为有效 JSON」判成功（**非退出码**——检查项失败时 doctor 可能返回非 0，但 stdout 仍是有效报告，须照常展示）。stdout 已有但不是 JSON → `sanitize_doctor_text` 当文本渲染，**不得**再 spawn 第二次。必须使用 `ccr_core::core::process_gateway::ManagedProcess`：`spawn` → 并发排空受限 stdout/stderr → 正常路径 `wait`；超时路径 `terminate_tree(grace)` 并 await reap。禁止只靠 `kill_on_drop(true)` 宣称无残留。外部调用 30s 超时（实现须提供可注入的 Duration seam），`stdin` 置 null。
- **doctor 快照归属**：doctor 标题必须携带开始前的 resolved profile；doctor 后再次 inspection。任一 profile/runtime 字段变化时，输出不得归属于旧快照，并以 local-drift 退出码结束。
- **doctor schema v1**：`{ codexVersion, overallStatus, checks: { <id>: { status, details: { <key>: <value> } } } }`。`details` **嵌套在 `checks.<id>` 之下，非顶层**。高亮 = 顶层 `codexVersion`/`overallStatus` + `checks` 里 `status != "ok"` 的 `id` + `status`。不得用键名子串扫描 `details`。
- **报告落盘**：仅非 dry-run 的 `--doctor` 将 CCR 再脱敏后的完整 doctor stdout 写入 `temp_dir()/codex-doctor-<user>-<ts>.<json|txt>`，打印路径。
- **安全**：`OPENAI_API_KEY`、`CODEX_API_KEY` 与 provider env key 只显「已设置/未设置」；doctor 中敏感 label 的值一律替换为 `<redacted>`，URL userinfo/query 在渲染和保存前移除；绝不打印 `auth.json` 原文、secret 片段、长度、哈希或 fingerprint，也不 log token。

### 4. Validation & Error Matrix

- 传入 `--doctor` 且 `codex` 不在 PATH（`which_on_path("codex") == None`）→ 中文 error + `std::process::exit(127)`。默认路径缺少 `codex` 不退出 127。
- `cleanup.respawned` 非空或 `discovery_issue` 非空 → `std::process::exit(2)`（在可选 doctor 之后判定）。
- runtime initialization / inspection / repair / repair verification / doctor 后 inspection 失败 → 输出 `runtime_consistency = unavailable`，继续可用独立阶段，最终 `std::process::exit(1)`。
- profile/route/credential 存在 `missing|mismatch` 且未修复、修复后仍漂移，或 doctor 期间快照变化 → `std::process::exit(3)`。
- 固定优先级：PATH missing `127`（仅 `--doctor`）> process remaining/unavailable `2` > runtime failure `1` > local drift `3` > success `0`。
- doctor 超时 / 无法 spawn → 记为诊断失败并 warning，**不 panic**；若无 respawn/local drift，退出码 0。超时后本次拉起的父进程与其孙进程均不得留在进程表。
- doctor `--json` stdout 非有效 JSON → 用已有 stdout 走文本路径，不得二次 spawn；若无 respawn/local drift，退出码 0。
- 报告写盘失败 → `report_path = None`，不影响主流程。
- pointer 冲突、profile/secret 缺失、provider env 缺失或不可读 keyring/auto → `repairable=false`，`--repair-runtime` 不猜测、不写文件。
- 退出码用 `std::process::exit`（先 `io::stdout().flush()`），沿用 `doctor_cmd` 先例。
- 不新增 `CcrError` 变体（变体冻结）。

### 5. Good/Base/Bad Cases

- Good: 改分类逻辑时用 `Vec<OsString>` 补 native、node wrapper、含 `ccr` 的 launcher 路径，以及 plain/exec/resume/login/任意工具参数反例。
- Good: 状态机测试使用 fake backend 编排 snapshot 与信号返回，分别断言 TERM exit、deadline KILL、新 PID、respawn、PID reuse、`Some(false)` / `kill=false` 和 discovery issue。
- Good: Unix 真实 fixture 启动同用户伪 app-server，并通过 `cleanup_report(true)` 证明生产 refresh 链路能看到 PID 且输出不含 sentinel argv。
- Good: doctor 解析改动时用真实 schema 样例断言 highlights：顶层 version/status + 非 ok 检查 id；ok 检查的 details 不得进入高亮。
- Good: route 漂移时同时显示实际 runtime env key 与目标 profile env key 的存在性，值全部隐藏。
- Good: `--dry-run --repair-runtime` 对 registry、profiles、secret store、config 与 auth 做 byte-for-byte 不变断言。
- Base: 分类器为纯函数，可脱离真实进程表测试；highlights 为纯函数，可脱离真实 codex 测试。
- Base: 无当前 profile 时报告 runtime-only 状态，不把某个 pointer 猜成事实。
- Bad: 按 `Process::name()` 宽匹配 `contains("codex")`——会误杀 `codex exec`/`resume`，且拿不到 `app-server` 判据。
- Bad: 只跟踪初始 PID 或只按 PID 判断存活；客户端换 PID 和 PID reuse 会分别导致漏杀与误杀风险。
- Bad: 只用最终“仍匹配目标”推断原身份已退出；argv 改变但身份仍活着时会伪造 Term/Kill 成功。
- Bad: 用 doctor 退出码判定成功——检查项失败（overallStatus=fail）时会丢弃有效报告。
- Bad: 用键名子串扫描 `checks.*.details`——会把 `search provider` / `configured servers` / 重复的 `model provider` 一并抽出。
- Bad: `--json` 非 JSON 后再 spawn 一次纯文本 doctor。
- Bad: 只用 `kill_on_drop(true)` 回收 doctor——Windows 上 `.cmd → node` 孙进程不会随 drop 退出。
- Bad: 看到 `auth.json` 有非空 key 就输出 Provider 凭据“valid”；这只能判定本地字段存在。
- Bad: 诊断前调用 `stable_current_profile()`；route mismatch 时它会清掉 registry pointer，破坏要报告的现场证据。

### 6. Tests Required

- `cargo test -p ccr-codex codex_process_service -- --test-threads=1`（显式 refresh 真实 fixture、argv 分类矩阵、动态身份、signal bool、PID reuse / settle respawned、空快照早停、走满 poll_rounds 再 KILL、owner/discovery fail closed）。
- `cargo test -p ccr-codex runtime_diagnostic -- --test-threads=1`（pointer、route、file secret、provider env、keyring、no-auth、无当前 profile、修复后二次验证与 secret-free Debug/JSON）。
- `cargo test -p ccr-cli --lib fix -- --test-threads=1`（highlights、process state、退出优先级、ManagedProcess 超时回收、非 JSON 只 spawn 一次）。
- `cargo test -p ccr-cli --lib codex_fix -- --test-threads=1`（`ccr codex fix` / `--dry-run` / `--repair-runtime` / `--doctor` 解析；`--repair-runtime` 不隐含 `--doctor`）。
- `cargo test -p ccr --test commands codex_fix -- --test-threads=1`（binary dry-run 无写入、默认路径不启动 fake `codex`、`--doctor` JSON/文本/落盘、runtime inspection 失败仍可执行 `--doctor`、退出码和无 secret 输出）。
- `just lint-strict`（跨平台编译 + 无 unwrap/panic 门）。
- Assertion points: 真实 dry-run 发现 fixture PID；原始 argv sentinel 不出现在结果；app-server 命中且 exec/resume/login/任意工具不命中；失败信号不进入 `terminated`；最终存活身份不会伪报终止；dry-run 受管文件字节不变；runtime failure=1、remaining/unavailable=2、local drift=3、`--doctor` 且 PATH missing=127。

### 7. Wrong vs Correct

#### Wrong

```rust
// sysinfo 的便捷 refresh 不加载 cmd/user；matcher 将永远看到空 cmdline
let mut system = System::new();
system.refresh_processes(ProcessesToUpdate::All, true);
```

#### Correct

```rust
let refresh = ProcessRefreshKind::nothing()
    .with_cmd(UpdateKind::Always)
    .with_user(UpdateKind::Always)
    .without_tasks();
system.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh);
is_codex_app_server(process.cmd()); // argv-aware, owner-scoped by the caller
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
