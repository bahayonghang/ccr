# T11：桌面控制命令并发与 OAuth 生命周期

## 目标与用户价值

控制命令不被无关长操作阻塞，OAuth 登录的 listener、pending、网络和取消具有同一完成责任者。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：TA-05、TA-07、TA-08 typed outcome。
- 代码证据：`ccr-ui/src-tauri/src/commands/handler_registry.rs:257-263`；`ccr-ui/src-tauri/src/commands/codex_auth.rs:595-599`；`ccr-ui/src-tauri/src/commands/codex_auth.rs:671-680`；`ccr-ui/src-tauri/src/commands/codex_auth.rs:1426-1432`。

## 范围和兼容约束

- 覆盖：ccr-ui/src-tauri/src/commands/handler_registry.rs；ccr-ui/src-tauri/src/commands/runtime_policy.rs；ccr-ui/src-tauri/src/commands/command_exec.rs；ccr-ui/src-tauri/src/commands/install.rs；ccr-ui/src-tauri/src/commands/usage.rs；ccr-ui/src-tauri/src/commands/codex_auth.rs；已有 OAuth service/storage 与对应事件 adapter；受影响生成 manifest、权限和操作 DTO。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T05 / 09-28-safe-persistence-backups；T06 / 09-28-usage-job-lifecycle

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：get/status/cancel 必须及时到达既有作业，同资源的新 start 仍受业务 admission 约束。
- R2：OAuth start 在返回 auth URL 前拥有真实已绑定 listener 和一致的 pending 状态。
- R3：已接受 socket、token exchange 和 reader/body 等待有真实 deadline，并可被取消。
- R4：接口重构不扩大登录权限或泄漏凭据，不改变其他命令的既有兼容格式。

## 验收标准

- [ ] AC1（R1）：逐 ID 执行 control-command-matrix.md 的 C01-C06：command、install、usage、OAuth 四个命令族的现有控制入口在 barrier 释放前到达 owner；已有 Parallel 查询保持可达。后台 start 已返回时，资源 admission 仍由真实 owner 持到完成与清理；同资源第二次 start 被拒绝、排队或按现有协议幂等复用同一 job，不创建并发作业。不存在的 get/status 操作明确标为不适用；ACL、确认和资源互斥保持。
- [ ] AC2（R2）：端口占用、bind 或 pending 保存失败均不返回可用 URL，不留下虚假活动登录；每个 login ID 仅一个终态。
- [ ] AC3（R3）：silent socket、慢 HTTP body 和取消均在期限加宽限内释放 listener/pending；失败和取消不会伪造登录成功。
- [ ] AC4（R4）：受影响 generated artifacts 无漂移，auxiliary-window ACL/确认测试不变，verifier/token sentinel 不进入事件或日志。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
