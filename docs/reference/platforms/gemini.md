# Antigravity CLI 平台指南

## 概览

CCR 继续把持久化平台 key 保持为 `gemini`，但面向用户的 Google CLI 集成现在指向 **Antigravity CLI**。这样可以保留既有 CCR profiles、历史记录、用量聚合、同步目录和 UI 书签，同时跟随 Google 从 Gemini CLI 迁移到 Antigravity CLI 的路径。

官方迁移参考：

- Google Developers Blog: <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>
- Antigravity CLI Overview: <https://antigravity.google/docs/cli-overview>
- Gemini CLI migration docs: <https://antigravity.google/docs/gcli-migration>

## 平台信息

| 项目 | 值 |
|------|----|
| CCR platform key | `gemini` |
| 显示名称 | Antigravity CLI |
| 推荐二进制 | `agy` |
| Profiles 文件 | `~/.ccr/platforms/gemini/profiles.toml` |
| Antigravity settings | `~/.gemini/antigravity-cli/settings.json` |
| Antigravity MCP 配置 | `~/.gemini/antigravity-cli/mcp_config.json` |
| 全局 skills | `~/.gemini/antigravity-cli/skills` |
| Gemini legacy/shared skills | `~/.gemini/skills` |
| Workspace MCP | `.agents/mcp_config.json` |
| Workspace skills | `.agents/skills` |

旧 `/gemini-cli` UI 路由和 `gemini` platform key 会继续作为兼容入口保留。新文档和可见 UI 应优先使用 Antigravity CLI 与 `/antigravity` 路由。

## 前置条件

- 使用本地 CLI runtime 时，需要已安装 Antigravity CLI（`agy --help`、`agy --version`）。
- 账号侧应具备合适的 Google API key、Enterprise / Standard / Google Cloud 或 paid API key 访问路径。
- 既有 CCR `gemini` profiles 可以继续复用；不要把平台 key 重命名为 `antigravity`。

## 快速开始

```bash
# Gemini 暂无 profile init 命令，请手工复制示例
mkdir -p ~/.ccr/platforms/gemini
cp examples/gemini/profiles.toml ~/.ccr/platforms/gemini/profiles.toml

# 切换到 Google / Antigravity profile 命名空间
ccr platform switch gemini

# 添加或编辑 profile
ccr add

# 在 CCR 外验证本地 Antigravity CLI 二进制
agy --version
```

Antigravity 官方迁移命令预览：

```bash
agy plugin import gemini
```

## Profile 配置

CCR profiles 仍位于 `~/.ccr/platforms/gemini/profiles.toml`：

```toml
[google-official]
description = "Google Antigravity / Gemini API"
base_url = "https://generativelanguage.googleapis.com/v1beta"
auth_token = "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
model = "gemini-2.0-flash-exp"
small_fast_model = "gemini-1.5-flash"
```

profile 生效后，CCR 会把 Antigravity settings 写入 `~/.gemini/antigravity-cli/settings.json`：

```json
{
  "env": {
    "GOOGLE_API_BASE_URL": "https://generativelanguage.googleapis.com/v1beta",
    "GOOGLE_API_KEY": "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "GEMINI_MODEL": "gemini-2.0-flash-exp",
    "GEMINI_SMALL_FAST_MODEL": "gemini-1.5-flash"
  }
}
```

| TOML 字段 | 环境变量 | 说明 |
|-----------|----------|------|
| `base_url` | `GOOGLE_API_BASE_URL` | API endpoint |
| `auth_token` | `GOOGLE_API_KEY` | Google API key，正常输出中会被掩码 |
| `model` | `GEMINI_MODEL` | 默认模型 |
| `small_fast_model` | `GEMINI_SMALL_FAST_MODEL` | 可选快速模型 |

## MCP、Skills 与 Workspace 路径

Antigravity MCP servers 与 settings 分文件存储：

```text
~/.gemini/antigravity-cli/mcp_config.json
```

CCR 写入远程 MCP servers 时使用 `serverUrl`，读取时继续兼容 legacy `url` / `httpUrl` 字段：

```json
{
  "mcpServers": {
    "example": {
      "serverUrl": "https://example.com/mcp",
      "type": "http"
    }
  }
}
```

Skills 查找顺序包括：

1. Workspace `.agents/skills`
2. Antigravity 全局 `~/.gemini/antigravity-cli/skills`
3. Gemini legacy/shared `~/.gemini/skills`

需要项目级 Antigravity MCP 配置时，workspace MCP 应使用 `.agents/mcp_config.json`。

## Sessions 与用量导入

CCR 会继续把历史 Gemini session 和用量数据聚合到内部 `gemini` platform key 下。旧 Gemini CLI logs（`~/.gemini/tmp/*/chats/session-*.json`）保持 import-compatible。

Antigravity session/log import 会在上游本地日志格式确认后再声明支持。不要仅凭 Gemini legacy parser 声称已经支持 Antigravity session import。

## 故障排查

### Settings 文件没有变化

检查 Antigravity 路径，而不是旧 Gemini 根目录 settings 文件：

```bash
ls -la ~/.gemini/antigravity-cli/settings.json
chmod 600 ~/.gemini/antigravity-cli/settings.json
```

然后确认 CCR profile 状态：

```bash
ccr platform switch gemini
ccr current
ccr validate
```

### MCP server 缺失

检查 `mcp_config.json`：

```bash
cat ~/.gemini/antigravity-cli/mcp_config.json
```

远程 servers 应优先使用 `serverUrl`；旧 `url` 和 `httpUrl` 只作为读取兼容字段保留。

### 旧 Gemini 路径仍然出现

部分旧路径会有意保留为 legacy/import-compatible 来源：

- `~/.gemini/skills`：共享 legacy skills
- `~/.gemini/commands` 与项目 `.gemini/commands`：legacy slash-command 文件
- `~/.gemini/tmp/*/chats/session-*.json`：legacy session import

它们不应再被文档为 Antigravity settings 或 MCP 的主路径。

## 安全说明

- `~/.ccr/platforms/gemini/profiles.toml` 中的 API keys 是明文；请保持严格文件权限。
- CCR 会在正常输出与历史记录中掩码 API keys，但 backups 仍可能包含 secrets。
- 分享 profile 示例时优先使用 `ccr export --no-secrets`。

## 相关命令

```bash
# profile create 流程会自动创建目录，也可按上文手工复制示例
ccr platform switch gemini
ccr list
ccr switch <profile>
ccr validate
ccr history --platform gemini
```
