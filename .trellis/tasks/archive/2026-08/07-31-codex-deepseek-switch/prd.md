# Codex DeepSeek 第三方接入支持

> rev2（2026-08-01）：按 Codex 审阅报告修订——7 项发现逐条核验全部属实并已吸收（2 阻断 / 2 高 / 3 中）；其中两处按实证校正范围：`CodexConfig` 已有 `#[serde(flatten)] other` 兜底、typed 整写不丢未知键；config.toml 经 MCP `bearer_token` 字段今天已承载明文密钥（含密非本任务首创）。核验记录见 `research/review-verification-rev2.md`。

## Goal

用户在 CCR 中配置一个 DeepSeek Codex profile 后，执行切换即可让 Codex（CLI / IDE 插件 / 桌面端共用同一份配置）直接使用 DeepSeek 模型；产生的 `~/.codex/config.toml` 与 DeepSeek 官方接入文档语义等价，且 `model_provider` 沿用 CCR 现有的固定 `custom` 段（不引入 `deepseek` provider id）。

## 背景与来源需求

- DeepSeek 已官方支持接入 Codex（Responses API 原生兼容），但与现有第三方中转「只换 base_url + key」不同，还要求：根级 `model`、`model_catalog_json`（指向 models.json 模型目录）、`preferred_auth_method = "apikey"`、`forced_login_method = "api"`，以及 provider 段内 `experimental_bearer_token`（API Key 直写 config.toml）。
- 当前仅 `deepseek-v4-flash` 可用；`deepseek-v4-pro` 预计 2026 年 8 月初开放。Codex 客户端版本需 >= 0.144.0。
- 上游事实与差距矩阵详见 `research/deepseek-codex-requirements.md`（含官方 config.toml 全文）。
- 用户约束：`model_providers` 沿用现有 `custom` 段；缺的是在该段和根级补齐上述字段。

## 明文边界（安全基线，rev2 定案）

bearer token 的明文只允许出现在以下位置，其余一切输出（日志、tracing、诊断 JSON、status、dashboard/常规 DTO、`extra` 投影、错误信息）严格掩码：

1. **磁盘既定落点**：`~/.codex/config.toml` provider 段（DeepSeek 官方格式所需）及其备份文件；CCR 侧 runtime secret store。
2. **用户显式触发的本地通道 A**：Profile 编辑器预填——typed profile DTO 的 `auth_token` 字段（现状即显式 `expose`，代码注释已标注掩码化改造归属独立 typed-ipc 任务，`ccr-ui/src-tauri/src/commands/codex.rs:1624`）。
3. **用户显式触发的本地通道 B**：Raw Source 编辑器整文读写 config.toml（`settings_raw.rs` 现有能力，受 raw-config-editor-contracts 约束）。

typed IPC 掩码化重构（sentinel 往返等）**不在本任务范围**，沿用仓库既定归属，另立任务。

## Requirements

### 范围（Must）

1. **根级字段透传**：Codex profile 可声明 `model_catalog_json`（字符串路径，如 `~/.codex/models.json`）与 `preferred_auth_method`；切换时写入 config.toml 根级，未声明或切走时移除（与现有 `model` / `model_reasoning_effort` 同一套写入/清理语义）。
2. **Bearer Token 认证模式（完整不变量）**：新增认证投递方式，把 profile 的 API Key 写入 `[model_providers.custom].experimental_bearer_token`，不落 auth.json、不依赖环境变量；该模式下自动派生 `preferred_auth_method = "apikey"` 与 `forced_login_method = "api"`（profile 显式声明可覆盖），保证 UI 只选模式即可产出完整 DeepSeek 组合。
3. **模型字段**：沿用现有 `profile.model` → 根级 `model` 机制（已支持，验收时纳入端到端断言即可）。
4. **切换幂等与清场**：重复切换同一 profile 结果稳定；切到其他 profile 或 `ccr codex off` 后，新增根级字段与 bearer token 不残留。
5. **诊断/修复一致性**：inspect/status 链路认识全部新字段——配置与 profile 一致时不得报漂移；漂移时 `ccr codex fix --repair-runtime`（显式授权开关，`crates/ccr-cli/src/commands/codex/fix.rs`）可安全重放修复。
6. **Secret 安全（写入契约）**：
   - profiles 存储沿用 Secret 脱敏 + runtime secret store，profiles.toml 永不落明文；
   - 内存承载遵守现有 route（非密）/ credential（密）分层：明文不得进入派生 `Debug` 的路由对象（承载点用不派生 Debug 的 AuthSelection 或掩码 Debug 的 `Secret`）；
   - config.toml 与其备份的写入必须走「先收权限后写内容」的 secret-aware 原子写（`AtomicWriter.secret(true)` 契约，禁止写后补权限）；CCR 内所有整写 config.toml 的入口（切换路径、UI Settings、Codex MCP、Unified MCP）统一收敛到 secret-aware 写入；
   - 输出侧遵守上文「明文边界」。
