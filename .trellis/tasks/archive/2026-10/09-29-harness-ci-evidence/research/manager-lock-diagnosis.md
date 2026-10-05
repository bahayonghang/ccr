# Manager 锁目录失败：只读诊断与待批准补充计划

日期：2026-09-30 UTC。任务：T07 `09-29-harness-ci-evidence`。基线 HEAD：`fe9d65972376dbb4c332fde9860c1e42ff2f2005`。

## 结论与证据边界

已确认 `managers` 集成测试二进制存在环境隔离缺口：6 个用例经共享写入层读取 `CCR_LOCK_DIR`，未持有该二进制现有的环境 mutex。另一个用例持有该 mutex，临时替换 `CCR_ROOT` 与 `CCR_LOCK_DIR`，并拥有随后清理的临时根目录。只保护环境变量修改者，无法保护未使用同一 guard 的读取者。

该缺口提供了与原始 `FileLockError` 一致的高置信竞态假设。原失败日志没有锁文件实际路径或线程交错记录，因此该次失败的具体交错仍未观测。不能声称已经动态复现根因。

本轮只运行 2 条诊断测试：单用例 1/1 通过；完整 managers 二进制 17/17 通过。保留原始 full CI FAIL，不用窄测试通过替代全量验收，不运行概率重试。测试前后 14 个相关源码/锁文件 SHA-256 相同。未修改源码、测试、spec、全局环境或用户配置。

## 原始失败

- 命令：`just ci`；2026-09-30T03:49:38.114489+00:00；退出码 1。
- 失败阶段：Test；`managers` 为 16 passed / 1 failed。
- 用例：`legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth`。
- 位置：`crates/ccr/tests/managers/legacy_registry.rs:53`，实际调用是 `manager.save(&config).unwrap()`。
- 错误：`FileLockError("无法打开锁文件: The system cannot find the path specified. (os error 3)")`。
- 原始记录：`.trellis/tasks/09-29-evergreen-harness-audit/research/final-ci-retry1.json` 与同目录 `.log`。
- 已重新核验日志 SHA-256：`acaa41ae374ef61a8d44733e49e09e5ef59f0800fafc78febaec2195c1fe1cc2`。日志 342–375 行保留 17 个用例、panic 与 Cargo 失败结果。

原 full CI 记录另有 2509 个交付源码文件前后字节相同的证据。该记录由父任务生成，本轮没有重跑全 CI。

## 调用链与目录所有权

1. `legacy_registry.rs:9–10` 创建临时目录 A，`config.toml` 属于 A。该用例的 `TempDir` 在 `manager.save` 执行时仍在作用域内。
2. `platform_config.rs:285–288` 只存储配置路径，没有存储锁目录。`save` 在 368–372 行清除旧路由字段并调用 `fileio::write_toml`。
3. `ccr-core/src/core/fileio.rs:94–113` 使用默认 `WriteOptions`，最终进入 `guarded_write::write_guarded`。
4. `ccr-core/src/core/guarded_write.rs:103–112` 从目标路径计算资源名，但通过 `LockManager::with_default_path()` 单独选择锁目录。资源名函数在 350–361 行生成 `gw_{stem}_{hash}`。配置路径为显式参数不会使锁目录变为显式参数。
5. `ccr-core/src/core/lock.rs:218–229` 读取进程 `CCR_LOCK_DIR`。变量缺失时回退到用户 home 下的 `.claude/.locks`。`LockManager` 保存读取到的 `PathBuf`。
6. 同一测试二进制的 `legacy_registry.rs:86–105` 调用 `setup_ccr_test_env()`。`managers.rs:6–9` 使用 `support/env.rs` 中同一个 binary-local 静态 mutex。
7. `support/env.rs:17–30` 在 guard 内创建临时根目录 B 与 B 的 `.locks`，然后修改两个进程环境变量。B 由 `CcrIntegrationTestEnv.temp_dir` 持有；失败用例 A 没有获取该 guard。
8. `support/env.rs:43–48` 在 guard 仍存活时恢复原环境变量。夹具销毁还会清理其持有的 B。未获取 guard 的其他用例可能已经保存 B 的锁目录路径，环境恢复不会更新这些 `PathBuf`。
9. `lock.rs:135–148` 先 `create_dir_all(parent)`，再 `OpenOptions::open(lock_path)`。原错误来自第二步，说明第一步已返回成功。`fs4` 实际加锁发生在 open 成功之后，所以该失败不在 fs4 的 `try_lock` 适配层。

