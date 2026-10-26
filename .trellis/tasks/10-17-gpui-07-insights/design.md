# GPUI 07：用量与会话观测：设计

## Change Boundary

- ccr-desktop/src/features/insights/、sessions/、monitoring/
- crates/ccr-app/src/usage/、sessions/、jobs/
- ccr-ui/src-tauri/src/services/usage.rs、home_insights.rs、agent_sessions.rs、llmusage_adapter/、usage_jobs.rs 的提取部分

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

先提取 State-free 计算到中立层，保留 Tauri wire DTO。cx/entity 更新只接收 snapshot；分页查询在 bounded blocking worker，stale result 有 generation。图表使用 GPUI 原生绘制，点数降采样但数值查询不丢精度；关闭页面停止订阅，不丢失 app-owned import。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
