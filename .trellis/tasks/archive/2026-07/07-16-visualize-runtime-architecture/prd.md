# 可视化 CCR 运行时架构

## Goal

基于仓库当前实现，生成一份可独立打开和发布的 Archify 架构图，让维护者能在一张图中识别 CCR 的运行时入口、核心组件、主要数据流、外部依赖和信任边界。

## Background

- CCR 当前包含 13 个 Rust workspace crate、Vue 3 + Tauri 桌面端和 VS Code 扩展；仓库已有文字与 Mermaid 架构说明，但缺少同时表达运行时关系和安全边界的可交互交付物（`docs/reference/architecture.md:1`）。
- CLI/TUI、Tauri 桌面端和 VS Code 扩展共享本机 CCR 配置事实源；桌面端不经过内置 HTTP API，而是通过 Tauri invoke 直接调用 Rust handlers（`docs/reference/architecture.md:60`）。
- Usage 路径同时涉及 `llmusage` 子进程和只读 SQLite 投影，是需要明确区分读写责任的外部边界（`ccr-ui/src-tauri/src/llmusage_adapter/mod.rs:1`、`crates/ccr-usage/src/db.rs:222`）。

## Requirements

- 使用用户指定的 Archify `architecture` 模式生成单文件 HTML，保留深浅主题切换和 PNG/JPEG/WebP/SVG 导出能力。
- 展示三类用户入口：CLI/TUI、Tauri 桌面端（Vue + Rust backend）和 VS Code 扩展。
- 将 Rust workspace crate 按运行时职责聚合为入口编排、共享领域/契约和持久化能力，避免把构建期依赖图误画成运行时服务拓扑。
- 清楚标出主要数据流：Tauri IPC、VS Code 到 `ccr` 子进程、配置读写、SQLite 访问、`llmusage` 同步事件、WebDAV 同步和外部签到/API 请求。
- 清楚标出至少四类信任边界：本机用户会话、本机进程边界、敏感本地数据边界、外部网络/第三方执行文件边界。
- 外部依赖按运行时角色归类展示，包括 AI CLI/runtime 文件、`llmusage`、WebDAV 服务和签到/供应商 API；不把普通编译依赖伪装成独立运行时组件。
- 图中不得包含真实 token、Cookie、账号、主机名或机器专属绝对路径。
- 最终 HTML 放在 `docs/public/architecture/ccr-runtime-architecture.html`，由 VitePress 作为静态资源原样发布，也可直接在浏览器打开。

## Out Of Scope

- 修改 CCR 运行时实现、依赖关系或安全策略。
- 为每个 command、Tauri handler、Vue 页面或全部第三方 crate 绘制节点。
- 生成多张独立流程图或重写现有中英文架构文档。

## Acceptance Criteria

- [x] `docs/public/architecture/ccr-runtime-architecture.html` 是自包含、可直接打开的 Archify HTML，并可切换深浅主题和使用导出菜单。
- [x] 主图同时覆盖运行时组件、关键数据流、外部依赖和信任边界，且主路径清晰、无交叉线遮挡核心节点。
- [x] CLI/TUI、Tauri、VS Code、`~/.ccr`、桌面 SQLite、`~/.llmusage/llmusage.db`、`llmusage` CLI、WebDAV 和外部服务均被准确定位。
- [x] 图中明确说明 `ccr-usage` 对 llmusage DB 为只读，而同步写入由外部 `llmusage` CLI 负责。
- [x] Archify schema/layout validation 与生成后 HTML check 均通过。
- [x] 最终 diff 只包含本任务规划文件、任务内 Archify 源 JSON 和目标 HTML，没有无关代码或生成物。
