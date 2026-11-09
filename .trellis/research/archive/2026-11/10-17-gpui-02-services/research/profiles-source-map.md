# Research: GPUI 04 Profiles source map

- Query: Locate the real 04 config/profile/converter command and route owners, reusable domain APIs, narrow shared orchestration, explicit fixture seams, and safety contracts required before native editing/apply/off work.
- Scope: internal; read-only prerequisite research while 02 implementation is active. No implementation, activation, Cargo/native execution or acceptance claim.
- Date: 2026-10-18

## Findings

### Scope, inventory and current-state limits

Read 04 `prd.md`, `design.md`, `implement.md`, parent `research/feature-matrix.md` and every inventory entry owned by `gpui-04-profiles`. The current inventory assigns **42 command IDs and 5 route IDs** to 04: config 13, config_extended 1, Claude profiles 10, Codex profiles 10, Grok profiles 7, converter 1. Entries are planned; this report does not mutate them. Routes have no redirects in this subset. Do not rerun the capture script over evidence.

The runtime/config sources are changing under assigned 02 owners. At inspection, `crates/ccr-app/src/config.rs` implements **ConfigReadService**, not a general ProfileService; its actual API is `for_claude`, `from_paths`, `list`. `ConfigRow` is a masked display projection with catalog revision. This read-only seam is real source but is not evidence that profile writes/lifecycle are implemented or checked.

### Actual responsibility map

| Source:line | Existing owner / API |
| --- | --- |
| `crates/ccr-app/src/config.rs:33-56` | ConfigReadService: path-only configuration list orchestration; snapshot plus Claude current-profile resolution; synchronous list must run on worker. |
| `ccr-ui/src-tauri/src/commands/config.rs:160-174` | Generic list now delegates through the shared read service and Tauri adapter. |
| `.../commands/config/adapter.rs:23-32` | Generic config surface remains explicit **Claude only**; missing platform differs from unsupported platform. |
| `.../commands/config/adapter.rs:178-284` | Activate/add/update/rename/duplicate/delete orchestration. Mutation validation, active-profile guards and application lock are here. |
| `crates/ccr-config/src/managers/config/repository.rs:12-48` | ConfigPatch / FieldPatch Set vs Remove; omitted fields retained; values intentionally have no Debug. |
| `.../repository.rs:136-198` | snapshot, mutate_versioned, strict patch with caller token and validator. Persistence remains catalog-owned. |
| `crates/ccr-config/src/managers/config/manager.rs:23-42`, `:86-99` | Explicit-path ConfigManager; canonical_toml and replace_toml(content, expected revision). |
| `crates/ccr-config/src/services/config_service.rs:66`, `:247-259` | ConfigService::new(Arc<ConfigManager>) and patch_config delegate to existing repository; reuse instead of matching wrappers. |
| `.../config_service.rs:293-317` | validate_all returns report counts and per-section results; it does not throw merely because invalid sections exist. |
| `.../config_service.rs:337-356` | backup_config and restore_config_from_backup: resource lock spans pre-restore backup and catalog save. |
| `.../config_service.rs:359-443` | Export including/redacting secrets and import Merge/Replace with optional preimage backup. No adapter SQL/TOML persistence is needed. |
| `crates/ccr-cli/src/application/profile_lifecycle.rs:88-112`, `:231`, `:253`, `:429` | ApplyProfileRequest, operation_lock, apply_profile, enable_profile, update_profile: synchronous shared lifecycle owner already consumed by CLI/TUI/Tauri. |
| `.../profile_lifecycle.rs:27-84` | Typed outcomes: Unchanged, Applied, AppliedWithWarning, RecoveryRequired; identifiers/warning codes/recovery paths, no profile payload. |
| `crates/ccr-cli/src/application/profile_off.rs:200-220` | needs_login_prep and profile_off_for_platform for Claude/Codex/Grok; profile off is not auth logout. |
| `ccr-ui/src-tauri/src/commands/profile_lifecycle.rs:9-22` | Tauri ProfileOutcome JSON conversion; keep wire schema/export layer here. |
| `.../profile_lifecycle.rs:105-150`, `:153-211` | Canonical profile export, raw read, validated raw save with activation-conflict and CAS status. Helpers already accept paths. |
| `ccr-ui/src-tauri/src/commands/claude.rs:365-511` | Claude JSON patch/build conversion. Platform_data/extra/auth_mode details are existing adapter semantics, not generic config-patch equivalence. |
| `ccr-ui/src-tauri/src/commands/codex.rs:1567-1661` | Codex build/patch and auth/runtime-aware projection; domain normalization remains CodexPlatform. |
| `ccr-ui/src-tauri/src/commands/grok.rs:607-810` | Grok patch, credential-action and custom rename orchestration; distinct outcomes retained on rename partial failure. |
| `crates/ccr-store/src/services/history_service.rs:19-28`, `:56-62`, `:114-120` | Config operation history owner; unrelated to ccr-db ui_state recent command history. |
| `crates/ccr-db/src/services/converter_service.rs:9-39` | Pure in-memory ConfigConverter::convert; already framework-free; no SQL or new application facade required. |

