# 审阅核验记录 rev2（2026-08-01）

> 对 Codex 审阅报告 7 项发现的逐条代码核验。结论：7/7 属实（2 阻断 / 2 高 / 3 中），其中发现 2、4 按实证校正了部分范围表述。全部已吸收进 prd/design/implement rev2。

## 发现 1 [阻断] Secret 验收与 UI 契约冲突 — ✅ 属实

- `ccr-ui/src-tauri/src/commands/codex.rs:1624-1625`：profile DTO 显式 `profile.auth_token.as_ref().map(ccr_core::Secret::expose)`，注释原文即"编辑表单预填需要原文：显式 expose（掩码化改造属 typed-ipc 任务）"——明文预填是现状既定通道，且掩码化重构已被仓库锚定为独立任务。
- `ccr-ui/src-tauri/src/commands/settings_raw.rs:311`（`codex_get_config_raw_text`）：Raw Source 编辑器整文读取 config.toml，第二条合法本地明文通道。
- **处置**：采纳审阅者提议的明文边界（两条用户显式触发的本地通道 + 磁盘既定落点允许明文，其余一律掩码；typed-IPC 重构独立任务）。已写入 prd「明文边界」与 design §3。

## 发现 2 [阻断] 写后补权限违反契约 — ✅ 属实（范围校正一处）

- 契约：`crates/ccr-core/src/core/atomic_writer.rs:165-175, 208-219`——`secret(true)` 在写入内容**之前**收紧临时文件到 0600（保留既有 owner-only mode），Windows 文档化 no-op。原设计"写后 `ensure_private_permissions`"确实违约。
- 现状违约点：`crates/ccr-codex/src/managers/codex_config.rs:202`（`NamedTempFile`+`fs::write`）、`:253`（备份 `fs::copy`）；`ccr-ui/src-tauri/src/commands/codex.rs:835`（`write_codex_config` 自带 tempfile 路径）；`ccr-ui/src-tauri/src/commands/unified_mcp.rs:572`（`write_json_config`：裸 `fs::write` + rename，连 NamedTempFile 都不是）。`codex_mcp.rs:71` 经 `write_codex_config`，同源。
- **校正 ①**：审阅暗示整写可能丢字段——实证 `CodexConfig` 定义（`codex.rs:100-102`）有 `#[serde(flatten)] pub other: HashMap<String, toml::Value>` 兜底，typed 整写**不丢**未知键（`experimental_bearer_token`/新根级键均保留）。问题只剩权限，另以 flatten 回归测试固化保证。
- **校正 ②**：Windows DACL 硬化超出 AtomicWriter 现有能力（文档化 no-op），属仓库级已知限制（Grok 任务同列后续候选），不纳入本任务。
- **先例**：`CodexMcpServer.bearer_token`（`codex.rs:132`）说明 config.toml 今天已承载明文密钥——本任务是把全部整写入口收敛到契约化路径的净改善。
- **处置**：design 新增 D5（四个写入点统一 `AtomicWriter.secret(true)`，备份弃 `fs::copy`）；implement 新增 A0 与 B0。

## 发现 3 [高] 明文入 Debug 路由对象 — ✅ 属实

- `crates/ccr-codex/src/platforms/codex.rs:44`：`RouteSelection` 派生 `Debug`；`:58`：`AuthSelection` 刻意不派生——分层意图明确。原设计把 `bearer_token: Option<String>` 放进 RouteSelection 确实破坏分层。
- `ccr_core::Secret`（`crates/ccr-core/src/core/secret.rs:33`）派生 `Clone/Default/PartialEq/Eq` 且 Debug/Display/默认 Serialize 全掩码——两种修法均可行。
- **处置**：选 `AuthSelection::WriteProviderBearerToken(Secret)`（分层 + 掩码双保险），RouteSelection 保持非密。design D2 已改。

## 发现 4 [高] UI 配不全 + Tauri 白名单丢字段 — ✅ 属实（细化一处）

- `ccr-ui/src-tauri/src/commands/codex.rs:264`：`EXPLICIT_PLATFORM_STRING_FIELDS` 无 `forced_login_method`（今天就配不了）、自然也无两个新键；`:1449-1480` `parse_platform_data_update` 只认白名单具名字段 + `extra`/`platform_data` 对象；投影 `:1634+` 无对应具名字段。
- **细化**：`extra` 对象透传通道存在（未知键可经 `extra` 往返），但前端编辑器（`codexProfileEditor.ts`）走具名字段序列化——按计划的具名字段路线，白名单/投影/编辑器三层都必须补，审阅结论对计划成立。
- **处置**：采纳"bearer 模式 = 完整不变量"（后端派生 `preferred_auth_method=apikey`、`forced_login_method=api`，显式覆盖优先）；implement B0 补白名单/投影/命令测试。

## 发现 5 [中] 修复命令写错 — ✅ 属实

- `crates/ccr-cli/src/commands/codex/fix.rs:65-66`：重放仅在 `decide_runtime_repair(dry_run, repair_runtime, ...)` 返回 Apply 时执行，需显式 `--repair-runtime`。
- **处置**：prd 验收与 implement A7 全部改为 `ccr codex fix --repair-runtime`（`--dry-run` 只预览）。

## 发现 6 [中] models.json 检查自相矛盾 — ✅ 属实

- 原设计"原样透传 `~`"与"检查文件存在"并存会把 `~` 当字面目录 → 假告警；`tracing::warn` 也不等于用户可见提示。
- **处置**：design D3 改为——仅在检查副本上展开 home 前缀；写盘保留原值；告警走切换命令用户可见输出（`ColorOutput::warning`）+ `tracing::warn`；无法展开则跳过检查。

## 发现 7 [中] JSONL 清单不完整 — ✅ 属实

- 五个 spec 均存在：`ccr-core/backend/atomic-writer.md`、`ccr-codex/backend/codex-app-server-cleanup.md`、`ccr-ui/frontend/profiles-page-contracts.md`、`ccr-ui/frontend/raw-config-editor-contracts.md`、`ccr-ui/frontend/sync-security-contracts.md`。
- `sync-security-contracts.md:68`：`pushSyncAsset('codex-config', ...)` 确认 config.toml 是加密同步资产（v2 加密信封）——bearer 随信封同步，需文档披露。
- **处置**：两份 jsonl 各补 5 条；prd 新增同步披露要求（Must 7）；docs 任务 C2 落实。

## 明文边界问题的答复

同意审阅者提议：保留「Profile 编辑器预填」与「Raw Source 编辑器」两条用户显式触发的本地明文通道，其余日志/诊断/status/常规 DTO 严格掩码，typed IPC 掩码化重构独立成任务。依据：该边界与代码现状注释（codex.rs:1624 "掩码化改造属 typed-ipc 任务"）及 raw-config-editor-contracts 的既定方向一致，本任务不扩大也不收窄现有明文面。
