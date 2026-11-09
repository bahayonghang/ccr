# Research: ccr facade 收拢 — 事实盘点（inventory）

- **Query**: 07-03-arch-ccr-facade PRD 前提逐项核查（dispatch 迁移 / 死依赖 / 消费面 / 守卫）
- **Scope**: internal
- **Date**: 2026-07-05
- **约束 spec**: `.trellis/spec/ccr/backend/public-api-boundary.md`（存在，路径确认）；`.trellis/spec/ccr-core/backend/ccr-error-freeze.md`

> 所有 `rg` 命令在仓库根执行。注意本机 shell 中裸 `rg` 被 alias 到 grep，需用 `/c/Users/lyh/scoop/shims/rg` 或在正常环境直接 `rg`。

---

## A. dispatch.rs（crates/ccr/src/cli/dispatch.rs，748 行）

### A1. 全部 use / 路径引用，按来源 crate 分类

文件头显式 `use`（L5-9）：

| 导入 | 实际来源 |
|---|---|
| `crate::cli::subcommands::{AllSyncAction, FolderAction}` | `ccr_cli::cli::subcommands`（经 `crates/ccr/src/cli/mod.rs:6` 转发） |
| `crate::cli::{CleanAction, Cli, Commands, DEFAULT_CLEAN_BACKUP_DAYS}` | `ccr_cli::cli::definitions`（`cli/mod.rs:3-7` 转发） |
| `crate::help` | **ccr crate 本地私有模块**（`crates/ccr/src/help.rs`，52 行，只依赖 clap + `crate::cli::build_cli_command`） |
| `ccr_core::core::error::CcrError` | ccr_core |
| `std::result::Result` | std |

正文内联路径（复现：`rg -o '\b[a-z_][a-z0-9_]*::' crates/ccr/src/cli/dispatch.rs | sort | uniq -c`）：

- `crate::commands::*`（110+ 处）→ 全部 `ccr_cli::commands`（lib.rs:147 桥）。含 `migration`、`sync_cmd`、`codex::{auth,profile,quota,sessions,sync_history,env}`、`claude::{auth,profile}`、`opencode::auth`、`skills_cmd`、`prompts_cmd`、`sessions_cmd`、`provider_cmd`、`SyncContentSelector`、`StatsArgs/BudgetArgs/PricingArgs/ImportMode` 等
- `crate::services::ui_service::UiService`（L198）→ `ccr_cli::services::ui_service`
- `crate::cli::subcommands::*`（Ui/Sync/TempToken/Platform/Check/Codex/OpenCode/Claude Action 枚举）→ ccr_cli
- `ccr_core::core::ColorOutput`（L686、703、735）
- **`crate::tui::*` → `ccr_tui::tui`（lib.rs:150 `pub use ccr_tui::tui`）**，见 A2

### A2. 循环依赖检查（关键）

`ccr-tui` 依赖 `ccr-cli`：`crates/ccr-tui/Cargo.toml:16` `ccr-cli = { path = "../ccr-cli" }`。

dispatch.rs 引用 ccr_tui 的位置（复现：`rg -n 'cfg\(.*tui|crate::tui' crates/ccr/src/cli/dispatch.rs`）：

| 行 | 符号 | 分支 |
|---|---|---|
| 174-176 | `crate::tui::run_tui()` | 无子命令 → 主 TUI |
| 373-375 | `crate::tui::codex_auth::run_codex_auth_tui()` | `ccr codex`（无 action） |
| 573-575 | `crate::tui::opencode_auth::run_opencode_auth_tui()` | `ccr opencode`（无 action） |
| 607-609 | `crate::tui::claude_auth::run_claude_auth_tui()` | `ccr claude`（无 action） |

全部在 `#[cfg(feature = "tui")]` 下，且各自有 `#[cfg(not(feature = "tui"))]` 降级分支（current/auth list/help）。4 个入口均为 `fn() -> Result<(), CcrError>` 形状（`ccr_core::core::error::Result`）。

**结论：dispatch.rs 整体迁入 ccr-cli 会形成 ccr-cli → ccr-tui → ccr-cli 循环，cargo 直接拒绝。硬否决点成立，必须剥离 TUI 分支**（可行剥离方式见 feasibility.md）。

