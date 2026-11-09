# Design — Codex DeepSeek 第三方接入支持

> rev2（2026-08-01）：吸收 Codex 审阅 7 项发现（核验记录 `research/review-verification-rev2.md`）。主要修订：bearer 凭据改经不派生 Debug 的 AuthSelection 承载（发现 3）；secret-aware 写入统一收敛（发现 2）；bearer 模式定义为完整不变量并补 Tauri 桥接层（发现 4）；明文边界定案（发现 1）；`--repair-runtime` / models.json 检查语义 / 上下文清单修正（发现 5/6/7）。
> 前置阅读：`research/deepseek-codex-requirements.md`、`.trellis/spec/ccr-codex/backend/backend-guidelines.md`、`.trellis/spec/ccr-core/backend/atomic-writer.md`。

## 1. 决策记录

### D1 Provider id 沿用 `custom`

Codex 只要求 `model_provider` 与 `[model_providers.<id>]` 段名一致，id 本身无语义。保持 `THIRD_PARTY_RUNTIME_PROVIDER_KEY = "custom"` 不动，与现有双路分发（Official/ThirdParty 都写 `custom` 段）及诊断逻辑完全兼容。官方文档的 `deepseek` id 仅是示例命名。

### D2 认证通道：新增 `provider_bearer_token` 模式（rev2 修订承载分层）

- **选定**：`CodexProfileAuthMode` 新增变体 `ProviderBearerToken`（序列化 `"provider_bearer_token"`）。profile 密钥仍存 `auth_token`（`ccr_core::Secret`，经 runtime secret store 托管、profiles.toml 落盘前擦除）。
- **承载分层（发现 3）**：`RouteSelection` 派生 `Debug`，现有 `AuthSelection` 刻意不派生——凭据不得进入路由枚举。bearer 明文经 **`AuthSelection` 新变体 `WriteProviderBearerToken(Secret)`** 承载（`Secret` 的 Debug/Display/默认 Serialize 均掩码，双保险），`RouteSelection::ThirdPartyCustom` 保持非密、不新增凭据字段。两个消费点：
  1. `apply_switch_spec` 构建 `custom` provider 表时，从 `spec.auth` 匹配该变体取值写入 `experimental_bearer_token`（provider 表每次整体重建，切走自然清除）；
  2. auth.json 侧沿用 ClearOpenAi 等价清理（清 tokens / OPENAI_API_KEY / provider keys），不写任何新内容。
- **完整不变量（发现 4）**：`normalize_auth_fields` 的 bearer 分支自动派生 `preferred_auth_method = "apikey"`、`forced_login_method = "api"`（platform_data 显式声明可覆盖）；同时 `requires_openai_auth = Some(false)`、清 `env_key`、清 `openai_login_method`。UI 只需选择 bearer 模式即产出官方要求的完整组合。
- **否决备选**：复用 `openai_api_key` 模式（auth.json `OPENAI_API_KEY` + `requires_openai_auth = true`）。理由：DeepSeek 官方仅验证 bearer 通道；auth.json 通道绑定 Codex 的 OpenAI 登录语义；官方脚本"删除冲突字段"的行为说明两通道并存会冲突。现有 `no_auth + env_key` 通道保持原样。

### D3 models.json：纯透传，不代写（rev2 修正检查语义）