### Why 02's read service should stay small

`ConfigReadService` performs one useful shared read composition, including current-marker resolution and masked projection. Adding a same-name method for every `ConfigManager`, `ConfigService`, `PlatformConfig`, lifecycle, HistoryService and converter operation would duplicate owners without introducing shared behavior.

Use existing domain owners directly from the desktop worker where an operation already exists. Extract into ccr-app only **actual code removed from Tauri and consumed by both adapters**, for example:

- Framework-free generic config add/update/delete policy combining active-resolution, platform validation, patch/CAS and application lock (`config/adapter.rs:190-284`). Wire DTO parsing/TS attributes stay Tauri-owned; shared inputs should use existing domain types or a small actual operation type, not OpenJsonValueDto.
- Raw-profile edit policy combining safe syntax/semantic validation, reject-empty, active-profile removal warning and catalog replacement (`commands/profile_lifecycle.rs:153-211`). Return a typed result and map to existing Tauri statuses; do not move the JSON serializer or TS exports wholesale.
- Managed backup restore selection/validation and restore composition (`config.rs:96-155`, `:291-307`) if both adapters actually call it. Keep ConfigService backup/save and core write ownership intact.
- Platform-specific patch normalization currently trapped in Tauri (`claude.rs`, `codex.rs`, `grok.rs`) only when native and Tauri both consume it. Move the narrow pure normalization/patch function, not the entire giant commands module. Do not copy auth normalization already owned by Claude/Codex/Grok domain types.

Do **not** rewrap `apply_profile` / `enable_profile` / `update_profile` merely to rename them, reimplement CatalogTransaction, clone profile SQL or add a second profile database. Execution/runtime ownership in 02 supplies blocking scheduling and job lifecycle, not business semantics.

### Critical CAS distinction for AC4.2

Generic `update_config(expected_version)` at `config.rs:368-378` passes a UI token to `adapter::update` (`config/adapter.rs:219-250`), then `ConfigManager::mutate_versioned(expected, ...)` checks it under repository ownership. The native editor must bind draft values, baseline and this revision together; omitted/masked credentials must remain omitted, not overwrite live credentials.

By contrast, `profile_lifecycle::update_profile` (`:429-441`) takes platform/name/target/patch but **no caller expected revision**. It reads a fresh token at operation start; `execute` checks that token against changes during the operation (`:316-340`). This protects execution consistency, not a stale editor that opened before another client saved. Claude/Codex Tauri update handlers (`claude_profiles.rs:99-123`, `codex_profiles.rs:130-157`) expose no draft token. Grok update (`grok.rs:720-765`) also has none.

