# 命令参考

本页按职责分组列出 CCR 当前所有**公开规范顶层命令**。嵌套子命令参数与 JSON 输出契约请查阅对应命令页或运行 `ccr help` 查看。

## 帮助（`ccr help`）

```bash
ccr --help
ccr help
ccr help claude profile
ccr <command> --help
```

`ccr help [COMMAND_PATH...]` 是公开顶层命令，支持直接解析多级嵌套子命令路径（如 `ccr help codex profile import`），不单独占用参考页。

## 平台与初始化

| 命令 | 用途 |
|---|---|
| [`platform`](./platform) | 查看可识别平台注册表状态（`ccr platform list [--json]`）与历史路径迁移对照 |
| [`claude`](./claude) | Claude 官方订阅账号快照（`auth`）与 Profile 目录/运行时路由（`profile` 13 动作） |
| [`codex`](./codex) | Codex 官方账号管理（`auth`）、Profile 路由（`profile`）、配额、会话垃圾箱、`fix` 与 `sync-history` |
| [`grok`](./grok) | Grok Build 官方 OAuth 账号（`auth`）与模型/第三方 Provider Profile 路由（`profile`） |
| [`project init`](./project-init) | 通过四段式向导在当前仓库初始化 Git、工作流引擎（`mini-trellis` / `trellis`）、忽略规则与本地 `ref/` |

用户级平台 Profile 目录通过 `ccr <claude|codex|grok> profile init` 按平台独立初始化；仓库级工作流通过 `ccr project init` 初始化，两者职责完全独立。

## 状态、诊断与临时覆盖

| 命令 | 用途 |
|---|---|
| [`current`](./current) | 并列查看 Claude 与 Codex 运行态总览（可见别名：`status`、`show`） |
| [`validate`](./validate) | 跨平台纯只读校验注册表与 Claude/Codex/Grok 配置及运行态（支持 `--json`） |
| [`doctor`](./doctor) | 聚合本地环境、平台配置、当前 Profile、认证状态与可选 `--online` 探活 |
| [`check`](./check) | 检测跨平台环境变量与配置冲突（`ccr check conflicts`） |
| [`temp`](./temp) | 交互式快速临时配置覆盖（不写入持久 Profile 目录） |
| [`temp-token`](./temp-token) | 命令式临时 Token 覆盖管理（`set` / `show` / `clear`） |

## 数据、同步与会话

| 命令 | 用途 |
|---|---|
| [`history`](./history) | 查看脱敏配置操作审计历史 |
| [`clean`](./clean) | 清理规划文件（`planfiles`）、过期备份（`backups`）与可重建存储缓存（`storage`） |
| [`sync`](./sync) | 基于 WebDAV 的配置资产同步（`config` / `status` / `push` / `pull`） |
| [`sessions`](./sessions) | 跨 CLI 会话索引、搜索、恢复与统计 |
| [`provider`](./provider) | Provider 端点连通性测试与密钥有效性验证 |

Profile 交换导入导出请使用各平台规范命令 `ccr <claude|codex|grok> profile export` 与 `ccr <claude|codex|grok> profile import`。

## 界面、成本与维护

| 命令 | 用途 |
|---|---|
| [`ui`](./ui) | 启动或更新 CCR UI（Web / Tauri 图形工作台） |
| [`tui`](./tui) | 终端交互界面（无子命令 `ccr` 及平台 TUI 入口说明） |
| [`stats`](./stats) | Token 用量与成本汇总、导入、导出与清理 |
| [`budget`](./budget) | 每日/每月 API 成本预算监控与设置 |
| [`pricing`](./pricing) | 自定义模型输入/输出定价表管理 |
| [`skills`](./skills) | Agent Skills 来源扫描、安装与清单管理 |
| [`prompts`](./prompts) | 系统提示词预设（Prompts）管理与应用 |
| [`update`](./update) | 检查或安装 CCR 版本更新 |
| [`version`](./version) | 显示详细版本号、构建信息与特性列表（别名：`ver`） |

## 推荐起步

```bash
ccr claude profile init                     # 初始化用户级 Claude Profile 目录（或 codex / grok）
ccr project init                            # 当前项目工作流向导（默认推荐 mini-trellis）
ccr project init --workflow mini-trellis    # 显式指定项目工作流引擎
ccr status                                  # 查看跨平台运行态总览（ccr current 别名）
ccr claude profile list                     # 查看 Claude Profiles
ccr codex auth current                      # 查看 Codex 官方账号状态
ccr grok profile list                       # 查看 Grok Profiles
ccr validate                                # 跨平台只读配置校验
ccr doctor                                  # 综合环境体检
```

## 已弃用与退休入口（7.x 兼容）

以下 11 个顶层旧入口已在 `7.x` 中从 `--help` 隐藏（调用时输出弃用警告或退休迁移错误，计划于 `8.0` 移除），仅供存量脚本迁移对照：

| 旧入口 | `7.x` 状态 | 规范替代命令 |
|---|---|---|
| [`switch`](./switch) | 已退休（直接报错） | `ccr <claude\|codex\|grok> profile switch <name>` |
| [`init`](./init) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile init` / `ccr project init` |
| [`list`](./list) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile list` / `ccr current` |
| [`add`](./add) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile create <name>` 或交互式 `ccr` |
| [`delete`](./delete) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile delete <name>` |
| [`enable`](./enable) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile enable <name>` |
| [`disable`](./disable) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile disable <name>` |
| [`clear`](./clear) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile off` |
| [`optimize`](./optimize) | 已弃用并隐藏 | SQLite `profiles.db` 自动维护规范顺序；手改投影用 `ccr <平台> profile open` |
| [`export`](./export) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile export` |
| [`import`](./import) | 已弃用并隐藏 | `ccr <claude\|codex\|grok> profile import` |

## 相关页面

- [CLI 工作流](/guide/cli-workflows)
- [配置模型](/guide/configuration)
- [迁移指南](/reference/migration)
