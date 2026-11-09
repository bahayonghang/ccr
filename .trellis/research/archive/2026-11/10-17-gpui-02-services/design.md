# GPUI 02：共享应用服务与运行时：设计

## Change Boundary

- crates/ccr-app/Cargo.toml、src/、tests/
- Cargo.toml、Cargo.lock
- ccr-desktop/src/runtime/、app.rs
- ccr-ui/src-tauri/src/commands/config/adapter.rs 及必要的服务 adapter、Cargo.toml/lock

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

复用领域 owner，只有真实两端共享编排进入 ccr-app。RequestKey(environment, platform, generation) 拒绝 stale writes。typed event 与日志分流；单一 Tokio owner 在退出阶段取消/等待任务。UI shutdown 不伪造 job terminal。Tauri DTO 映射留在 adapter，避免 export_to 路径漂移。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