Therefore the 04 two-client stale-editor requirement is **not already satisfied on all profile edit paths**. The smallest new mechanism is an expected-version extension in the existing lifecycle/domain mutation owner, preserving the current default API behavior for legacy callers. Exact new API name is not prescribed and is **not implemented**. Native editor reads must return a corresponding catalog revision; a profile-only JSON payload is not a version baseline. Do not compare revision outside the lock, then invoke an unversioned lifecycle; that leaves a race. Do not use a plain repository patch to sidestep active rename/apply journaling.

Repository `snapshot.version` now means `document.revision` in profiles.db, not a TOML-byte hash. Some old comments still describe original bytes; use the current catalog contract. Raw saves do carry the catalog token to `replace_toml` and return `conflict` on mismatch (`commands/profile_lifecycle.rs:201-210`). The `force` flag bypasses only active-profile-removal refusal; it does **not** bypass stale-token CAS.

### Lifecycle, compensation and active state

- `apply_profile` validates target/enabled/auth rules, takes operation lock, snapshots, then executes (`profile_lifecycle.rs:231-249`). `enable_profile` enables policy and activates within the same operation (`:253-284`); saving enabled first then invoking apply separately is invalid.
- `execute` declares runtime/registry/auth/secret files, writes interrupted record, opens thread-local WriteJournal, enters CatalogTransaction, verifies runtime files before catalog COMMIT, then commits journal (`:287-380`). Keep the **whole** synchronous lifecycle in one worker: no await, nested spawn or detached work inside the journal.
- Revision conflict, failed write and external compensation conflict have different outcomes. AppliedWithWarning means committed with ancillary work pending/failed; do not report generic save failure and auto-repeat activation. RecoveryRequired preserves external changes and gives recovery paths; do not replace files unconditionally.
- Apply uses caller-owned operation UUID (`ApplyProfileRequest.operation_id`); same-ID replay returns saved outcome and does not increment usage/history or reapply (`:197-222`). An interrupted marker is not permission for automatic activation after restart.
- Active config disable/delete must acquire `operation_lock`, **then** resolve current profile, **then** mutate. Generic adapters do this (`config/adapter.rs:225-247`, `:276-283`). Claude valid file marker then valid registry fallback must remain authoritative. Do not hold an outer application lock while invoking lifecycle functions that acquire it themselves.
- Claude/Codex rename through `update_profile` seeds exact stored section, preserves unknown TOML values/datetimes, updates current/default references and applies renamed profile only if active (`:429-500`). Same-name editing does not implicitly activate.
- Grok Tauri rename currently has custom save-new/apply/delete sequence with `RenameApplyFailed` / `RenameCleanupFailed`, preserving both names as needed (`grok.rs:757-809`). Do not silently replace that with Claude semantics or claim a single transaction already exists. Its explicit credential_action and unsafe entry-state rules must survive any reviewed consolidation.
- `profile_off_for_platform` owns off cleanup/backups, not a mere marker clear. Claude/Codex/Grok each clean only approved managed runtime state. Codex `login_prep_codex_dirs` (`crates/ccr-codex/src/platforms/codex.rs:174-191`) can include default home when only inherited CODEX_HOME redirects; explicit CCR_CODEX_DIR limits the target. Never fixture-test off using only CODEX_HOME and assume real home is isolated.

### Secrets, raw source, import/export and restoration boundaries

**Do not copy current dedicated profile projection blindly.** Generic `ConfigReadService` masks via Secret::Display. But `claude.rs:513-528` and `codex.rs:1592-1625` explicitly expose auth_token for legacy edit-prefill; Codex list also computes env_export/shell_export_script (`:1611-1617`). Those payloads must not become native list/store/Debug/screenshot data. Native list uses masked display rows; sensitive edit/export content stays in the scoped editor/explicit export action and is cleared on teardown. Grok `profile_to_dto` (`grok.rs:425-456`) exposes presence/auth mode and sanitized URL, not inline secret.

