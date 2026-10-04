# Manager 锁目录候选修复：独立审查

任务：T07 `09-29-harness-ci-evidence`。审查者：`/root/independent_check`。独立复核时间和机器可读证据见同目录 `manager-lock-independent-review.json`。

## 结论

候选补丁静态审查通过，可以提交新增范围批准。补丁只在 2 个测试文件的 6 个用例中增加现有环境夹具的命名绑定。全部 17 个用例已分类：6 个缺少保护的默认锁目录读取者、1 个已有保护的环境修改/读取者、10 个不涉及 `CCR_ROOT` / `CCR_LOCK_DIR` 默认路径的用例。

代码确认存在环境夹具隔离缺口。原 `just ci` 失败的具体原因未查明；目录清理与锁文件打开交错是与错误相符的假设。既有 2 次诊断均通过，没有动态观测原失败的精确路径或交错。不得将本审查写成历史根因已确认或修复已验收。

本轮只写本报告及其 JSON 证据。未应用候选，未改源码或 spec，未运行新的测试、lint、type-check 或概率重试。

## Findings (fixed)

无源码修复。本轮为新增范围批准前的只读候选审查。

## Findings (not fixed)

### P1：6 个默认锁目录读取者未持有现有环境 guard

- 文件：`crates/ccr/tests/managers/general.rs` 与 `crates/ccr/tests/managers/legacy_registry.rs`。
- 事实：显式配置或 settings 文件路径没有消除共享写入层对 `CCR_LOCK_DIR` 的读取。另一个测试在同一进程内临时修改该变量。
- 未修复原因：两个测试文件均不在 T07 `design.md:6–12` 的批准白名单内。主会话负责取得新增范围批准。
- 建议：批准 `manager-lock-candidate.patch` 的 6 行差异，然后执行本文验收门槛。不要修改生产锁策略、重试、超时或测试断言。

### P2：夹具规范需要明确传递环境依赖

- 文件：`.trellis/spec/ccr/backend/test-fixtures.md`。该文件已在 T07 白名单内。
- 事实：现有第 49 行允许不读取进程环境的显式路径测试直接使用 `tempdir()`。现有条目没有列明 manager 保存操作的传递读取链。
- 未修复原因：本轮只允许写审查证据；规范差异由主会话随候选提交批准并实施。
- 建议：仅在 `Contracts` 增加以下 1 条，不复制五套工具的独立规则文件：

> Tests that call `ConfigManager::save`, `PlatformConfigManager::save`, or `SettingsManager::save_atomic` / `restore` must hold a named `CcrIntegrationTestEnv` binding for the full operation scope. These calls reach `guarded_write`, which reads `CCR_LOCK_DIR` even when target paths or a `SettingsManager` lock argument are explicit. Apply this rule in Claude Code, Codex, Grok Build, Kimi Code, and OMP.

## 实际调用链

1. `crates/ccr-config/src/managers/platform_config.rs:368–372`：`PlatformConfigManager::save` 调用 `fileio::write_toml`。
2. `crates/ccr-config/src/managers/config/manager.rs:70–71` 与 `config_file_handler.rs:92–98`：`ConfigManager::save` 经文件处理器调用 `write_toml_opts`。
3. `crates/ccr-core/src/core/fileio.rs:94–113`：两条 TOML 路径均进入 `guarded_write::write_guarded`。
4. `crates/ccr-cli/src/managers/settings.rs:220–227,368–383`：`save_atomic` 直接调用 `write_guarded`；`restore` 调用 `save_atomic`。`write_options`（96–104 行）提供备份与密文选项，没有传递 `self.lock_manager`。
5. `crates/ccr-core/src/core/guarded_write.rs:103–112`：从目标文件计算资源名，但通过 `LockManager::with_default_path()` 另行选择锁目录。
6. `crates/ccr-core/src/core/lock.rs:218–229`：默认锁目录来自进程 `CCR_LOCK_DIR`。`FileLock::new` 在 127–140 行先创建父目录，再打开锁文件；原错误对应 139–140 行的打开失败。实际文件加锁在打开之后发生。