除 ccr_tui 外，dispatch.rs 未引用任何依赖 ccr-cli 的 crate。

main.rs（42 行）引用：`ccr::cli::{Cli, CommandDispatcher, build_cli_command}`、`ccr::{init_logger, init_file_only_logger}`（ccr_core 转发）、`ccr::cli::dispatch::handle_error`、`clap::FromArgMatches`、`cli.is_tui_mode()`（定义在 `ccr-cli/src/cli/definitions.rs:70`，`#[cfg(feature = "tui")]`，ccr-cli 自带空 `tui` feature）。main.rs 无任何 ccr_tui 直接引用。

### A3. Cargo.toml 依赖差异（ccr vs ccr-cli）

ccr 有而 ccr-cli 没有的依赖：`ccr-tui`(optional)、`ccr-store`（ccr-cli 也有 ccr-store，忽略）——实际差集：**ccr-tui、urlencoding/sysinfo/unicode-width 的平台条件差异**（ccr-cli 把 libc/urlencoding/sysinfo/unicode-width 放在 `[target.'cfg(unix)'.dependencies]`，ccr 是无条件依赖）。dispatch.rs 用到的外部 crate 只有 `ccr_core`（ccr-cli 已依赖）与 std——**dispatch 本身不需要 ccr-cli 新增任何依赖**（TUI 分支剥离后）。

ccr src 实际引用的 extern crate 全集（复现：`rg -o '\b[a-z_][a-z0-9_]*::' crates/ccr/src --no-filename | sort | uniq -c`，再逐一核对）：`ccr_cli`、`ccr_core`、`ccr_store`（lib.rs:306-310）、`ccr_tui`（lib.rs:150）、`clap`（main.rs/help.rs）、`tokio`（main.rs `#[tokio::main]`）。**其余 ~25 个 [dependencies]（anyhow、ccr-types、chrono、dirs、reqwest、serde、thiserror、toml、base64、blake3、colored、comfy-table、dialoguer、filetime、futures、fs4、once_cell、rayon、reqwest_dav、rpassword、tracing 系、uuid、walkdir、urlencoding、sysinfo、unicode-width…）在 src 0 引用**（tests 用到 indexmap/serde_json/tempfile，见 B）。

### A4. handle_error（dispatch.rs:734-748）

- 公开路径 `ccr::cli::dispatch::handle_error`；全仓唯一消费方 `crates/ccr/src/main.rs:32`（复现：`rg -n 'handle_error' --glob '*.rs'`）。
- `public_api_compat.rs` 的快照只扫描 `crates/ccr/src/lib.rs`（`include_str!("../src/lib.rs")`，L117），`cli` 模块内部不在快照内 → **迁走 handle_error / dispatch 不触发快照变更**（只要 lib.rs 不动）。
- `ccr-error-freeze.md:20`：`exit_code()/is_fatal()/user_message()` 单一消费者即此函数；`:59` 已预留"把渲染搬到 dispatch 旁"为 facade 议题。迁移与该 spec 方向一致，无冻结冲突（CcrError 本体不动）。

---

## B. 死依赖验证（PRD 称 4 个：ccr-config / ccr-skills / ccr-codex / ccr-sync）

复现：
```
rg -n 'ccr_config::|ccr_skills::|ccr_codex::|ccr_sync::' crates/ccr/src
rg -n 'ccr_config::|ccr_skills::|ccr_codex::|ccr_sync::' crates/ccr/tests
```

| crate | src 引用 | tests 引用 | 判定 |
|---|---|---|---|
| ccr-skills | 0 | 0 | **死依赖，可删** |
| ccr-codex | 0 | 0 | **死依赖，可删** |
| ccr-sync | 0 | 0 | **死依赖，可删** |
| ccr-config | 0 | **6 处 / 2 文件**：`tests/commands/claude_profile.rs:8,122,126`、`tests/commands/codex_profile.rs:7,113,117`（`ProfileConfig` + `profile_to_section`） | **非纯死依赖**：需移入 `[dev-dependencies]`（ccr 目前**没有** dev-deps 段）或改写测试。`profile_to_section` 不经任何 ccr 转发路径可达（ccr::models 只转发 Platform/PlatformConfig/PlatformPaths/ProfileConfig 四个类型），故不能简单改成 `ccr::` 路径 |

