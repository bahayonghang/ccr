# T01：配置仓储并发一致性与纯查询

## 目标与用户价值

所有配置调用者对同一资源使用一致的读改写边界，列表和诊断不隐式修改文件。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：CLI-01、CLI-06、TA-02。
- 代码证据：`crates/ccr-cli/src/platforms/claude.rs:258-262`；`crates/ccr-config/src/platforms/base.rs:482-504`；`crates/ccr-config/src/managers/config/manager.rs:31-46`；`crates/ccr-config/src/managers/config/manager.rs:89-108`。

## 范围和兼容约束

- 覆盖：crates/ccr-config/src/managers/config/；crates/ccr-config/src/services/config_service.rs；crates/ccr-config/src/platforms/base.rs；crates/ccr-cli/src/platforms/claude.rs；相关平台 writer 的委托调用。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

无子任务前置；仍须用户批准规划后才可实施。

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：同资源的读取、业务变更、校验和保存必须属于同一跨进程 RMW 临界区。
- R2：所有新业务调用必须显式提供平台；纯查询不得 bootstrap、autofix 或回写 current marker。
- R3：字段校验、冲突、current/default 和 secret/backup 规则在各 writer 一致。

## 验收标准

- [ ] AC1（R1）：两个独立进程经不同 adapter 并发修改不同 profile 后，两项修改都存在。
- [ ] AC2（R2）：registry 排序变化不改变指定平台结果；list/current/validate 前后文件清单与内容相同。
- [ ] AC3（R3）：非法 patch、重名和版本冲突均返回明确错误且磁盘不变；合法更新保留未编辑字段。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