原失败测试在 `legacy_registry.rs:53` 调用 `manager.save` 时，自己的配置临时目录仍在作用域内。可能被其他测试结束时清理的是保存于 `LockManager` 中的临时锁目录。环境恢复不会修改已经保存的 `PathBuf`。该路径与清理顺序未由原日志直接记录。

## 17 个测试的完整分类

下表只分类与 `CCR_ROOT` / `CCR_LOCK_DIR` 及默认锁目录有关的路径。历史记录的用户名或时钟读取不计入这两个变量的读取者。行号对应未应用候选的当前源码。

| 模块 / 测试 | 源码行 | 分类与候选处理 |
| --- | --- | --- |
| `general::test_config_manager_lifecycle` | 39 | 配置保存，增加 guard |
| `general::test_config_manager_section_operations` | 72 | 配置保存，增加 guard |
| `general::test_config_section_validation` | 110 | 内存配置校验，保持不变 |
| `general::test_config_sorting_and_filtering` | 151 | 内存排序，保持不变 |
| `general::test_settings_manager_atomic_operations` | 187 | settings 保存，增加 guard |
| `general::test_settings_update_from_config` | 223 | 内存键值映射，保持不变 |
| `general::test_settings_backup_and_restore` | 253 | settings 保存与恢复，增加 guard |
| `general::test_settings_validation` | 300 | 内存 settings 校验，保持不变 |
| `general::test_history_manager_add_and_load` | 331 | 显式 SQLite 路径，保持不变 |
| `general::test_history_manager_filtering` | 371 | 显式 SQLite 路径，保持不变 |
| `general::test_history_manager_recent_limit` | 428 | 显式 SQLite 路径，保持不变 |
| `general::test_history_manager_stats` | 464 | 显式 SQLite 路径，保持不变 |
| `general::test_history_entry_env_changes` | 517 | 构造与掩码历史数据，保持不变 |
| `general::test_config_and_settings_integration` | 561 | 配置及 settings 保存，增加 guard |
| `legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth` | 8 | 平台配置保存，增加 guard |
| `legacy_registry::per_platform_current_profile_helpers_round_trip` | 64 | 内存 UnifiedConfig，保持不变 |
| `legacy_registry::config_manager_default_ignores_legacy_current_platform_routing` | 86 | 已有同一夹具 guard，保持不变 |

历史测试使用 `Database::init(explicit_path)`（`crates/ccr-store/src/storage/database.rs:52–73`），SQLite pool 保留传入路径（`crates/ccr-core/src/core/sqlite.rs:29–39`），`HistoryManager::new(db)` 只保存该数据库（`crates/ccr-store/src/history.rs:175–176`）。测试未进入 `init_default`。`HistoryEntry::add_env_change` 只向数据结构写入掩码值（151–164 行）。

配置校验和排序分别在 `config/types.rs:159–195`、`config/ccs_config.rs:80–82`；settings 映射和校验在 `crates/ccr-types/src/claude_settings.rs:270–276,336–370`。上述路径均操作已有内存值。

## 候选范围与 guard 生命周期

- 候选 SHA-256：`a7082c421018a9b54167b3069ec3000cc21210bdebe3ca135cf6d7f52e775738`。
- 差异：`general.rs` 新增 5 行，`legacy_registry.rs` 新增 1 行，删除 0 行。未改变测试总数、断言、文件路径或临时目录构造。
- 独立执行 `git apply --check` 返回 0；候选未应用。
- `managers.rs:6–9` 将两个测试模块连接到同一 `support/env.rs` fixture。`support/env.rs:8,17–29` 先获取同一静态 mutex，再设置两个环境变量。
- `let _env = ...` 是命名绑定。绑定持有 guard 至测试作用域结束，不会像 `let _ = ...` 一样立即丢弃返回值。
- 新绑定位于每个用例首行。其后的同步保存、最终读取、断言及后创建临时资源的清理均在该绑定的作用域内；这 6 个测试没有将相关写入派发到未等待的后台任务。
- `Drop` 在 mutex 仍存活时恢复环境变量（`support/env.rs:43–48`）。字段随后按声明顺序销毁；恢复后释放 `_guard`，再清理旧临时目录。受保护的下一个用例会在读取路径前创建并设置自己的目录，因此不再保留前一用例的锁路径。无需改写现有 fixture。
- 7 个环境相关测试通过现有 mutex 互斥，10 个其他测试继续由默认测试并行策略运行。候选未新增 `#[serial]`、`--test-threads=1`、`RUST_TEST_THREADS`、概率重试或阈值变更。

