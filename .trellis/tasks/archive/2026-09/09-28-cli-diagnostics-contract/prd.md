# T04：CLI 诊断结果与退出码契约

## 目标与用户价值

CLI 诊断准确区分状态，认证模式校验与实际 apply 一致，脚本可依赖退出码。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：CLI-05、CLI-06、CLI-09、CLI-10。
- 代码证据：`crates/ccr-cli/src/commands/lifecycle/validate.rs:255-307`；`crates/ccr-cli/src/platforms/claude.rs:339-345`；`crates/ccr-config/src/models/platform.rs:115-120`；`crates/ccr/tests/commands/validate.rs:125-161`。

## 范围和兼容约束

- 覆盖：crates/ccr-cli/src/commands/lifecycle/validate.rs；crates/ccr-cli/src/services/validate_service.rs；crates/ccr-config/src/managers/config_validator.rs；crates/ccr/src/main.rs；crates/ccr-cli/src/commands/doctor_cmd.rs；相关平台能力枚举和诊断测试。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：诊断 severity 在 binary 边界决定退出码，错误报告不得返回成功退出。
- R2：平台及 auth-mode 校验使用与 apply 相同的领域规则。
- R3：诊断只读，平台能力说明与实际可达入口一致。

## 验收标准

- [ ] AC1（R1）：invalid、corrupt、unreadable fixture 的 ccr validate 非零退出；合法配置和 warning-only 按已记录策略退出。
- [ ] AC2（R2）：合法 subscription 与 API-key fixture 在 validate/apply 预检中一致；缺失、损坏和权限错误分类不同。
- [ ] AC3（R3）：诊断前后 fixture 文件不变；doctor 覆盖当前支持的 Claude/Codex/Grok，保留 adapter 的支持状态明确。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
