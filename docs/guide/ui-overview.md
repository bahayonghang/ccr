# UI 概览

`ccr ui` 是 CCR 推荐的图形入口。它不是旧浏览器端的皮肤，而是独立的 Vue 3 + Tauri 应用。

## 推荐使用方式

```bash
ccr ui -p 15173 --backend-port 38081
```

默认端口：
- 前端：`15173`
- 后端：`38081`

## 启动顺序

`ccr ui` 会按以下顺序寻找可运行的 UI：
1. 当前目录或父目录下的 `ccr-ui/`
2. 用户目录 `~/.ccr/ccr-ui/`
3. 首次使用时提示从 GitHub 下载

## 运行模式

### 开发预览模式
- 前端运行在本地 HTTP 端口
- 主要用于开发与调试前端界面
- 不再依赖 `crates/ccr` 中已移除的 legacy Web API

### 桌面模式
- UI 工程同时提供 Tauri 壳
- 适合需要原生桌面窗口、系统集成时使用
- 文档主线仍以 `ccr ui` 启动独立 `ccr-ui` 为主

## 与 CLI 的关系

| 入口 | 角色 | 适合场景 |
|------|------|----------|
| `ccr` | CLI / TUI / 核心管理入口 | 自动化、脚本、日常命令操作 |
| `ccr ui` | 推荐图形界面入口 | 日常可视化管理、模块导航、统一操作 |
| `ccr-ui` | 独立图形应用工程 | 前端开发、Tauri 桌面运行 |

## 当前 UI 覆盖的能力面

- 平台模块：Claude Code、Codex、Gemini CLI、Droid，以及保留中的 Qwen 分组
- 配置与扩展：configs、mcp、slash-commands、agents、plugins、hooks、statusline、output-styles
- 数据与运营：usage、monitoring、budget、pricing
- 工具与环境：commands、converter、sync、checkin、opencode、WSL、SSH、skills / market

详细分组见 [`UI 模块地图`](/guide/ui-modules)。

## 什么时候优先使用 UI

优先使用 `ccr ui` 的场景：
- 需要浏览多个平台模块
- 需要看 dashboard、usage、monitoring 一类可视化信息
- 需要集中管理 skills、statusline、配置与同步能力

优先使用 CLI 的场景：
- CI / shell automation
- 快速切换、脚本化导入导出
- 对命令参数和输出格式有精确要求

## 相关页面
- [`UI 模块地图`](/guide/ui-modules)
- [`ui 命令`](/reference/commands/ui)