六行候选足以覆盖已确认的同二进制读取者集合。结论限定于这组调用路径；正式修复结果仍需应用后的门槛证明。

## Verification

| 项目 | 结果与边界 |
| --- | --- |
| 原 full CI | `final-ci-retry1.json` / `.log` 保留 exit 1；`managers` 为 16 passed / 1 failed |
| 原 full CI 日志哈希 | 独立重算匹配 `acaa41ae374ef61a8d44733e49e09e5ef59f0800fafc78febaec2195c1fe1cc2` |
| 既有 exact 诊断 | 1 passed / 0 failed；只复核 `manager-lock-exact.json` 和日志 |
| 既有 managers 诊断 | 17 passed / 0 failed；日志测试名与当前 17 个源码用例完全一致 |
| 两次诊断日志 | 4 份 stdout/stderr SHA-256 均独立复算一致 |
| 两次诊断源码 | 两组 before/after 的 14 个文件哈希相同，并与本轮当前字节相同 |
| 诊断锁文件 | exact 为 1 个配置锁；suite 为 4 个配置锁、3 个 settings 锁；实际文件清单匹配凭据 |
| 候选静态范围 | 2 文件、6 行新增、0 行删除；`git apply --check` 通过 |
| Lint | 未运行；候选未应用，待新增范围批准 |
| TypeCheck | 未运行；候选未应用，待新增范围批准 |
| 新测试运行 | 0 次；本轮没有增加概率复现 |

诊断 runner 只设置 Cargo 子进程的 `CCR_ROOT`、`CCR_LOCK_DIR`、`RUSTUP_AUTO_INSTALL=0`，保留默认并行及 `--skip export_bindings`。两次通过只提供当前诊断结果，原 full CI 失败继续保留。原 aggregate 凭据记录的 2509 个交付文件字节保持结论属于该轮运行；本审查独立重算范围为上述 14 个诊断输入。

## 批准后的文件与验收要求

新增源码白名单仅为：

1. `crates/ccr/tests/managers/general.rs`：5 个命名 guard 绑定。
2. `crates/ccr/tests/managers/legacy_registry.rs`：1 个命名 guard 绑定。

已批准的 `test-fixtures.md` 增加本文建议的 1 条传递读取合同。主会话将批准、候选哈希、验收结果与原失败记录写回 T07 工件。

| 顺序 | 必须通过的命令 / 核查 |
| --- | --- |
| 1 | 应用前核对 14 个诊断输入及候选哈希；`just version-check`；`just fmt-check` |
| 2 | `cargo test -p ccr --all-features --test managers legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth -- --exact --skip export_bindings` |
| 3 | `cargo test -p ccr --all-features --test managers -- --skip export_bindings`，全 17 个用例通过 |
| 4 | 保留当前 fixture spec 的三条必需检查：`cargo test -p ccr --test platforms -- --nocapture --skip export_bindings`；`cargo test -p ccr --test commands -- sync_content --nocapture --skip export_bindings`；`cargo test -p ccr --test managers -- --nocapture --skip export_bindings` |
| 5 | `cargo clippy -p ccr --all-targets --all-features -- -D warnings`；`just lint-strict`；`just test`；`python scripts/ci/check_workflow_governance.py` |
| 6 | 父任务实际 `just ci`，记录输入哈希、完整输出、退出码与源码前后校验 |
| 7 | `git diff --check`、T07 上下文校验及独立差异复核 |

第 4 项保留现有 spec 的 `Tests Required`，防止批准计划漏项；未要求更改这些测试。所有 Rust gate 保留默认并行。绑定导出继续由独立 gate 负责。完整 `just ci` 成功才可关闭本地整体验收失败。其他操作系统、托管 SHA 和新客户端加载边界保持各自证据状态。

强模型负责调用链、范围和最终验收审查。取得补充批准后，较便宜模型可执行固定两文件六行补丁与固定命令。Claude Code、Codex、Grok Build、Kimi Code、OMP 使用同一约束；客户端可用能力与原生加载结果仍由各自证据证明。
