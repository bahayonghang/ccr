# Codex Auth 注册表版本门与未知字段保留

## Goal

防止较旧版本 CCR 加载并重写 `auth_registry.toml` 时丢失较新版本写入的字段（未知字段保留）；在未来出现不兼容的主版本时，让较旧 CCR 以只读方式工作，不破坏注册表（版本门）。

## Authorization and Status

2026-10-06 用户已授权按顺序实施现有父子任务。本任务现有 PRD、design.md 与 implement.md 已由主会话审阅，作为首个实施子任务启动。旧的规划阶段说明保留为历史记录。

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务，并从本任务（P2）开始 brainstorm。本版为规划产物，未开始实施；实施需用户批准最终规划摘要后 `task.py start`。规划基线：dev @ 73b47563。

## Background and Confirmed Facts

来源：审计 F14（中，B），见 `.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md`。codex-auth 使用 `schema_version` 门，较新版本拒绝加载。

- `CodexAuthRegistry`（`models/codex_auth.rs:291-308`）与 `CodexAuthAccount`（`:226-268`）都没有扩展字段；未知字段在反序列化时被丢弃，保存时丢失。
- `version` 默认 `"1.0"`（`models/codex_auth.rs:310-312`），代码只在测试中读取（`codex_auth.rs:967`、`codex_auth_service.rs:2456`、`codex_registry_store.rs:120`），没有逻辑分支依赖它。
- 字段历来以追加方式增加且未升版本：`usage_ledger`（37702ed5）、`CodexAuthAccount` 的 `api_base_url`、`api_provider_name`、`plan_type`。实际风险是旧版本 CCR 保存时丢弃新增字段。
- 唯一写入方是 `CodexRegistryStore::save`（`codex_registry_store.rs:58-81`）；`load`（`:45-56`）在文件不存在时返回默认值。ccr-vscode 只读取该文件（`codexAuthReader.ts:27`、`codexRuntimeReader.ts:616`）；Tauri 只计算其哈希（`ccr-ui/src-tauri/src/commands/codex.rs:575`），账号字段经显式 DTO 映射（`ccr-ui/src-tauri/src/commands/codex_auth.rs:354`）；导出使用显式 DTO `CodexAuthExportAccount`（`codex_auth_service.rs:1731-1765`）。保留的未知字段不会进入桌面端 DTO 或导出文件。
- 注册表写入点：
  - 用户命令：`save_current`（`codex_auth_service.rs:954`）、`switch_account`（`:1246`）、`delete_account`（`:1304`）、`update_account_description`（`:1448`）、`rename_account`（`:1570`）、`import_accounts`（`:2029`）。除 `update_account_description` 外，其余命令在保存注册表之前已写入快照、runtime 或删除文件。
  - 后台：`update_registry_metadata`（`codex_oauth_token_service.rs:351-358`，配额与刷新路径）、`sync_current_auth_registry`（`codex_auth_service.rs:727-755`）。
- 先例：`ccr-types` 约定用 `#[serde(flatten)] other` 保留未知字段（`crates/ccr-types/src/lib.rs:10`）；TOML flatten 先例 `ccr-config/src/managers/platform_config.rs:76`；workspace `toml = "1.1.4"`。TUI 按错误前缀本地化的先例：`relogin_required_detail`（`codex_quota_service.rs:22-27`，`ccr-tui/src/tui/codex_auth/ui.rs:972`）。
- 结构字面量构造点：`CodexAuthRegistry { .. }` 全部在测试模块中（`codex_usage_estimation.rs:1666`、`:1688`、`:1765`；`codex_auth_service.rs:2463`、`:3250`、`:3285`；`codex_registry_store.rs:129`；`platforms/codex.rs:3034`）。`CodexAuthAccount { .. }` 生产构造点为 `codex_auth_service.rs:931`（`save_current`）与 `:2004`（导入），其余在测试模块中（含 `ccr-tui/src/tui/codex_auth/ui.rs:2785`）。
- 保护边界：只保护包含本修复的版本之间的往返；已发布的旧版本 CCR 仍会丢字段，也不会遵守版本门。

## Decisions