lib.rs re-export 核查：lib.rs **没有任何 `pub use ccr_config::/ccr_skills::/ccr_codex::/ccr_sync::` 直连**（复现：`rg -n 'ccr_(config|skills|codex|sync)::' crates/ccr/src/lib.rs` → 0）。`pub use managers::{…}` / `pub use sync::{…}` / `pub use models::{…}` 全部经 `ccr_cli` 模块路径转发，类型可达性由 ccr-cli 的依赖保证，**删除这 4 个直接依赖不影响 lib.rs 编译**。

附带事实（超出 PRD 声称）：如 A3，ccr 的 [dependencies] 中 src 0 引用的还有 ~25 个；tests 额外用 `indexmap`(8)、`serde_json`(5)、`tempfile`(12)（复现：`rg -o '^use (indexmap|serde_json|tempfile|toml|base64|chrono|clap)' crates/ccr/tests`）→ 这三个 + ccr-config 需要 `[dev-dependencies]`。

---

## C. 消费面盘点

### C6. ccr-ui/src-tauri 对 `ccr::` 的消费（复现：`rg -n '\bccr::' ccr-ui/src-tauri/src`）

排除注释后共 **3 处真实消费**：

| 位置 | 符号 |
|---|---|
| `commands/claude.rs:15` | `ccr::platforms::ClaudePlatform` |
| `commands/claude.rs:16` | `ccr::services::ClaudeAuthService` |
| `commands/config.rs:183` | `ccr::commands::switch_command` |

`ccr::application/sessions/sync/managers/models` 在 src-tauri 0 消费。src-tauri 依赖 `ccr = { path=…, default-features = false }`（无 tui）。

### C7. `use ccr_cli::` 全量清单

复现：`rg -n 'ccr_cli::' ccr-ui/src-tauri/src crates/ccr-tui/src crates/ccr/src`

- **src-tauri**（4 处，全在 install 流）：`ccr_cli::services::install_detect`（模块）、`ccr_cli::services::install_service::InstallService`（install.rs:8 + main.rs:151）、`ccr_cli::services::install_types::{AttemptId, CancelResult, DetectionResult, HostCapabilities, InstallPlan, ManualCatalog, PlanOutcome, RingBufferSnapshot}` —— 注意**全部走模块路径，不经 services/mod.rs 的平铺 re-export**。
- **ccr-tui**：`lib.rs:3` `pub use ccr_cli::{models, platforms, services}`（自建别名墙）+ `tui/app.rs:8` `use ccr_cli::managers::{TuiConfigManager, TuiTabId}`。TUI 内部经 `crate::models/services/platforms::` 消费的符号全集（复现：`rg -o 'crate::(models|managers|services|platforms)::[A-Za-z_]+' crates/ccr-tui/src | sort -u` + 展开 grouped use）：
  - models（18）: ClaudeLoginState, ClaudeProfileAuthMode, ClaudeRuntimeMode, ClaudeRuntimeSummary, ClaudeAuthRegistry, ClaudeCurrentAuthInfo, CodexAccountQuota, CodexAuthAccount, CodexAuthItem, CodexAuthJson, CodexAuthRegistry, CodexAuthTokens, CodexQuota, CodexRuntimeMode, CodexRuntimeSummary, CodexUsageActivation, CodexToOpenCodeMigrationReport, LoginState, OpenAiAuthMethod, OpenCodeAuthItem, OpenCodeAuthRegistry, OpenCodeLoginState, OpenCodeReadSnapshot, Platform, PlatformConfig, PlatformPaths, ProfileConfig
  - services（15）: AuthReadSnapshot, ClaudeAuthItem, ClaudeAuthService, CodexAuthService, CodexOAuthTokenService, CodexQuotaService, CodexRollingUsage, CodexUsageRecord, CodexUsageService, OpenCodeAuthService, OpenCodeQuotaService, OpenCodeRollingUsage, OpenCodeUsageRecord, OpenCodeUsageService
  - platforms: ClaudePlatform, create_platform；managers: TuiConfigManager, TuiTabId
