# T09 R2 后端环境隔离补充

日期：2026-09-28。范围：独立审查确认的 Claude Settings 入场与读写环境竞态。主会话授权在 T09 R2 内补齐；本报告不表示 T09 全部验收或父任务完成。

## 变更与责任

- `commands/claude_settings.rs` 的 `claude_get_settings`、`claude_update_settings` 增加可选 `expected_environment_id: Option<String>`，保持原返回类型。前端 IPC 键为 `expectedEnvironmentId`。
- `commands/claude.rs` 在 registry 读锁内核对当前环境 ID 并捕获一个 `Arc<dyn ExecutionEnvironment>`，释放锁后执行 I/O。同次远端更新只通过该 Arc 读取和写入，不在 await 后重新选择 active。
- 不匹配或显式期待环境但 registry 为空时返回 `settings_environment_changed: active environment does not match the settings session`，不执行配置读取、写入或修改闭包。
- 缺省/空值 `None` 保持旧调用的入场语义；空 registry 仍回退 Local。其他 shared `update_settings` 调用也复用单次捕获，不要求全部接口新增参数。
- Local 继续调用既有 `SettingsManager::update_atomic_async`，没有替换备份、权限或原子写 owner。仅删除本次重构造成无调用者的 `save_settings`、`write_active_claude_settings_raw`。
- registry 源仅调整两个命令 input schema 与 client declaration。生成产物只改变 `generated/claude.ts`、`command-manifest.json`、`commandCapabilities.ts`。

## 环境身份边界

Local ID 为 `local`；WSL 为 `wsl:<distro>`；正常持久 SSH 为 `ssh:<DB host id>`。`EnvironmentRegistry::register` 本身不拒绝重复 ID；刷新可替换相同逻辑 ID 的对象。expected ID 只绑定逻辑目标，不作为跨请求 revision。入场后捕获的 Arc 独立于 registry refresh、A→B→A 回切及同 ID 对象替换。

前端仍须使用编辑会话 epoch 隔离旧响应。一次操作在 A 入场后切到 B，操作可以完成 A 的读写；前端不能把该返回值归属于 B 或新的 A 会话。此补充不宣称 ID 检查能够识别入场前已发生并结束的 A→B→A。

## 已验证行为

- 将原三次 active 读取逻辑机械移到同一 registry helper 后，真实双环境 barrier 反例失败：读取 A 后切 B，A 写次数为 0，断言期望 1。日志 `research/backend-environment-red.log`，源 SHA 和修复前源快照见 `research/backend-environment-red.json`。该失败直接执行生产更新 helper，没有模拟成功保存结果。
- 修复后 `commands::claude::` 共 39 个测试通过，包含 6 个新增环境测试：读写目标固定；期待 A 而入场 B 时读取与更新都零 I/O 拒绝；读返回原目标数据；A→B→A 同 ID replacement 不改变原写目标；空 registry 的显式/旧调用行为；Local 原子 owner 与备份/未知字段保留。
- barrier 使用开始与恢复两阶段，没有时间延迟制造竞态。测试总超时只防止失败时无限等待。远端 fixture 不连接真实 SSH，Local 使用临时 settings、backup、lock 路径。
- 精确 inventory 生成测试通过。生成前备份 260 个已有 generated 文件，生成后无文件缺失，仅上述 3 个预期文件改变。命令没有删除 generated 目录。证据：`research/backend-generated-snapshot.json`、`backend-generation-evidence.json`、`backend-inventory-generation.log`。
- 比对生成前后全部 348 个 manifest 描述，只有两个命令的 `input_type` 改变。授权、确认、风险、超时、审计和并发元数据不变。

## 最终检查与冻结

- 全 registry 测试 21/21 通过，日志 `research/backend-registry-check.log`。
- 正式 `cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings` exit 0，日志 `research/backend-clippy-bin.log`。依赖 command-macros 保留已有 linker warning；没有因此更改警告等级。
- 额外 `--all-targets -- -D warnings` exit 101，4 项既有测试代码 lint：`main.rs` / `state.rs` 的 `items_after_test_module`、`claude.rs` 既有 statusLine 的 `bool_assert_comparison`、`codex.rs` 既有测试的 `await_holding_lock`。均核对 baseline `34d8a85e0e48b793733835e0304c8ed33940fcee` 中已有对应源码，没有扩大本补充清理范围。原日志 `research/backend-clippy.log` 保留；不能把 bin 检查描述为 all-targets 通过。
- `just fmt-check` exit 0（含 workspace/Tauri fmt 与 5 项 JSON formatter 测试）、`git diff --check` exit 0。日志 `research/backend-fmt-check.log`、`backend-diff-check.log`。
- 生成窗口和 Cargo 窗口已经释放。产品与测试已冻结；最终文件 SHA 和 gate 命令见 `research/backend-verification.json`。
- 前端 domain/Settings adapter 与 session 行为由 `/root/implement_t09` 接入并验证；本报告不替代该验证或独立审查。

## 未验证边界

未执行真实 SSH/WSL 保存、原生 WebView、跨平台 OS 权限验收或全量 bindings guard。本次不改变远端文件传输的并发/原子协议，不对跨请求同 ID 环境配置修改提供 revision CAS。完整父任务 gates 由 T10 负责。没有提交、推送、归档或任务状态变更。
