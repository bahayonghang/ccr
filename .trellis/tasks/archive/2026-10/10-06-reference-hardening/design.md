# 设计：参考加固审计框架

## 审计维度表（每维：CCR 现状锚点 / codex-auth 做法 / 风险 / 修复类别）

| 维度 | CCR 现状 | codex-auth | 初判类别 |
| --- | --- | --- | --- |
| 备份策略 | 每次写前备份（runtime_switch 标签），保留为 auth 前缀共享池 10 个（跨标签共享，codex_config.rs:41、:271-299） | 仅内容变化时备份，保留 5 | A（去重命中返回既有相同备份路径，保持回滚契约） |
| 权限加固 | 写路径 ensure_private_permissions（codex_auth_service.rs:908、:1509、:1964 等）；读取/无变化路径覆盖度待查 | 每次接触 hardenSensitiveFile | A |
| 原子写 | AtomicWriter 覆盖快照/注册表/配置 | replaceFilePreservingPermissions | A（查缺） |
| 身份模型 | account_id（codex_auth_service.rs:714、codex_oauth_token_service.rs:333） | user_id::account_id | B（后续任务提案） |
| 注册表健壮性 | toml serde（codex_registry_store.rs）；未知字段保留待查 | schema_version 迁移门 | A/B 待判 |
| 导入/导出 | export_accounts / encrypted / import（codex_auth_service.rs:1700 起） | import/export/--cpa、密钥指纹 | A（查脱敏） |
| 日志脱敏 | spec 要求不记录 tokens（ccr-codex backend-guidelines:93） | — | A（查缺口） |
| 清理/保留 | backup_current_auth 为 dead_code（:1223）；备份清理按前缀 | clean 白名单 | A |
| 文件命名 | 账号名直用（validate_account_name） | record_key 派生 + base64url | B（后续任务提案） |

## 类别定义

- A：不改变运行逻辑与用户可见文本（仅日志级文本可调整）；文件副作用级变更（如备份去重）须保持 commit_plan 回滚契约并在报告显式说明。
- B：行为变化候选（含用户可见文本变更、保留池缩减、身份模型）→ 转为后续任务提案（位置/证据/建议）；本任务不实施。

## 证据要求

- 每条 finding：file:line + 代码摘录 +（可行时）合成实验输出；分级 高/中/低。
- 报告落点：research/codex-auth-audit.md（维度表 + finding 列表 + 分级 + 修复映射）。

## 约束

- 修复执行放在子任务 1 合并之后（同文件冲突风险：codex_auth_service.rs、codex_oauth_token_service.rs、codex_config.rs）。
- 不改 ccr-types 公共契约；不新增错误变体。
