# 实施计划：导入身份校验

前置：P1通过、按序P5评估完成。用户已确认拒绝冲突输入、保留现有加密导出规则。

白名单：`crates/ccr-codex/src/services/codex_auth_service.rs`、P1共享身份helper的必要校验接口、`codex_auth_refresh_lock.rs` 中同时取得原身份与输入身份锁所需的窄接口、`crates/ccr-cli/src/commands/codex/auth/import.rs`与相关测试。不扩展 DTO、通用事务、crypto 格式或 CLI 强制加密行为。

1. 全包待写条目preflight，校验账号名、JSON、token/metadata身份一致性；生成完整键。
2. 导入避免先删旧快照再原子替换；输入校验失败无目录/backup/snapshot/registry副作用。
   预检后取得 P1 凭据操作锁并复核目标，覆盖本次替换和注册表提交；输入拒绝发生在锁文件/目录创建前。补刷新与force导入交叉回归。
3. CLI错误传播；encrypted/plaintext同一校验入口。
4. 测试一致/不一致、缺claims、API/metadata-only、Merge/force/Replace、valid-first-invalid-second与加密输入，保留既有计数语义。
5. 写 `research/encrypted-export-assessment.md`，确认当前CLI加密默认及TUI/Tauri缺失入口。
6. 运行Codex/CLI相关测试与strict clippy/fmt，独立check；父任务执行最终workspace正式门与just ci。仅合成数据，不提交/归档。
