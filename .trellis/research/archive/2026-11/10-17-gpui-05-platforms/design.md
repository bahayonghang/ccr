# GPUI 05：平台设置与认证：设计

## Change Boundary

- ccr-desktop/src/features/platforms/、features/auth/
- crates/ccr-app/src/auth/、platform_settings/ 的实际共享编排
- ccr-ui/src-tauri/src/commands/claude_auth.rs、codex_auth/、grok.rs、codex_settings.rs 等必要 adapter

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

现有 auth owner 与 typed settings service 负责状态。浏览器登录使用已存在的外部浏览器/callback 契约；回调端口、state/nonce、cancel 规则复用，不能通过通用 shell 字符串启动。只为实际差异创建平台策略，避免一个巨大 render 按平台分支。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
