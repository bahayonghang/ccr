# Research: GPUI 06 resources source map

- Query: Trace all retained 06 commands and routes to actual owners, extraction boundaries, fixture seams, persistence and process/path safety before assigning implementation.
- Scope: internal source research only; active task remains `10-17-gpui-04-profiles`; 06 has not been activated.
- Date: 2026-10-18
- Read first: `resources-preflight.md`, approved 06 PRD/design/implement, parent architecture/feature matrix/live migration inventory, `code_map.md`, workflow and affected spec indexes/guides. Research-role isolation forbids loading implement/check JSONL; this report does not claim those manifests were loaded.
- Evidence limit: no builds, tests, network, account access, install/uninstall or native operations were executed. File:line anchors describe the current shared workspace and can drift during implementation.

## Findings

### Decisions that matter before coding

1. The baseline is exactly **80 commands / 38 route records = 24 surfaces + 14 redirects**. Route count is not capability count. Antigravity Agents/Plugins return frontend unsupported stubs; Codex Slash Commands is an explainer; `/skills` is currently the external skillport migration surface. Do not manufacture CRUD to satisfy a route count or mark a placeholder complete.
2. Direct domain reuse: builtin prompts, prompt presets, MCP presets/sync and Skills have real `ccr-skills` owners. Several owners still lack explicit fixture constructors and durable CAS/process safety; direct delegation alone does not satisfy AC6.2/6.3.
3. Actual framework-free Tauri extraction candidates: scoped Claude MCP policy; Claude resource transformations over the 05 shared Settings owner; Codex agent file/source catalog operations; Antigravity resource files; OpenCode Markdown resources; closed system-prompt file policy over 05 raw-file persistence. Keep IPC derives, AppState admission/cache invalidation and wire shapes in Tauri. Do not create a generic string dispatch facade.
4. Existing source debt blocks safe native exposure: unsanitized resource file names, shell-joined npx arguments, ambient roots, unchecked stored target/checkouts, unversioned writes and no cooperative process cancellation. Fix at the existing responsibility owner with targeted regressions before connecting native mutation. These are observed preexisting gaps, not claims that the approved product must gain new resource families.
5. Reuse 05's extracted typed Settings/raw persistence and environment identity; do not fork a second settings parser/writer in 06. Preserve original native draft tokens and unknown fields. Bounded workers do not turn an uninterruptible `.output()` call into a cancellable job.

### Files found and responsibilities

| Area | Current source anchors and responsibility |
| --- | --- |
| Scoped Claude MCP | `ccr-ui/src-tauri/src/commands/claude_mcp_config.rs:42` context; `:257` scopes; `:339` approvals; `:540` effective list; `:600` CAS retry; `:797` secret-aware patch |
| Claude wrappers | `commands/claude_mcp.rs:1`, `claude_agents.rs:1`, `claude_slash.rs:1`, `claude_plugins.rs:1`, `claude_hooks.rs:1`; `claude_settings.rs:42` output styles / `:114` statusline / `:269` presets; all paths below are relative to `ccr-ui/src-tauri/src/` unless qualified |
| Claude Settings owner | `commands/claude.rs:154` captured environment update; `:178` Local SettingsManager atomic update; `:195` ambient output-style directory. `crates/ccr-types/src/claude_settings.rs:399/419/440` SlashCommand/Agent/Plugin preserve nested extensions via flattened `other` |
| Codex MCP | `commands/codex_mcp.rs:5/50/84/129`; parent `codex.rs:816` read / `:828` writer / `:1424` validation / `:1432` unknown-field merge / `:1439` field parser |
| Codex local agents | `commands/codex_agents.rs:84` scope, `:133` filename join, `:261` malformed-inclusive sorted inventory, `:387` structured delta, `:473` raw+structured parsing, `:499` direct writer |
| Codex remote agent sources | `commands/codex_agent_sources.rs:306` CCR metadata roots; `:339` atomic helper; `:454` GitHub parsing; `:579` retry; `:818` tree/catalog scan; `:1022` installation change states; `:1274` install / `:1384` sync / `:1469` accept / `:1494` untrack |
| Antigravity | `commands/gemini.rs:21` ambient root (creates directory even during resolution); `:93` MCP aliases; `:149` recursive command precedence; `:224` ambiguous name lookup; `:482` read-only extension manifests |
| OpenCode | `commands/opencode.rs:126` global/legacy roots, `:187` scope; `:380` flat name validation; `:394` Markdown writer; `:401` frontmatter delta; `:594` global/project inventory; `:634/668` agent/command upsert; `:721` plugin file scan |
| Unified MCP | `commands/unified_mcp.rs:127/169` separate platform readers; `:252` aggregated list; `:334` request conversion; `:435/493/516` mutations; `:573` AtomicWriter helper |
| Presets and Skills | `crates/ccr-skills/src/managers/{mcp_preset_manager,prompts_manager,builtin_prompts}.rs`; `crates/ccr-skills/src/services/skills_service.rs`; do not copy their install format or inventories into app/native |
| System instructions | `commands/system_prompts.rs:35` closed PROMPT_FILES list, `:94` path mapping, `:111` backups, `:151` Claude rules metadata, `:204` versioned save; builtin DTO only in `commands/builtin_prompts.rs` |

### Command group → owner → native action candidates

The exact 80 IDs and current handler anchors are in the appendix. All are **candidates**, not implemented or verified native actions.

