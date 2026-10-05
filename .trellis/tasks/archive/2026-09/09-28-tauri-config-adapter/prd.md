# T03：Tauri 配置接口与 CLI 呈现层解耦

## 目标与用户价值

配置页切换和启用恢复可用，并通过显式平台的共享业务接口完成。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：TA-01、TA-02、CLI-07。
- 代码证据：`ccr-ui/src/features/configs/hooks/useConfigsPage.ts:55`；`ccr-ui/src/api/generated/config.ts:26`；`ccr-ui/src-tauri/src/commands/config.rs:200-204`；`crates/ccr-cli/src/commands/profile/switch.rs:24-26`。

## 范围和兼容约束

- 覆盖：ccr-ui/src-tauri/src/commands/config.rs；ccr-ui/src-tauri/src/commands/handler_registry.rs；ccr-ui/src/api/domains/config.ts；ccr-ui/src/api/tauri.ts 的兼容委托；ccr-ui/src/features/configs/；同源生成的 config IPC 产物。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：配置页的 switch 与 enable 使用真实共享用例，不再调用永久失败的 legacy_switch_error 路径。
- R2：config 的 add、update、rename、duplicate、delete 统一委托 T01，并严格处理 patch 字段。
- R3：保留 registry、ACL、确认与生成客户端单一来源，旧无平台请求不得猜测操作对象。

## 验收标准

- [ ] AC1（R1）：隔离 fixture 中从配置页发起切换和启用均成功，runtime 与当前 profile 一致。
- [ ] AC2（R2）：未知 key、错误类型、目标缺失和重名均拒绝且无写入；并发独立修改不丢失。
- [ ] AC3（R3）：命令清单和客户端生成无漂移；旧请求缺乏可确定平台时返回明确 platform_required 或兼容迁移错误，且不修改状态。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
