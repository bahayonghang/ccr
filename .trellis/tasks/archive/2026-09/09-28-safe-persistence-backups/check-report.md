# T05 独立检查报告

日期：2026-09-28。角色：`trellis-check`。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。环境：Windows；补充验证使用 WSL Ubuntu-24.04 / Rust 1.95.0。分支 `dev`。

## 结论

发现并修复 1 项局部兼容回归：Windows 新备份丢失源文件只读属性。修复后，本机备份、OAuth 存储及桌面适配器的定向行为检查通过。T02/T11 可以使用下述已验证接口继续实施。

本结论仅覆盖 T05 的定向契约。Linux guarded-write 24 项和 pending store 5 项已补充通过，包含对应 Unix mode 分支。完整 `ccr-core` 包级检查已有 Windows 锁测试失败，macOS 和完整根/Tauri 门禁未完成。不得将本报告表述为全包、全部平台或父任务验收通过。

## 检查范围与方法

- 读取完整保存的 hook 输出、`check.jsonl` 及其 spec/research、当前 PRD/design/implement、实施报告；未读取 `implement.jsonl`。
- 使用 `trellis-check`；出现测试运行器错误后使用 `ccr-gate-recovery` 做窄范围检查。
- 检查 guarded backup 的命名、碰撞、原权限/DACL、轮换、原子写入口；核对 `atomic_writer` 的改动仅为两个既有 DACL helper 的 `pub(super)` 可见性。
- 检查 profile-off 唯一操作目录、快照补偿、旧/新备份 reader。保留 T01 的 `clear_current_if_present` 协调修改。
- 检查 pending store 的 secret/no-backup 策略、操作锁与叶子写锁顺序、camelCase 磁盘兼容、默认脱敏、错误传播及 Tauri 原路径和 disk-first memory 发布。
- 本轮仅修改 `guarded_write.rs`、对应 core spec 和本报告。未修改其他代理文件、任务状态、用户配置；未提交或归档。

## Findings (fixed)

### P2：Windows 备份发布后丢失源文件只读属性

- File：`crates/ccr-core/src/core/guarded_write.rs:331-348`。
- Issue：`copy_backup_new` 在复制前设置源权限，然后调用 `NamedTempFile::persist_noclobber`。本地依赖 `tempfile 3.27.0` 的 Windows `persist` 会先调用 `SetFileAttributesW(..., FILE_ATTRIBUTE_NORMAL)`；因此 DACL 保留，但源文件的只读属性在发布时丢失。该行为与旧 `fs::copy` 保留文件权限及 AC2 的兼容目标不一致。
- Reproduction：新增 `test_backup_preserves_windows_readonly_attribute` 后、修复前运行单例，结果 `0 passed / 1 failed / exit 1`，断言为 `backup must preserve the source read-only attribute`。日志：`C:/Users/lyh/AppData/Local/rtk/tee/1790595577_cargo_test.log`。
- Fix：保存源权限快照，发布成功后在仍打开的文件句柄上恢复 Windows 只读属性；DACL 仍在任何 payload 字节写入前建立。权限恢复错误继续返回错误。未改变公开接口或备份命名。
- Regression：补充只读属性保留与只读源文件连续 16 次碰撞失败后的临时文件清理测试（同文件 `635`、`660`）。断言旧目标和旧备份内容不变，临时文件不残留。
- Spec：同步 `.trellis/spec/ccr-core/backend/atomic-writer.md` 的 Windows 发布行为及测试要求。
- Result：修复后的整个 `guarded_write` 过滤集 `25 passed / 79 filtered / exit 0`，包含原 DACL、三子进程、碰撞和轮换用例。

检查期间新增测试最初使用 `set_readonly(false)` 恢复权限，被 Clippy 拒绝。已改为恢复完整的原权限快照；未添加 lint 豁免，最终严格 Clippy 通过。

## Findings (not fixed)

### N1：完整 core 包级门禁仍有既有 Windows 锁测试失败

- 实施报告记录 `cargo test -p ccr-core -- --test-threads=1` 为 `101 passed / 1 failed / exit 1`，独立单例同样失败。失败用例为 `core::lock::tests::grok_auth_lock_preserves_native_holder_metadata`，`lock.rs:319` 在持文件范围锁时调用 `fs::read`，得到 OS 33。
- 本轮核对该源码，并执行 `git diff --exit-code -- crates/ccr-core/src/core/lock.rs`，结果为 0；T05 未修改该文件。本轮未重复整个包级检查，也未降低其断言。上面的 101/1 是实施阶段证据，不能替换为本轮执行数量。
- 归属：既有锁测试/平台契约，由主会话交 T10 处理。该失败使完整 core gate 保持未通过。