- `ConfigPatch` retains omitted fields and handles null removal explicitly; generic ConfigPatchInput has strict allowed fields and no secret Debug (`config/adapter.rs:65-155`). Dedicated Claude/Codex platform_data replacement semantics differ; preserve documented unknown-field behavior and auth-mode cleanup, not a universal JSON merge.
- Generic `ConfigService::export_config(false)` supports redacted output (`config_service.rs:359-384`). Dedicated Claude/Codex `profiles_export_payload_from_path` **rejects** `include_secrets=false` and emits canonical secret TOML (`commands/profile_lifecycle.rs:105-132`). Do not merge these commands into one behavior without reviewed compatibility rationale.
- Raw reads return canonical catalog TOML, revision, profiles.db path and exists (`profile_lifecycle.rs:135-150`); missing returns empty/token empty/exists false without creating a catalog. Raw saves validate syntax/shape/nonempty, current-profile removal and CAS before mutation (`:153-211`). Source content is plaintext and Local-only in handlers (`claude_profiles.rs:144-179`, `codex_profiles.rs:50-85`).
- **Spec drift to reconcile:** the general `raw-config-editor-contracts.md` says verbatim bytes/comments survive and guarded file backup is used. Current **profile-specific catalog** contract (`ccr-config/backend/profile-repository.md`) and actual `canonical_toml`/`replace_toml` explicitly do not preserve comments/formatting and write profiles.db, not a live TOML. Do not promise verbatim profile round trips or recreate a TOML store to satisfy the older general wording. Other runtime raw settings remain a different 05 surface.
- `ConfigService::validate_all` returns invalid_count/results. Current Tauri validate_configs ignores those and returns "All configurations are valid" whenever the read succeeded (`config.rs:244-254`). Native validation must render the real report, not infer validity from Ok. Preserve or deliberately review legacy Tauri text separately.
- Import Merge/Replace uses the domain safe parser and preimage backup under resource lock (`config_service.rs:388-443`). Restore holds resource lock across pre-restore backup+save (`:346-356`). Backup failure must prevent overwrite. A backup is secret content and must use existing private writers.
- Managed restore resolver accepts a **same-directory relative backup filename** with current filename prefix and `.bak` suffix, rejecting absolute/traversal/nested paths (`config.rs:96-140`). Do not expose arbitrary file-path restore just because a native file picker can select it. Existing tests cover this at `config.rs:439-476`.
- Existing restore helper parses `toml::from_str::<CcsConfig>` and interpolates full parser Display (`config.rs:143-153`), a source/credential diagnostic risk. Before sharing that path, use domain safe parsing/diagnostic contracts and sentinel tests; do not preserve the unsafe error construction as part of DTO compatibility.
- `clean_backups` is **existing Tauri-local orchestration**, scanning real `HOME/.ccr`, max depth 3, deleting `.bak`/`.backup` and counting successes (`config.rs:499-530`). It ignores CCR_ROOT and skips traversal/delete errors. Do not call unchanged in a fixture or expose arbitrary recursive deletion. A narrow explicit-root helper can preserve deletion-set semantics and retain default Tauri behavior while native/fixtures bind an approved root. Do not broaden cleanup to catalogs, auth, journals or every subdirectory.

### 42 command IDs -> candidate native actions

Action names below are **mapping candidates**, not implemented enum/API claims. Every entry needs real UI+domain wiring and verification before completion. Source lines were inspected in the current workspace; reconcile concurrent changes at implementation handoff.

