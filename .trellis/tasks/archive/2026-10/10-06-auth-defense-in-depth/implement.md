# 实施计划：权限与脱敏

前置：P3 独立验收通过。按当前用户顺序实施授权补齐原草案行为；不扩大加密/UI协议。

白名单：`crates/ccr-core/src/core/{atomic_writer,guarded_write}.rs` 及必要 mod 导出；`crates/ccr-codex/src/{utils.rs,models/codex_auth.rs,services/codex_auth_service.rs,services/codex_oauth_token_service.rs,services/codex_quota_service.rs,services/codex_runtime_service.rs,services/openai_quota_core.rs}`；权限 Result 传播所需 `codex_registry_store.rs`、`codex_model_provider_store.rs`；Codex Auth TUI 真正受影响的 ui/app/i18n 固定文本与断言。runtime service 限于凭据/配置 Debug 脱敏和必要权限错误传播，不改变 runtime 提交协议。

正式六尺寸 EN/ZH 矩阵位于 `crates/ccr-tui/src/tui/ui.rs`，该文件仅添加相关测试状态和文字/颜色断言。

1. 新增独立 owner-only 权限 API，保留现有 preserve-DACL 合同；Windows宽DACL→当前SID，验证bytes/mtime/file identity，缺失与版本冲突。
2. utils 复用新API，调整必要调用者；Unchanged执行侧版本校验元数据加固。
3. rename回退用私有原子写；注入rename失败验证回退、发布前写入失败与源保留。
4. 认证模型与原始auth map Debug脱敏，测试secret/identity marker不出现。
5. HTTP状态+allowlisted code固定错误，测试任意响应体/unknown code/Unicode不泄漏，force refresh、repair、relogin分类保持。
6. 运行 core 相关测试、`cargo test -p ccr-codex --all-features -- --skip export_bindings`、`cargo test -p ccr-tui --all-features -- --skip export_bindings`、scoped clippy、fmt。正式TUI EN/ZH六尺寸矩阵：80x24、100x22、100x30、120x22、140x40、180x50，并保留紧凑尺寸退化检查。
7. 独立Trellis check，写回core/Codex/TUI相关spec与实际平台边界。workspace lint/test/ci由父任务统一运行。仅合成临时数据，不提交或归档。
