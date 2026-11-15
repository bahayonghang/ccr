# 多平台配置指南

> 本页聚焦在同一个 `~/.ccr/` 注册表下并行管理多个 AI CLI 平台的推荐 CLI 路径。

## 平台能力概览

| 平台 | CLI 能力层级 | 权威配置库 | 宿主运行时设置目标 |
|------|--------------|------------|--------------------|
| **Claude Code** (`claude`) | 完整 `profile` + `auth` | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json`、`~/.claude.json` |
| **Codex** (`codex`) | 完整 `profile` + `auth` | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml`、`~/.codex/auth.json` |
| **Grok** (`grok`) | 完整 `profile` + `auth` | `~/.ccr/platforms/grok/profiles.db` | `~/.grok/config.toml`、`~/.grok/user-settings.json` |
| **Antigravity CLI** (`gemini`) | 保留兼容适配器（`legacy_adapter`） | 无 CLI `profile`/`auth` 子命令 | `~/.gemini/antigravity-cli/settings.json`（及 `/antigravity` UI） |
| **Factory Droid** (`droid`) | 保留兼容适配器（`legacy_adapter`） | 无 CLI `profile`/`auth` 子命令 | `~/.factory/settings.json` |
| **Qwen CLI** (`qwen`) | 预留 / Stub | 不预建配置库 | 预留平台键 |

## 目录结构

```text
~/.ccr/
├── config.toml                            # 平台注册表（各平台独立维护 current_profile）
├── platforms/
│   ├── claude/
│   │   └── profiles.db                    # Claude SQLite 配置库
│   ├── codex/
│   │   └── profiles.db                    # Codex SQLite 配置库
│   └── grok/
│       └── profiles.db                    # Grok SQLite 配置库
├── history/                               # 脱敏操作审计历史
├── backups/                               # 自动备份目录
└── logs/                                  # 按日轮转运行日志
```

## 快速初始化与并列管理

### 1. 初始化受支持平台的配置库

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
```

每个命令都会从内置模板初始化对应平台的 SQLite `profiles.db`，互不覆盖。

### 2. 安全创建与编辑各平台 Profile

创建或修改凭据时，请通过标准输入（`--auth-token-stdin` / `--api-key-stdin`）或环境变量名（`--env-key`）传入密钥，避免在命令行参数中暴露明文密钥：

```bash
# Claude：通过 stdin 传入密钥
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin

# Codex：通过 stdin 传入密钥
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create relay \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin

# Grok：绑定环境变量名，或通过 --api-key-stdin 传入直接密钥
ccr grok profile create xai --env-key XAI_API_KEY
printf '%s' "$GROK_RELAY_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --api-key-stdin
```

如需在 `$VISUAL` / `$EDITOR` 中一次性调整多个字段或标签，可打开配置库的临时 TOML 投影：

```bash
ccr claude profile open
ccr codex profile open
ccr grok profile open
```

### 3. 并行切换与查看运行时状态

CCR 7.x 不需要“先切换全局平台再切换配置”——各平台路由彼此独立，随时按平台显式切换：

```bash
ccr claude profile switch work
ccr codex profile switch relay
ccr grok profile switch xai

# 查看运行时总览与平台注册表
ccr current --verbose
ccr platform list
```

若要让某个平台退出 CCR Profile 路由并回到官方登录态，执行对应平台的 `profile off`：

```bash
ccr claude profile off
ccr codex profile off
ccr grok profile off
```

## 配置导出、分享与迁移（`ExchangeV1`）

使用各平台的 `profile export` 与 `profile import` 在团队或设备间同步配置：

```bash
# 导出不含密钥的 ExchangeV1 TOML 供代码评审或分享
ccr claude profile export --no-secrets -o claude-team.toml

# 在目标机器上先预演导入，再按冲突策略增量合并
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict skip
```

> 如需完全用导入文件替换目标配置库，请显式传入 `--replace -y`。

## 兼容适配器平台的日常检查

`gemini`（Antigravity CLI）与 `droid`（Factory Droid）不提供 CLI `profile` 切换表面，但可通过统一体检命令检查宿主配置文件健康度：

```bash
ccr doctor --platform gemini
ccr doctor --platform droid
ccr doctor --all-platforms
```

Antigravity CLI 的本地设置、MCP 服务器（`~/.gemini/antigravity-cli/mcp_config.json`）、斜杠命令与插件也可直接在 `ccr ui` 的 `/antigravity` 模块中图形化管理。

## 相关文档

- [平台支持概览](/reference/platforms/)
- [配置示例索引](./index)
- [故障排除指南](./troubleshooting)
- [快速开始](/guide/quick-start)
- [UI 概览](/guide/ui-overview)
