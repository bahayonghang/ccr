# CCR 配置示例（平台模板与结构参考）

本目录提供 CCR（Claude Code Configuration Switcher）的内置平台模板与 TOML 交换结构参考。

- **支持完整 CLI / TUI Profile 与 Auth 管理的平台（`claude`、`codex`、`grok`）**：主存储为 SQLite `~/.ccr/platforms/<platform>/profiles.db`，通过 `ccr <claude|codex|grok> profile init` 从本目录内置模板初始化，日常通过 `profile create/edit/open/export/import` 管理（无需手工复制 `profiles.toml`）。
- **保留兼容适配器平台（`gemini`、`droid`）**：不提供 CLI / TUI 的 `profile` 或 `auth` 子命令；对应目录下的 TOML 文件仅作历史字段参考。

## 📁 内容一览

```
examples/
├── README.md                     # 本文件
├── claude/
│   ├── profiles.example.toml     # Claude Code 内置初始化模板（include_str!）
│   └── profiles.toml             # 旧文件（历史遗留测试夹具，可能包含非 UTF-8 内容）
├── codex/
│   └── profiles.toml             # Codex CLI 内置初始化模板（include_str!）
├── grok/
│   └── profiles.toml             # Grok Build 内置初始化模板（include_str!）
├── gemini/
│   └── profiles.toml             # Antigravity CLI（key: gemini）历史结构参考
└── droid/
    └── profiles.toml             # Factory Droid 历史结构参考
```

## 🚀 快速使用

```bash
# 1. 初始化平台 Profile 配置库（~/.ccr/platforms/<platform>/profiles.db）
ccr claude profile init
ccr codex profile init
ccr grok profile init

# 2. 通过 $VISUAL / $EDITOR 查看或编辑 Profile 目录投影
ccr claude profile open
ccr codex profile open
ccr grok profile open

# 3. 通过命令行安全写入密钥创建 Profile（避免命令行历史泄露）
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create relay \
  --base-url https://your-relay.example.com/v1 \
  --model claude-sonnet-4-5-20250929 \
  --auth-token-stdin

printf '%s' "$GROK_API_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-example \
  --api-key-stdin \
  --reasoning-effort high

# 4. 查看与切换激活配置
ccr claude profile list
ccr claude profile switch relay
ccr current

# 5. 导出脱敏分享模板与非破坏性合并导入（ExchangeV1）
ccr claude profile export --no-secrets -o team-claude.toml
ccr claude profile import team-claude.toml --dry-run
ccr claude profile import team-claude.toml
```

## 🗂️ 示例文件解读

- `claude/profiles.example.toml`：Anthropic 官方与兼容转发服务模板。
- `codex/profiles.toml`：OpenAI Codex 官方、中转站与 DeepSeek Responses API 模板。
- `grok/profiles.toml`：Grok 官方会话与第三方中转站模板（支持 `api_key` 或 `env_key`）。
- `gemini/profiles.toml` / `droid/profiles.toml`：保留兼容适配器历史结构参考；真实宿主检查请使用 `ccr platform list` 与 `ccr doctor --platform gemini` / `ccr doctor --platform droid`（Antigravity 本地设置亦可在 `ccr ui` 的 `/antigravity` 页面管理）。

## 🔑 常用字段与管理命令

### `enabled`

控制配置是否允许被激活：

```bash
ccr claude profile disable old_api
ccr claude profile enable old_api
```

### 分类与标签筛选导出

使用 `provider`、`provider_type`、`account`、`tags` 标记来源、类型与环境，并按标签导出：

```toml
[anyrouter_main]
provider = "anyrouter"
provider_type = "official_relay"
account = "work"
tags = ["production", "relay"]
```

```bash
ccr claude profile export --tag production --no-secrets -o prod-claude.toml
```

## 🔒 安全提示

1. 导出含密钥备份默认限制为当前用户可读写（`0600`），对外分享请务必加 `--no-secrets`。
2. 创建或修改密钥时优先使用 `--auth-token-stdin` / `--api-key-stdin` 或环境变量名 `--env-key`。
3. 请勿将真实 API Key 提交到公共仓库。

## 📚 更多信息

- 主项目文档：[`README.md`](../README.md) / [`README_CN.md`](../README_CN.md)
- 命令与平台参考：参见 `docs/reference/commands/` 与 `docs/reference/platforms/`
- 反馈：GitHub Issues <https://github.com/bahayonghang/ccr/issues>