| Group / count | Actual behavior / owner | Native candidate and compatibility constraints |
| --- | --- | --- |
| Claude MCP 4 + unified 4 | Claude wrappers delegate `claude_mcp_config`; unified covers Claude/Codex/Gemini, not OpenCode. Unified update supports **Claude only**, add/delete support the other two (`unified_mcp.rs:493`). | One discoverable MCP manager with platform/scope filters and dedicated platform detail controls; preserve each adapter protocol. Do not silently widen unified update or lose Codex fields. Exact failure parity before a reviewed merge. |
| Claude agents 4 | Typed `ClaudeSettings.agents` list/add/replacement-by-name/delete through captured-environment Settings update. | List/detail/create/edit/delete and existing toggle/folder projection; typed patch must preserve untouched `Agent.other`, rather than replacement from a minimal form. No remote agent filesystem install exists here. |
| Claude slash commands 4 | `ClaudeSettings.slash_commands` array; typed CRUD and not-found errors. | Commands list/detail/editor; retain folder data, but verify the attempted toggle wrapper rather than assuming it works (see below). Stored command text is configuration, not permission to execute it. |
| Claude plugins 4 | `ClaudeSettings.plugins` metadata array, not package manager execution. | Metadata create/edit/remove/toggle. Do not relabel metadata removal as uninstalling a physical plugin or execute JavaScript. |
| Claude styles/statusline/hooks/prompts 8 | Two commands each. Styles scan/write `~/.claude/output-styles/*.md`; statusline opaque value in Settings.other; grouped hooks replace typed HooksConfig; prompt preset catalog is PromptsManager. | Separate editor types under one resources workspace. Hooks/statusline save configuration only; preserve unknown grouped hook keys. Presets and closed system instructions are distinct stores. Output-style frontend delete is presently broken, not a real delete endpoint. |
| Codex MCP 4 | Full typed CodexMcpServer parser, unknown `other`; the lightweight structs are Tauri-local (`codex.rs:40/107`), and their persistence/orchestration is there too; ccr-codex has a separate config manager owner API. | Dedicated stdio/http editor retains auth injection/env header references/timeouts/tool scope/required/disabled semantics; masked summary separate from scoped secret draft. |
| Codex local agents 7 | Global/project TOML list/add/update/delete/rename/copy/validate; structured edits preserve other TOML fields; raw input is parsed and reserialized. | Full agent editor + project context, copy/rename and TOML validation. Do not claim raw comments/formatting preserved. Resource identity is safe scoped file identity, not untrusted `path` from a list. |
| Codex sources 9 | Source list/add/remove/sync/catalog; install/sync installed/accept local/untrack. Actual GitHub REST/raw downloads, no git checkout or archive extraction here. | Sources/catalog/installed views; retain `partial`, truncated/error diagnostics, tracked-only replace, local/upstream conflict decision. Accept adjusts baseline; untrack leaves installed file. Removing source also removes its tracking records (see below). |
| Antigravity 9 | MCP 4 JSON, recursive slash 4 TOML, extensions 1 read-only manifest scan. | MCP + folder-aware commands + extension inventory. Do not invent Agents/Plugin CRUD from unsupported frontend functions. |
| OpenCode 9 | Agent Markdown 4, command Markdown 4, local plugin file inventory 1. | Global/project frontmatter/body editors and plugin file metadata, preserving extensions. MCP/plugin package-reference mutations are route-only behavior over 05 Settings and must be mapped additionally. |
| MCP presets/sync 7 | McpPresetManager list/get/install, McpSyncManager source list/sync one/all; Tauri maps per-target failures. | Catalog/detail/credential review/install selected targets; source sync one/all with explicit partial outcomes and reread each successful target. Defaults requested Codex+Gemini; no all-platform transaction claim. |
| Builtin prompts 3 | Pure `ccr_skills::{get_builtin_prompts,get_prompt_by_id,get_prompts_by_category}`. | Search/category/details/variable preview in prompt picker. Unknown category maps General; unknown ID returns None; no arbitrary on-disk path. |
| System prompts 4 | Closed platform+ID list/get/save/create, Local guard, versioned raw writer. | Four platform instruction editors with explicit plaintext warning, original-token conflict/reload, metadata/rules sidebar. Preserve exact IDs, aliases, backup selection and create-if-missing behavior. |

### Route-only behavior and honest merges (all 24 surfaces)

