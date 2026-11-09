# DeepSeek 接入 Codex：上游要求与 CCR 差距分析

> 来源：https://api-docs.deepseek.com/zh-cn/quick_start/agent_integrations/codex/ （2026-07-31 抓取）
> 全文快照：`.trellis/workspace/lyh/deepseek-codex-doc.md`

## 1. 上游事实（DeepSeek 官方文档）

1. **模型可用性**：当前仅 `deepseek-v4-flash` 支持接入 Codex；`deepseek-v4-pro` 预计 2026 年 8 月初支持。
2. **协议**：DeepSeek API 原生支持 Responses API（`wire_api = "responses"`），与 Codex 现行协议一致。
3. **Codex 版本门槛**：models.json 中 `minimal_client_version: "0.144.0"`。
4. **官方手动配置**（`~/.codex/config.toml`）：

```toml
model = "deepseek-v4-flash"
model_provider = "deepseek"
preferred_auth_method = "apikey"
forced_login_method = "api"
model_reasoning_effort = "high"
model_catalog_json = "~/.codex/models.json"

[model_providers.deepseek]
name = "deepseek"
base_url = "https://api.deepseek.com/"
wire_api = "responses"
experimental_bearer_token = "<DeepSeek API Key>"
```

5. **模型目录 `~/.codex/models.json`**：向 Codex 声明 DeepSeek 模型元数据（context_window 1048576、supported_reasoning_levels low/high/max、apply_patch_tool_type freeform、base_instructions 全文等），包含 `deepseek-v4-flash` 与 `deepseek-v4-pro` 两个条目。内容较大（约 40KB，内嵌 Codex 系统提示词全文），由 DeepSeek 官方脚本（`https://cdn.deepseek.com/api-docs/codex-deepseek-setup.sh` / `-en.ps1`）生成，会随上游演进。
6. **认证方式**：API Key 直接写在 config.toml 的 provider 段 `experimental_bearer_token` 字段（不经 auth.json、不经环境变量）；配合 `preferred_auth_method = "apikey"` + `forced_login_method = "api"` 跳过 ChatGPT 登录。
7. **官方脚本行为**（可借鉴）：备份现有 config.toml 到 `~/.codex/backup-deepseek/`；只改写必要字段，保留 MCP servers / 信任级别等既有配置；写入前校验 TOML/JSON 语法；冲突字段删除并逐条说明。

## 2. CCR 现状（crates/ccr-codex/src/platforms/codex.rs）

第三方切换路径 `RouteSelection::ThirdPartyCustom` 固定写 `model_provider = "custom"` + `[model_providers.custom]`，字段：`name` / `base_url` / `wire_api`（仅 responses，chat 自动迁移）/ `requires_openai_auth` / 可选 `env_key`。

根级字段由 `apply_common_settings` 管理（写入/移除）：`model`、`approval_policy`、`sandbox_mode`、`model_reasoning_effort`、`forced_login_method`、`disable_response_storage`、`sandbox_workspace_write.network_access`、`cli_auth_credentials_store`。

认证模式 `CodexProfileAuthMode`：`openai_chatgpt` / `openai_api_key`（auth.json OPENAI_API_KEY + requires_openai_auth=true + 强制 file 存储）/ `provider_env_key`（provider 段 env_key + shell 导出）/ `no_auth`。

诊断/修复链路：`parse_current_auth_intent`、`inspect_runtime`、`diagnostic_route_status`、`diagnostic_credential_status`、`spec_matches_runtime_without_auth`、`ccr codex fix` 重放。

清场路径：`apply_runtime_route_without_auth`（`ccr codex off` / clear）显式 remove 根级管理字段列表。

## 3. 差距矩阵