| Inventory command ID | Actual handler/source | Candidate native action and critical behavior |
| --- | --- | --- |
| `list_configs` | `commands/config.rs:160` | `Configs::LoadClaude`; existing shared masked list + revision. |
| `switch_config` | `config.rs:177` | `Configs::Apply` / explicit `EnableAndApply`; use lifecycle outcomes. |
| `add_config` | `config.rs:190`, `config/adapter.rs:190` | `Configs::Create`; strict patch/platform validation. |
| `delete_config` | `config.rs:204`, adapter `:276` | `Configs::DeleteConfirmed`; active guard and existing confirmation semantics. |
| `rename_config` | `config.rs:218`, adapter `:253` | `Configs::Rename`; shared update_profile lifecycle, add draft-token mechanism for native edits. |
| `duplicate_config` | `config.rs:231`, adapter `:259` | `Configs::Duplicate`; clone domain section under mutation; reject target collision. |
| `update_config` | `config.rs:368` | `Configs::SavePatch`; original draft expected_version, no silent conflict overwrite. |
| `validate_configs` | `config.rs:244` | `Configs::Validate`; display actual ValidationReport counts/results. |
| `import_config` | `config.rs:258` | `Configs::ImportConfirmed`; existing Merge/Replace + backup. |
| `restore_config` | `config.rs:291` | `Configs::RestoreManagedBackupConfirmed`; managed leaf validation, safe parse/preimage backup. |
| `export_config` | `config.rs:311` | `Configs::Export`; preserve redacted/include-secret choice and scoped secret output. |
| `get_history` | `config.rs:332` | `Configs::LoadOperationHistory`; ccr-store HistoryService, not ui_state recents. |
| `clear_history` | `config.rs:357` | `Configs::ClearOperationHistoryConfirmed`; existing history owner/policy. |
| `claude_list_profiles` | `claude_profiles.rs:11` | `Profiles<Claude>::Load`; masked native projection, actual current/can_off. |
| `claude_get_profile` | `claude_profiles.rs:40` | `Profiles<Claude>::OpenEditor`; scoped secret + catalog revision. |
| `claude_add_profile` | `claude_profiles.rs:61` | `Profiles<Claude>::Create`; existing build/auth validation and domain save. |
| `claude_update_profile` | `claude_profiles.rs:99` | `Profiles<Claude>::SaveOrRename`; shared lifecycle, required native draft CAS extension. |
| `claude_delete_profile` | `claude_profiles.rs:130` | `Profiles<Claude>::DeleteConfirmed`; platform/current guards. |
| `claude_apply_profile` | `claude_profiles.rs:277` | `Profiles<Claude>::Apply`; typed ProfileOutcome, no early current UI change. |
| `claude_profile_off` | `claude_profiles.rs:253` | `Profiles<Claude>::OffConfirmed`; Local login-prep cleanup, not logout. |
| `claude_export_profiles` | `claude_profiles.rs:191` | `Profiles<Claude>::ExportCanonical`; dedicated secret-only behavior retained. |
| `claude_get_profiles_raw` | `claude_profiles.rs:144` | `Profiles<Claude>::OpenSourceAfterWarning`; Local/scoped canonical content+token. |
| `claude_save_profiles_raw` | `claude_profiles.rs:159` | `Profiles<Claude>::SaveSource`; typed invalid/conflict/activation_conflict; force never bypasses CAS. |
| `codex_list_profiles` | `codex_profiles.rs:11` | `Profiles<Codex>::Load`; current/auth source/can_off without exposing list secrets. |
| `codex_add_profile` | `codex_profiles.rs:100` | `Profiles<Codex>::Create`; domain normalization, fields not guessed from Claude. |
| `codex_update_profile` | `codex_profiles.rs:147` | `Profiles<Codex>::SaveOrRename`; lifecycle, native draft CAS extension, refresh dependent views. |
| `codex_delete_profile` | `codex_profiles.rs:163` | `Profiles<Codex>::DeleteConfirmed`; domain deletion and refresh. |
| `codex_get_profile_env` | `codex_profiles.rs:582` | `Profiles<Codex>::ExportEnvironment`; explicit sensitive env/script output, no execution. |
| `codex_apply_profile` | `codex_profiles.rs:206` | `Profiles<Codex>::Apply`; shared lifecycle/route/auth semantics. |
| `codex_profile_off` | `codex_profiles.rs:183` | `Profiles<Codex>::OffConfirmed`; explicit target roots, cleanup official-login suppressors. |
| `codex_export_profiles` | `codex_profiles.rs:233` | `Profiles<Codex>::ExportCanonical`; preserve dedicated secret-only mode. |
| `codex_get_profiles_raw` | `codex_profiles.rs:50` | `Profiles<Codex>::OpenSourceAfterWarning`; Local catalog read. |
| `codex_save_profiles_raw` | `codex_profiles.rs:65` | `Profiles<Codex>::SaveSource`; same original token/force protocol. |
| `grok_list_profiles` | `grok.rs:1266` | `Profiles<Grok>::Load`; typed Local/unsupported envelope and actual activation diagnosis. |
| `grok_get_profile` | `grok.rs:1284` | `Profiles<Grok>::OpenEditor`; safe presence/auth/URL DTO + native revision baseline. |
| `grok_add_profile` | `grok.rs:1304` | `Profiles<Grok>::Create`; official/third-party and credential_action rules. |
| `grok_update_profile` | `grok.rs:1317` | `Profiles<Grok>::SaveOrRename`; preserve partial-failure statuses, add native draft CAS inside owner. |
| `grok_delete_profile` | `grok.rs:1331` | `Profiles<Grok>::DeleteConfirmed`; blocked/drifted/unsafe-state/force semantics retained. |
| `grok_apply_profile` | `grok.rs:1345` | `Profiles<Grok>::Apply`; shared apply lifecycle and Grok runtime owner. |
| `grok_profile_off` | `grok.rs:1370` | `Profiles<Grok>::OffConfirmed`; shared off lifecycle, Local only. |
| `convert_config` | `converter.rs:78` | `Converter::ConvertPreview`; ConfigConverter::convert in worker, output stats/warnings, no filesystem apply. |
| `clean_backups` | `config.rs:499` | `Configs::CleanManagedBackupsConfirmed`; explicit safe root and existing bounded deletion set. |