| Retained surface(s) | Current behavior and native disposition candidate |
| --- | --- |
| `/claude-code/system-prompts`, `/codex/system-prompts`, `/antigravity/system-prompts`, `/opencode/system-prompts` (4) | Shared closed instruction-file editor, platform-specific IDs/limits; Claude rules list is metadata-only and not arbitrary-file editor. Reuse same native pane with platform entry points. |
| `/mcp-manager`, `/codex/mcp`, `/antigravity/mcp`, `/opencode/mcp` (4) | Unified hub + dedicated field coverage. OpenCode MCP is **frontend orchestration** (`api/domains/opencode.ts:241–278`) reading `settings.mcp` and writing replacement map through 05's Settings API, hence no four OpenCode MCP IDs in the 80. Extract actual policy, retain sibling fields and CAS. |
| `/slash-commands`, `/antigravity/slash-commands`, `/opencode/commands` (3) | Claude settings array vs recursive Gemini TOML vs OpenCode frontmatter. Shared list/editor chrome is appropriate; serializers and identities remain separate. |
| `/codex/slash-commands` (1) | `features/codex/CodexSlashCommandsView.tsx:33` explicitly states no manageable CRUD; links Sessions/Agents/MCP. Reviewed merge into Codex resource help/shortcuts is honest. `api/domains/codex.ts:986` list returns empty; mutation wrappers throw unsupported. Do not implement a new Codex command runtime. |
| `/agents`, `/agents/:name`, `/codex/agents`, `/opencode/agents` (4) | Claude catalog/detail + Codex advanced local/source editor + OpenCode Markdown. Claude detail edits model/tools/system prompt, copies system prompt, displays folder, confirms delete and returns to list (`features/platform/agents/AgentDetailView.tsx:25`). `getAgent` selects the first name match (`api/domains/claude.ts:325`), not a folder-disambiguated identity. Toggle is an attempted wrapper action with a contract gap below. |
| `/antigravity/agents` (1) | `api/domains/gemini.ts:170–198` is empty/unsupported; descriptors/base features do not prove real support. Native explicit unsupported disposition with reviewed merge into platform capabilities, not fake empty successful catalog. |
| `/plugins`, `/opencode/plugins` (2) | Claude metadata CRUD/toggle. OpenCode combines package-reference string list in settings.plugin **and** local JS/TS file metadata (`configs/plugins.ts:143`, `api/domains/opencode.ts:341–365`). Removing a reference does not delete the local file; local JS is never loaded into the CCR process. |
| `/antigravity/plugins` (1) | Current route's `listGeminiPlugins` is unsupported stub (`api/domains/gemini.ts:201–229`); actual `gemini_list_extensions` remains real read-only command. Reviewed extension inventory + explicit unsupported mutation is a possible merge, not permission to add install/uninstall. |
| `/hooks`, `/output-styles`, `/statusline` (3) | Real separate editors. Claude hooks support official grouped command/prompt/agent types; preserve opaque payloads and pending writes. Styles create/update backend expects `Vec<OutputStyle>` but wrapper update(name,patch) sends object, delete sends `{}` (`api/domains/claude.ts:475–493`); source debt must be resolved explicitly, not counted as working delete. |
| `/skills` (1) | Existing `SkillsMigrationView` describes removal of embedded Skills management and external skillport detection/open; its shell command IDs belong09. Approved06 independently includes existing ccr-skills capabilities: supply true native domain inventory/install lifecycle after safety seams, while retaining the migration/help link dependency09. Record separately from 80-command coverage, no resurrected Tauri IPC requirement. |

Redirects: `/mcp` and `/mcp/unified` → MCP manager (2); `/skills-manager`, `/skillport-manager`, `/skills/add`, `/skills/hub`, `/skills/:platform/:name`, `/market`, `/opencode/skills` → Skills (7); `/gemini-cli/{slash-commands,mcp,agents,plugins,system-prompts}` → corresponding Antigravity surface (5). Preserve discoverable navigation/alias rationale in inventory; no duplicate implementations required.

### MCP scope, secrets, concurrency and preset mismatch

- Claude precedence is Local project entry in user state → project `.mcp.json` → user root (`claude_mcp_config.rs:257–320`). Local and user share the user-state file but target different objects; Windows project-key normalization is tested. Project approvals read `.claude/settings.json` then `settings.local.json`; disabled beats approved/enable-all; project pending/disabled must not appear effective. Source/scope/hidden_by/effective/diagnostics are meaningful UI data, not decoration.
- Existing Claude context can be structurally constructed **inside its Tauri module only** with `user_state_path` and `project_root`; move actual framework-free functions + typed context together and expose injected construction. Its current scope writer uses three fresh CAS attempts and `BackupPolicy::None`, secret for Local/User, nonsecret Project (`:592–628`). Retain legacy policy and label original editor conflict behavior separately from field mutation replay; do not infer existing backups.
- Env/header values are masked and mask placeholders retained on patch; null removes explicit keys; empty arrays/maps/strings have deliberate no-op rules (`:755–819`). `sanitized_config` masks only env/headers, so a raw-config field is not automatically safe if arbitrary extensions contain secrets. URL/args/command can also contain credentials. Native safe projection must not derive Debug/retain unreviewed raw maps; use owner-owned secret handling, distinct scoped plaintext editor.
- Codex `merge_codex_mcp_server` merges **unknown other only**, not omitted known fields (`codex.rs:1432`). An enabled-only parsed replacement can erase known args/env/etc unless full-snapshot policy is preserved or owner patched deliberately; do not copy Claude merge semantics blindly.
- Existing Claude agent/slash/plugin updates deserialize a **complete typed record**, then replace the matching array item. Their frontend toggles send only `{enabled}` (`api/domains/claude.ts:363/407/444`), while Agent needs name/model, SlashCommand name/description/command, Plugin id/name/version (`ccr-types/src/claude_settings.rs:399–455`). Agent/command actually use `disabled`, not `enabled`. These attempted toggle actions are not evidence of working behavior. Antigravity slash toggle likewise sends `{enabled}` (`api/domains/gemini.ts:153`) to a backend requiring command (`gemini.rs:449`). Native should use explicit typed original-snapshot mutation over05's CAS owner and preserve unedited extensions; any Tauri wrapper repair is scoped compatibility work with real regression, not an unchecked promise of parity.
- Unified Codex/Gemini list exposes unmasked env and loses many dedicated fields, errors are partly only traced (`unified_mcp.rs:127/169/305`). Unified add on existing key replaces object; dedicated Codex add rejects duplicates; unified update deliberately unsupported outside Claude. Review semantic merges, don't collapse these distinctions unnoticed.
- Preset/sync domain currently uses `~/.claude/claude.json` (`mcp_preset_manager.rs:179/490`), while scoped Claude MCP uses `ClaudeRuntimePaths.state_file` (normally `~/.claude.json`). This observed path mismatch is preexisting compatibility behavior. Do not quietly redirect old Tauri preset source during extraction. Native can make source explicit and report the chosen path; a global compatibility correction needs an explicit decision/evidence.
- Preset install writes `.json.tmp`/`.toml.tmp` then rename (`:390/419`), without guarded CAS, backups or secret option. Real new native credential writes need the existing core guarded policy at that domain owner; do not copy these helper implementations into ccr-app. `McpSyncManager::sync_preset` creates a fresh ambient target manager even when its source were injected: injecting only source is insufficient.