| DeepSeek 要求                                 | CCR 现状                                                                                  | 差距                                                   |
| --------------------------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| 根级 `model`                                  | ✅ `profile.model` → 根级写入                                                             | 无                                                     |
| `model_provider` 对应 provider 段 id          | ✅ 固定 `custom`（用户明确要求沿用）                                                      | 无（id 仅需与段名一致，语义等价）                      |
| `model_reasoning_effort = "high"`             | ✅ platform_data 已支持                                                                   | 无                                                     |
| `forced_login_method = "api"`                 | ⚠️ 仅在 openai_chatgpt / openai_api_key 模式保留；其他模式被 `normalize_auth_fields` 清除 | 新认证模式下需允许保留                                 |
| `preferred_auth_method = "apikey"`            | ❌ 从不写入                                                                               | **新增根级管理字段**                                   |
| `model_catalog_json = "~/.codex/models.json"` | ❌ 从不写入                                                                               | **新增根级管理字段**                                   |
| `~/.codex/models.json` 模型目录               | ❌ 不涉及                                                                                 | 供给策略需设计决策（透传路径 vs 内置资产 vs 自动下载） |
| provider 段 `wire_api = "responses"`          | ✅ 默认值                                                                                 | 无                                                     |
| provider 段 `experimental_bearer_token`       | ❌ 现有密钥通道仅 auth.json / env_key                                                     | **新增认证模式**（密钥落入 config.toml，安全敏感）     |

## 4. 涉及面（切换写入之外）

- **诊断/修复**：上表新字段全部要进入 SwitchSpec 对比，否则 `ccr codex fix` 会报假漂移或修复时丢字段。
- **清场**：`apply_runtime_route_without_auth` 的 remove 列表需补 `preferred_auth_method` / `model_catalog_json`（bearer token 随 provider 段整体重建自然清除）。
- **Secret 处理**：`save_profile` → `persist_profile_secret` / `scrub_profile_secret_fields`（crates/ccr-codex/src/services/codex_runtime_service.rs）按 auth_mode 分派，新模式需接入；bearer 模式下 config.toml 变为含密文件（备份 `~/.codex/backups/config.*.bak` 同样含密），展示/日志/DTO 不得回显。
- **CLI**：`ccr codex profile set-field` 仅放行 `CodexPlatform::editable_fields()`（description/model/small_fast_model/provider/provider_type/account/tags）；`wire_api`、`auth_mode` 等 platform_data 键当前只能通过 ccr-ui 编辑器或手改 `~/.ccr/platforms/codex/profiles.toml` 配置。
- **UI（ccr-ui）**：
  - `src/utils/codexProfileEditor.ts` 表单已有 `wire_api`/`env_key`/`auth_mode`/`model_reasoning_effort`；`DEPRECATED_AUTH_MODES = ['openai_chatgpt', 'provider_env_key']`。未知 auth_mode 会回落 `no_auth`——若后端加新模式而 UI 不认识，用 UI 编辑该 profile 会静默改写认证模式（数据破坏风险），因此 UI 类型与编辑器必须同步。
  - `src/types/codex.ts` 的 `CodexProfileAuthMode`、`src/api/generated/codexAuth.ts`（生成物）需同步。
  - Provider 模板契约见 `.trellis/spec/ccr-ui/frontend/provider-template-contracts.md`：模板仅存非密字段，DeepSeek 模板可预填 base_url/model/名称，绝不预填 key。
- **示例/文档**：`examples/codex/config.example.toml`、`docs/examples/codex-cli-config.toml` 需补 DeepSeek 形态示例。

## 5. 关键开放决策（供 design.md 定案）

1. **密钥通道**：A) 忠实官方文档用 `experimental_bearer_token`（新模式，密钥入 config.toml）；B) 复用现有 `openai_api_key` 模式（auth.json，未经 DeepSeek 官方验证是否可用）。
2. **models.json 供给**：A) 纯透传（profile 声明路径，文件由用户/官方脚本生成）；B) CCR 内置 DeepSeek 目录资产并代写；C) 切换时从 CDN 下载（引入网络依赖，不可取）。
3. **新 auth_mode 命名**与 profile 内密钥存放位置（复用 `auth_token` Secret 字段 + runtime secret store）。
