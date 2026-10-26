# GPUI 03：Codex 设计系统与原生外壳：设计

## Change Boundary

- ccr-desktop/src/theme/、shell/、app.rs
- ccr-desktop/src/locales/（实施时按最简资源格式确定）
- ccr-desktop/tests/native_shell.rs、DESIGN.md

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

Kit 处理成熟输入/焦点/组件行为，Rust theme 映射唯一 token；不从 HTML 拼 UI。导航 scope 与 query scope 共用环境 ID。原生窗口最低 960×640；窄窗将详情改为页面内 drill-in。对屏幕阅读器和中文输入分别测试，headless layout 不替代系统 IME。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