### Exact injection seams and missing boundaries

| Owner | Safe seam now / required narrow extension |
| --- | --- |
| Claude settings resources | SettingsManager already owns atomic updates. Reuse 05's captured environment/injected manager; preserve remote support for those typed operations, and reject unsupported remote filesystem resources before any Local path resolution. Output styles currently uses HOME/USERPROFILE directly (`claude.rs:195`); inject explicit styles root. |
| Claude MCP | Context has user-state path/project root, but visibility is pub(crate). Inject context plus complete scoped lock root on a synchronous worker. Reads should resolve paths without creating them. Approval files derive only from approved project root. |
| Codex config/MCP | `codex_config_path`/agents dir are ambient (`codex.rs:325/330`); read/write helpers already take a file path but are private and writer only AtomicWriter. Reuse05 owner on that same config file, not second parse/persist path. |
| Codex local agents | Existing project request points an existing project root at `.codex/agents`; global ambient. Extract typed context for global agents root + approved optional project root + locks/backups. No public framework-free resource service currently found in ccr-codex. |
| Codex sources | Inject **all** `.ccr/platforms/codex/agent-sources/{sources.json,installs.json,catalogs/}` and agents root, HTTP transport/client with bounded limits, clock and cancellation. Passing AppState only for HTTP/cache invalidation is unnecessary coupling. List/detail routines also perform synchronous filesystem work inside async handlers. |
| Antigravity | Inject settings, mcp_config, extensions, user `.gemini/commands` and explicit project command roots separately. They are not all under `antigravity-cli`. Current resolver mkdir-on-read and current_dir fallback must not operate on real cwd/home in fixtures. |
| OpenCode | Inject config/legacy and project roots. Existing root-from-home helpers private; `list_scope_candidates` discovers cwd. Keep global/project same-name items distinct. 05 owns shared config/tui/json path operations; 06 owns Markdown agents/commands + local plugin metadata. |
| PromptsManager | `new(platform)` resolves home-derived paths without directory creation (`prompts_manager.rs:29`); save creates the catalog parent. No public injected constructor found. Inject platform instruction directory and `.ccr/platforms/<platform>/prompts.toml` explicitly. Current array replacement removes existing presets before validating every incoming item (`claude_settings.rs:302`); partial failure possible. Validate batch first and let the domain own one durable versioned mutation; preserve documented targeted upsert vs array replacement. |
| McpPresetManager / McpSyncManager | Constructors ambient; Sync target methods create further ambient managers (`:479`). Inject source path, platform target map, CCR/backup/lock roots. Pure builtin catalog `get_builtin_presets()` needs no home construction. |
| SkillsService | Current private fields home_dir, ccr_skills_dir, sources_path, platforms_path, summary_cache_path; `new` uses both home and OS cache (`skills_service.rs:166`), unit-only `build_test_service(root)` demonstrates full safe seam at`:2814`. Add narrow public explicit-path construction at owner, and validate configurable platform target paths; don't expose a UI arbitrary write root. |
| System prompts | Closed IDs map from explicit home, but backups separately resolve ambient PlatformPaths. Inject closed path map + backup roots; `save_prompt_file(spec,path,backup_dir,content,token)` is already a useful private seam. Reuse05 raw-file service and preserve Local guard. `list_for_home` currently test-only list seam, not complete get/save injection. |

Additional reuse constraints: `ccr-codex/src/managers/codex_config.rs:45` already has `CodexConfigManager::new(config_path, auth_path, backup_dir, LockManager)`; prefer it/05's settled service where its behavior matches rather than inventing home resolution. Do not confuse its general JSON/TOML manager with Tauri's lightweight structs or assume it already implements agent/source transactions. System prompt save currently passes `secret:false` into the versioned raw writer (`system_prompts.rs:212`), unlike credential-bearing Settings source saves; preserve/explicitly review this owner policy rather than stating all prompt backups are private already. Codex `validate_agent_toml` reads and validates an **existing stored named file**, not an arbitrary draft-string validation API (`codex_agents.rs:771`). Native draft validation can reuse the parser without adding a new IPC ID; keep those native and existing-command actions distinct.

### Filesystem and process safety: preserve real guards; repair actual gaps

**Reusable core primitives, not a second implementation:** `ccr-core/core/guarded_write.rs:140` versioned guarded write, `:348` versioned delete, `:386` backups; `AtomicWriter` owns durability/permissions and `WriteJournal` handles coordinated compensation where appropriate. `normalized_resource_path` at`:426` is lexical lock identity and explicitly **does not resolve symlinks**; it is not an authorization boundary. Existing scoped lock directory must encompass one synchronous worker operation, never leak across await/spawn. `ccr-core/core/process_gateway.rs:68` ManagedProcess owns process tree and bounded lines; see exact spec contracts below.

