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

Factory Droid 在 CLI 中作为保留兼容适配器（`legacy_adapter`）存在，Qwen 保留为部分支持/stub；当前路由没有为它们提供独立平台主页。平台支持状态以 [平台支持](/reference/platforms/) 为准，UI 页面以 [UI 模块地图](./ui-modules) 为准。

## GPUI 原生桌面的执行环境

独立的 `ccr-desktop`（`ccr-gpui`）在底部状态栏切换本地/SSH 当前执行环境，全局设置分为外观、工作台偏好、执行环境、诊断四页。平台配置、MCP、Agent 会话、命令/安装、同步和用量随所选环境读取；失败保留 SSH 主机且不回落本地。签到和全局设置与环境无关，SSH 离线仍可打开。Tauri 的 SSH 页面继续使用系统 OpenSSH，不随 GPUI 切换为 russh。

SSH 用量需要主机安装 llmusage 与带 sqlite3 的 Python3：显式导入在主机执行，报告使用包含 WAL 的只读 SQLite backup，经 SFTP 拉取最多 64MiB 的私有快照（下载最多 10 分钟），由 `ccr-usage` 按 schema/能力门控，不迁移远端数据库。缺工具、断线、快照或清理失败显示不可用，不显示本机数据或假零值。

SSH WebDAV 支持四个固定文件资产与远程 HOME 内有界普通文件夹；敏感固定资产继续使用 v2 加密。远端方案数据库/SQLite sidecar、解绑 WebDAV 和新建纯空目标目录尚不支持；多文件写回可能部分完成并保留远端备份。真实主机与跨平台验收另行验证。

## Tauri 数据边界

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
