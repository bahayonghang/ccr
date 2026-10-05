# P2 历史 CI 复核证据（diagnosis only）

日期：2026-09-06。执行角色：Trellis implement（Cursor Grok 4.6）。未改产品代码，未安装工具，未 `gh run rerun`，未提交。独立审查：trellis-check PASS。

当前工作树：`dev` / `cf4aa21de5fdee2460f59c7ec84248ed186d46b7`（ahead origin/dev 8）。本机：Windows 10.0.26200 AMD64，`rustc`/`cargo` 1.98.0，Bun 1.4.2，TEMP=`C:\Users\lyh\AppData\Local\Temp\`。F1 route smoke 已在归档子任务修复；本项只诊断历史 Root/Tauri/Frontend 失败。

**总则：当前测试通过 ≠ 历史根因已证明，不得记“已修复”。**

## 命令账本

| # | cwd | 命令 | exit | 时长 | cold/warm | 结论 |
|---|---|---|---|---|---|---|
| 1 | `D:\Documents\Code\Github\ccr` | `cargo test -p ccr-cli --all-features --locked --offline non_dry_run_doctor_persists_sanitized_report` | 0 | 18.97s 墙钟；compile 18.37s；测试体 0.03s | crate compile cold（本会话重编 `ccr-store`/`ccr-cli`）；测试运行 warm | 1 passed / 334 filtered。**未复现**（本机 Windows、非 llvm-cov、非 Ubuntu） |
| 2a | 同上 | `just tauri-process-smoke` attempt 1 | 0 | 29.96s；gateway 10 tests 0.66s；ccr-core 5 tests 1.38s | 本会话第一次 process smoke；Tauri 测试二进制 compile ~20.84s | 10+5 passed。stdin/flood **未复现** |
| 2b | 同上 | `just tauri-process-smoke` attempt 2 | 0 | 4.04s；gateway 0.61s；ccr-core 1.45s | warm | 未复现。按设计继续第 3 次 |
| 2c | 同上 | `just tauri-process-smoke` attempt 3 | 0 | 4.08s；gateway 0.64s；ccr-core 1.42s | warm | 未复现。三次上限，停止 |
| 3 | 同上 | `cargo llvm-cov --version`（探测，未安装） | 101 | <3s | n/a | `error: no such command: llvm-cov`。**未跑** `just coverage-rust`。该 gate **UNVERIFIED**，不得标通过 |
| 4 | `D:\Documents\Code\Github\ccr\ccr-ui` | `bun run test:smoke --coverage` | 0 | 116.41s 墙钟；vitest 114.27s | `@vitest/coverage-v8@4.1.11` 已在 node_modules，无新安装 | 151 files / 722 tests passed；lines 72.17%（阈值 70%）。这是 **当前 React 门**，不是历史 Vue teardown 的复现 |
| R | 同上仓库 | `gh run view {32684239641,32684151306,32684151228} --log-failed` | 0 | 2.8s / 4.3s / 3.0s | n/a | 只读。原文：`research/gh-run-*-log-failed.txt`。无 rerun/cancel |

## 1. Root 32684239641 — doctor 报告未持久化

### 历史

- SHA：`fac5611f5587dcbe842ef8c133072a26f7ef7be5`（PR `chore(deps): bump thiserror 2.0.18→2.0.20`）
- 工作流：Root CI，job **Root Coverage 70-85**，`ubuntu-24.04`，runner image 20260816.277.1
- 原命令：`just coverage-rust` → `cargo llvm-cov --workspace --all-features --json --output-path target/coverage-workspace.json`
- 环境：`rust-toolchain.toml` 当时 pin **1.95.0**（日志：`rustup default 1.95.0`，`cfg(coverage)`）；`CARGO_INCREMENTAL=0`，`RUSTFLAGS=-D warnings`。同 SHA 的 **Root Tests (windows-2025)/(macos-15) 成功**；Ubuntu 普通 `cargo test` 不在矩阵里，Linux 单测只出现在 coverage job
- 首故障：`commands::codex::fix::tests::non_dry_run_doctor_persists_sanitized_report` FAILED。`fix.rs:1112` `report should be persisted`。`332 passed; 1 failed`，2.26s。llvm-cov 包装的 `cargo test --workspace --all-features` exit 101。未生成 `target/coverage-workspace.json`

同文件当时与现在一样：`save_report` 用 `std::fs::write(...).ok()` 吞掉 IO 错误；`persist_report=true` 且 JSON 解析成功时仍设 `failed: false`。测试只 `expect` 路径，不读 `failed`/`note`，因此 **spawn 失败与写盘失败在断言上无法区分**。该测试体 5s timeout；整个 lib 套件 2.26s 结束，故 **不是 5s doctor timeout**。

### 当前对应命令

- 本机：账本 #1，exit 0，Windows / rustc 1.98.0 / **无** llvm-cov
- 当前测试行：`crates/ccr-cli/src/commands/codex/fix.rs:1151`（断言 `:1158`）；`save_report` 仍在 `:679` 吞 IO；`:576` 仍 `failed: false`
- `just coverage-rust`：**UNVERIFIED**（缺 `cargo-llvm-cov`，禁止安装）

### 分类

**not reproduced**（本机 focused `cargo test`）。**不是 vanished path。不得记 fixed。**

高可信但未证实的假设（按可证伪性）：

1. Hosted Linux + llvm-cov 下 `save_report` 写 `temp_dir` 失败（权限/EMFILE/只读等）→ `None`。OS 错误码 **UNVERIFIED**（日志无 `io::Error`）
2. Fake doctor spawn 在 coverage 下快速失败 → `failed_with` 且 `report_path=None`，断言仍是同一句
3. rustc 1.95 + `cfg(coverage)` 与当前 1.98 普通测试不是同一运行时。本机通过不能外推

### 可证伪下一步（若要动产品须另批）

不要调高 timeout、不要串行化整个 workspace、不要降 coverage 阈值。

1. 在 **已有** llvm-cov 的 Linux 环境跑 `just coverage-rust`（当前 HEAD），同时跑 **不带** llvm-cov 的 focused doctor 测试。仅 coverage 失败 → 仪器化/FD；两者都失败 → Linux IO/temp
2. 测试诊断（仍属产品/测试 diff，需批准）：断言失败时打印 `outcome.failed`、`outcome.note`、`std::env::temp_dir()`，区分 spawn vs write
3. 负例：把 TEMP 指到只读目录，期望当前代码 `report_path=None` 且 `failed=false`。若该负例成立，才证明“吞 IO”足以制造历史断言

### 候选产品方案（未实施，本 P2 **不建议立刻落地**）

历史 OS 根因未证明。若另批语义修复，最小方向：

- `save_report` 改为 `Result<PathBuf, io::Error>`（或保留路径 + 错误），写失败进入 `DoctorOutcome.note`，必要时 `failed=true`
- 文件名用更高分辨率（nanos/pid），避免秒级碰撞（本失败不是强碰撞证据：仅一处 `persist_report=true` 测试）
- focused 测试：`failed==false` **且** 文件存在；另加不可写 TEMP 负例
- 必跑：账本 #1；有工具时再 `just coverage-rust`。禁止用串行/`--test-threads=1` 或放宽 70/85 冒充修复

## 2. Tauri 32684151306 — Windows process smoke 5s

### 历史

- SHA：`8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455`（PR `async-trait` bump in `ccr-ui/src-tauri`）
- 工作流：Tauri Rust CI，job **Tauri Smoke (windows-2025)**。同 run：**macos-15 smoke、Linux validation、gateway coverage 均成功**
- 原命令：`just tauri-process-smoke` → `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml process::gateway`（随后才是 `cargo test -p ccr-core core::process_gateway`；失败发生在第一段）
- 环境：pwsh，`CARGO_INCREMENTAL=0`。先 `cargo install just`（release，~2m32s），再冷编 Tauri 测试
- 首故障（并行，墙钟 **5.04s**，与 descriptor **5s** 对齐）：
  - `foreground_stdin_is_written_and_streams_are_collected`：`assert!(output.status.success())` 失败（`gateway.rs:941`）
  - `foreground_output_flood_is_capped_and_terminated`：`assert!(output.stdout_truncated)` 失败（`:969`）
  - `8 passed; 2 failed; 480 filtered`
- 日志 **没有** `timed_out`/`duration`/`stdout.len`。两测试均为 `powershell.exe` + `Duration::from_secs(5)`。`execute_command` 超时会 `timed_out=true` 并 `terminate_tree`，成功位通常为 false；flood 若在写满 64KiB 前被 5s 杀掉，则 `stdout_truncated` 可为 false

当前测试仍在 `gateway.rs:909` / `:953`，仍是 powershell + 5s，未改 timeout。

### 当前对应命令

账本 #2a–2c：三次原参数，**均 exit 0**。gateway 10 passed（当前 505 filtered，套件变大）；核心 5 passed。本机 gateway 墙钟 0.61–0.66s，远低于 5s。attempt 1 相对本会话为 cold compile，仍未打到 5s 上限。

### 分类

**not reproduced**（本机 Windows ×3）。**不得记 fixed。** 冷启动/并发抢 powershell 仍是高可信假设：历史两失败同时收在 ~5.04s，macOS 同 SHA 通过。本机无法模拟 windows-2025 冷 runner。

### 可证伪下一步（若要动产品须另批）

禁止把 5s 调高、禁止串行化该 smoke、禁止吞 teardown。

1. 当前 HEAD 在 **hosted windows-2025** 跑原参数 `just tauri-process-smoke`（新 run，不是 rerun 32684151306）
2. 若再失败：仅给断言加上 `status`/`timed_out`/`duration`/`stdout.len()`（测试诊断，另批）。若 `timed_out==true` 且 flood `stdout.len()<<64KiB`，冷启动/超时假设成立
3. 反证：同 runner 上先 warm 一次 powershell 再跑；若仅 cold 失败，根因是环境而非 cap 语义

### 候选产品方案（未实施，**不建议**用放宽 timeout 当修复）

三次未复现，本 P2 不改 `gateway.rs`。若 hosted 复现且诊断确认 5s 内 powershell 未完成 stdin/flood：另批讨论用更轻的 helper 二进制，或把“进程 timeout”与“flood 必须在 timeout 内截断”拆开——那是测试设计，不是把 5s 改成 15s。

## 3. Frontend 32684151228 — Vue usage.store teardown

### 历史

- SHA：同 Tauri，`8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455`
- 工作流：Frontend CI，job **Vue and Docs Validation**，`ubuntu-24.04`，Bun 由 workflow 安装（当时前端仍是 Vue）
- 原命令：`just frontend-check` **先通过**（123 files / 626 tests）。随后 `just frontend-coverage` 失败
- 首故障：coverage 跑完 626 passed 后 **1 unhandled** `EnvironmentTeardownError: [vitest-worker]: Closing rpc while "onUserConsoleLog" was pending`，源文件 **`tests/usage.store.smoke.test.ts`**。vitest exit 1 → recipe `frontend-coverage` line 792。lines 当时 74.96%，**不是阈值失败**。artifact `vue-coverage` 仍上传了

该文件在 `8e262ce2852f1f79f4a192119813b771f0fd9fb8`（2026-08-23）随 Vue 工具链测试删除；处置清单：`.trellis/tasks/archive/2026-08/08-22-react-foundation/legacy-tests-disposition.md`（`usage.store`）。当前树 **无** `ccr-ui/tests/usage.store.smoke.test.ts`。

### 当前对应命令

账本 #4：`cd ccr-ui && bun run test:smoke --coverage`（`vitest.smoke.config.ts`，lines≥70）。exit 0；151/722；lines 72.17%。本机 Bun 1.4.2 vs 仓库 pin 1.4.0。F1 已不在此套件中失败。

### 分类

**vanished path（已删除的 Vue 测试）**。当前 React coverage 通过 **不能** 解释或关闭历史 teardown。禁止复活 Vue 文件，禁止为“消除 teardown”去吞 IO/worker 错误。

### 可证伪下一步

1. 不要再跑已删路径
2. 若 **当前** hosted `just frontend-coverage` / `bun run test:smoke --coverage` 再出现 `EnvironmentTeardownError`，当作新的 Vitest worker 问题，单独开任务；与 32684151228 的 Pinia/Vue store 不是同一故障
3. 本机通过 ≠ hosted Bun 1.4.0 通过（UNVERIFIED）

### 候选产品方案

**无。** 不改 `fix.rs`/`gateway.rs`，不改 coverage 阈值，不改 timeout。

## Ledger

| ID | Surface | Status | Classification |
|---|---|---|---|
| H1 | Root 32684239641 | open | not reproduced（本机 focused `cargo test`）。`just coverage-rust` UNVERIFIED。不得记 fixed |
| H2 | Tauri 32684151306 | open | not reproduced（本机 `just tauri-process-smoke` ×3）。hosted windows-2025 UNVERIFIED。不得记 fixed |
| H3 | Frontend 32684151228 | wontfix | vanished path：`tests/usage.store.smoke.test.ts` 已随 Vue 删除。当前 React coverage 通过 ≠ 已修复。未复活该文件 |

未重开 F4/F6。本项不改 `fix.rs` / `gateway.rs` / justfile / 阈值。

## UNVERIFIED

- Hosted Linux 上 doctor persist 的真实 `io::Error` / spawn 错误
- 本机或 hosted **`just coverage-rust` / cargo-llvm-cov**（工具缺失，未装）
- Linux/macOS 本机复现
- rustc 1.95 vs 当前 1.98 在 llvm-cov 下的差异
- Hosted windows-2025 冷 powershell（本机三次未打到 5s）
- **当前 HEAD 的 hosted CI**（禁止 rerun 历史 run；也未触发新 workflow）
- 真实桌面 WebView、五工具客户端、provider 联通
- 本机 Bun 1.4.2 vs CI Bun 1.4.0

## 产品修复建议（未应用）

**不建议在本 P2 落地任何产品 diff。** 证据不足以把历史失败写成已修复的代码缺陷；三次 Windows smoke 与 focused doctor 测试均为未复现。

仅在另批时考虑：Root 的 persist IO 可见性（见 §1 候选）。Tauri 不要加 timeout。Frontend 不要复活 Vue 测试。

## 工具与审查

- 适用工具：本会话 shell + 源码 + `gh run view --log-failed`（只读）
- 未把诊断交给无 shell 的 plan/explore；未安装 llvm-cov/just/bun 包
- 独立强模型审查：**PASS**（trellis-check；未改产品代码、未安装工具、未重跑 hosted）
- 父任务回写：适用工具=Codex/Claude/Cursor 强模型诊断；检查结果见上表；缺证据见 UNVERIFIED