- Codex `agent_file_path` merely joins `format!("{name}.toml")` (`codex_agents.rs:133`). Add/update/rename/copy/delete and remote-install target_name consume it without a shared basename validator. Inventory is **flat**; a flat resource identifier rule belongs at the extracted owner, separate from a general profile-name validator. `validate_profile_name` only rejects blank/reserved profile labels (`ccr-config/.../repository.rs:204`) and is not path validation. `skills_ext/toggle.rs:265` permits ASCII alnum/_/- only; do not copy it indiscriminately and forbid legitimate Unicode/spaces/dots without evidence. At minimum reject rooted/drive/UNC, separator, dot/dotdot, NUL/ADS and unsafe Windows aliases as appropriate to actual filename semantics; tests must preserve legitimate names.
- Existing Codex `write_agent_file` is direct fs::write (`:499`); rename happens **before** later serialization/write (`:604/687`), so failure may leave moved files. Source install uses this helper, sync/accept trust stored target_path (`codex_agent_sources.rs:1409/1479`). Revalidate derived scoped target identity against tracked metadata, never let tampered catalog/installed records become arbitrary write/read. Use original revision under durable policy; local/upstream hash check before write is not atomic CAS. Preserve conflict, tracked-only replace, accept-baseline and untrack-without-delete semantics. Existing source remove deletes source metadata and catalog only (`:1197`); install tracking and installed files remain, so later operations can encounter a missing source. Do not silently convert source removal into uninstall or cascading untrack.
- Output styles appends `.md` to item.name without validation (`claude_settings.rs:97`), fs::writes multiple files; no CAS/backups, no true delete. Owner must validate flat names/confinement and validate whole batch before mutation. Never present save-as/delete capability that backend does not implement.
- Antigravity commands genuinely support nested `folder` paths and project override (`gemini.rs:149`); do **not** flatten them under Codex-style basename rules. Current create accepts folder + name through unvalidated join (`:398–425`). Resolve a validated relative folder composed of normal components under selected command root; reject escape/root/prefix/ADS and verify existing ancestors. Existing update/delete select by basename across recursive user/project roots and reject multiple matches (`:224–254`); do not silently pick a winner for duplicate names. TOML update regenerates only description+prompt (`:453`) and drops unknown fields: native unknown-field contract needs an owner fix, not a new parser in the UI.
- OpenCode flat names reject blank and `/`/`\\` (`opencode.rs:380`), but not the whole Windows filename/alias surface. Existing writes are direct fs::write, deletes direct remove_file; scans follow file symlinks through read/metadata. Frontmatter updates preserve unsubmitted keys (`:401`), but values/raw body may contain secrets. Root/project/file symlinks and junctions must be explicitly authorized or rejected for mutation; keep read-only symlink inventory provenance rather than materializing it as local file.
- Skills `file_get` **already** rejects absolute/root paths and canonicalizes target + installation root, rejects outside target (`skills_service.rs:746–769`), with parent/absolute/inside tests. This protects that read only; it is not proof install/delete/source paths are contained. Skill install directory is slugified from descriptor name, but configurable platform relative_path and persisted checkout_path still need root authority. `resolve_source_root` accepts manifest skills_dir via root.join (`:1543`); absolute/traversing manifests require inspection/rejection if treated as managed checkout. Preserve legitimate explicit local source selection as read scope instead of conflating it with destination authority.
- Skills `load_platforms` returns defaults without creating the config (`skills_service.rs:668`); however inventory cache may persist to summary_cache_path through `save_summary_cache` (`:725`) and create runtime/cache directories. A native list read is not automatically zero-write. Test business source bytes separately from explicitly owned caches and keep all cache roots injected.
- Skills direct managed installation writes SKILL.md + `.ccr-skill.json` only (`:950`), not the whole support-file tree. Existing source fixture creates `docs/guide.md` but does not assert support-file installation; do not claim archive/package completeness or expand it silently. `content_save` AtomicWrites descriptor then metadata, no original-token CAS or pair transaction (`:1482`). `remove_installation`/`remove_skill` remove inventoried paths recursively (`:2425`), and source_remove trusts persisted checkout_path (`:1892`). Resolve authorized managed roots, compare recorded identities, do not delete symlink targets or mutate shared symlink installs as copies.
- For nonexistent targets, canonicalize the **existing authorized root/closest existing ancestor**, validate remaining relative components before creating any directories, and verify no escaping symlink/junction/reparse component. For existing target, resolve/compare actual identity. Canonical check then path-based write still has a replacement race; choose owner-level handle/locked policy consistent with existing safety boundary and test it, rather than claiming lexical starts_with alone solves it. This is a suggested minimal policy, not an existing helper found in this search.
- Skills `run_git` uses argv (`:1515`), but `clone` lacks an explicit option separator/URL policy and reads unbounded process output with no deadline or cancellation. `pull --ff-only` preserves existing no-merge policy. No permission to execute arbitrary Git protocol/helper commands should be inferred from a text source field. Reuse existing process/command-policy owners, keep credential-bearing stderr out of native summaries.
- **Skills npx is shell injection exposure:** builder concatenates raw source_ref/selected skills into a string (`:1954–2012`); runner executes `cmd /C` or `sh -lc` (`:2015`). Route review must use structured executable/argv with explicit platform handling and argument validation, never shell text interpolation. Use app-owned managed tree, bounded stdout/stderr/line and timeout, cooperative cancellation plus joined cleanup. Cancellation after partial target mutation must report actual partial outcome/reread, not invent all-or-nothing rollback. `npx_status`/capabilities also spawn blocking probes; keep them on worker with limits and missing-tool state.
- Codex source downloads have retries/ETag/blob cache/truncated partial status; retry sleeps and response bodies have no cancellation/bounds in their current functions. Shared transport should capture cancellation and bounded body/page/catalog limits while preserving partial diagnostics. This path has no archive extraction and no Git subprocess; do not add a downloader runtime or credential account dependency unnecessarily.
- No zip/tar/archive import entry was found in the inspected 06 commands or ccr-skills service. AC6.3 therefore still requires rejecting unsupported archive inputs and preserving any later-selected existing owner boundary; it does not authorize inventing generic extraction. `ccr-store` is not a resource install filesystem owner; no need to add a DB or duplicate session/archive SQL there.

