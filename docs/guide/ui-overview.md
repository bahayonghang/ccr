# UI 概览

`ccr ui` 是 CCR 推荐的浏览器与桌面入口。它不是 `ccr web` 的皮肤，而是独立的 Vue 3 + Axum + Tauri 应用。

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

### 浏览器模式
- 前端运行在本地 HTTP 端口
- 后端通过 Axum 提供 API
- 适合日常管理、演示、模块浏览

### 桌面模式
- UI 工程同时提供 Tauri 壳
- 适合需要原生桌面窗口、系统集成时使用
- 文档主线仍以浏览器模式的 `ccr ui` 命令为主

## `ccr ui` 与 `ccr web` 的关系

| 命令 | 角色 | 适合场景 |
|------|------|----------|
| `ccr ui` | 推荐图形界面入口 | 日常可视化管理、模块导航、统一操作 |
| `ccr web` | Legacy 轻量 API | 脚本、CI、兼容场景、纯 HTTP 调用 |

## 当前 UI 覆盖的能力面

- 平台模块：Claude Code、Codex、Gemini CLI、Droid，以及保留中的 Qwen / iFlow 分组
- 配置与扩展：configs、mcp、slash-commands、agents、plugins、sessions、hooks、statusline、output-styles、provider health
- 数据与运营：usage、monitoring、budget、pricing
- 工具与环境：commands、converter、sync、checkin、opencode、WSL、SSH、skills / market

详细分组见 [`UI 模块地图`](/guide/ui-modules)。

## 什么时候优先使用 UI

优先使用 `ccr ui` 的场景：
- 需要浏览多个平台模块
- 需要看 dashboard、usage、monitoring 一类可视化信息
- 需要集中管理 skills、statusline、provider health、sessions

优先使用 CLI 的场景：
- CI / shell automation
- 快速切换、脚本化导入导出
- 对命令参数和输出格式有精确要求

## 相关页面
- [`Web 指南`](/guide/web-guide)
- [`UI 模块地图`](/guide/ui-modules)
- [`ui 命令`](/reference/commands/ui)