All `commands/` paths in this table are under `ccr-ui/src-tauri/src/`. Native actions need actual confirmation/risk checks based on the existing entry policy; removing IPC is not permission to remove those checks. A candidate row is not a completed inventory row.

Converter support caveat: `ConverterCliType` advertises ClaudeCode/Codex/Gemini/Qwen (`commands/converter.rs:9-24`), but actual ConfigConverter parse/generate supports only ClaudeCode/Codex (`converter_service.rs:44-72`); other formats return unsupported. convert_mcp/commands/agents are request fields but the inspected conversion implementation does not read them outside tests (`:279-310`). Preserve/report existing supported behavior; do not silently implement new formats or claim unchecked toggles filter output. Converter input/output can contain credentials; keep scoped and nonlogged.

### 5 route IDs -> candidate native surfaces

| Route ID / current path | Candidate surface / mapping caveat |
| --- | --- |
| `claude-code-profiles` / `/claude-code/profiles` | Actual Claude profile list/detail/editor/source/apply/off surface. |
| `grok-profiles` / `/grok/profiles` | Actual Grok list/editor/activation diagnosis and guarded actions. |
| `converter` / `/converter` | Explicit text conversion preview/result surface, supported formats only. |
| `configs` / `/configs` | Generic explicit-Claude configuration catalog/editor/import/export/history/backups. |
| `codex-profiles` / `/codex/profiles` | Actual Codex profile list/editor/source/env export/apply/off surface. |

`configs` and `claude-code-profiles` overlap stored data but have different existing operation/export surfaces. A single native navigation entry may host both through discoverable actions **only with a recorded reviewed merge rationale** covering every command. Do not remove one inventory route simply because both lists render similar records. No native route/action implementation is asserted here.

### Explicit fixture seams and safe baseline reuse

