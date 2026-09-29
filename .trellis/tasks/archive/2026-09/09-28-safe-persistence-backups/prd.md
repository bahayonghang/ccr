# T05：敏感状态持久化与备份唯一性

## 目标与用户价值

每次备份有独立身份，OAuth 等敏感状态从写入开始即遵守现有安全持久化契约。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：CLI-08、TA-06、TA-07 storage。
- 代码证据：`crates/ccr-core/src/core/guarded_write.rs:264-307`；`crates/ccr-cli/src/application/profile_off.rs:82-86`；`ccr-ui/src-tauri/src/commands/codex_auth.rs:518-521`；`crates/ccr-codex/src/utils.rs:87-111`。

## 范围和兼容约束

- 覆盖：crates/ccr-core/src/core/guarded_write.rs；crates/ccr-cli/src/application/profile_off.rs；ccr-ui/src-tauri/src/commands/codex_auth.rs；crates/ccr-codex/src/utils.rs；OAuth pending 的现有共享存储边界。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

无子任务前置；仍须用户批准规划后才可实施。

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：同资源同秒多次备份不能覆盖旧前镜像。
- R2：旧格式备份继续可以发现和恢复，secret/ACL 行为不降低。
- R3：OAuth pending 凭据必须由统一 secret writer 持久化，权限设置失败必须传播。
- R4：敏感内容不得泄漏到错误、日志、事件或研究 fixture。

## 验收标准

- [ ] AC1（R1）：固定时钟下连续三次修改保留三份不同的前镜像，且轮换数遵守现有限额。
- [ ] AC2（R2）：旧/新格式混合目录的列出、恢复、轮换测试通过，敏感前镜像权限满足既有规则。
- [ ] AC3（R3）：OAuth pending 使用 secret:true 和 BackupPolicy::None；创建、替换、取消、过期清理及权限失败后，目标目录和配置的备份目录不得新增 verifier/state 的备份或历史副本。权限失败不发布不受保护的新凭据文件，handler 返回明确失败，原有目标字节保持完整。
- [ ] AC4（R4）：合成 secret sentinel 在失败、取消、过期清理的可见输出中均不存在。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