- **crates/ccr/src**：`cli/mod.rs:3-7`（Cli/Commands/build_cli_command/subcommands/Clean*）+ `lib.rs:147`（七模块桥）。

### C8. re-export 墙逐条判定（ccr-cli/src/{models,managers,services}/mod.rs）

判定口径：**"消费方"= 经墙路径（`crate::X::`、`ccr_cli::X::`、`ccr::X::`、ccr-tui 别名）访问**。直接从源 crate（ccr_codex/ccr_skills…）import 的不算。复现方法：对每个符号
```
rg -c '\bSYM\b' crates/ccr-tui/src ccr-ui/src-tauri/src crates/ccr/src crates/ccr/tests
rg -c '(crate|ccr_cli)::(models|managers|services)::SYM\b' crates/ccr-cli/src
# + 人工展开 grouped: rg -U 'use crate::(models|managers|services)::\{[^}]*\}' crates/ccr-cli/src --multiline
```

**models/mod.rs**（4 个 allow 组 + 1 个非 allow 组）：

| 条目 | 判定 | 消费方 |
|---|---|---|
| `CodexAuthTokens` | 保留 | ccr-tui |
| codex_auth 组：AuthIntent, AuthState, AuthStateStatus, CodexProfileAuthMode, CredentialStoreKind, CurrentAuthInfo, ImportMode | 保留 | ccr-cli 内部 `crate::models::` |
| codex_auth 组：CodexAccountQuota, CodexAuthAccount, CodexAuthItem, CodexAuthJson, CodexAuthRegistry, CodexQuota, CodexRuntimeMode, CodexRuntimeSummary, CodexUsageActivation, LoginState | 保留 | ccr-tui（部分兼 ccr-cli 内部） |
| codex_auth 组：**OpenAiAuthMethod** | 保留（**锁定**） | ccr-tui + ccr-cli + `public_api_compat.rs:12`（`ccr::models::OpenAiAuthMethod`） |
| codex_auth 组：**CodexAuthExport, CodexAuthExportAccount, CodexProfileSecret, CodexProfileSecretStore, normalize_auth_map_for_intent** | **可删** | 0 墙路径消费（src-tauri 用 `ccr_codex::CodexAuthExportAccount` 直连） |
| opencode_auth 组：CodexToOpenCodeMigrationItem, CodexToOpenCodeMigrationStatus | 保留 | ccr-cli 内部 |
| opencode_auth 组：CodexToOpenCodeMigrationReport, OpenCodeAuthItem, OpenCodeAuthRegistry, OpenCodeLoginState, OpenCodeReadSnapshot | 保留 | ccr-tui |
| opencode_auth 组：**OpenCodeAuthAccount, OpenCodeCurrentAuthInfo, OpenCodeOpenAiAuth** | **可删** | 0 |
| `ccr_config::{Platform, PlatformConfig, PlatformPaths, ProfileConfig}`（无 allow） | 保留（**锁定**） | lib.rs 桥 + prelude + 快照 + 全员 |
| ccr_types 组（7 个 Claude*） | 保留 | ccr-cli 内部 + ccr-tui + `ccr::models::ClaudeAuthRegistry`（tests/commands/claude_profile.rs:7） |

**managers/mod.rs**（17 条 allow）：

