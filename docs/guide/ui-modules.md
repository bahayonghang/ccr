# UI 模块地图

本页按能力分组介绍 `ccr-ui` 已暴露出来的主要模块，避免把路由树原样搬成文档。

## 平台模块

### Claude Code
- 主平台页
- settings 子页

### Codex
- 主平台页
- MCP、profiles、slash commands、auth、settings

### Gemini CLI
- 主平台页
- MCP、agents、slash commands、plugins

### Factory Droid
- 主平台页
- MCP、agents、slash commands、plugins、models、profiles、droids

### 预留分组：Qwen / iFlow
- 在 UI 中已有一级与二级分组
- 文档按 reserved / partial 说明，不把它们写成已完整交付的平台能力

## 配置与扩展模块

- `configs`
- `mcp`
- `slash-commands`
- `agents`
- `plugins`
- `sessions`
- `hooks`
- `output-styles`
- `statusline`
- `provider-health`

## 数据与运营模块

- `usage`
- `monitoring`
- `budget`
- `pricing`

## 工具与环境模块

- `commands`
- `converter`
- `sync`
- `checkin`
- `opencode`
- `wsl`
- `ssh`
- `ccr-control`

## Skills 与市场模块

- `skills`
- `skills/add`
- `market`

这部分建议与 CLI 的 `skills` / `prompts` 一起理解：CLI 负责精确操作，UI 负责浏览、筛选、安装和查看详情。

## 相关页面
- [`UI 概览`](/guide/ui-overview)
- [`CLI 工作流`](/guide/cli-workflows)
- [`Web 指南`](/guide/web-guide)
