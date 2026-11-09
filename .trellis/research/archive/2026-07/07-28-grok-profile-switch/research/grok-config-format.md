# Research：Grok CLI（xAI Grok Build）配置格式与第三方 API key 实践

> 调研日期：2026-07-28（rev2，按 Codex 审阅修正 auth.json 与默认模型两处事实）。
> 来源：xAI 官方文档 docs.x.ai（settings / settings/reference / build/overview / enterprise / developers/quickstart）；xai-org/grok-build 仓库 user-guide（HEAD 02d9359：`02-authentication.md`、`05-configuration.md`、`11-custom-models.md`）；TrueFoundry AI Gateway 集成文档；用户本机 `~/.grok/config.toml` 实样。
> 其中 `02-authentication.md` 已于 rev2 全文复核。

## 1. 配置文件与作用域

| 作用域 | 路径 | 用途 |
|---|---|---|
| 环境变量 | `GROK_*`、`XAI_API_KEY` 等 | 会话/CI 覆盖 |
| 用户 | `~/.grok/config.toml`（或 `$GROK_HOME/config.toml`；Windows `%USERPROFILE%\.grok\config.toml`） | 个人默认值 —— **CCR 的写入目标** |
| 会话凭据 | `~/.grok/auth.json`（+ `~/.grok/mcp_credentials.json`） | `grok login` 产物 —— **CCR 永不读写** |
| 项目 | 仓库内 `.grok/config.toml` | 仓库共享 MCP/插件/权限 |
| 托管/策略 | `managed_config.toml` / `requirements.toml`（用户级与 /etc 级） | 企业下发与策略钉死 |

配置优先级：CLI flags > 环境变量 > config.toml > managed/requirements > 内置默认。

## 2. 自定义模型（BYOK / 第三方中转）Schema

```toml
[model.<alias>]                # alias 仅本地标识，用于 -m / /model / [models].default
model = "model-id"             # 发送给 API 的模型 id
base_url = "https://api.example.com/v1"   # OpenAI 兼容端点
name = "Display Name"          # 模型选择器显示名
description = "..."            # 可选
api_key = "sk-..."             # 内联 key（可选；官方建议改用 env_key）
env_key = "XAI_API_KEY"        # 持有 key 的环境变量名；string 或 array（第一个非空生效）
api_backend = "responses"      # chat_completions | responses | messages
temperature = 0.7
top_p = 0.95
max_completion_tokens = 8192
context_window = 1000000       # 驱动 auto-compact 时机
extra_headers = { "x-api-key" = "..." }   # 逐字发送的额外请求头
query_params = { api-version = "..." }    # 追加到每个请求 URL
env_http_headers = { "X-Tenant" = "TENANT_TOKEN" }
supports_backend_search = true # 端点是否支持 Grok 托管的服务端搜索工具
supports_reasoning_effort / reasoning_effort / stream_tool_calls / max_retries / inference_idle_timeout_secs

[models]
default = "<alias>"            # 会话默认模型；【重要】该键可缺省——缺省时回落上游内置默认
web_search = "<alias>"         # web_search 工具所用模型（可选）

[endpoints]                    # 可选：为所有 [model.*] 提供共享 base_url
models_base_url = "https://gateway.example.com/v1"
```

要点：

- `[endpoints].models_base_url` 存在时 `[model.*]` 可省略 `base_url`（继承）；显式 `base_url` 优先。CCR 托管条目始终写显式 `base_url`。
- 覆盖内置模型：以内置名作段名（如 `[model.grok-build]`）只写需覆盖字段。
- **默认模型事实（rev2 修正）**：上游新会话默认模型由 catalog 决定，当前 HEAD 为 `grok-4.5`；docs 示例中的 `default = "grok-build"` 是针对 coding 场景的推荐而非出厂默认。**CCR 官方路线在 profile 未显式指定模型时应移除 `models.default` 键回落上游默认，而不是硬编码任何别名**（上游默认会随版本漂移）。
- 改完 config 后可用 `grok inspect` 查看发现结果。

## 3. 认证与凭据（rev2 修正）

**事实修正**：Grok **存在** `~/.grok/auth.json`——浏览器 OIDC / 设备码 / 外部 auth provider 登录的会话凭据都存在该文件（Unix 0600），后台自动刷新、外部改动 hot reload，`grok logout` 清除。`mcp_credentials.json` 存 MCP OAuth token。

每请求凭据解析优先级（官方 `02-authentication.md` Auth Precedence 一节，三层）：

1. **Per-model `api_key` 或 `env_key`**（`[model.*]` 内）——存在即胜出
2. **活跃会话 token**（auth.json）
3. **`XAI_API_KEY`** 全局回退（无会话 token 时；兼容 `GROK_CODE_XAI_API_KEY`）

