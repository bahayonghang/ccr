# UI 模块地图

本页按用户任务整理当前 `ccr-ui/src/shell/routeCatalog.ts` 暴露的页面。重定向入口不会被描述成独立功能。

## 平台工作区

| 平台 | 路由 | 当前子页面 |
|---|---|---|
| Claude Code | `/claude-code` | settings、profiles、auth |
| Codex | `/codex` | MCP、profiles、agents、sessions、slash commands、auth、settings |
| Antigravity CLI | `/antigravity` | MCP、agents、slash commands、plugins；旧 `/gemini-cli` 路径重定向 |
| OpenCode | `/opencode` | providers、MCP、agents、commands、plugins、settings；skills 汇入统一 skills 页 |

Factory Droid 与 Qwen 仍属于 CLI platform domain，不应从 CLI 支持状态推断出不存在的 UI 平台主页。

## 配置与扩展

| 能力 | 路由 | 说明 |
|---|---|---|
| 配置集 | `/configs` | 浏览、筛选和管理共享配置 |
| MCP | `/mcp-manager` | 统一 MCP 管理；旧 `/mcp` 与 `/mcp/unified` 重定向到此页 |
| Slash commands | `/slash-commands` | 通用命令资源管理 |
| Agents | `/agents` | 通用列表与 detail 路由 |
| Skills | `/skills` | 统一 skills 迁移页；manager、hub、market 和 detail 旧入口重定向 |
| 扩展 | `/plugins`、`/hooks` | plugins 与 hooks 管理 |
| 输出 | `/output-styles`、`/statusline` | 输出样式和状态栏配置 |

## 数据与运营

| 能力 | 路由 | 说明 |
|---|---|---|
| Usage | `/usage` | usage dashboard；旧 `/stats` 重定向到此页 |
| Monitoring | `/monitoring` | 监控 feed；旧通用 `/sessions` 重定向到此页 |
| 成本控制 | `/budget`、`/pricing` | 预算与模型定价 |
| Check-in | `/checkin` | 账号列表、执行状态和 account dashboard |

Codex 的 session 页面仍位于 `/codex/sessions`，与已下线的通用 SessionsView 不是同一路径。

## 工具与环境

| 能力 | 路由 | 说明 |
|---|---|---|
| Commands | `/commands/ccr` 等 | 按 client 展示命令工作区；旧 `/ccr-control` 重定向 |
| Converter | `/converter` | 配置格式转换 |
| Sync | `/sync` | 固定配置资产的 WebDAV 同步控制台 |
| WSL | `/wsl` | WSL 环境管理 |
| SSH | `/ssh` | SSH 环境管理 |
| App settings | `/settings` | UI 外观、行为和诊断设置 |

## 路由维护规则

- 用户文档记录稳定能力组，不为每个内部 route 创建页面。
- route 新增、删除或改为 redirect 时，同步检查本页和双语镜像。
- 平台 capability 与 UI route 是不同契约；只有路由实际注册后才称为 UI 页面。
- 精确 CLI 参数保留在 [命令参考](/reference/commands/)，内部 Vue/Tauri 边界保留在 [架构](/reference/architecture)。

## 相关页面

- [UI 概览](./ui-overview)
- [CLI 工作流](./cli-workflows)
- [平台支持](/reference/platforms/)