- profile `platform_data.model_catalog_json`（字符串路径）**原样**写入 config.toml 根级——`~` 由 Codex 展开，CCR 写盘值不做任何改写。
- 存在性提醒（发现 6）：仅在**检查用副本**上展开 home 前缀（`~/` / `~\` → `dirs::home_dir()`），文件不存在时通过切换命令的用户可见输出告警（`ColorOutput::warning`）并同步 `tracing::warn`；无法展开或相对路径则跳过检查不告警。检查绝不阻塞切换，绝不影响写盘原值。
- **否决备选**：内置资产代写（约 40KB、内嵌上游 prompt 全文、随上游漂移、有覆盖用户自有 models.json 风险）；切换时 CDN 下载（网络依赖 + 供应链面）。

### D4 `preferred_auth_method`：bearer 派生 + 显式覆盖

bearer 模式下由不变量派生（见 D2）；platform_data 显式声明 `preferred_auth_method`（值域 `apikey | chatgpt`，大小写归一，非法值 `ValidationError`）时以显式值为准，且允许非 bearer 模式单独使用（一般第三方 relay 也可能需要）。根级写入/移除语义同现有管理字段。

### D5 Secret-aware 写入统一（rev2 新增，发现 2）

**契约**：secret 权限必须在临时文件写入内容**之前**生效（`AtomicWriter.secret(true)`：Unix 先 0600 再写、保留既有 owner-only mode；Windows 为文档化 no-op）。原方案"写后 `ensure_private_permissions`"违约，废弃。

| 写入点 | 现状 | 变更 |
| --- | --- | --- |
| `CodexConfigManager::atomic_write`（切换路径，config.toml + auth.json 共用） | `NamedTempFile` + `fs::write`，无 secret 语义 | 改走 `ccr_core::AtomicWriter`，config.toml 与 auth.json 均 `secret(true)`（auth.json 本就是密文件，一并对齐契约） |
| `CodexConfigManager::backup_file` | `fs::copy`（Unix 隐式带 mode，Windows 无保证） | 改为读源 + `AtomicWriter.secret(true)` 写备份，显式契约化 |
| `ccr-ui/src-tauri/.../codex.rs::write_codex_config`（Settings + Codex MCP 共用） | 自带 NamedTempFile + `fs::write` | 改走 `AtomicWriter.secret(true)` |
| `ccr-ui/src-tauri/.../unified_mcp.rs::write_json_config`（codex 分支） | `fs::write` 裸临时文件 + rename | 改走 `AtomicWriter.secret(true)` |

- **核验校正**：`CodexConfig` 已有 `#[serde(flatten)] pub other: HashMap<String, toml::Value>` 兜底，Settings/MCP typed 整写**不会**剥掉 `experimental_bearer_token` / 新根级键——无数据丢失问题，仅需权限收敛；以 flatten 回归测试固化该保证。
- **先例**：config.toml 今天已通过 `CodexMcpServer.bearer_token` 承载明文密钥，含密非本任务首创；本任务把全部整写入口收敛到契约化路径，是对既有状况的净改善。
- **范围外**：Windows DACL 硬化为仓库级已知限制（AtomicWriter 文档化 no-op，Grok 任务同列后续候选），不在本任务解决。
- Debug 卫生：`CodexConfig`（含 flatten other）与 `toml::Value` 均可 Debug——实现时核查不得整体 Debug 打印配置对象到日志（`rg` 检查 + 测试断言归入明文边界验收）。

## 2. 数据流与触点（crates/ccr-codex/src/platforms/codex.rs 为主）

```
profiles.toml(platform_data) ─→ build_switch_spec ─→ SwitchSpec ─→ apply_switch_spec ─→ config.toml
                                                         │                                （AtomicWriter.secret）
                       inspect_runtime / fix --repair-runtime ←─┘（同一 SwitchSpec 对比矩阵）
```

| 触点 | 变更 |
| --- | --- |
| `SwitchSpec` | 新增 `model_catalog_json: Option<String>`、`preferred_auth_method: Option<String>`（均非密）；路由枚举不新增凭据字段 |
| `AuthSelection` | 新增 `WriteProviderBearerToken(Secret)`（维持不派生 Debug） |
| `build_switch_spec` | 解析新字段 + bearer 模式校验（auth_token/base_url 必填、Official 路由禁用）+ 不变量派生对接 |
| `apply_common_settings` | 根级 `model_catalog_json` / `preferred_auth_method` 写入/移除（`set_optional_root_string` 既有语义） |
| `apply_switch_spec` | provider 表按 `spec.auth` 写 `experimental_bearer_token`；auth.json 侧 bearer arm 执行 ClearOpenAi 等价清理 |
| `apply_runtime_route_without_auth` | remove 列表补 `model_catalog_json`、`preferred_auth_method` |
| `parse_current_auth_intent` | provider 段存在非空 `experimental_bearer_token` → `AuthIntent::ProviderBearerToken`（优先级在 env_key 判定之前） |
| `diagnostic_route_status` | `root_string_matches` 矩阵补两个根级键 |
| `diagnostic_credential_status` | bearer arm：期望 secret 与 config.toml provider 段实际值对比 → Missing/Mismatch/Match，repairable = true |
| `runtime_auth_source` | 新枚举值（如 `CodexRuntimeAuthSource::ConfigBearerToken`），只报来源、不带值 |
| `spec_matches_runtime_without_auth` | 保持"不比对凭据"定位：bearer 值归 credential 侧；本函数只比路由字段（含两个新根级键） |
| `validate_profile` | bearer 模式：auth_token/base_url 必填；`preferred_auth_method` 值域校验 |
| `codex_runtime_service.rs` | `persist_profile_secret` / `scrub_profile_secret_fields` / `build_env_export`（bearer 无 env 导出）新增 arm |
| `models/codex_auth.rs` | `CodexProfileAuthMode` 新变体 + `as_str`/解析/`openai_login_method()`（返回 None） |
| `managers/codex_config.rs` | 写入/备份按 D5 契约化 |

