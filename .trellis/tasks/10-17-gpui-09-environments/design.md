# GPUI 09：执行环境与桌面集成：设计

## Change Boundary

- ccr-desktop/src/platform/、shell/、runtime/
- crates/ccr-app/src/environments/ 的框架无关部分
- ccr-ui/src-tauri/src/platform/、ssh/、desktop_shell.rs 与必要 adapter

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

提取 ExecutionEnvironment 及必要实现，移除 ts/tauri 耦合由 adapter 映射。托盘/通知只在实际需要时选最新兼容 crate，不预先堆多个窗口栈。共享业务数据沿原路径，GPUI UI prefs 独立命名空间。原生桌面非浏览器自动化证据。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