### N2：本轮 CLI 测试首次运行异常，具体原因未查明

- 命令：`cargo test -p ccr-cli profile_off -- --test-threads=1`。使用包默认 features；`ccr-cli` 声明 `default = []`。
- Cargo 已完成编译，测试进程 `D:/Documents/Code/Github/ccr/target/debug/deps/ccr_cli-442c371c21e1c1ca.exe profile_off --test-threads=1` 返回 `0xc0000135, STATUS_DLL_NOT_FOUND`；命令 exit 1，未得到有效测试计数。日志：`C:/Users/lyh/AppData/Local/rtk/tee/1790595720_cargo_test.log`。
- 窄查：对该路径运行 Visual Studio `dumpbin /DEPENDENTS`，只见 Windows/CRT 导入名称；没有定位到缺失 DLL。未输出整份环境变量，未改 PATH 或安装依赖。
- 增加 `--nocapture` 后复跑，结果 `10 passed / 335 filtered / exit 0`。未修改运行环境。并行工作区中其他代理仍可能触发重编译；未记录首轮二进制 SHA，不能证明两轮二进制字节相同，也不能将恢复归因于 `--nocapture`。
- 未修改产品代码掩盖运行器错误。首轮异常与复跑成功均保留，停止扩大环境调查。

### N3：实施阶段两次访问冲突仍无根因结论

实施报告记录两次 `0xc0000005, STATUS_ACCESS_VIOLATION`，随后窄用例及完整 guarded-write 过滤集通过。失败发生阶段和原因未确认。日志中同时出现的 mbx 告警不构成因果证据。本轮没有复现该访问冲突，未更改缓存或链接器配置。

### N4：跨平台与后续生命周期验收尚未完成

- Linux guarded-write 与 pending store 的 Unix mode 分支已通过，证据见下方补充验证。macOS 未运行；独立 AtomicWriter 的完整 umask 矩阵不在本次 Linux 过滤集内，不能由这些结果推定通过。
- OAuth listener bind、socket deadline、流程身份/重启恢复、控制命令可达性仍归 T11。本轮没有改动这些边界或声称真实 OAuth 网络/原生 UI 验收通过。
- 未运行全仓 `just ci`、完整 `just lint-strict`、完整 `just tauri-ci`、全部 Rust/Tauri 测试或发布构建。
- 正式前端 `bun run lint:ci` 的既有基线仍记录两个用户临时脚本中的 5 条 `no-console`：`ccr-ui/.tmp-desktop-probe.mjs`、`ccr-ui/.tmp-insights-visual.mjs`。本角色未复跑该前端门禁、未修改或忽略两文件；Rust Clippy 通过不改变该背景。

### N5：Linux 测试构建有本任务外的 unused-imports 告警

Linux pending 测试构建在 `crates/ccr-codex/src/services/codex_process_service.rs:574-575` 报告 1 条 `unused_imports` 告警，涉及 `SysinfoProcessBackend` 与 `process_refresh_kind`。`git diff --exit-code` 确认该文件未修改。测试 exit 0；本次未运行 Linux Clippy，不能将 Windows Clippy 的既有通过结果扩展到 Linux。遵照补验范围保留该代码，交父任务/T10 处理。

## 四项 AC 结果

| AC | 结果 | 证据与边界 |
| --- | --- | --- |
| AC1：唯一前镜像和轮换 | Windows 与 Linux writer 定向通过 | Windows core 25 项、主会话 Linux core 24 项均覆盖固定时钟前镜像、三独立子进程版本集合、碰撞、目标保持、keep-10、稳定排序。profile-off 同秒目录及补偿仍以 Windows CLI 10 项为证据。 |
| AC2：旧/新备份兼容及权限 | Windows 消费者与 Linux mode 定向通过 | Windows 验证 `ConfigManager` 发现、`SettingsManager` 实际恢复、DACL 与只读属性。Linux core 24 项验证旧/新命名、轮换、基础恢复及 Unix `0400`/`0600` 备份 mode；未运行 Linux CLI reader 过滤集或 macOS。 |
| AC3：secret/no-backup 与失败传播 | Windows 与 Linux 存储定向通过 | Windows 保留真实 DACL、删除拒绝、Tauri disk-first memory 和原路径证据。Linux pending 5 项补验创建 `0600`、替换保留 `0400`、创建/替换/取消/过期无副本、模拟权限拒绝保持原目标，以及 camelCase 无损兼容。未将 Linux 模拟权限拒绝表述为原生 OS 拒绝。 |
| AC4：失败/取消/过期脱敏 | 存储与适配器定向通过 | Windows 的 IPC/tracing 证据保持；Linux pending 5 项同时验证默认 Debug/Serialize、格式错误、CLI `user_message` 和权限拒绝错误的 sentinel 脱敏。当前 CLI 没有 pending 操作命令，未运行真实 CLI OAuth 流程；监听器 timeout 事件仅做源码核对，网络流程归 T11。 |