| Source:line | Reusable today / limit |
| --- | --- |
| `ccr-config/.../config/manager.rs:23`, `config_service.rs:66` | ConfigManager::new(path), ConfigService::new(Arc<manager>) inject catalog storage. Catalog writes still resolve lock directory through core default unless an injected owner is added. |
| `ccr-config/src/models/platform.rs:423` | Public PlatformPaths fields permit explicit domain paths; production new() remains domain default resolver. |
| `ccr-config/src/models/claude_runtime_paths.rs:24` | resolve_with(explicit_home, env_getter) is a pure resolver; no process env mutation needed. |
| `ccr-cli/src/managers/settings.rs:49-61` | SettingsManager::new(settings, backups, LockManager) already accepts all three. |
| `ccr-core/src/core/lock.rs:207-233` | LockManager::new(explicit_dir) vs with_default_path. Reuse existing lock identity/policy. |
| `ccr-cli/src/platforms/claude.rs:59-72` | ClaudePlatform currently constructs paths/settings/default lock internally; no public from_parts found. A narrow injected constructor can reuse SettingsManager; not yet implemented. |
| `ccr-cli/src/platforms/grok.rs:144-159` | GrokPlatform::from_parts(PlatformPaths, config_path) already exists; runtime owner retains CAS/entry-state rules. |
| `ccr-codex/src/platforms/codex.rs:238-266` | for_codex_dir changes runtime root but still resolves PlatformPaths ambiently; full from_parts(paths, config_manager, runtime_service) is available. |
| `ccr-codex/src/services/codex_runtime_service.rs:86-97` | from_parts(paths, codex_dir, config_manager) exists. |
| `ccr-codex/src/managers/codex_config.rs:73-84` | with_codex_dir injects runtime files but still calls LockManager::with_default_path. Do not claim complete isolation from one path argument. |
| `ccr-cli/src/services/claude_auth_service.rs:216-235` | from_parts(ccr_claude_dir, claude_dir, state_file) exists; it chooses local lock subdirectory internally. Keep lock identity aligned with lifecycle if exposing this seam. |
| `ccr-store/src/storage/database.rs:56`, `history.rs:175`, `services/history_service.rs:19` | Database::init(explicit_path) -> HistoryManager::new(database) -> HistoryService::new(Arc<manager>) isolates config operation history. No schema duplication. |
| `ccr-config/src/managers/config/catalog/migrate.rs:29-40` | migrate_catalogs resolves every platform from ambient paths; explicit migrate_paths is **pub(crate)** and has a private failpoint type. Do not promise public injected migration API exists. A narrow public owner entry can be added when actually needed. |

Lifecycle `record_path`, `operation_paths`, `execute`, replay and ancillary history currently re-resolve ambient paths (`profile_lifecycle.rs:128-163`, `:214`, `:296`, `:314`, `:385`). Constructing only a fixture platform instance is not sufficient while these helpers still select real defaults. The minimal fixture-capable extension belongs in that existing owner and should receive the concrete platform/path/history/lock dependencies it actually needs; keep default entrypoints delegating compatibly. Do not invent a universal dynamic dispatch container or relocate lifecycle into ccr-app.

**Concurrent 02 improvement observed:** `ccr-cli/src/application/profile_contract.rs:63-128` now includes a ProcessFixture selected by CCR_PROFILE_CONTRACT_ROOT, validates a fixture marker and explicit child domain roots, and does not mutate HOME/USERPROFILE in that path. Tauri `commands/config/contract_tests.rs:10-40` launches each real contract in a child with CCR_ROOT/DATA/LOCK, Claude paths, CCR_CODEX_DIR and GROK_HOME. Parent tests remain parallel. The legacy fallback at profile_contract.rs:120 still uses new_with_home_env; do not run that path and claim compliance. This is changing code owned by 02, not independently tested by this researcher. Reuse finalized harness instead of copying it into 04 tests.

04 can consume these domain path seams for its own fixture and later **profile/config portion** of a safe Tauri baseline. They do not isolate the whole Tauri process: app DB, usage archive migration sources, preferences, logger, OAuth recovery, local/remote environment discovery, WebView storage and process runners have separate owners. 01 `research/baseline.md:54-70` already records those boundaries. Do not expand 04 into a baseline rewrite or launch Tauri against real accounts to obtain timings. Retain the no-global-HOME rule; absence of real usage data is not proof that startup cannot touch other user files.

### Safety contracts/specs and verification anchors