### Tauri 桥接层触点（ccr-ui/src-tauri，发现 4）

| 触点 | 变更 |
| --- | --- |
| `commands/codex.rs` `EXPLICIT_PLATFORM_STRING_FIELDS` | 补 `model_catalog_json`、`preferred_auth_method`、`forced_login_method`（具名字段保存当前被白名单丢弃；`extra` 对象透传通道虽在但编辑器走具名字段） |
| profile DTO 投影（`codex.rs` 具名字段 + `extra` 剔除循环） | 补三个具名字段；同步从 `extra` 剔除避免双写 |
| `write_codex_config` / `unified_mcp.rs::write_json_config` | 按 D5 secret 化 |
| 命令层测试 | 新字段白名单/投影往返 + flatten 保留回归 |

## 3. 明文边界与 Secret 安全（红线，rev2 定案）

**允许明文的位置（全集）**：
1. 磁盘既定落点：config.toml provider 段 + 其备份（均经 D5 secret-aware 写入）+ runtime secret store；
2. 用户显式触发通道 A：Profile 编辑器预填（typed DTO `auth_token`，现状显式 `expose`，掩码化归属独立 typed-ipc 任务——代码注释已锚定）；
3. 用户显式触发通道 B：Raw Source 编辑器整文读写（raw-config-editor-contracts 约束下的既有能力）。

**其余一律掩码**：日志/tracing、诊断 JSON、status、dashboard DTO、`extra` 投影、错误信息。测试断言以此边界为准。

**同步披露**：config.toml 属 `codex-config` 加密同步资产（v2 加密信封，`pushSyncAsset('codex-config', ...)`，见 sync-security-contracts）。bearer 随信封同步为预期行为——仅文档披露，不改机制。

## 4. UI 同步（ccr-ui 前端）

- `src/types/codex.ts`：`CodexProfileAuthMode` 增 `'provider_bearer_token'`；生成物 `src/api/generated/codexAuth.ts` 按仓库生成流程再生成（查 `command-manifest.json` 的生成命令，禁止手改生成物）。
- `src/utils/codexProfileEditor.ts` / `CodexProfileEditorModal.vue`：
  - auth_mode 选项增 bearer（不入 `DEPRECATED_AUTH_MODES`）；表单增 `model_catalog_json`；派生字段 `preferred_auth_method` / `forced_login_method` 以默认派生态呈现，显式覆盖走高级入口。
  - 序列化单源规则扩展：bearer 值走既有 auth_token secret 输入通道；`env_key` 仅 env 模式、bearer 相关仅 bearer 模式序列化。
  - **关键兼容修复**：未知 auth_mode 回落 `'no_auth'` 的路径对新模式失效（纳入类型后消除静默改写）。
- 模板：DeepSeek 内置模板（`platforms.codex`：base_url `https://api.deepseek.com/`、model `deepseek-v4-flash`、名称/官网/取 key 链接）；遵守 provider-template-contracts——不含任何密钥字段；非预置模型走 custom model path 既有机制（不写全局 custom-models.toml）。
- i18n：zh-CN / en-US 双份键。

## 5. 兼容性与发布

- platform_data 为增量键：旧 profiles 无新键 → 行为不变；新键对旧版本 CCR 是未知键，读取忽略。无迁移脚本。
- 存量四种 auth_mode 的 spec 对比矩阵不变（新增键均为 `Option`，None 时 remove 语义与现状一致）。
- UI Settings / MCP 整写靠 `CodexConfig.other` flatten 保留新键（已核验），回归测试固化。
- Rust / Tauri / UI 版本经 `just version-sync` 同步发布，避免旧 UI + 新后端跨版本混用窗口。

## 6. 回滚

- 单 PR 交付，回滚 = revert commit。
- 运行时残留：`ccr codex off`（`clear_active_profile_runtime`）清除全部 CCR 管理字段并恢复入口 auth 快照；新字段进入其 remove 列表后该路径天然覆盖。
- 最大风险 = 诊断假漂移（fix 循环重写）：以"切换 → 立即 inspect 零漂移"幂等测试作为合并门槛；修复重放仅经 `--repair-runtime` 显式授权。

## 7. 评审门

- `rust-security-reviewer`：bearer 承载分层、D5 写入契约、明文边界、备份/同步披露全链路（必审）。
- `tauri-ipc-reviewer`：命令层白名单/投影/写入路径变更（本次命中，必审）。
- `frontend-quality-reviewer`：UI 改动跨多文件 + 触及 types/utils（必审）。
