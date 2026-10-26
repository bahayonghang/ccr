# GPUI 10：性能、三平台与交付验收：设计

## Change Boundary

- ccr-desktop/benches/、tests/、justfile、README.md、打包配置
- justfile、scripts/ci/ci_surface_policy.py、scripts/drift/check_dependency_drift.py 及对应测试
- 必要 .github/workflows/ 新桌面 lane；既有 context 名不变
- 父任务 research/ 与本任务 evidence.md

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

release fixtures + 指标采集脚本输出统一 JSON/Markdown；command graph 回归验证新命令不串台且错误退出码透传。CI relevance 增加 desktop surface 的显式触发，独立构建锁文件审计；仅在完整验收后判定可替代 Tauri。签名/发布作为外部操作另行授权，不阻塞本地可审阅产物。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
