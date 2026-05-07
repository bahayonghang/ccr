# 入口选择

本页只说明当前仍受支持的入口。内置 Web API、`ccr web`、旧的全局平台切换心智模型都不再是推荐主路径。

## 当前入口

| 入口 | 角色 | 适合场景 |
|---|---|---|
| `ccr <command>` | 主 CLI 入口 | 自动化、脚本、精确命令执行 |
| `ccr` | 默认 TUI 入口 | 在终端内交互式浏览和切换 |
| `ccr ui` | 推荐图形入口 | 浏览模块、查看状态、日常图形化管理 |
| `ccr-ui` | 独立 UI 工程目录 | 前端开发与 Tauri 桌面调试 |

## 如何选择

### 选择 CLI

```bash
ccr current
ccr claude profile list
ccr codex auth list
ccr sync all status
```

### 选择 TUI

```bash
ccr
```

### 选择 CCR UI

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

## 边界说明

- `ccr current` 是当前的运行时总览入口。
- `ccr claude profile ...` 与 `ccr codex profile ...` 是当前的 auth/profile 主路径。
- `ccr platform list` 仍保留为注册表兼容视图。
- `ccr switch <name>`、`ccr <name>`、`ccr platform switch/current/...` 已退休，只返回迁移指引。

## 相关页面

- [`CLI 工作流`](/guide/cli-workflows)
- [`配置模型`](/guide/configuration)
- [`UI 概览`](/guide/ui-overview)
- [`tui 模式说明`](/reference/commands/tui)
