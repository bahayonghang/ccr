# GPUI 06：MCP 与扩展资源：设计

## Change Boundary

- ccr-desktop/src/features/resources/
- crates/ccr-app/src/resources/（实际共享逻辑）
- ccr-ui/src-tauri/src/commands/unified_mcp.rs、mcp_presets.rs、claude_*、codex_*、gemini.rs、opencode.rs 的目标 adapter

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

同类 list/detail/editor 复用原生组件；serialization、验证和安装策略留领域 owner。新增共享编排仅针对两个 adapter 的实际共用部分。后台安装用第 02 的任务生命周期；未知字段保持而非全量覆盖。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