- D1（2026-10-06 用户）：未知字段保留与版本门两者都做。
- D2（2026-10-06 用户）：遇到更新主版本的注册表时进入只读模式：账号列表与配额可用；6 个写注册表的用户命令被拒绝并提示升级；后台注册表元数据写入记录 warn 后跳过。

## Requirements

| ID  | 要求                                                                                                                                                                                                |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R1  | 未知字段保留：`CodexAuthRegistry` 与 `CodexAuthAccount` 的未知字段在 load → save 往返后原样保留。`save_current` 覆盖已有账号时丢弃该账号旧记录的未知字段（快照已更换为新凭据）。                    |
| R2  | 版本规则：只比较主版本号；本版本支持主版本 1；新建注册表写 `"1.0"`；保存时保留文件中已有的 `version` 值，不降级。追加字段不升版本号。                                                               |
| R3  | 只读模式：`version` 主版本大于 1 或无法解析时，注册表为只读。6 个用户命令在产生任何文件副作用之前返回只读错误；`CodexRegistryStore::save` 对只读注册表同样拒绝写入（兜底）。                        |
| R4  | 后台写入：`update_registry_metadata` 与 `sync_current_auth_registry` 遇到只读错误时记录 warn 并返回成功，不中断配额、刷新、加载流程。快照与 runtime token 同步不受只读模式影响。                    |
| R5  | 加载：主版本更新且完整结构无法解析时，`load` 返回的错误保留前缀「解析注册表失败: 」，并说明该文件由更新版本的 CCR 写入、需要升级 CCR。                                                              |
| R6  | 文本：服务层只读错误使用固定前缀常量（CLI 与桌面端显示该中文文本，与其他服务错误一致）；Codex Auth TUI 的保存、切换、删除、重命名错误 toast 识别该前缀并以 EN/ZH 显示（TUI 没有改描述与导入入口）。 |
| R7  | 约束：不新增 `CcrError` 变体（使用 `ConfigError`）；保存仍走文件锁、写前备份、`AtomicWriter::secret(true)`；不改变 1.x 注册表的任何现有行为与文本。                                                 |

## Acceptance Criteria

- [x] AC1（R1）：含顶层未知键、未知表与账号级未知键的注册表，经 `load` 后 `save`，未知键与值仍在；`save_current --force` 覆盖账号后该账号旧的未知字段被移除。
- [x] AC2（R2）：新建注册表 `version = "1.0"`；`version = "1.7"` 的注册表可写，保存后仍为 `"1.7"`。
- [x] AC3（R3）：`version = "2.0"` 与 `version = "abc"` 时，6 个用户命令返回只读错误，且注册表、快照、runtime `auth.json` 的字节均未改变；`list` 与账号读取正常。
- [x] AC4（R4）：只读注册表下，`update_registry_metadata` 与 `sync_current_auth_registry` 返回 `Ok`，注册表字节不变。
- [x] AC5（R5）：`version = "2.0"` 且包含当前结构无法解析的字段类型时，`load` 错误以「解析注册表失败: 」开头并包含升级提示。
- [x] AC6（R6）：TUI 在 EN 与 ZH 下把只读错误显示为本地化文本，并显示文件中的版本值；有测试断言两种语言。
- [x] AC7（R7）：既有注册表、账号命令、导入导出与同步测试语义不变；`cargo test -p ccr-codex`、`cargo test -p ccr-tui`、`just lint-strict`、`just test` 通过。

2026-10-06 本地验收：独立 AC1–AC7 证据见 `research/independent-check.md` 与 `research/workspace-validation.md`；最终跨子任务源码通过父任务 `just ci`，Codex 381 passed、2 ignored，TUI 253 passed，Rust workspace 2038 passed、0 failed、16 ignored。最终集成证据见父任务 `research/integration-validation.md`。已发布旧版本的兼容行为仍不受本修复保护。

## Out of Scope

- 主动升级注册表主版本，或任何不兼容的结构变更。
- 让已发布的旧版本 CCR 遵守版本门或保留字段。
- 导出/导入携带未知字段（导出使用显式 DTO，维持现状）。
- TUI 中常驻的只读状态指示（只在命令被拒绝时提示）。
- P1 身份键字段本身（10-06-auth-identity-key）。

## Dependencies

- 10-06-auth-identity-key（P1）依赖本任务：P1 新增的注册表字段依赖 R1 在本修复之后的版本间保留。
