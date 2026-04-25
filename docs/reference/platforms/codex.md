# Codex 平台配置指南

## 概览

CCR 在 Codex 平台下会把统一 profile 配置转换为 Codex CLI 配置文件，核心路径：

- 输入：`~/.ccr/platforms/codex/profiles.toml`
- 输出：`~/.codex/config.toml`、`~/.codex/auth.json`

> 说明：本仓库同时提供 GitHub Copilot for VS Code 的工作区资产，它们位于仓库内 `.github/*` 与共享 `.claude/skills/`。那一套能力与这里描述的 Codex CLI 运行时配置是分开的。

切换时有两种模式：

1. `official_relay`（官方模式）：重置 `config.toml` 和 `auth.json`
2. 其他类型（第三方模式）：read-modify-write，仅更新 Provider 相关字段，保留其他已有配置

CCR 保留 `custom` 作为第三方 Codex profile 的运行时 provider 命名空间；此类 profile 写入时，根字段 `model_provider` 始终固定为 `"custom"`，真实上游身份由 CCR profile metadata 或 `[model_providers.custom].name` 表示。

## 配置流程

### 1. Profile -> Codex 配置写入

执行 `ccr switch <profile>` 后，Codex 平台按以下规则写入：

- 顶层字段（`~/.codex/config.toml`）：`model`、`model_provider`、`model_reasoning_effort`、`approval_policy`、`sandbox_mode` 等
- 活跃 Provider 表固定为 `[model_providers.custom]`：`name`、`base_url`、`wire_api`、`requires_openai_auth`、`env_key`
- 凭据文件（`~/.codex/auth.json`）：`OPENAI_API_KEY` 或 `<env_key>` 对应的 key

### 2. 运行时 provider namespace

对所有第三方 profile：

1. 根字段 `model_provider` 固定写入 `"custom"`
2. 活跃 provider 配置固定写入 `[model_providers.custom]`
3. 真实上游 provider 身份继续来自 profile metadata（例如 `provider_id` / `provider`）或 `[model_providers.custom].name`

这意味着 `model_provider` 在第三方模式下表示 CCR 保留的运行时命名空间，而不是上游厂商名。历史遗留的 `[model_providers.<legacy_id>]` 表可能继续存在，但当前运行时只使用 `custom`。

如果你已经切换了运行时 provider，但旧历史在 Codex CLI / App 中仍然不可见，请使用 `ccr codex sync-history --provider <ID>` 同步 rollout / SQLite / 侧边栏元数据。URL+Key profile 在 CCR 中的运行时 provider 是 `custom`；想让最近 `openai` 会话在 URL+Key profile 下可见时，用 `--provider custom`。想恢复官方 profile 视图时，用 `--provider openai`。建议先加 `--dry-run` 预览。

## 字段配置（重点）

下面是 Codex 配置最关键的字段说明。

### model

| 字段 | 位置 | 类型 | 是否必填 | 说明 |
|------|------|------|----------|------|
| `model` | profile 顶层 | string | 否 | 主模型名，切换后写入 `~/.codex/config.toml` 顶层 `model` |

示例：

```toml
model = "gpt-5-codex"
```

### model_reasoning_effort

| 字段 | 位置 | 类型 | 是否必填 | 允许值 |
|------|------|------|----------|--------|
| `model_reasoning_effort` | platform_data（profile 扁平字段） | string | 否 | `minimal` / `low` / `medium` / `high` / `xhigh` |

行为说明：

- 校验时严格枚举校验，非法值会报错。
- 输入大小写不敏感，写入时会规范化为小写。

示例：

```toml
model_reasoning_effort = "high"
```

### base_url

| 字段 | 位置 | 类型 | 是否必填 | 说明 |
|------|------|------|----------|------|
| `base_url` | profile 顶层 | string | 第三方模式必填 | Provider API 基础地址，必须以 `http://` 或 `https://` 开头 |

示例：

```toml
base_url = "https://api.example.com/v1"
```

### key（auth_token / env_key / OPENAI_API_KEY）

