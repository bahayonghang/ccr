# P2 实施与本地验证

状态：IMPLEMENTED_LOCAL_CHECKS_PASS。独立 Trellis check 与 workspace 集成门由主会话执行。未提交、未归档。

日期：2026-10-06。环境：Windows，分支 `dev`，实施前 HEAD `c524ac07`。测试使用临时目录和合成凭据，未操作真实账号。

## 产品改动

| 文件 | 改动 |
| --- | --- |
| `crates/ccr-codex/src/models/codex_auth.rs` | Registry/Account 增加 flatten `extra: toml::Table`；Default 与本文件测试构造点补空表 |
| `crates/ccr-codex/src/services/codex_registry_store.rs` | 主版本判定、共享只读错误文本与版本提取；锁内保存兜底；结构解析失败时尝试读取 version；未知字段往返与版本门测试 |
| `crates/ccr-codex/src/services/codex_auth_service.rs` | 6 个命令的副作用前检查；公开无副作用的 `ensure_registry_writable` 供 TUI 预检；后台 current_auth 元数据跳过；合成命令矩阵、force 覆盖、双向 token 同步测试 |
| `crates/ccr-codex/src/services/codex_oauth_token_service.rs` | 只读元数据保存记录 warn 后返回成功；专门测试；构造点补空表 |
| `crates/ccr-codex/src/platforms/codex.rs` | 既有测试的 Registry/Account 构造点补空表 |
| `crates/ccr-tui/src/tui/codex_auth/app.rs` | 保存/删除/重命名/切换错误本地化；Profile 退出前检查注册表；EN/ZH 与版本矩阵测试 |
| `crates/ccr-tui/src/tui/codex_auth/ui.rs` | 既有测试 Account 构造点补空表 |

`codex_usage_estimation.rs` 的 Registry 字面量均使用 `..Default::default()`，Account 夹具通过反序列化构造，无需修改。未改变导出 DTO、桌面 DTO、ccr-vscode 或 P1/P3/P4 的产品逻辑。

## 调用链边界补充

`switch_account` 的 `ensure_current_runtime_supports_openai_switch` 经 `current_profile_name` 调用 `load_or_create_default`，并可能创建锁目录。因此版本门放在 `ensure_managed_auth_supported` 之后、该运行时校验之前。主会话已确认顺序调整。

TUI `switch_selected_account` 在调用服务前执行 `profile_off_for_platform`。新增服务预检保护该前置副作用。测试通过可注入的 Profile 退出回调验证未调用退出步骤；生产包装器仍调用原 `profile_off_for_platform(Platform::Codex)`，只丢弃原本未使用的成功结果。测试回调仅能修改临时夹具。

用户命令测试比较整个临时 home 的文件字节和目录集合：2 个版本（`2.0`、`abc`）× 2 种快照目录状态，覆盖 6 个命令，另覆盖同名 rename。保存、切换、删除、改描述、force rename 和携带合成凭据的 import 均在文件副作用前拒绝。

后台写入和 store.save 的只读兜底在锁内执行，允许创建注册表锁目录与空锁文件；注册表、快照、runtime/config 与备份集合保持要求的状态。快照与 runtime 的双向 token 同步继续，注册表元数据不写入。

## 验证结果

所有测试保持默认并行度。没有修改覆盖阈值或跳过新增测试。

| 命令 | 最终结果 | 退出码 |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features flatten_probe_preserves_unknown_tables_and_account_fields -- --skip export_bindings` | 实施前 test-only flatten 探针：1 passed，293 filtered；通过后改为实际 load/save 回归测试 | 0 |
| `cargo test -p ccr-codex --all-features codex_registry_store -- --skip export_bindings` | 10 passed，289 filtered | 0 |
| `cargo test -p ccr-codex --all-features read_only -- --skip export_bindings` | 修正后台锁文件夹具断言后：12 passed，298 filtered | 0 |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | 308 passed，2 ignored，2 suites | 0 |
| `cargo test -p ccr-tui --all-features -- --skip export_bindings` | 252 passed，2 suites；包含 TUI 预检与本地化测试 | 0 |
| `cargo clippy -p ccr-codex -p ccr-tui --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | 完成，无警告 | 0 |
| `just fmt-check` | JSON 格式检查 5 tests/11 files，workspace 与 Tauri cargo fmt 检查通过；最后改动后复核通过 | 0 |
| `git diff --check -- crates/ccr-codex crates/ccr-tui` | 通过 | 0 |

`cargo fmt -p ccr-codex -p ccr-tui` 作为独立修复命令执行，退出码 0；格式验证单独执行。

完整 ccr-codex 测试在增加 TUI 使用的无副作用服务预检方法前完成。此后完整 ccr-tui 测试编译使用最终 ccr-codex，严格 Clippy 也检查最终两个 crate。workspace 集成测试保留给主会话。

## 首次失败与修正

1. 合成导入测试第一次编译：`export_accounts` 仅接受 `include_secrets: bool`，测试错误传入账号列表，产生 E0061，退出码 1。测试改为现有 `CodexAuthExport` 格式并附加合成 auth_data。
2. 第一轮 read_only 测试：10 passed、1 failed，退出码 1。后台保存按设计先获取锁，产生 `.locks/codex_auth_registry.lock`，测试误要求目录也完全不变。测试仅允许这两个锁基础设施路径，其余目录、文件字节和备份集合继续比较；复测通过。用户命令的完整目录不变断言保持不变。
3. TUI 预检测试第一次编译：注入回调要求 `Result<()>`，原 `profile_off_for_platform` 返回 `Result<ProfileOffResult>`，产生 E0308，退出码 1。生产包装器显式 `.map(|_| ())`，保持原调用与错误传播；完整 TUI 复测通过。

## 未运行与限制

- NOT_RUN：本子代理不重复运行 `just lint-strict`、`just test` 或 `just ci`。主会话执行 workspace 集成门后才能声明 AC7 的完整集成验收。
- IGNORED：既有 `benchmark_list_sessions_inventory_cache`、`benchmark_compute_rolling_usage_cache`，保留原 ignore 属性。
- NOT_RUN：Unix-only 权限测试，本机为 Windows。
- NOT_RUN：真实账户、在线配额/refresh、原生终端视觉与用户目录行为。TUI 证据来自临时夹具与状态/Toast 断言。
- 字段保留保护包含本修复的 CCR 版本之间的往返；既有旧版本仍可能丢弃未知字段。

spec 写回由主会话负责，未计入本子代理产品改动。