### Existing tests and safe future verification commands (NOT run here)

Fixture anchors:

- Claude MCP tests `claude_mcp_config.rs:943` use TestProcessEnv + temp home/project; production context itself accepts explicit paths. Tests cover local/project/user precedence, approval, Windows keys, secret retention, env merge, transport change, unrelated state, CAS retry/exhaustion/concurrency and owner-only user state. Migrate fixture plumbing to injected roots/child-only env where required, not global HOME mutation or serializing the whole suite.
- Codex local agent tests `codex_agents.rs:816–888`: global path resolution and pure unknown/skills-field merge only. They do **not** prove persisted create/update/rename/delete safety, CAS, failed rename recovery or source installation.
- Codex sources tests `codex_agent_sources.rs:1527`: repo URL parse, category inference and local/upstream conflict classification; no HTTP failure/cancel/install-compensation fixture found. Add injected fake transport, no real GitHub/account request.
- OpenCode tests `opencode.rs:1011`: path mapping, frontmatter roundtrip, body-only parsing, agent/command metadata preservation; use temporary roots for actual read-save-reread/conflict and malicious name/symlink regressions.
- Skills unit helper `skills_service.rs:2814` fully isolates home+cache fields; `:2943/2963/2981` file_get traversal/absolute/inside; `:3002` local source install/sync/remove, `:3125` duplicate/filter, `:3207` local Git clone/pull/remove, `:3261` offline validation, `:3322` command preview. Existing tests do not cover npx shell injection, live process cancellation/descendant cleanup, install fault/permissions or tampered metadata. `crates/ccr-skills/tests/skills_ext_*` cover scanner/agents/taxonomy/toggle/trash/versioning; don't automatically expose every extension capability as newly requested product scope.
- System prompt tests `system_prompts.rs:323` cover closed aliases/path/list/rules/version conflict and size-warning behavior. `save_prompt_file` has explicit temp paths and backup seam; retain parser/secret-safe error mapping.

Recommended commands from repository root, after owner fixture seams are safe (actual pinned Cargo/runtime wrapper selected by coordinator; preserve `--locked`, default test parallelism):

```text
cargo test -p ccr-skills --locked -- --skip export_bindings
cargo test -p ccr-app --locked -- --skip export_bindings
cargo test -p ccr-core --locked process_gateway -- --skip export_bindings
cargo clippy -p ccr-skills -p ccr-app --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked claude_mcp_config -- --skip export_bindings
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked codex::agents -- --skip export_bindings
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked codex::agent_sources -- --skip export_bindings
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked opencode -- --skip export_bindings
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked system_prompts -- --skip export_bindings
just desktop-test
just desktop-test-ui
just desktop-lint
just fmt-check
```

Tauri filters above correspond module names, but coordinator must inspect test count to avoid a zero-test false pass. Run actual affected `gemini`, `codex::mcp`, `claude::settings`, presets, bindings/inventory gates after extraction; existing test absence means add meaningful fixtures, not report zero tests passed. Frontend parity anchors: `tests/mcp/mcp-panels.smoke.test.tsx`, `tests/api/typed-codex-system-prompts.smoke.test.ts`, `tests/platforms/platform-surface-unify.smoke.test.ts`, Settings lossless/capability suites and typed-command boundary. No validation command may perform implicit fmt/version-sync repair.

Core native acceptance: synthetic resource list/filter/detail, dirty draft/cancel, valid save + actual reread, same-token conflict preserving external bytes, invalid input/permission error/retry, missing-tool/install failure, cooperative cancel with joined tree cleanup, normal exit with pending job; actual native evidence distinct from headless. Destructive real UI tests require applicable computer-use action policy; use owned synthetic fixtures. User explicitly deferred scaling/DPI/size tests; do not add them to this task's current core execution or mark them passed.

### Suggested non-overlapping 06 ownership partitions

1. **MCP policy owner:** extracted scoped Claude MCP and unified orchestration; Codex MCP semantic adapter; existing preset/sync domain hardening. One writer owns `claude_mcp_config.rs`, `claude_mcp.rs`, `unified_mcp.rs`, `mcp_presets.rs` and `ccr-skills/managers/mcp_preset_manager.rs`. Coordinate 05's shared config writer, avoid concurrent parent `codex.rs` writes.
2. **Codex agent/source owner:** `codex_agents.rs`, `codex_agent_sources.rs`, new focused framework-free service files, corresponding fixture tests. Own all source catalog/install metadata transactions and path boundaries together; no second agent edits local agent writer simultaneously.
3. **Other resource owner:** Claude typed array/hooks/statusline/style/presets, `gemini.rs`, `opencode.rs`, system prompts and PromptsManager. Giant parent `claude.rs/codex.rs` exports/imports remain single coordinator writer (or explicitly transfer once); `gemini.rs` and `opencode.rs` each single writer shared with any still-active05 work. Reuse05 typed/raw Settings APIs; no competing whole-file extraction.
4. **Skills/domain owner:** `skills_service.rs` plus narrowly necessary core process gateway use; owns injected roots/cache, source/read/write/remove guard, structured npx/Git runner and fault/cancel tests. Native resource UI owner consumes typed APIs only after contract freeze. Manifest/locks and inventory remain coordinator-only. Concurrency slots may require sequential passes; this is a file responsibility map, not permission to start06 early.