| 字段 | 位置 | 类型 | 是否必填 | 用途 |
|------|------|------|----------|------|
| `auth_token` | profile 顶层 | string | 取决于认证模式 | 切换时用于写入 `auth.json` |
| `env_key` | platform_data | string | Provider key 模式必填 | 指定写入到 `auth.json` 的 key 名 |
| `OPENAI_API_KEY` | `~/.codex/auth.json` | string | OpenAI key 模式可写入 | 当使用 OpenAI API key 模式时写入 |

认证模式判定：

- 当 `requires_openai_auth = true`：走 OpenAI 认证语义，`env_key` 会被忽略。
- 当 `requires_openai_auth = false` 且提供了 `env_key`：要求 `auth_token` 非空，并写入 `auth.json[env_key]`。
- 未显式设置 `requires_openai_auth` 时，默认按是否存在 `env_key` 推断。

## 其他常用字段

| 字段 | 位置 | 说明 |
|------|------|------|
| `wire_api` | platform_data | `responses` 或 `chat`，默认 `responses` |
| `provider_type` | profile 顶层 | `official_relay` 表示官方模式；其他值走第三方模式 |
| `approval_policy` | platform_data | 透传到 `config.toml` 顶层 |
| `sandbox_mode` | platform_data | 透传到 `config.toml` 顶层 |
| `network_access` | platform_data | 透传到 `config.toml` 顶层 |
| `disable_response_storage` | platform_data | 透传到 `config.toml` 顶层（bool） |
| `provider_model` | platform_data | 可选；写入 `[model_providers.custom].model` |

## 推荐配置示例

### 官方模式（重置到 Codex 默认）

```toml
[official]
description = "Codex 官方模式"
provider = "openai"
provider_type = "official_relay"
```

### 第三方模式（Provider env key）

```toml
[duckcoding]
description = "DuckCoding OpenAI 兼容"
base_url = "https://jp.duckcoding.com/v1"
auth_token = "sk-..."
model = "gpt-5-codex"
provider = "duckcoding"
provider_type = "third_party_model"
wire_api = "responses"
env_key = "DUCKCODING_API_KEY"
requires_openai_auth = false
model_reasoning_effort = "high"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
network_access = "enabled"
disable_response_storage = true
```

### 第三方模式（OpenAI auth 语义）

```toml
[openai-proxy]
description = "OpenAI auth via proxy"
base_url = "https://proxy.example.com/v1"
model = "gpt-5-codex"
provider = "proxy"
provider_type = "third_party_model"
wire_api = "responses"
requires_openai_auth = true
model_reasoning_effort = "medium"
```

## 写入结果示例

上面的 `duckcoding` 切换后，`~/.codex/config.toml` 典型结果（注意运行时命名空间固定为 `custom`）：

```toml
model_provider = "custom"
model = "gpt-5-codex"
model_reasoning_effort = "high"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
network_access = "enabled"
disable_response_storage = true

[model_providers.custom]
name = "DuckCoding OpenAI 兼容"
base_url = "https://jp.duckcoding.com/v1"
wire_api = "responses"
requires_openai_auth = false
env_key = "DUCKCODING_API_KEY"
```

`~/.codex/auth.json` 典型结果：

```json
{
  "DUCKCODING_API_KEY": "sk-..."
}
```

## 校验与排错

### 常见校验失败

1. `wire_api` 非法
- 仅允许 `responses` / `chat`

2. `model_reasoning_effort` 非法
- 仅允许 `minimal/low/medium/high/xhigh`

3. 第三方 profile 缺少 `base_url`
- 且 URL 必须以 `http://` 或 `https://` 开头

4. Provider key 模式缺少 `auth_token`
- 当 `env_key` 生效时，`auth_token` 必须提供

### 建议检查命令

```bash
ccr platform switch codex
ccr list
ccr validate
ccr switch <profile>
```

## 安全建议

1. 不要提交包含真实 token 的文件到 Git。
2. 分享配置使用 `--no-secrets`。
3. 保护权限：

```bash
chmod 600 ~/.ccr/platforms/codex/profiles.toml
chmod 600 ~/.codex/auth.json
```

## 相关文档

- [GitHub Copilot 工作区支持](/guide/github-copilot-workspace)
- [平台总览](./index)
- [平台迁移](./migration)
- [示例索引](../../examples/)
