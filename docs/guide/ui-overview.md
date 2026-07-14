# UI 概览

CCR UI 是共享 `~/.ccr/` 配置与运行状态的 Vue 3 + Tauri 图形界面。它与 CLI/TUI 使用同一组 Rust domain crate，不是独立的配置系统。

## 启动入口

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

默认前端端口为 `15173`，后端端口为 `38081`。`ccr ui` 依次查找开发 checkout、`~/.ccr/ccr-ui/` 安装，并在缺失时进入下载或更新流程。

开发 `ccr-ui` 时使用：

```bash
cd ccr-ui
bun run dev:web -- --host 127.0.0.1 --strictPort
```

需要原生窗口和 Tauri invoke 时使用 `bun run tauri:dev`。纯浏览器预览可以验证路由和展示，但不能完成所有桌面命令。

## 与其他入口的关系

| 入口 | 适用场景 |
|---|---|
| `ccr <command>` | 自动化、精确参数、脚本和诊断输出 |
| `ccr` 无子命令 | 终端内的快速 profile/auth 操作 |
| `ccr ui` | 可视化配置、平台管理、usage、monitoring 和桌面工具 |
| `ccr-ui/` checkout | UI 开发、测试和 Tauri 构建 |

## 当前能力面

- 平台工作区：Claude Code、Codex、Antigravity CLI、OpenCode。
- 配置与扩展：profiles、auth、settings、MCP、agents、slash commands、plugins、hooks、output styles、statusline、skills。
- 数据与运营：usage、monitoring、budget、pricing、check-in。
- 工具与环境：commands、converter、WebDAV sync、WSL、SSH。

Factory Droid 已在 CLI platform domain 实现，Qwen 保留为部分支持/stub；当前路由没有为它们提供独立平台主页。平台支持状态以 [平台支持](/reference/platforms/) 为准，UI 页面以 [UI 模块地图](./ui-modules) 为准。

## 数据边界

```text
Vue view/store
  -> src/api/domains/*
  -> Tauri invoke
  -> src-tauri/src/commands/*
  -> workspace domain crate
```

`src/api/tauri.ts` 是 legacy compatibility facade。新增前端业务 API 应进入 domain module。Usage 数据通过 `ccr-usage` 和桌面 llmusage adapter 读取，不由 Vue 直接解析 transcript。

## 选择 UI 的场景

优先使用 UI：

- 需要跨多个平台工作区浏览与比较状态；
- 需要查看 usage、monitoring、check-in 或成本面板；
- 需要集中管理 MCP、agents、skills、plugins 和同步资产。

优先使用 CLI：

- CI 或 shell automation；
- profile/auth 的可重复脚本操作；
- 需要 JSON 或明确退出码的诊断流程。

## 相关页面

- [入口选择](./entrypoints)
- [UI 模块地图](./ui-modules)
- [`ui` 命令](/reference/commands/ui)
- [架构](/reference/architecture)