### Related specs and external references

- Read: `.trellis/spec/ccr-skills/backend/{index,backend-guidelines}.md`; `ccr-store/backend/{index,backend-guidelines}.md`; `ccr-app/backend/{index,shared-desktop-services,configuration-services}.md`; `ccr-ui/frontend/{index,raw-config-editor-contracts,platform-surface-contracts}.md`; `ccr-core/backend/{index,atomic-writer,managed-process}.md`; `ccr-codex/backend/index.md` and affected filesystem/security section of its backend guidelines; `ccr-types/backend/{index,backend-guidelines}.md`; `ccr-cli/backend/index.md`; `guides/{index,cross-layer-thinking-guide,code-reuse-thinking-guide}.md`.
- Old skills/store/core example commands suggest `--test-threads=1`; user's current explicit default-parallel/skip-bindings instruction takes precedence. Solve test isolation rather than weakening parallelism.
- No external research/network performed. GitHub API/raw bases and skills.sh references described here are string constants in repository source, not verified current third-party API guarantees. No dependency versions were selected or changed.

## Caveats / Not Found

- No claim of accepted06 implementation, safe install subprocess, complete archive boundary, native Skills CRUD, passed test, platform run or performance improvement.
- 05 extraction is future work in this recovery context; re-read its actual settled service API before06 implementation. Do not build against assumed file/module names.
- A prompt/resource can contain credential text even if its schema is not an auth schema. Native list/Debug/error/prefs need safe projections, and raw drafts require explicit scoped lifetime and stale completion checks.
- Root confinement helper for arbitrary new resource writes was not found. Existing normalized lock path, profile validator, restrictive toggle name validator and safe file_get each solve different problems; none can be advertised as a universal safe join.
- Preservation of existing compatibility versus repair of the observed preset path mismatch/output-style wrappers needs an explicit coordinator disposition. Do not silently change Tauri behavior while moving code.

## Appendix: exact retained command IDs → current handlers

The following table is populated from the retained inventory plus current handler definitions, without recapturing or modifying the inventory. Each command maps to its group above; line references are a source snapshot, not verification results.