| 条目 | 判定 | 消费方 |
|---|---|---|
| BudgetManager, CostTracker, PricingManager | 保留（锁定） | lib.rs 桥（快照行 75-79）+ ccr-cli 内部 |
| config::{CcsConfig, ConfigManager, ConfigSection, GlobalSettings, ProviderType} | 保留（锁定） | lib.rs 桥 + ccr/tests + ccr-cli |
| PlatformConfigEntry, PlatformConfigManager, UnifiedConfig | 保留（锁定） | lib.rs 桥 + ccr/tests + ccr-cli |
| settings::{ClaudeSettings, SettingsManager} | 保留（锁定） | lib.rs 桥 + ccr/tests + ccr-cli |
| temp_override::{TempOverride, TempOverrideManager} | 保留（锁定） | lib.rs 桥 + `tests/workflows/temp_override.rs:5` |
| history::{HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType} | 保留 | lib.rs 桥(HistoryManager) + `tests/managers/general.rs:9-11` + ccr-cli 内部 |
| CodexConfigManager | 保留 | ccr-cli 内部 `crate::managers::CodexConfigManager` |
| TuiConfigManager, TuiTabId | 保留 | ccr-tui `tui/app.rs:8` |
| **CachedCodexConfigManager** | **可删** | 0 |
| **TuiConfig** | **可删** | 0 |
| **history::{EnvChange, HistoryStats}** | **可删** | 0 |
| **ccr_sync::{SyncConfig, SyncConfigManager}**（managers 路径） | **可删（整行）** | 0（lib.rs 桥的 SyncConfig 走 `sync::` 模块，非 managers） |
| **ConfigFileHandler** | **可删** | 0 |
| **ConfigValidator, ValidationReport** | **可删** | 0 墙路径（内部走 `config_validator::` 模块路径） |
| **McpPresetManager, McpSyncManager, get_builtin_presets** | **可删** | 0（src-tauri 走 `ccr_skills::` 直连，mcp_presets.rs:4,72…） |
| **CachedSettingsManager** | **可删** | 0 墙路径 |
| **SyncFolderManager**（managers 路径） | **可删** | 0（桥走 sync:: 模块） |

**services/mod.rs**：

| 条目 | 判定 | 消费方 |
|---|---|---|
| BackupService, ConfigService, HistoryService, SettingsService, SkillsService, ValidateService | 保留（锁定） | lib.rs 桥（快照行 102-104）+ tests + ccr-cli |
| `pub use ccr_codex::services::codex_session_service;`（模块行） | 保留（锁定） | `public_api_compat.rs:17`（legacy 测试） |
| **`codex_session_service::CodexSessionInventory`（平铺行）** | **可删** | 0 墙路径（tauri/tests 走模块路径） |
| ccr_codex 大组中保留：AuthReadSnapshot, CodexAuthService, CodexOAuthTokenService, CodexQuotaService, CodexRollingUsage, CodexUsageRecord, CodexUsageService, OpenCodeAuthService, OpenCodeQuotaService, OpenCodeRollingUsage, OpenCodeUsageRecord, OpenCodeUsageService, CodexHistoryBackupPruneResult, CodexHistoryProviderBuckets, CodexHistorySyncOptions, CodexHistorySyncResult, CodexHistorySyncService, CodexHistorySyncStatus, CodexSessionTrashService | 保留 | ccr-tui / ccr-cli 内部 |
| ccr_codex 大组中**可删**：CodexAuthCacheAction, CodexHistoryBackupSummary, CodexHistoryRestoreResult, CodexHistoryVisibilityDiagnostics, CodexRuntimeCommitPlan, CodexRuntimeService, CodexSessionDetail, CodexSessionExport, CodexSessionMessage, CodexSessionRestoreSummary, CodexSessionService, CodexSessionSummary, CodexSessionTrashSummary, CodexTrashedSessionRecord, CodexUsageStats, OpenCodeReadSnapshot, OpenCodeUsageStats | **可删（17 个）** | 0 墙路径（src-tauri 的 CodexSessionService 走 `ccr_codex::` 直连，codex.rs:18-20；tui 的 OpenCodeReadSnapshot 走 models 路径） |
| ClaudeAuthItem, ClaudeAuthService | 保留 | ccr-tui + src-tauri（`ccr::services::ClaudeAuthService`）+ ccr-cli |
| **ClaudeAuthReadSnapshot** | **可删** | 0 |
| **doctor_service::{DoctorCheck, DoctorReport, DoctorRunOptions, DoctorService, DoctorStatus}（平铺）** | **可删（5 个）** | 0 墙路径（内部走 `doctor_service::` 模块） |
| MultiBackupService | 保留 | ccr-cli 内部 |
| runtime_overview::{PlatformStatusCard, RuntimeOverview, RuntimeOverviewService, StatusAuthKind, StatusHealth} | 保留 | ccr-cli 内部（commands/profile/current.rs:14-17） |
| SyncService（services 路径） | **可删** | 0（桥走 sync:: 模块） |
| **UiService（平铺）** | **可删** | 0（dispatch 走 `ui_service::UiService` 模块路径） |
| **InstallService（平铺）+ install_types 平铺组 {AttemptId, CancelResult, DetectionResult, HostCapabilities, InstallEvent, InstallFlowError, InstallPlan, ManualCatalog, PlanOutcome, RingBufferSnapshot}** | **可删（11 个）** | 0 墙路径（src-tauri 全走 `install_service::` / `install_types::` 模块路径） |

