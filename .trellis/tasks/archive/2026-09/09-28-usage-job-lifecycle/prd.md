# T06：Usage 后台任务取消与进程清理

## 目标与用户价值

Usage 同步的开始、取消、超时、清理和终态与真实子进程生命周期一致。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：TA-03、TA-04。
- 代码证据：`ccr-ui/src-tauri/src/commands/usage.rs:446-449`；`ccr-ui/src-tauri/src/commands/usage.rs:1539-1546`；`ccr-ui/src-tauri/src/commands/usage.rs:1570-1590`；`ccr-ui/src-tauri/src/llmusage_adapter/cli.rs:139-159`。

## 范围和兼容约束

- 覆盖：ccr-ui/src-tauri/src/commands/usage.rs；ccr-ui/src-tauri/src/state.rs 的 usage job 状态；ccr-ui/src-tauri/src/usage_jobs.rs；ccr-ui/src-tauri/src/llmusage_adapter/cli.rs；ccr-ui/src-tauri/src/process/gateway.rs；对应 generated DTO 和 usage 前端状态映射。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

无子任务前置；仍须用户批准规划后才可实施。

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：发布 job 前同时登记取消能力，取消请求不直接宣告进程已退出。
- R2：终态幂等，成功取消不能被普通失败覆盖，完成后取消不能改写结果。
- R3：流式执行真正执行 descriptor deadline，并限制清理和 reader join。
- R4：保留 llmusage 外部 CLI、ccr-usage 只读 SQL owner 与现有输出限制。

## 验收标准

- [ ] AC1（R1）：runner 被 barrier 暂停时立即取消，释放 barrier 后 spawn 次数为零；active 占用直到 runner 真正完成。
- [ ] AC2（R2）：Cancelled、Finished、Failed 的已终结记录不被迟到事件或重复 cancel 覆盖；清理失败可见。
- [ ] AC3（R3）：静默、EOF 后仍存活、持有 stderr 和清理失败的假进程均在配置期限加清理宽限内结束，返回真实结果。
- [ ] AC4（R4）：stdout 无换行 flood 仍受 1 MiB 单行限制；stderr 保留上限和 DTO secret 规则不退化；adapter 不新增 usage SQL。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
