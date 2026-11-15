# CLI 工作流

本页按日常任务组织 CCR 的核心 CLI 路径。当前推荐的 `auth` / `profile` 主路径围绕显式的 Claude、Codex 与 Grok 平台子命令展开，不再依赖隐式的“全局当前平台”心智模型。

## 工作流 1：初始化配置库并查看运行时状态

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
ccr current
ccr platform list
```

适用场景：
- 首次初始化 `~/.ccr/platforms/<platform>/profiles.db`
- 确认当前各平台运行时是否就绪（`ccr current`，别名 `ccr status` / `ccr show`）
- 查看平台注册表中的已启用条目与 `current_profile`（`ccr platform list [--json]`）
- 在代码仓库根目录配置项目工作流时使用 `ccr project init`

> `ccr platform list` 是注册表兼容与概览视图；实际运行时状态请看 `ccr current`。

## 工作流 2：Claude Runtime / Profile

```bash
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin

ccr claude profile open
ccr claude profile list
ccr claude profile switch work
ccr claude profile current
ccr claude profile off
```

适用场景：
- 通过 `--auth-token-stdin` 安全创建或用 `profile open` 在 `$VISUAL` / `$EDITOR` 中批量编辑 Claude 配置库
- 将选定 Profile 应用到 `~/.claude/settings.json`
- 执行 `ccr claude profile off` 退出 Profile 模式，回到 Claude 官方登录态

## 工作流 3：Codex Runtime / Profile 与多账号

```bash
# 官方登录态多账号管理
ccr codex auth save work
ccr codex auth list
ccr codex auth switch work
ccr codex auth off

# 运行时 Profile 路由管理
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create proxy \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin
ccr codex profile list
ccr codex profile switch proxy
ccr codex profile current
ccr codex profile off
```

适用场景：
- 明确区分 `codex auth`（官方登录账号快照）与 `codex profile`（`~/.codex/config.toml` + 运行期 `auth.json` 路由）
- 切换第三方中转或模型服务，并在执行 `profile off` 时清理 CCR 托管路由，为 `codex login` 准备干净环境

## 工作流 4：Grok Runtime / Profile 与官方会话

```bash
# 使用环境变量名或标准输入创建 Grok Profile
ccr grok profile create xai --env-key XAI_API_KEY
printf '%s' "$GROK_RELAY_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --api-key-stdin

ccr grok profile list
ccr grok profile switch xai
ccr grok profile off
ccr grok auth current
```

适用场景：
- 管理 `~/.grok/config.toml` 与 `~/.grok/user-settings.json` 的 Profile 路由
- 使用 `--env-key <ENV_VAR>` 引用环境变量或通过 `--api-key-stdin` 安全传入直接密钥

## 工作流 5：Profile 备份、分享与迁移（`ExchangeV1`）

```bash
# 导出脱敏配置用于团队分享
ccr claude profile export --no-secrets -o claude-team.toml

# 预演导入并按策略增量合并
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict skip
```

适用场景：
- 从 `profiles.db` 导出标准 `ExchangeV1` TOML 文件进行备份或跨机器迁移
- 使用 `--dry-run` 预演导入变更，并通过 `--on-conflict skip|overwrite|rename`（或全量替换 `--replace -y`）控制合并行为

## 工作流 6：校验与统一诊断

```bash
ccr current --verbose
ccr validate --json
ccr doctor --online
```

当前行为：
- `ccr current --verbose`：展示运行时总览与注册表目标详情
- `ccr validate [--json]`：对注册表以及 Claude / Codex / Grok 的配置与宿主设置做静态校验
- `ccr doctor [--online] [--all-platforms]`：执行统一只读体检（含 `gemini` / `droid` 保留兼容适配器的本地检查与可选在线探活）

## 工作流 7：同步、历史、清理与交互界面

```bash
ccr history --limit 50
ccr sync config
ccr sync all status
ccr clean backups --days 30 --dry-run

# 进入图形界面或终端 TUI
ccr ui -p 15173 --backend-port 38081
ccr
```

- `ccr ui`：推荐的图形界面入口
- `ccr`：默认终端交互入口（TUI）

## 升级与命令迁移

如果你之前的脚本仍在使用旧版隐式切换命令或已隐藏的顶层单文件命令，请查阅 [`迁移指南`](/reference/migration) 与 [`平台命令迁移`](/reference/platforms/migration) 获取完整对照表。

## 相关页面

- [`快速开始`](/guide/quick-start)
- [`配置模型`](/guide/configuration)
- [`入口选择`](/guide/entrypoints)
- [`命令参考`](/reference/commands/)