高置信假设的最小交错：A 读取 B 的 `CCR_LOCK_DIR` → A 创建/确认 B 的锁父目录 → B 的夹具结束并清理 B → A 打开锁文件，Windows 返回 OS error 3。精确路径和这一事件顺序未由原日志直接记录；其他外部文件系统清理仍没有观测证据。

## 同二进制全部 17 个用例复核

下表的“读取”专指 `CCR_ROOT` / `CCR_LOCK_DIR` 及默认锁目录，未将用户名等无关环境读取归入本次竞态。

| 用例（省略模块前缀） | 相关路径 | 同一环境 guard | 处理建议 |
| --- | --- | --- | --- |
| `test_config_manager_lifecycle` | `ConfigManager::save` → fileio → guarded write；general.rs:39,59 | 无 | 增加现有 fixture guard |
| `test_config_manager_section_operations` | 同链路；general.rs:72,87,98 | 无 | 增加现有 fixture guard |
| `test_config_section_validation` | 内存配置验证 | 不涉及 | 保持不变 |
| `test_config_sorting_and_filtering` | 内存配置排序 | 不涉及 | 保持不变 |
| `test_settings_manager_atomic_operations` | `SettingsManager::save_atomic` → guarded write；general.rs:187,207 | 无 | 增加现有 fixture guard |
| `test_settings_update_from_config` | 内存 settings 更新 | 不涉及 | 保持不变 |
| `test_settings_backup_and_restore` | `save_atomic` → guarded write；general.rs:253,267,279 | 无 | 增加现有 fixture guard |
| `test_settings_validation` | 内存 settings 验证 | 不涉及 | 保持不变 |
| `test_history_manager_add_and_load` | 显式临时 SQLite 路径；general.rs:331–335 | 不涉及 | 保持不变 |
| `test_history_manager_filtering` | 显式临时 SQLite 路径；general.rs:371–375 | 不涉及 | 保持不变 |
| `test_history_manager_recent_limit` | 显式临时 SQLite 路径；general.rs:428–432 | 不涉及 | 保持不变 |
| `test_history_manager_stats` | 显式临时 SQLite 路径；general.rs:464–468 | 不涉及 | 保持不变 |
| `test_history_entry_env_changes` | 构造掩码记录，未修改进程环境；general.rs:517–553 | 不涉及 | 保持不变 |
| `test_config_and_settings_integration` | config save + settings save；general.rs:561,580,592 | 无 | 增加现有 fixture guard |
| `legacy_registry_fields_load_without_becoming_clean_write_routing_truth` | PlatformConfigManager save；legacy_registry.rs:8,53 | 无 | 增加现有 fixture guard |
| `per_platform_current_profile_helpers_round_trip` | 内存 UnifiedConfig；legacy_registry.rs:64 | 不涉及 | 保持不变 |
| `config_manager_default_ignores_legacy_current_platform_routing` | 临时改写两个环境变量，并通过默认路径读取；legacy_registry.rs:86–105 | 已有 | 保持不变 |

合计：6 个未保护的默认锁目录读取者；1 个已经保护的环境修改/读取者；10 个不涉及该环境路径的用例。

相关传递证据：`ConfigManager::save` 在 `ccr-config/src/managers/config/manager.rs:70–71` 调用 `ConfigFileHandler::save`，后者在 `config_file_handler.rs:92–98` 调用 `write_toml_opts`。`ccr-cli/src/managers/settings.rs:220–227` 的 `save_atomic` 直接调用 `write_guarded`；即使测试向 `SettingsManager::new` 传了显式 `LockManager`，该写入路径仍使用 guarded write 的默认锁目录。历史测试使用 `ccr-store/src/storage/database.rs:52–73` 的显式 `Database::init(path)`，随后 `HistoryManager::new(db)`（history.rs:175–176），未进入 `init_default`。

现有规范 `.trellis/spec/ccr/backend/test-fixtures.md:31–34,46–49` 已要求环境夹具；直接 `tempdir()` 只适用于不读取进程环境的显式路径测试。六个用例不满足这一前提。

## 已执行诊断及凭据

两次调用均使用 `manager-lock-run.py`。仅给 Cargo 子进程复制的环境设置独立 `CCR_ROOT`、`CCR_LOCK_DIR` 和 `RUSTUP_AUTO_INSTALL=0`；未改变父进程或系统环境。所有诊断输出和持久保留的外层锁目录均在当前 research 目录的 `manager-lock-*` 下。既有 Rust 测试内部的 `tempfile` 生命周期保持原样。