**统计**：可删约 **58 个符号**（models 8、managers 14、services ~36）；保留约 70 个，每条均有明确消费方。锁定项（快照/legacy 测试锁死，动 lib.rs 才能动）：managers 16 项、services 6 项+codex_session_service 模块行、models 的 Platform 四件套 + OpenAiAuthMethod、`models::skills` 全组、platforms 5 项、sync 组 13 项。

### C9. ccr-tui/src/lib.rs 的 re-export

`rg -n 'ccr_tui::' --glob '*.rs'` 全仓仅 2 处：`crates/ccr/src/lib.rs:150`（`pub use ccr_tui::tui`，快照锁定）和 `public_api_compat.rs:62`（快照字符串本身）。**`ccr_tui::{models, platforms, services}` 别名 0 外部消费**，纯 TUI 内部便利（约 15 个文件经 `crate::models/…` 使用）→ 可删该行，TUI 内部改 `use ccr_cli::…` 直连；不影响快照（快照锁的是 `ccr_tui::tui` 这条，保留 `pub mod tui` 即可）。

---

## D. 守卫与测试面

### D10. public_api_compat.rs（229 行，3 个测试）

- `legacy_public_paths_remain_available`：锁 `ccr::application::switch_platform`、`SwitchPlatformRequest`、`ccr::commands::ImportMode`、`ccr::managers::ConfigManager`、`ccr::models::OpenAiAuthMethod`、`models::prompt::PromptPreset`、`models::mcp_preset::McpServerSpec`、`models::skills::SkillOperationResponse`、`services::codex_session_service::CodexSessionInventory`、`sessions::{SessionFilter, SessionIndexer, SessionSummary}`。
- `stable_prelude_paths_remain_available`：锁 prelude 18 项。
- `crate_root_public_reexport_snapshot_is_intentional`：**逐行文本快照，仅针对 lib.rs**（`include_str!("../src/lib.rs")`），从 `pub mod cli;` 起扫描 `pub use`/`pub mod` 行 + 特定注释。`crates/ccr/src/cli/mod.rs`（8 行）的内容**不在快照内**——dispatch 迁走后只要 lib.rs 一字不动、`ccr::cli` 模块继续 re-export 同名符号（Cli/Commands/build_cli_command/CommandDispatcher/dispatch::handle_error），快照与 legacy 测试都不变。

### D11. 集成测试面

- `crates/ccr/tests/commands/` 实为 **10 个文件 / 54 个 `#[test]`**（PRD 写"24 个集成测试"与实际不符；复现：`rg -c '#\[(tokio::)?test\]' crates/ccr/tests`；无 `#[tokio::test]`）。tests 全目录合计 155 个 test fn。
- tests/commands 全部是**黑盒子进程测试**：`Command::new(env!("CARGO_BIN_EXE_ccr"))`（10 文件全命中）→ 只依赖 ccr 二进制行为，**dispatch 内部搬家零影响**；受影响的只有 B 节的 `ccr_config` dev 依赖问题。
- 白盒路径消费：`tests/{managers,workflows,platforms}` + `commands/sync_content.rs` 经 `ccr::managers/services/models/commands` 桥路径 import（详见 C8 表），墙瘦身不得删这些符号。
- dispatch 路由逻辑可测性现状：`CommandDispatcher::dispatch(cli: &Cli) -> Result<(), CcrError>` 是无状态静态方法，但 match 分支**直接 await 真实 *_command**（读写真实 HOME/CCR_ROOT），路由选择与执行耦合；`handle_error` 调 `std::process::exit`。当前形状只能黑盒测（已有 legacy_routing.rs 4 例）或测纯输出分支（Version/Help）；要"直接单测路由正确性"需把 match 的目标改为可注入/可枚举的形状——这是迁移时补测试的实质工作量。