7. **同步披露**：config.toml 已是 `codex-config` 加密同步资产（v2 加密信封，见 sync-security-contracts）。bearer token 会随之同步属预期行为，需在用户文档中明确披露；不改同步机制。
8. **向后兼容**：存量 Codex profiles（openai_chatgpt / openai_api_key / provider_env_key / no_auth）行为不变；不做 profiles 存储格式迁移（新字段均为 platform_data 增量键）；UI Settings / MCP 整写路径对新字段的保留能力（`CodexConfig.other` flatten 兜底，已核验存在）以回归测试固化。
9. **UI 同步（含 Tauri 桥接层，rev2 扩充）**：
   - Tauri 命令层：`EXPLICIT_PLATFORM_STRING_FIELDS` 白名单补 `model_catalog_json` / `preferred_auth_method` / `forced_login_method`（当前白名单缺失则具名表单字段在保存时被丢弃，`ccr-ui/src-tauri/src/commands/codex.rs:264`）；profile DTO 投影补对应具名字段；补命令层测试。
   - Profile 编辑器支持新认证模式与新字段；bearer 模式下派生字段以只读/默认态呈现，显式覆盖走高级入口；用 UI 打开/保存新模式 profile 不得把 auth_mode 静默改写为 `no_auth`（当前未知模式会回落，属数据破坏，必须堵住）。
   - 新增 DeepSeek 内置 provider 模板（预填 base_url `https://api.deepseek.com/`、模型 `deepseek-v4-flash` 等非密字段），遵守 provider-template-contracts：绝不预填/存储 API Key。
10. **示例与文档**：`examples/codex/` 与 `docs/` 补 DeepSeek 形态示例（含 models.json 获取方式说明——官方脚本/文档链接）、备份文件含密与加密同步披露；不携带真实凭据。

### 非目标（Won't，本期）

- 不自动生成/下载/维护 models.json 内容（约 40KB 且随上游演进，由用户经 DeepSeek 官方脚本或手动创建；CCR 仅透传路径）。
- 不做 typed IPC 掩码化重构（sentinel 往返）——独立任务，现状边界见上文。
- 不做 Windows DACL 硬化（`AtomicWriter.secret` 在 Windows 为文档化 no-op，属仓库级已知限制，延续 Grok 任务先例列为后续候选）。
- 不校验 Codex CLI 版本 >= 0.144.0（可作为 doctor 后续候选）。
- 不扩展 `ccr codex profile set-field` 的可编辑字段白名单（CLI 侧沿用现状：UI 编辑器或手改 profiles.toml）。
- 不涉及 OpenCode / Grok / Claude 平台；不做 DeepSeek 账号 check-in / 配额查询；不改同步机制本身。

### 约束

- `model_provider` 固定 `custom`（用户明确要求，且与现有双路分发架构一致）。
- `wire_api` 保持 `responses`（DeepSeek 原生支持；现有校验拒绝其他值的行为不变）。
- 遵守仓库红线：masking、破坏性变更前备份、文件锁、原子写全部保留。

## Acceptance Criteria

- [ ] 配好 DeepSeek profile（base_url + key + model=deepseek-v4-flash + model_catalog_json + model_reasoning_effort=high，`preferred_auth_method`/`forced_login_method` 由 bearer 模式自动派生）后执行切换，生成的 config.toml 与 research 文档中官方样例语义等价（provider 段名为 `custom`，其余字段一一对应），auth.json 不新增 OPENAI_API_KEY。
- [ ] 从 DeepSeek profile 切到官方 profile / 其他第三方 profile / `ccr codex off`，`model_catalog_json`、`preferred_auth_method`、`experimental_bearer_token` 全部消失；反向切回再次齐全。
- [ ] 切换后立即 inspect：零漂移；人为篡改 config.toml 中任一新字段后 `ccr codex fix --repair-runtime` 可修复回 profile 期望值（`--dry-run` 只预览不写入）。
- [ ] Secret 链路断言：bearer 明文不出现在日志、tracing、status/诊断 JSON、dashboard DTO、`extra` 投影中（明文边界之外零出现）；config.toml 及其备份写入走 secret-aware 原子写（Unix 断言临时文件先 0600 后写内容语义，沿用 AtomicWriter 既有测试基建）。
- [ ] 兼容回归：存量四种 auth_mode 既有测试全部通过（`crates/ccr/tests/commands/codex_profile.rs`、`codex_fix.rs`、`current.rs` 等）；bearer 在场时经 UI Settings 保存 / MCP 增删改一轮，`experimental_bearer_token`、`model_catalog_json`、`preferred_auth_method` 原样保留（flatten 回归测试）。
- [ ] UI：Tauri 命令层测试覆盖新字段白名单与投影往返；Profile 编辑器创建/编辑 DeepSeek 形态 profile 往返保存不丢字段、auth_mode 不回落；DeepSeek 模板出现在 Codex 模板选择器且不含任何密钥字段；相关 smoke tests + `bun run type-check` + `bun run lint` 通过。
- [ ] `just lint-strict` 与 `just test` 通过；UI 侧 `just frontend-check-quick` 通过。
- [ ] 示例/文档更新完成（含 models.json 获取方式、备份含密、加密同步披露），不含真实凭据。

## Notes

- 技术定案（bearer 承载分层、secret-aware 写入收敛范围、models.json 检查语义、Tauri 桥接触点）见 `design.md`；执行顺序与验证命令见 `implement.md`。
- 真机验证（用真实 DeepSeek key 跑通 codex 会话）依赖用户提供 key，列为人工验收项，不阻塞自动化验收。