- `.trellis/spec/ccr-config/backend/{index,backend-guidelines,profile-repository,test-fixtures}.md`: profiles.db single owner; explicit mutation versus pure queries; catalog revision, strict patch, unknown values, no source leaks; constructors do not initialize/migrate.
- `.trellis/spec/ccr-cli/backend/{index,profile-application-lifecycle,profile-off-login-prep,grok-profile-runtime,test-fixtures}.md`: one-worker journal; lock order; replay; active guards; off versus auth logout; Grok credential_action/unsafe entry-state semantics. Existing fixture/global-env and serial examples do not override this user's stricter explicit fixture/default-parallel rules.
- `.trellis/spec/ccr-core/backend/{atomic-writer,ccr-error-freeze}.md`: guarded write/backup rotation/atomic replacement/private permissions/resource identity; no new CcrError domain variants. Repository-specific operation -> resource -> catalog/leaf contracts take precedence over historical legacy RMW examples in the generic atomic-writer document.
- `.trellis/spec/ccr-codex/backend/codex-provider-bearer-runtime.md`: bearer belongs only in approved runtime/secret owner, not diagnostic DTO/Debug/template/auth.json; auth-mode cleanup and warnings preserved.
- `.trellis/spec/ccr-ui/frontend/{raw-config-editor-contracts,profiles-page-contracts,api-facade-boundary}.md`: draft and source session binding; plaintext warning; original token; explicit force; form auth-derived fields; Tauri exports remain stable. Apply profile-catalog-specific canonical formatting rule rather than general raw-verbatim wording.
- `.trellis/spec/ccr-store/backend/{index,profile-activity}.md`: config history versus recorded profile activity; don't initialize a writable HistoryManager merely to show last-use. Read-only activity projection already has its own owner.

For actual 04 verification: two clients share one fixture catalog and draft revision; concurrent save proves one success/one conflict; test original draft retention/reload cancellation, nullable/missing/unknown fields, private-file metadata, safe sentinel diagnostics, backup failure and every journal stage/external conflict. Lifecycle tests must assert actual runtime/current/enabled state and structured warning/recovery outcomes, not just an Ok handler result. Existing Tauri profile_lifecycle application_contract functions (`commands/profile_lifecycle.rs:26-96`) and config contract tests offer real-adapter anchors, subject to the finalized isolated process harness. Raw-specific tests at `commands/profile_lifecycle.rs:291-446` cover missing/canonical read, invalid/empty/activation-conflict/stale-token/private modes. Current line positions may move under owners.

Approved gates remain affected ccr-config/ccr-cli/ccr-codex/store tests, `just desktop-test`, `just desktop-test-ui`, `just secret-write-check`, `just tauri-bindings-check` when shared/wire types change, version/fmt/lint/test and final root CI at parent integration. Use --locked and default Rust test parallelism with --skip export_bindings. No command in this report has been executed; no unrun gate is passed.

## External References

None needed. This is an internal responsibility/contract map over the current source, not new dependency/API version research.

## Caveats / Not Found

- Only this file was written. No product/Cargo/manifests/status/inventory changes, git commands, Cargo/check/native execution, user-config access or app launch occurred. 03/04 were not activated.
- Candidate native actions in the mapping are proposals for traceability, not existing symbols or completed implementation. All 42 command/5 route rows need actual owner verification before inventory completion.
- Current internal CAS does not universally provide stale-editor CAS. New versioned lifecycle/injection APIs described above do not exist yet and are not promised as ready.
- Tauri raw export/list secret exposure, unsafe restore parser formatting, generic validation success text, cleanup root selection and converter flag support are **observed compatibility/security gaps**, not authority for broad unrelated rewrites. Repair only the touched shared boundary with concrete tests; escalate a genuinely scope/compatibility/security change to the coordinator.
- 02 fixture work is concurrent; observations are source-only and do not replace 02 independent check. Re-read those few final interfaces at 04 dispatch rather than trusting this snapshot after further edits.
- No implement.jsonl/check.jsonl was loaded under the research-role isolation rule. The RTK.md path was previously absent; no new assumption about its contents was made.
