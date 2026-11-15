# 入口选择

本页只说明当前仍受支持的入口。内置 Web API、`ccr web` 以及旧的全局平台切换心智模型均已退场，不再作为可用主路径。

## 当前入口

| 入口 | 角色 | 适合场景 |
|---|---|---|
| `ccr <command>` | 主 CLI 入口 | 自动化、脚本、精确命令执行与 JSON 诊断 |
| `ccr` | 默认 TUI 入口 | 在终端内交互式浏览和切换 Claude / Codex / Grok 配置与账号 |
| `ccr ui` | 推荐图形入口 | 浏览完整模块、查看状态与用量、日常图形化管理 |
| `ccr-ui` | 独立 UI 工程目录 | 前端开发与 Tauri 桌面调试 |

## 如何选择

### 选择 CLI

```bash
ccr current
ccr claude profile list
ccr codex auth list
ccr grok profile list
ccr project init
ccr sync all status
```

### 选择 TUI

```bash
ccr
ccr codex
ccr grok auth
```

### 选择 CCR UI

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

## 边界与历史命令迁移说明

- `ccr current`（别名 `ccr status`、`ccr show`）是当前的运行时总览入口。
- `ccr <claude|codex|grok> profile ...` 与 `ccr <claude|codex|grok> auth ...` 是当前的权威配置库（`profiles.db`）与账号管理入口。
- `ccr platform list [--json]` 保留为平台注册表查询视图。
- 已移除或弃用的旧入口对照：
  - `ccr platform switch|current|info|init|profile` 已从 CLI 解析树中移除；请改用 `ccr <claude|codex|grok> profile <动作>`、`ccr current` 与 `ccr platform list`。
  - `ccr switch <name>` 与 `ccr <name>` 快捷入口已退休并返回迁移指引；请改用 `ccr <claude|codex|grok> profile switch <name>`。
  - 顶层 `ccr init|list|add|delete|enable|disable|clear|optimize|export|import` 在 7.x 中已隐藏并标记弃用（计划于 8.0 移除）；请改用 `ccr <claude|codex|grok> profile init|list|create|delete|enable|disable|off|open|export|import`（项目级工作流初始化请用 `ccr project init`）。

## 相关页面

- [`CLI 工作流`](/guide/cli-workflows)
- [`配置模型`](/guide/configuration)
- [`UI 概览`](/guide/ui-overview)
- [`迁移指南`](/reference/migration)
- [`tui 模式说明`](/reference/commands/tui)
