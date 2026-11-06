# 分层配置元数据共享批次

基线 `96376961`。本批仅抽取 Claude/Codex/Grok 现有来源清单，Tauri 保持既有 wire shape；原生消费者另外集成。

- 共享 owner：`ccr_app::platform_settings::layers`。只投影路径、存在、大小、mtime 与只读策略，不读取配置正文或猜测合并有效值。
- 未指定项目保持 unresolved；managed/requirements/project/profile overlay 不变成可写用户配置。Codex overlays 保持确定顺序。
- 独立检查见 `batch-check-20261106.md`。实际通过：共享 layer 测试1、Tauri raw settings测试7、当时共享/Tauri严格clippy；均为该源码版本的 fixture/headless 证据。
- 原生 runtime/control 各1通过，包含 fixture 系统根与显式项目拒绝、managed只读；原生交互后由用户明确延期。

其他并行批次的最终集成门禁尚未执行。本收据不表示所有平台配置、原生视觉或完整迁移验收通过。
