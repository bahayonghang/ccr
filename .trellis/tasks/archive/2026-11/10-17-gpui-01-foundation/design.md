# GPUI 01：独立工程与开发命令：设计

## Change Boundary

- ccr-desktop/Cargo.toml、Cargo.lock、.cargo/config.toml、src/main.rs、tests/、justfile、README.md
- justfile、.gitignore、code_map.md
- scripts/version/version-sync.ps1、version-sync.sh 及对应测试

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

独立 [workspace]，使用 gpui-kit 单一入口与 dev-only test-support。根只转发子 justfile，子命令在 ccr-desktop cwd 中执行，避免 .cargo/config 搜索差异。维护 doctor 的平台先决条件清单而不自动装软件。probe 使用 fixture 内容、不访问个人配置；先初始化 Kit/平台，再创建 Root/window。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
