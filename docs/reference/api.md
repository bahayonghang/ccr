# Legacy Web API（已移除）

`crates/ccr` 中的 legacy HTTP 路由已移除。本页仅作为迁移说明保留。

## 迁移方向

- 图形界面：使用 [`ccr ui`](/reference/commands/ui) 启动独立 `ccr-ui`
- 桌面集成：使用 `ccr-ui/src-tauri` 的 Tauri IPC 命令
- CLI 自动化：继续使用 `ccr` 命令本身

## 说明

- 旧的 `/api/*` 路由不再作为受支持接口提供
- 仓库中残留的 `src/web/**` 目录仅待物理删除，不再参与编译
- 若需要了解当前 UI 入口，请查看 [UI 概览](/guide/ui-overview)
