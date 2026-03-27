# 入口选择

本页只说明当前支持的入口，不再把已移除的内置 Web API / `ccr web` 当成可用选项。

## 当前入口

| 入口 | 角色 | 适合场景 |
|---|---|---|
| `ccr <command>` | 主 CLI 入口 | 自动化、脚本、精确命令操作 |
| `ccr` | 默认 TUI 入口 | 终端内交互式切换配置 |
| `ccr ui` | 推荐图形入口 | 浏览模块、集中管理、日常图形化操作 |
| `ccr-ui` | 独立 UI 工程 | 前端开发、Tauri 桌面调试 |

## 如何选择

### 选 CLI

适合这些情况：

- 需要稳定的脚本接口
- 只想执行一个明确命令
- 要把 `ccr` 接入 shell alias、CI 或自动化任务

常见入口：

```bash
ccr platform list
ccr switch <name>
ccr sync all status
ccr sessions list
```

### 选 TUI

适合这些情况：

- 你在纯终端环境里工作
- 主要任务是浏览并切换配置
- 想用键盘快速在配置之间来回切换

默认构建下，直接运行：

```bash
ccr
```

详细行为见 [`tui 模式说明`](/reference/commands/tui)。

### 选 CCR UI

适合这些情况：

- 需要跨多个能力面浏览状态和配置
- 需要 skills、monitoring、sessions、statusline、checkin 等可视化模块
- 需要本地 `ccr-ui/` checkout 的开发体验

启动方式：

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

## 入口边界

- `ccr` 是事实源，命令定义在 `crates/ccr/src/cli/definitions.rs`
- `ccr ui` 只是图形入口，不是第二套配置系统
- `ccr-ui` 是工程目录名，不是推荐给普通用户直接记忆的主命令

## 相关页面

- [`CLI 工作流`](/guide/cli-workflows)
- [`UI 概览`](/guide/ui-overview)
- [`UI 模块地图`](/guide/ui-modules)
- [`tui 模式说明`](/reference/commands/tui)