## Verification

以下为本检查角色实际运行的命令。过滤集结果不等于包级测试结果。

| 命令 | 结果 | 范围 |
| --- | --- | --- |
| `cargo test -p ccr-core guarded_write -- --test-threads=1 --nocapture`（修复前基线） | exit 0；23 项 | 实施者原有过滤集。 |
| `cargo test -p ccr-core test_readonly_backup_collision_failure_removes_temporary -- --test-threads=1 --nocapture` | exit 0；1 项 | 新增碰撞清理覆盖，初版断言。最终加强的 16 次断言由完整过滤集验证。 |
| `cargo test -p ccr-core test_backup_preserves_windows_readonly_attribute -- --test-threads=1 --nocapture`（修复前） | exit 1；0 通过/1 失败 | 确认新备份的属性回归。 |
| `cargo test -p ccr-core guarded_write -- --test-threads=1 --nocapture`（修复后） | exit 0；25 通过/79 过滤 | 全部本域原有与新增用例。 |
| `cargo test -p ccr-codex oauth -- --test-threads=1` | exit 0；9 通过/222 过滤 | 6 项 pending store 及 3 项原有 OAuth 用例；在本轮 backup-only 修复前运行，pending 使用 BackupPolicy::None。 |
| `cargo test -p ccr-cli profile_off -- --test-threads=1` | exit 1；无有效计数 | STATUS_DLL_NOT_FOUND，见 N2。 |
| `cargo test -p ccr-cli profile_off -- --test-threads=1 --nocapture` | exit 0；10 通过/335 过滤 | 两个测试目标的聚合输出；保留 T01 清理 helper。 |
| `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::codex::auth::tests -- --test-threads=1` | exit 0；5 通过 | 环境设置 `CCR_SKIP_ICON_GENERATION=1`；主测试目标另有 523 过滤，guard 集成目标匹配 0 项。没有将 0 项目标算成行为验证。 |
| `cargo clippy -p ccr-core -p ccr-codex -p ccr-cli --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | exit 0 | 最终本任务代码及所列包的全部 targets/features；不同于完整 workspace lint。 |
| `cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings` | exit 0 | Tauri binary；存在工具链 linker-messages 提示，命令未因该提示失败。 |
| `python scripts/quality/check_secret_writes.py` | exit 0 | 实际 pending owner 和 adapter 已在扫描名单；静态规则不替代行为测试。 |
| `rustfmt --edition 2024 --config skip_children=true --check` + 六个本任务 Rust 文件 | exit 0 | core writer/guarded writer、profile_off、pending store、services/mod、Tauri codex_auth。 |
| scoped `git diff --check` | exit 0 | 本任务已跟踪代码/spec/script；新 report/store 另外读取确认。 |
| `git diff --exit-code -- crates/ccr-core/src/core/lock.rs` | exit 0 | 既有失败文件未修改。 |

- Lint：上述三包严格 Clippy、Tauri binary Clippy 通过；完整 workspace lint 未运行。
- TypeCheck：上述 Clippy 对 Rust targets/features 完成类型检查，Tauri auth 测试目标也已编译；没有独立执行 `cargo check`，T05 未改前端 TypeScript。
- Tests：本域最终定向测试通过；完整 core 包级已有失败，完整跨平台门禁未通过。

## Linux 补充验证（2026-09-28）

本次补验未修改生产代码、测试源码、依赖清单或任务状态；未占用 Windows 编译目录。

### 环境与命令

- WSL distribution：`Ubuntu-24.04`。cwd：`/mnt/d/Documents/Code/Github/ccr`。
- 直接运行 `/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo`；未调用 rustup shim、未安装工具链。实际版本输出为 `rustc 1.95.0 (59807616e 2026-04-14)`。
- `PATH` 前置同一工具链目录；`RUSTC`、`RUSTDOC` 分别指向该目录中的真实二进制。
- `CARGO_TARGET_DIR=/tmp/ccr-architecture-linux-01a0e781`。未添加 `--all-features` 或改变包默认 features。

首轮命令：

```text
/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test --offline --locked -p ccr-codex oauth_pending -- --test-threads=1
```

离线缓存缺少 `aead v0.6.1`，Cargo 返回 101，Windows 包装命令返回 1，没有运行测试。证据：`check-linux-oauth-pending-offline.json` / `.log`。

按已授权边界保留 `--locked` 补齐缺少依赖后执行：

```text
/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test --locked -p ccr-codex oauth_pending -- --test-threads=1
```

结果：**5 passed / 0 failed / 0 ignored / 224 filtered，exit 0**。总耗时 54.83 秒，包含依赖下载和编译；测试自身 0.09 秒。可执行文件：`/tmp/ccr-architecture-linux-01a0e781/debug/deps/ccr_codex-d450ec27deaa5260`。完整证据：`check-linux-oauth-pending.json` / `.log`。

5 项用例分别覆盖创建/替换/取消/过期无副本、错误及默认输出脱敏、旧明文形状无损读写、模拟权限拒绝时保持旧/缺失目标、真实 Unix 文件 mode。最后一项断言首次创建为 `0600`，将既有目标设为 `0400` 后替换仍为 `0400`。

### 纳入主会话 Linux writer 证据

读取并核对 `root-linux-guarded-write.json` / `.log`。主会话使用相同真实工具链及独立 Linux target 运行：

```text
/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test --offline --locked -p ccr-core guarded_write -- --test-threads=1
```

结果：**24 passed / 0 failed / 0 ignored / 88 filtered，exit 0**。包含 `test_backup_preserves_private_unix_mode` 与 `test_write_guarded_secret_sets_owner_only_mode`，以及固定时钟、多进程、轮换及 CAS 过滤集。这是主会话执行的证据，本角色未重复运行，也未将其计为完整 core 包级门禁。

### 锁文件与源码边界

两次 pending 命令及主会话 writer 命令前后的 `Cargo.lock` SHA256 均为：

```text
ca1f03e85b5e0d888109ac8af7e7943fc1ceecd20a0f6eeb6a60052c5155db79
```

补验时 pending store 源文件 SHA256 仍为 `6F4CCC7384183467D07CA78056325954FCBF6430CDA41CE71B93861C270E8627`，与首次 Windows 检查一致。Linux 编译告警见 N5；本次未执行 Linux Clippy、macOS、原生桌面、真实 OAuth 网络或完整 CI。

## 下游接口与集成边界

- **T02 可使用**：`backup_guarded`、`write_guarded`、versioned/CAS 的公开调用签名未变；旧/新备份可由既有 reader 发现，Settings 恢复路径通过。profile-off 独立目录及现有补偿通过。下游仍须完成自身跨文件事务和版本保护，不能把基础 writer 的成功当作全流程事务成功。
- **T11 可使用**：`ccr_codex::services::{CodexOAuthPendingStore, CodexOAuthPendingState}`；`save(&state)`、`load(now)`、`clear()` 使用原 `ccr_core::Result`。存储操作锁覆盖保存/清理，guarded path lock 保持 leaf lock。失败保留旧 memory 的责任由 Tauri adapter 履行。
- **路径兼容**：桌面继续以 `PlatformPaths`/CCR_ROOT 构造 `with_path`；不能在 T11 重构时无条件改用 `new()`，否则 CCR_DATA_DIR 与 CCR_ROOT 不同时会改变既有 pending 路径。未迁移或复制凭据。
- **验证边界**：允许依赖已验证的 Windows/Linux 定向契约继续实施；不据此完成或归档 T05。macOS、其他未覆盖的平台组合及整体门禁在父任务/T10 继续验收。

## 首次 Windows 检查源文件指纹

以下 SHA256 对应首次 Windows 检查报告写入前的本任务代码。共享工作区后续修改会使对应证据需要重新判断；本次 Linux 补验另行确认 pending store 指纹保持不变。

| 文件 | SHA256 |
| --- | --- |
| `crates/ccr-core/src/core/guarded_write.rs` | `3921FFB0A7C104D14C2639A600B126E55581810118E41C0870D085E4E8EFA8AE` |
| `crates/ccr-core/src/core/atomic_writer.rs` | `AA042DDDE0C9FBA3E19FB13905310AC2A2695F09254C94232FD76BDC2546E81E` |
| `crates/ccr-cli/src/application/profile_off.rs` | `B6A9643D5BCD71179A660CBED70C98AE7FE44790E78E4A2C1CFF0742A472792B` |
| `crates/ccr-codex/src/services/codex_oauth_pending_store.rs` | `6F4CCC7384183467D07CA78056325954FCBF6430CDA41CE71B93861C270E8627` |
| `crates/ccr-codex/src/services/mod.rs` | `F4ACE6ACE4ACD61DB59D2A99A7A0B48002EDD4DFE6A18074F092811D897BD7F3` |
| `ccr-ui/src-tauri/src/commands/codex_auth.rs` | `1895E218644B5BFE63F594DF722D4C6950A1D50F278E341C4F56BEF9BFD30B2F` |

图标批量改动由主会话单独处理。本轮末再次核对相关图标路径 `git diff --name-only` 为空，未自行恢复或删除任何资源。
