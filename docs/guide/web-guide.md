# Legacy Web 迁移说明

`crates/ccr` 内置的 legacy Web API / Web UI 已移除，不再作为受支持入口。

## 现在应该使用什么

| 入口 | 角色 | 适合场景 |
|------|------|----------|
| `ccr` | CLI / TUI 主入口 | 脚本、自动化、日常命令操作 |
| `ccr ui` | 推荐图形入口 | 日常图形化管理、模块浏览 |
| `ccr-ui` | 独立图形应用工程 | 前端开发、Tauri 桌面运行 |

## 迁移建议

- 原先依赖 `ccr web` 做图形管理：改用 `ccr ui`
- 原先依赖内置浏览器页面：改用 `ccr-ui` 的前端/Tauri 形态
- 原先依赖命令自动化：继续直接调用 `ccr` CLI

## 相关页面
- [`UI 概览`](/guide/ui-overview)
- [`UI 模块地图`](/guide/ui-modules)
- [`ui 命令`](/reference/commands/ui)