CCR 边界推论：

- CCR 只经营第 1 层（config.toml 内的 per-model 凭据）；**auth.json 完全归 grok login/logout 管理，CCR 不读不写不备份**（自动刷新 + hot reload 意味着任何外部快照都会过期甚至制造 token 回滚风险——与 Codex refresh_token_reused 教训同类）。
- 官方 profile 的语义因此是**纯模型选择器**：认证由第 2/3 层自然接管；per-model 凭据字段必须为空，否则会按第 1 层优先级劫持官方请求。

认证方式矩阵：Browser OIDC（`grok login`）/ Device code（`--device-auth`）/ External auth provider（`auth_provider_command`）/ API key（`XAI_API_KEY` env 或 `model.api_key`）。企业可用 `requirements.toml` `disable_api_key_auth = true` 强制 SSO（BYOK 第三方端点不受影响）。

## 4. 最佳实践（社区 + 官方）

1. **优先 `env_key` 而非明文 `api_key`**（settings/reference 原文）。CCR 双模式支持：inline 服务开箱即用，env_key 服务安全偏好；文档与 TUI 提示以 env_key 为推荐。
2. 第三方中转标准接法：`[model.<alias>]` + `base_url` + 凭据 + `api_backend`（responses 或 chat_completions，取决于网关）。
3. `env_key` 支持 string 或 array（首个非空生效）；**CCR MVP 仅支持单字符串**，array 形态校验拒绝并提示。
4. 诊断/切换显示应关注的环境覆盖：`XAI_API_KEY`、`GROK_CODE_XAI_API_KEY`、`GROK_DEFAULT_MODEL`、`GROK_HOME`。

## 5. 用户本机实样（2026-07-28，key 已脱敏）

```toml
[cli]
installer = "internal"

[model.custom]
model = "grok-4.5"
base_url = "https://api.tangguo.xin/v1"
name = "Grok 4.5"
api_key = "sk-***MASKED***"
api_backend = "responses"
context_window = 1000000
supports_backend_search = true

[models]
default = "custom"

[session]
auto_compact_threshold_percent = 85
load_envrc = true

[memory]
enabled = true
[memory.session]
save_on_end = true

[ui]
fork_secondary_model = "custom"
max_thoughts_width = 120
theme = "rosepine-moon"
yolo = false
compact_mode = false

[subagents]
enabled = true

[marketplace]
default_skills_installs_purged = true
```

结论：

- 用户已用 `custom` 作第三方条目别名，且 `[ui].fork_secondary_model = "custom"` 引用该别名 → CCR 接管 `[model.custom]` 可承接现状；**但正因存在这类外部引用与用户原始内容，官方切换不能简单删除该条目**：入口状态必须记录原始 `[model.custom]`（含"原不存在"标记）与原始 `[models].default`，切回官方时"原存在则恢复原条目、原不存在才删除"（CORR-001）。
- 其余段落（cli/session/memory/ui/subagents/marketplace）必须在 read-modify-write 中保留。

## 6. 与 Codex 平台架构的映射（rev2 修正）

| Codex（现有） | Grok（设计） |
|---|---|
| `~/.codex/config.toml` `model_provider = "custom"` + `[model_providers.custom]` | `[models].default = "custom"` + `[model.custom]` |
| `~/.codex/auth.json`（CCR 读写、含入口快照回写） | `~/.grok/auth.json` **存在但 CCR 永不触碰**（自动刷新 + hot reload，快照必然过期） |
| `wire_api`（仅 responses） | `api_backend`（chat_completions/responses/messages 三值，缺省 responses） |
| `requires_openai_auth` / `env_key` | `env_key`（单层语义；MVP 仅单字符串） |
| 入口快照 `profile_entry_auth_state.json`（auth.json 内容） | 入口状态 `profile_entry_config_state.json`：整份 config.toml + 结构化记录原始 `[model.custom]` / 原始 `[models].default` |
| `CodexProfileAuthMode`（4 值） | `GrokProfileAuthMode`：`inline_api_key` / `env_key` / `session`（官方=纯模型选择器） |
| 删除当前 profile → reconcile 指针（指向首个剩余但不 apply） | **不复用该语义**：删除当前激活 profile 默认拒绝（CORR-003，见 core design） |

## 7. 引用

- https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md（auth.json、Auth Precedence、hot reload）
- https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md（[model.*] 字段、凭据解析、[endpoints]）
- https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md（配置优先级、custom models）
- https://docs.x.ai/build/settings 、 https://docs.x.ai/build/settings/reference 、 https://docs.x.ai/build/enterprise
