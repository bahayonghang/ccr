# GPUI 08：命令、同步与签到：设计

## Change Boundary

- ccr-desktop/src/features/jobs/、sync/、checkin/
- crates/ccr-app/src/jobs/、sync/、checkin/
- ccr-ui/src-tauri/src/commands/command_exec.rs、install.rs、sync.rs、checkin.rs、checkin_jobs.rs 的实际共享提取

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

复用 ccr-core ManagedProcess、ccr-sync、ccr-checkin；UI 仅发 typed action。runner 完成 process wait/reap/reader cleanup 才写 terminal，cancel action 只请求取消。snapshot query 用于先订阅后恢复，关闭 UI 不 drop 中途持久化操作。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