| Group | Exact ID | Current implementation |
| --- | --- | --- |
| system_prompts | `system_prompts_list` | `ccr-ui/src-tauri/src/commands/system_prompts.rs:225` |
| system_prompts | `system_prompts_get` | `ccr-ui/src-tauri/src/commands/system_prompts.rs:258` |
| system_prompts | `system_prompts_save` | `ccr-ui/src-tauri/src/commands/system_prompts.rs:277` |
| system_prompts | `system_prompts_create` | `ccr-ui/src-tauri/src/commands/system_prompts.rs:297` |
| claude | `claude_list_mcp_servers` | `ccr-ui/src-tauri/src/commands/claude_mcp.rs:9` |
| claude | `claude_add_mcp_server` | `ccr-ui/src-tauri/src/commands/claude_mcp.rs:20` |
| claude | `claude_update_mcp_server` | `ccr-ui/src-tauri/src/commands/claude_mcp.rs:35` |
| claude | `claude_delete_mcp_server` | `ccr-ui/src-tauri/src/commands/claude_mcp.rs:50` |
| claude | `claude_list_agents` | `ccr-ui/src-tauri/src/commands/claude_agents.rs:4` |
| claude | `claude_add_agent` | `ccr-ui/src-tauri/src/commands/claude_agents.rs:12` |
| claude | `claude_update_agent` | `ccr-ui/src-tauri/src/commands/claude_agents.rs:31` |
| claude | `claude_delete_agent` | `ccr-ui/src-tauri/src/commands/claude_agents.rs:53` |
| claude | `claude_list_slash_commands` | `ccr-ui/src-tauri/src/commands/claude_slash.rs:4` |
| claude | `claude_add_slash_command` | `ccr-ui/src-tauri/src/commands/claude_slash.rs:14` |
| claude | `claude_update_slash_command` | `ccr-ui/src-tauri/src/commands/claude_slash.rs:33` |
| claude | `claude_delete_slash_command` | `ccr-ui/src-tauri/src/commands/claude_slash.rs:55` |
| claude | `claude_list_plugins` | `ccr-ui/src-tauri/src/commands/claude_plugins.rs:4` |
| claude | `claude_add_plugin` | `ccr-ui/src-tauri/src/commands/claude_plugins.rs:12` |
| claude | `claude_update_plugin` | `ccr-ui/src-tauri/src/commands/claude_plugins.rs:31` |
| claude | `claude_delete_plugin` | `ccr-ui/src-tauri/src/commands/claude_plugins.rs:53` |
| claude | `claude_get_output_styles` | `ccr-ui/src-tauri/src/commands/claude_settings.rs:42` |
| claude | `claude_update_output_styles` | `ccr-ui/src-tauri/src/commands/claude_settings.rs:84` |
| claude | `claude_get_statusline` | `ccr-ui/src-tauri/src/commands/claude_settings.rs:114` |
| claude | `claude_update_statusline` | `ccr-ui/src-tauri/src/commands/claude_settings.rs:125` |
| claude | `claude_list_hooks` | `ccr-ui/src-tauri/src/commands/claude_hooks.rs:4` |
| claude | `claude_update_hooks` | `ccr-ui/src-tauri/src/commands/claude_hooks.rs:13` |
| claude | `claude_list_prompts` | `ccr-ui/src-tauri/src/commands/claude_settings.rs:269` |
| claude | `claude_update_prompts` | `ccr-ui/src-tauri/src/commands/claude_settings.rs:290` |
| codex | `codex_list_mcp_servers` | `ccr-ui/src-tauri/src/commands/codex_mcp.rs:5` |
| codex | `codex_add_mcp_server` | `ccr-ui/src-tauri/src/commands/codex_mcp.rs:49` |
| codex | `codex_update_mcp_server` | `ccr-ui/src-tauri/src/commands/codex_mcp.rs:83` |
| codex | `codex_delete_mcp_server` | `ccr-ui/src-tauri/src/commands/codex_mcp.rs:124` |
| codex | `codex_list_agents` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:506` |
| codex | `codex_add_agent` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:524` |
| codex | `codex_update_agent` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:574` |
| codex | `codex_delete_agent` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:636` |
| codex | `codex_rename_agent` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:663` |
| codex | `codex_copy_agent` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:716` |
| codex | `codex_validate_agent_toml` | `ccr-ui/src-tauri/src/commands/codex_agents.rs:771` |
| codex | `codex_list_agent_sources` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1132` |
| codex | `codex_add_agent_source` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1142` |
| codex | `codex_remove_agent_source` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1197` |
| codex | `codex_sync_agent_source` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1210` |
| codex | `codex_get_agent_source_catalog` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1237` |
| codex | `codex_install_source_agent` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1274` |
| codex | `codex_sync_source_install` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1384` |
| codex | `codex_accept_local_source_install` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1469` |
| codex | `codex_untrack_source_install` | `ccr-ui/src-tauri/src/commands/codex_agent_sources.rs:1494` |
| gemini | `gemini_list_mcp_servers` | `ccr-ui/src-tauri/src/commands/gemini.rs:284` |
| gemini | `gemini_add_mcp_server` | `ccr-ui/src-tauri/src/commands/gemini.rs:304` |
| gemini | `gemini_update_mcp_server` | `ccr-ui/src-tauri/src/commands/gemini.rs:331` |
| gemini | `gemini_delete_mcp_server` | `ccr-ui/src-tauri/src/commands/gemini.rs:358` |
| gemini | `gemini_list_slash_commands` | `ccr-ui/src-tauri/src/commands/gemini.rs:380` |
| gemini | `gemini_add_slash_command` | `ccr-ui/src-tauri/src/commands/gemini.rs:391` |
| gemini | `gemini_update_slash_command` | `ccr-ui/src-tauri/src/commands/gemini.rs:434` |
| gemini | `gemini_delete_slash_command` | `ccr-ui/src-tauri/src/commands/gemini.rs:467` |
| gemini | `gemini_list_extensions` | `ccr-ui/src-tauri/src/commands/gemini.rs:482` |
| opencode | `opencode_list_agents` | `ccr-ui/src-tauri/src/commands/opencode.rs:898` |
| opencode | `opencode_add_agent` | `ccr-ui/src-tauri/src/commands/opencode.rs:906` |
| opencode | `opencode_update_agent` | `ccr-ui/src-tauri/src/commands/opencode.rs:925` |
| opencode | `opencode_delete_agent` | `ccr-ui/src-tauri/src/commands/opencode.rs:930` |
| opencode | `opencode_list_commands` | `ccr-ui/src-tauri/src/commands/opencode.rs:944` |
| opencode | `opencode_add_command` | `ccr-ui/src-tauri/src/commands/opencode.rs:952` |
| opencode | `opencode_update_command` | `ccr-ui/src-tauri/src/commands/opencode.rs:971` |
| opencode | `opencode_delete_command` | `ccr-ui/src-tauri/src/commands/opencode.rs:976` |
| opencode | `opencode_list_local_plugins` | `ccr-ui/src-tauri/src/commands/opencode.rs:990` |
| unified_mcp | `unified_list_mcp_servers` | `ccr-ui/src-tauri/src/commands/unified_mcp.rs:252` |
| unified_mcp | `unified_add_mcp_server` | `ccr-ui/src-tauri/src/commands/unified_mcp.rs:435` |
| unified_mcp | `unified_update_mcp_server` | `ccr-ui/src-tauri/src/commands/unified_mcp.rs:493` |
| unified_mcp | `unified_delete_mcp_server` | `ccr-ui/src-tauri/src/commands/unified_mcp.rs:516` |
| builtin_prompts | `list_builtin_prompts` | `ccr-ui/src-tauri/src/commands/builtin_prompts.rs:58` |
| builtin_prompts | `get_builtin_prompt` | `ccr-ui/src-tauri/src/commands/builtin_prompts.rs:70` |
| builtin_prompts | `get_builtin_prompts_by_category` | `ccr-ui/src-tauri/src/commands/builtin_prompts.rs:79` |
| mcp_presets | `list_mcp_presets` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:22` |
| mcp_presets | `get_mcp_preset` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:37` |
| mcp_presets | `install_mcp_preset` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:52` |
| mcp_presets | `install_mcp_preset_single` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:99` |
| mcp_presets | `list_source_mcp_servers` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:139` |
| mcp_presets | `sync_mcp_server` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:165` |
| mcp_presets | `sync_all_mcp_servers` | `ccr-ui/src-tauri/src/commands/mcp_presets.rs:212` |