| 假设 / 目的 | 命令 | 开始时间 UTC | 退出码 / 结果 | 凭据 |
| --- | --- | --- | --- | --- |
| 无并发环境修改者时，目标写入链是否仍失败 | `cargo test -p ccr --all-features --test managers legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth -- --exact --skip export_bindings` | 03:55:59.143184 | 0；1 passed；0 failed | `manager-lock-exact.json`、同前缀 stdout/stderr、hashes-before/after |
| 原二进制所有用例默认并行时是否重现 | `cargo test -p ccr --all-features --test managers -- --skip export_bindings` | 03:56:47.581206 | 0；17 passed；0 failed | `manager-lock-suite.json`、同前缀 stdout/stderr、hashes-before/after |

单用例产生 1 个 `gw_config_*.lock`；全套产生 4 个 `gw_config_*.lock` 和 3 个 `gw_settings_*.lock`，位于外层隔离锁目录。该事实验证了显式配置/settings 路径的默认锁目录副作用。日志不记录 fixture 临时目录清理交错。剩余第 3 条窄测试预算未使用。

## 待批准补充范围

优先级 P1：修复既有集成测试夹具使用。以下两个源码文件均不在当前 T07 `design.md` 白名单，需主会话取得补充范围批准后才能应用。本报告与候选 patch 不构成实施授权。

| 文件 | 精确改动 | 必须通过 |
| --- | --- | --- |
| `crates/ccr/tests/managers/general.rs` | 在上表 5 个默认锁目录读取用例的首行增加 `let _env = crate::setup_ccr_test_env();`；保留原显式配置、settings、备份及 tempdir 路径 | `cargo test -p ccr --all-features --test managers -- --skip export_bindings`；全 17 个用例通过 |
| `crates/ccr/tests/managers/legacy_registry.rs` | 在失败用例首行增加 `let _env = setup_ccr_test_env();`，使用已有 import | 原失败 exact 命令通过；managers 全套通过 |

候选为 `manager-lock-candidate.patch`，只增加 6 行，未应用。`_env` 是命名绑定，持有现有 RAII guard 至该用例结束；不得改为立即丢弃的 `let _ = ...`。先获取 guard，再创建该用例其他临时资源，使操作和资源清理发生在 guard 生命周期内。只保护共享进程环境使用者，10 个无关用例维持默认并行。

不修改 `support/env.rs`、生产 `LockManager`、guarded write、fs4 依赖、超时、重试次数、测试阈值或全局线程设置。不引入新的恢复机制。

补充批准后，由强模型复核六个调用点和 fixture 生命周期；执行者可使用较便宜模型完成上述确定性 6 行补丁。Claude Code、Codex、Grok Build、Kimi Code、OMP 均可承担有 shell 能力的执行；工具名称不替代默认并行和证据门槛。适用工具与传递环境依赖应回写现有 test-fixtures 规范及 T07 记录，避免新增重复规则。规范回写也应纳入补充批准的精确差异。

建议在 `test-fixtures.md` 第 49 行的显式路径规则后追加以下英文条目，保留该文件既有五工具适用范围：

> Explicit config or settings paths still require the integration binary's shared environment fixture when the write path calls `fileio` / `guarded_write`, because that layer resolves `CCR_LOCK_DIR`. Hold a named fixture binding for the full test. Passing a `LockManager` to `SettingsManager::new` does not isolate `save_atomic` from the guarded-write lock directory.

## 批准后的验收顺序

1. 应用前重新核对 14 个相关文件哈希与并发变更；核对候选仅涉及上述两个文件的 6 行。
2. `just version-check`、`just fmt-check`；随后 exact 失败用例和 managers 全套，保持默认并行与 `--skip export_bindings`。
3. `just lint-strict` 和 `just test`，后者保留 workspace/all-features/default parallelism/binding skip。保留原失败，不添加自动重试。
4. 父任务执行实际 `just ci`，记录命令、退出码、输入哈希、输出和源码前后字节校验。只有完整入口成功才能关闭当前整体验收失败。
5. `git diff --check`；T07 上下文验证；独立审查确认规范与批准范围一致。Windows 本地通过不关闭 macOS、新客户端会话、托管 SHA 等既有未验证边界。

本轮无 commit、push、PR、任务归档、用户配置访问或源码修复。
