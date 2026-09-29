# T07：命令工作台任务恢复与事件生命周期

## 目标与用户价值

用户离开并返回命令页后可以继续观察和取消正在运行的命令，历史不遗漏终态。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：F01、F02、F03。
- 代码证据：`ccr-ui/src/features/commands/useCommandsPage.ts:49`；`ccr-ui/src/features/commands/useCommandsPage.ts:99-179`。

## 范围和兼容约束

- 覆盖：ccr-ui/src/features/commands/；ccr-ui/src/shell/eventBridge；命令 stream store、query keys 与 history owner；现有 command job status domain wrapper。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T11 / 09-28-desktop-control-oauth-lifecycle

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：后台任务状态由外壳级 owner 保存，路由页面只选择和呈现。
- R2：终态和历史只处理一次，事件丢失后可以从 snapshot 恢复。
- R3：异步监听在路由卸载前后均可清理。

## 验收标准

- [ ] AC1（R1）：任务运行中离开命令页再返回，job ID、状态、输出和取消能力恢复。
- [ ] AC2（R2）：终态先于 start 响应、重复/乱序事件均不回退终态或重复写 history；旧 job 事件不能覆盖新 job，提交期间禁止重复启动。
- [ ] AC3（R3）：listen Promise 在卸载后才 resolve 时仍调用 unlisten，重新进入不会新增重复监听。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
