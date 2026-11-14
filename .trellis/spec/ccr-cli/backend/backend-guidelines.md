# ccr-cli Backend Guidelines

> CLI/application domain crate.

## Scope

`crates/ccr-cli` owns command definitions, command dispatch, CLI presentation, application services, and command-facing managers. Domain logic should call shared crates rather than duplicating lower-level behavior.

Reference files:

- `crates/ccr-cli/src/lib.rs`
- `crates/ccr-cli/src/cli/`
- `crates/ccr-cli/src/commands/mod.rs`
- `crates/ccr-cli/src/application/`

## Structure

Follow the current command split:

- `cli/` for Clap definitions and subcommand enums.
- `commands/` for user-facing command handlers grouped by area.
- `services/`, `managers/`, and `platforms/` for application orchestration that is still CLI-specific.
- `models/` mostly re-exports shared model crates for command use.

Do not put Ratatui rendering in this crate; that belongs in `ccr-tui`. Do not put persistent session SQL here; that belongs in `ccr-store` or `ccr-db`.

## Output And Logging

Human CLI output may use `println!`, tables, and colored output in command handlers. JSON modes should serialize stable output structs. Diagnostic/runtime logs should use `tracing`, not direct stderr, unless a test-only debug helper is explicitly isolated.

Never print secrets. Use masking helpers for tokens, provider keys, auth files, and config values.

## Scenario: CLI Table Style And Truncation Compatibility

### 1. Scope / Trigger

- Trigger: adding or changing a `comfy-table` table, table preset, width constraint, ANSI color, or command snapshot.
- Applies because `comfy-table 8` replaced positional presets with `TableStyle` and changed its default truncation indicator from `...` to `…`.

### 2. Signatures

- Default ASCII table: `crate::commands::common::new_table() -> comfy_table::Table`.
- Full UTF-8 table: `crate::commands::common::new_utf8_table() -> comfy_table::Table`.
- Style load owned by the shared helper: `Table::load_style(presets::UTF8_FULL)`.

### 3. Contracts

- Command handlers construct tables through `new_table` or `new_utf8_table`; do not call `Table::new` directly.
- Both helpers explicitly retain the CLI's `...` truncation indicator so an upstream default change cannot silently alter script-visible text.
- `new_utf8_table` is the only owner of the full UTF-8 preset. Callers still own headers, column order, padding/constraints, alignment, and content arrangement.
- Captured/non-TTY output remains free of ANSI styling. Explicit TTY styling continues to respect `NO_COLOR`.
- A dependency migration must not change human output intentionally unless the command contract and its tests are updated in the same batch.

### 4. Validation & Error Matrix

- Direct `Table::new` in a CLI table path -> replace it with the matching shared helper before accepting the change.
- `load_preset` or another removed pre-8 style API remains -> compile failure; migrate the style only in the shared helper.
- Narrow table renders `…` instead of `...` -> compatibility regression.
- Captured/non-TTY table contains an ANSI escape -> output regression for pipes, snapshots, and Windows test processes.
- Border, column order, or alignment changes without an explicit command requirement -> reject as an unrelated output change.

### 5. Good/Base/Bad Cases

- Good: a command starts with `let mut table = new_utf8_table();` and configures only its own header, rows, and widths.
- Base: an intentionally ASCII table uses `new_table()` and otherwise keeps the default ASCII style.
- Bad: calling `Table::new().load_style(UTF8_FULL)` locally, because it bypasses the shared truncation contract.

### 6. Tests Required

- `cargo test -p ccr-cli commands::common::table::tests` asserts exact UTF-8 borders/column order, 12-column truncation with `...`, non-TTY plain text, and ANSI/`NO_COLOR` behavior.
- Run `cargo test -p ccr-cli --all-features`, `cargo test -p ccr-tui --all-features`, and `cargo test -p ccr --test commands -- --test-threads=1` after table or dependency changes.
- Run Rust 1.95 check, `just lint-strict`, `just test`, `just tauri-ci`, and both Cargo-lock audits before accepting a `comfy-table` upgrade.

### 7. Wrong vs Correct

#### Wrong

```rust
let mut table = Table::new();
table.load_style(comfy_table::presets::UTF8_FULL);
```

#### Correct

```rust
let mut table = crate::commands::common::new_utf8_table();
```

## Error Handling

Command handlers should return `ccr_core::Result<T>` or `anyhow::Result<T>` where the local command already uses it. Preserve actionable errors from shared crates and handle them at the dispatcher boundary.

Do not use panics for invalid user input. Clap validation, typed command enums, and `CcrError` should carry invalid states.

Codex Auth plaintext import must return the service error after printing its failure. The dispatcher receives Err and preserves failure status. Encrypted import uses the same domain preflight after decryption. Preserve the existing rule that CLI exports containing credentials use encrypted export; `--no-secrets` exports metadata. No new plaintext credential-export flag or password DTO is implied by import validation.

## Scenario: Project Workflow Bootstrap

### 1. Scope / Trigger

- Trigger: adding or changing `ccr project init`, dual workflow-engine orchestration (`trellis` / `mini-trellis`), host platform menus, workspace fingerprint conflict guards, minimum postcondition checks, four-stage wizard/summary presentation, or the fixed project ignore rules.
- Applies to the Clap command tree (`cli/definitions.rs`, `cli/subcommands/project.rs`, `cli/help_config.rs`), dispatcher (`cli/dispatch.rs`), `commands/project/{mod,init,workflow,fingerprint,wizard,render,gitignore}.rs`, command integration tests (`crates/ccr/tests/commands/{project_init,help}.rs`), and bilingual command reference (`docs/reference/commands/{project-init,index}.md` and `docs/en/reference/commands/{project-init,index}.md`).

### 2. Signatures

- CLI signature: `ccr [-y|--yes] project init [--workflow <trellis|mini-trellis>]` (`--workflow` uses `clap::ValueEnum` with `ignore_case = true`; no `--json` flag and no path positional argument).
- Rust types & entrypoints:
  - `Commands::Project { action: ProjectAction }`
  - `ProjectAction::Init { workflow: Option<WorkflowEngine> }`
  - `WorkflowEngine::{Trellis, MiniTrellis}`
  - `project_init_command(auto_yes: bool, workflow: Option<WorkflowEngine>) -> ccr_core::Result<()>`
- Default engine policy:
  - Interactive (`!auto_yes`, `--workflow` omitted): `Select` menu ordered `[MiniTrellis, Trellis]`, highlighting index `0` (`mini-trellis`) by default; explicit `--workflow` skips this prompt.
  - Non-interactive (`-y` / `--yes`, `--workflow` omitted): defaults to `WorkflowEngine::Trellis` for backward compatibility; `--workflow mini-trellis` selects `MiniTrellis`.
- Host platform matrix (menu order, default all checked in interactive mode, all passed in `-y` mode):
  - `trellis` (5 items): Claude Code (`--claude`), Codex (`--codex`), Cursor (`--cursor`), Oh My Pi (`--omp`), Grok (`--grok`).
  - `mini-trellis` (4 items): Claude Code (`--claude`), Codex (`--codex`), Cursor (`--cursor`), Oh My Pi (`--omp`) — excludes unsupported `--grok`.
  - Downstream invocation: `<trellis|mini-trellis> init -u <developer> <selected-flags-in-order...> [--yes]`. Append `--yes` last only when `auto_yes` is true; never pass `-s` / `--skip-existing`.
- Workspace fingerprint markers (`Uninitialized`, `Trellis`, `MiniTrellis`, `Mixed`):
  - `TrellisMarkers` (5 items, any match): `.trellis/workflow.md`, `.trellis/scripts/task.py`, `.trellis/.template-hashes.json`, `.trellis/.version`, or `AGENTS.md` containing `Managed by Trellis.` (checked within `<!-- TRELLIS:START -->...<!-- TRELLIS:END -->` blocks when present, otherwise full file).
  - `MiniTrellisMarkers` (6 items, any match): `.trellis/.mini-trellis-manifest.json`, `.trellis/spec/guides/mini-memory.md`, `.claude/commands/mini-trellis/remember.md`, `.agents/skills/mini-trellis-remember/SKILL.md`, `.omp/extensions/mini-trellis/index.ts`, or `AGENTS.md` containing `Managed by mini-trellis.` (same managed-block rule).
- Minimum workflow postconditions (checked after downstream CLI exits zero):
  - `trellis`: `<cwd>/.trellis/workflow.md` and `<cwd>/.trellis/scripts/task.py` are files.
  - `mini-trellis`: `<cwd>/.trellis/.mini-trellis-manifest.json` and `<cwd>/.trellis/scripts/add_session.py` are files, and `<cwd>/AGENTS.md` is a file containing `Managed by mini-trellis.`.
- Fixed ignore rules, in this order: `.agents/`, `.claude/`, `.codex/`, `.cursor/`, `.omp/`, `.grok/`, `ref/`. They do not vary with the selected workflow engine or platforms. `.agents/` is the shared Codex directory. `.trellis/` stays tracked. Kimi stays off both menus and is not an ignore rule. A file that only has the older four rules (`.agents/`, `.claude/`, `.codex/`, `ref/`) gains `.cursor/`, `.omp/`, and `.grok/` on the next run. Source: Grok session `01a1c998-2fde-7572-910a-b5d6808135e4` (2026-11-11, user accepted the init fix and asked to commit); implemented by `PROJECT_IGNORE_RULES` in `crates/ccr-cli/src/commands/project/gitignore.rs` and stated in `docs/reference/commands/project-init.md`.

### 3. Contracts

- The process current directory is the only project root; do not add upward/downward project discovery or a path argument implicitly.
- Print the header block (`项目工作流初始化`, `目标目录`, `运行模式`, plus `工作流预设` when `--workflow` is explicitly supplied), then run four numbered stages in strict order:
  1. `[1/4] 检查 Git 仓库`: `git rev-parse --show-toplevel` detects either an existing repo root (`ExistingRoot`) or membership in a parent worktree (`ParentWorktree`, warns with `跳过嵌套 git init` and reuses the parent repo without creating a nested `.git`). Outside any worktree, run `git init` (`Initialized`).
  2. `[2/4] 配置工作流与宿主平台`:
     - Immediately reject `WorkspaceState::Mixed` (`ensure_not_mixed_workspace`) with `ValidationError` (90) before any prompt or `git config user.name` call.
     - Interactive mode resolves the engine (`--workflow` or `Select` defaulting to `mini-trellis`), validates `validate_markers_for_target` (stopping before platform/username prompts on cross-engine conflict), shows `MultiSelect` for that engine's platforms (re-prompting with warning on empty selection), silently probes `git config user.name` to prefill the `Input` default (Enter accepts a non-empty prefill; blank input when unset warns and re-prompts), and prints `工作流引擎`, `宿主平台`, and `开发者` confirmation lines.
     - `-y` mode resolves the engine (`explicit.unwrap_or(Trellis)`), validates `validate_markers_for_target`, strictly reads `git config user.name` with the resolved Git binary (`ExternalCommandError` on spawn failure, `ValidationError` (90) on non-zero exit or blank stdout), selects all supported platforms for that engine (`5` for `trellis`, `4` for `mini-trellis`), and prints confirmation lines.
     - Same-engine markers classify the run as `WorkflowInitMode::Reinit` (`幂等更新`); `Uninitialized` classifies as `WorkflowInitMode::Fresh` (`首次初始化`).
  3. `[3/4] 执行 <trellis|mini-trellis> 初始化`: resolve the selected engine binary on `PATH`, wrap inherited `stdin`/`stdout`/`stderr` output between start/end boundary lines (`┌─ 外部工具输出 (<cmd>) ──...` / `└─ 外部工具执行完毕 ──...` on non-dumb TTY; `--- 开始执行 <cmd> ---` / `--- <cmd> 执行结束 ---` when captured or `TERM=dumb`), and require both exit code `0` and the engine's minimum postcondition before continuing.
  4. `[4/4] 准备 ref/ 目录与 .gitignore 忽略规则`: keep an existing `ref/` directory, fail with `FileIoError` (51) if `ref` is a file/non-directory without touching `.gitignore`, or create a missing `ref/` directory (even when `.gitignore` is already complete). Merge only missing rules from the fixed seven-rule list while preserving existing text and LF/CRLF line endings, skipping the write when unchanged and using `AtomicWriter` when updated.
- Terminal capability adaptation (`WizardCapabilities` / `WizardThemeKind`):
  - `dialoguer` uses the custom `ColorfulTheme` only on styled non-dumb TTY stderr, falling back to `SimpleTheme` under non-TTY, `NO_COLOR`, `CLICOLOR=0`, or `TERM=dumb`.
  - After stage 4 succeeds, render a 6-row summary table (`Git 仓库`, `工作流引擎`, `宿主平台`, `开发者`, `ref/ 目录`, `.gitignore`) via `new_utf8_table()` (or ASCII `new_table()` under `TERM=dumb`), calling `force_no_tty()` and omitting cell ANSI styling whenever `!caps.style`, while keeping `...` truncation.
  - Print the ready line (`项目工作流初始化完成（Git、<Trellis|mini-trellis>、Agent 忽略规则和 ref/ 均已就绪）`) and engine-specific `print_next_steps`:
    - `mini-trellis`: `mini-trellis doctor` and `mini-trellis mem recall "project init"`.
    - `trellis`: `trellis doctor` and `ccr project init --help`.
  - Any stage failure stops immediately without rolling back completed stages and must never print the summary table, ready line, or `下一步` block. Idempotent rerun in the same directory is the recovery model.

### 4. Validation & Error Matrix

- Bare `ccr project` or invalid `--workflow <value>` -> Clap parse failure (exit code 2); never initialize implicitly.
- Git executable missing or `git init` non-zero -> `ExternalCommandError` (75, `Git 阶段`); do not prompt, run workflow CLI, create `ref/`, or write `.gitignore`.
- Dual-engine mixed workspace (`TrellisMarkers` and `MiniTrellisMarkers` both present) -> `ValidationError` (90, `工作流冲突`混杂提示); block before prompts, `git config user.name`, workflow CLI, `ref/`, or `.gitignore`.
- Existing `trellis` workspace with target `mini-trellis` -> `ValidationError` (90); block before platform/username prompts or `mini-trellis init`, and instruct running `mini-trellis migrate --dry-run` then `mini-trellis migrate`.
- Existing `mini-trellis` workspace with target `trellis` (including bare `ccr -y project init`) -> `ValidationError` (90); block before `trellis init` (preventing exit-0 silent hybrid corruption) and instruct running `mini-trellis uninstall` first if switching.
- Workflow, platform, or username interact failure -> `UiError` (74); do not run workflow CLI.
- `-y` cannot start `git config user.name` -> `ExternalCommandError` (75); do not run workflow CLI.
- `-y` and Git user name is missing or blank -> `ValidationError` (90, `用户名阶段`); do not run workflow CLI, create `ref/`, or write `.gitignore`.
- Selected workflow executable (`trellis` or `mini-trellis`) missing or exits non-zero -> `ExternalCommandError` (75, `Trellis 阶段` / `mini-trellis 阶段`); keep Git result, do not create `ref/` or write `.gitignore`.
- Selected workflow exits zero but misses any minimum postcondition artifact -> `ValidationError` (90, `缺少最低工作流文件`); do not create `ref/` or write `.gitignore`.
- `ref` is a file (or non-directory) -> `FileIoError` (51, `ref/ 阶段`); do not write `.gitignore`.
- `.gitignore` read/write fails -> `FileIoError` (51, `.gitignore 阶段`); keep completed Git, workflow, and `ref/` results.
- All seven fixed rules already exist -> success without rewriting `.gitignore` (`.gitignore 已包含全部项目忽略规则，跳过写入`), while still creating a missing `ref/` directory.

### 5. Good/Base/Bad Cases

- Good: interactive `ccr project init` defaults to `mini-trellis` (4 platforms without Grok), prefills `git config user.name`, wraps `mini-trellis init` output, validates manifest + `add_session.py` + managed `AGENTS.md`, creates `ref/`, atomically merges the 7 ignore rules, and prints the 6-row summary card plus `mini-trellis doctor` next steps.
- Good: `ccr -y project init` preserves backward compatibility with `trellis`, `git config user.name`, all 5 flags, and trailing `--yes`; `ccr -y project init --workflow mini-trellis` runs `mini-trellis` with its 4 flags and `--yes`.
- Good: running `--workflow trellis` (or bare `-y`) inside an existing `mini-trellis` repo fails preflight with exit code 90 and `mini-trellis uninstall` guidance before `trellis init` can silently corrupt `AGENTS.md` and hooks.
- Good: a monorepo child reports the parent Git root, skips nested `git init`, and scopes workflow files, `ref/`, and `.gitignore` to the child directory.
- Base: rerunning the same engine marks the summary row as `幂等更新`, omits `-s`, keeps `ref/` contents, leaves a complete `.gitignore` byte-for-byte unchanged, and renders zero ANSI escapes under pipes, `NO_COLOR=1`, or `TERM=dumb` (ASCII borders).
- Bad: checking only `<cwd>/.git` and creating a nested repository inside a parent worktree.
- Bad: passing `--grok` to `mini-trellis`, passing `-s`/`--skip-existing` on reinit, or passing `--kimi`.
- Bad: treating downstream exit code zero as complete without checking the selected engine's minimum postcondition files.

### 6. Tests Required

- `cargo test -p ccr-cli commands::project -- --skip export_bindings --test-threads=1`
  - Assert dual-engine defaults, argv ordering (`5` vs `4` platforms, middle-flag removal, trailing `--yes`, no `--grok` for `mini-trellis`, no `-s`), four-state workspace fingerprint detection and conflict hints, engine-specific minimum postconditions, `.gitignore` seven-rule idempotent merging (LF/CRLF, legacy four-rule upgrade), and capability-aware rendering (`NO_COLOR`, captured pipe, `TERM=dumb` ASCII table, `...` truncation, zero ANSI).
- `cargo test -p ccr --test commands project_init -- --skip export_bindings --test-threads=1`
  - Assert end-to-end stage order, cwd isolation, parent-worktree reuse, `-y` default `trellis` (5 flags) and `--workflow mini-trellis` (4 flags), cross-engine & mixed preflight exit-90 blocks, missing binary (75) and missing postcondition (90) boundaries, `ref/` preservation/conflict (51), `.gitignore` mtime preservation when complete, summary card & engine-specific next steps, failure-tail suppression, and pipe/`NO_COLOR`/`TERM=dumb` degradation.
- `cargo test -p ccr --test commands help -- --skip export_bindings --test-threads=1`
  - Assert `ccr project init --help` and `ccr help project init` parity, `--workflow` values (`trellis`, `mini-trellis`), interactive vs `-y` defaults, 5 vs 4 platform sets (Grok exclusion for `mini-trellis`), the 7 fixed `.gitignore` rules, and `ccr init` separation.
- Run `cd docs && bun run audit && bun run build` (or `just docs-check`), `just version-check`, `just fmt-check`, and `just lint-strict`.

### 7. Wrong vs Correct

#### Wrong

```rust
if !root.join(".git").exists() {
    Command::new("git").arg("init").status()?;
}
Command::new("mini-trellis")
    .args(["init", "-u", user, "--grok", "-s"])
    .status()?;
fs::write(root.join(".gitignore"), ".agents/\n.claude/\n.codex/\n")?;
```

Checking only `.git` creates nested repositories in subdirectories. Skipping fingerprint preflight lets cross-engine runs corrupt the workspace; `--grok` is unsupported by `mini-trellis`, and `-s` breaks downstream `handleReinit` template-hash tracking. Overwriting `.gitignore` drops existing entries and omits `.cursor/`, `.omp/`, `.grok/`, and `ref/`.

#### Correct

```rust
let (git, git_outcome) = ensure_git_repository(&root, caps)?;          // detects parent worktrees
let request = resolve_workflow_request(auto_yes, workflow, &git, &root, caps)?; // fingerprint guard + 5/4 platforms
run_workflow_init(&root, &request, caps)?;                             // bounded TTY output + engine postcondition
let ref_outcome = ensure_ref_dir(&root, caps)?;                        // keep directory, reject file
let gitignore_outcome = ensure_project_gitignore(&root, caps)?;        // 7-rule merge + no-op + AtomicWriter
println!("{}", render_summary_table(&summary, caps));                  // new_utf8_table / new_table in dumb mode
print_next_steps(next_steps_for_engine(request.workflow));
```

This enforces cross-engine safety, preserves ownership boundaries, adapts cleanly to non-TTY/`NO_COLOR`/`TERM=dumb`, and keeps retries idempotent.

## Claude Profile Auth Mode Contract

`ClaudePlatform::apply_profile` branches on auth mode: `Subscription` calls `clear_ccr_managed_vars()` and writes no key from `CCR_MANAGED_KEYS`; `ApiKey` calls `settings.apply_managed_env(section.to_managed_env_pairs())` and writes the registered overrides. Keys outside the registry are user-owned and survive both branches. A third-party profile therefore **only works under `api_key`**.

Auth mode has two layers — keep them separate:

- `ClaudeAuthService::resolve_profile_auth_mode` — literal/stored resolution (explicit `platform_data.auth_mode` wins over inference). Do not change this; tests depend on its literal semantics.
- `ClaudeAuthService::effective_auth_mode` — normalization layer on top of resolve: if resolved is `Subscription` **and** `is_api_key_shaped`, return `ApiKey`. `ClaudePlatform::profile_auth_mode` delegates here so apply / validate / `profile_to_json` stay consistent.

`is_api_key_shaped` is intentionally conservative: `provider_type == "third_party_model"`, or `base_url` and `auth_token` both non-empty. **Do not** include model-mapping fields — `ANTHROPIC_DEFAULT_*_MODEL` is valid on official subscription (snapshot pinning), so that would false-positive and fail `section.validate()`.

Correction happens at two points and must stay idempotent: `normalize_profile` (save - persists the corrected `auth_mode`) and `apply_profile` (defensive - self-heals stale on-disk profiles). Defensive apply must persist the corrected profile through the guarded profiles writer **before** loading or modifying runtime settings. If persistence fails, return an actionable error naming the profile and leave `settings.json` byte-for-byte unchanged. Each correction emits a `tracing::warn`; never log `auth_token` / full `base_url`.

Model-mapping fields are typed on `ProfileConfig` / `ConfigSection` and mapped in `ConfigSection::to_managed_env_pairs` (ccr-config, keyed by `ccr_types::env_keys` constants); `custom_model_option`(`_name`) -> `ANTHROPIC_CUSTOM_MODEL_OPTION`(`_NAME`). New env keys must be added to `ccr_types::env_keys::CCR_MANAGED_KEYS`; `ClaudePlatform::get_env_var_names` derives from that registry. Typing a previously-untyped key auto-migrates existing TOML (serde captures it into the typed slot instead of `other`/`platform_data`).

`ClaudeSettings` itself is `ccr_types::ClaudeSettings` (single workspace shape); `managers/settings.rs` is a pure IO adapter (`SettingsManager`: load/save/backup/restore) plus a re-export, and must not grow local settings types or env-mutation logic.

## Scenario: Claude API-Key Profile Runtime Env

### 1. Scope / Trigger

- Trigger: adding or changing Claude profile fields that write Claude Code environment variables, profile apply behavior, doctor diagnostics, or Claude state-file ownership.
- Applies to `ProfileConfig`, `ConfigSection`, `ccr_config::profile_to_section`, `ccr_config::section_to_profile`, `ClaudeSettings`, `ClaudePlatform`, Tauri Claude profile JSON, and command integration tests.

### 2. Signatures

- `ConfigSection::{default_fable_model, default_*_model_name, claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic}`
- `ProfileConfig::{default_fable_model, default_*_model_name, claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic}`
- `ConfigSection::to_managed_env_pairs()` (ccr-config)
- `env_keys::CCR_MANAGED_KEYS` (ccr-types)
- `ClaudeSettings::{apply_managed_env, clear_ccr_managed_vars, has_managed_overrides, managed_env_entries}` (ccr-types)
- `ClaudePlatform::get_env_var_names()`
- `ClaudePlatform::apply_profile(name)`
- `ccr_config::ClaudeRuntimePaths::{from_env, resolve_with}` owns all
  user-level Claude settings/credentials/state/backup path priority.
- Test fixtures: `TestHome` must isolate `CLAUDE_CONFIG_DIR`, `CLAUDE_JSON_PATH`, `CCR_SETTINGS_PATH`, and `CCR_BACKUP_DIR`.

### 3. Contracts

- `SettingsManager`, `ClaudePlatform`, `ClaudeAuthService`, and doctor consume
  `ClaudeRuntimePaths`; they must not reimplement environment priority or
  default `.claude` joins. The authoritative contract is in
  `ccr-config/backend/backend-guidelines.md`.
- API-key Claude profiles write only typed, managed env keys into the `env`
  object stored at `ClaudeRuntimePaths::settings_file`; do not add ad hoc env
  writes in command handlers.
- Subscription apply, auth switch, profile off, and lifecycle clear call `clear_ccr_managed_vars()` and remove every key in `CCR_MANAGED_KEYS`, including non-Anthropic runtime keys such as `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `API_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`.
- User-owned keys outside `CCR_MANAGED_KEYS`, including `ANTHROPIC_API_KEY` and `ANTHROPIC_CUSTOM_HEADERS`, are never written or deleted by profile operations. Unknown prefix keys may still affect Claude Code; doctor owns the warning path rather than cleanup guessing ownership.
- `managed_env_entries()` is the single source for lifecycle clear preview, empty-state detection, confirmation count, and removal scope.
- Every new typed env field must be added to both `ProfileConfig` and `ConfigSection`, both conversion directions, an `ccr_types::env_keys` constant, `CCR_MANAGED_KEYS` (and `NON_ANTHROPIC_MANAGED_KEYS` when unprefixed), `ConfigSection::to_managed_env_pairs`, Tauri JSON parse/serialize, UI form state, and provider template mappers when templates can fill it. Registry-to-mapping equality tests must fail if any path drifts.
- Profile apply, auth switching, and doctor must not create or modify
  `ClaudeRuntimePaths::state_file`. In particular, `hasCompletedOnboarding` is
  private Claude Code state; API-key apply leaves an existing state file
  byte-for-byte unchanged and does not create a missing one.
- Tauri Claude MCP user/local mutations are the only CCR-owned state-file
  write surface. They resolve the file through `ClaudeRuntimePaths`, replay
  only the target `mcpServers` subtree on the latest JSON object, and use
  guarded-write CAS for at most three attempts. Top-level fields such as
  `oauthAccount` and `primaryApiKey`, unknown fields, and unrelated project
  entries must round-trip unchanged.
- User/local MCP state writes use `secret: true` with no backup. Project-scope
  `.mcp.json` writes use the same CAS loop with `secret: false` and no backup.
- `ccr doctor` checks API-key profiles for placeholder-looking tokens, active-profile env mismatches, and GLM 1M profiles missing compact-window configuration; it does not diagnose onboarding state.

### 4. Validation & Error Matrix

- Profile token is placeholder-like -> `doctor` warning; do not print or infer a real token.
- Profile expected env differs from `settings.json.env` -> `doctor` warning recommending re-apply.
- GLM model contains `[1m]` and `claude_code_auto_compact_window` is empty -> `doctor` warning recommending `1000000`.
- `.claude.json` missing, unparsable, unreadable, or lacking
  `hasCompletedOnboarding` -> no profile-apply or onboarding-doctor failure;
  the profile/auth path does not own this file.
- An MCP state root is not a JSON object -> return an actionable error and
  leave the file unchanged.
- One or two MCP CAS conflicts -> reread, replay the deterministic subtree
  mutation, and retry. Three conflicts -> return an actionable retry error and
  do not report the operation as successful.
- A mismarked `subscription` profile with API-key shape -> use effective `api_key` in apply, auth switch, and runtime summary; persist the corrected literal before runtime mutation.
- Corrected profile persistence fails -> return an actionable error and do not modify `settings.json`.
- A registered env key is written but not cleared on subscription/off switch -> regression; add it to the registry/mapping equality fixture and switch-cleanup tests.
- An unregistered pair reaches `apply_managed_env` -> ignore it and preserve any existing user-owned value.

### 5. Good/Base/Bad Cases

- Good: typed GLM profile writes `ANTHROPIC_DEFAULT_FABLE_MODEL`, all `*_MODEL_NAME` vars, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `API_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`, then switching to subscription clears them.
- Good: apply/off/auth switch/clear removes all registered keys while preserving `ANTHROPIC_API_KEY`, `ANTHROPIC_CUSTOM_HEADERS`, and unrelated env entries.
- Good: applying a stale API-key-shaped profile persists `auth_mode = "api_key"` before settings mutation; a retry is idempotent.
- Good: a custom `CLAUDE_CONFIG_DIR` makes profile apply, auth switching, and
  doctor observe the same settings, credentials, and state files.
- Good: API-key apply leaves a state file containing `oauthAccount`,
  `primaryApiKey`, and unknown fields byte-for-byte unchanged.
- Good: a Tauri user/local MCP mutation changes only the selected MCP subtree
  while preserving the rest of the latest state-file object.
- Base: API-key apply succeeds without creating a missing resolved state file.
- Bad: `retain(|key, _| !key.starts_with("ANTHROPIC_"))` in a normal mutation path, because it deletes keys CCR does not own.
- Bad: writing `hasCompletedOnboarding` into either `settings.json` or the
  Claude state file.
- Bad: describing the MCP path lock as protection against Claude Code itself;
  non-cooperating external processes do not acquire CCR's lock.
- Bad: checking `CLAUDE_JSON_PATH` or joining `.claude/settings.json` directly
  in a CLI consumer instead of using `ClaudeRuntimePaths`.
- Bad: storing runtime env keys only in `platform_data` or UI-only state, because apply and doctor cannot round-trip them reliably.

### 6. Tests Required

- `cargo test -p ccr-cli platforms::claude -- --test-threads=1`
  - Assert API-key apply writes expected registered env, preserves unregistered Anthropic keys, leaves existing state bytes unchanged, and does not create a missing state file.
  - Assert corrupt `.claude.json` is unchanged and does not block `settings.json` apply.
  - Assert subscription apply clears every registered key and preserves user-owned keys.
  - Assert stale auth mode persists before runtime mutation and write failure leaves settings bytes unchanged.
- `cargo test -p ccr-cli claude_auth -- --test-threads=1`
  - Assert auth switch and runtime summary use effective auth mode and explicit managed override detection.
- `cargo test -p ccr-config -- --test-threads=1`
  - Assert each typed field maps to the correct env key in `to_managed_env_pairs` and stale keys clear on profile switch (combined with `apply_managed_env`).
- `cargo test -p ccr-cli managers::settings -- --test-threads=1`
  - Assert disk-level read→apply→write→read keeps unknown fields and non-managed env intact.
- `cargo test -p ccr --test commands doctor -- --test-threads=1`
  - Assert placeholder, mismatch, and compact-window warnings without an onboarding warning.
- `cargo test -p ccr-desktop --manifest-path ccr-ui/src-tauri/Cargo.toml claude_mcp -- --test-threads=1`
  - Assert user/local add, update, and delete preserve unrelated state, replay a single conflict, fail after three conflicts, and do not silently lose concurrent mutations.
- `cargo test -p ccr --test commands claude_profile -- --test-threads=1`
  - Assert command-level switch/off behavior remains compatible.

### 7. Wrong vs Correct

#### Wrong

```rust
settings.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
```

This deletes user-owned Anthropic variables and still misses CCR-managed non-Anthropic runtime keys.

#### Correct

```rust
let preview = settings.managed_env_entries();
settings.clear_ccr_managed_vars();
assert_eq!(preview.len(), removed_count);
```

This keeps preview, count, and cleanup on the same explicit ownership registry while preserving user keys.

## Scenario: Claude Credential Snapshots And Settings CAS

### 1. Scope / Trigger

- Trigger: changing Claude official-auth save/switch/list behavior or any local `settings.json` mutation in CLI/Tauri.
- Applies to `ClaudeAuthService`, `SettingsManager`, Claude profile apply/off/clear/temp flows, and Tauri Claude agents/hooks/plugins/slash/settings/statusline commands.

### 2. Contracts

- Windows and Linux official credentials use `ClaudeRuntimePaths::credentials_file`. macOS save/switch returns an explicit Keychain-not-supported error before touching a credentials file.
- `.credentials.json`, `auth/<name>.json`, and `auth_registry.toml` write through `write_guarded` with `secret: true` and `BackupPolicy::None`; credential fields are `Secret` with explicit plaintext persistence serializers.
- Before switch overwrites an existing credentials file, its stable in-memory identity token must match a valid registry snapshot. Missing current credentials may switch; corrupt or unmatched credentials must be preserved and return guidance to run `ccr claude auth save`.
- Runtime identity comes from the matching snapshot's `oauth_account`. Only an unmatched login may fall back to the current state-file `oauthAccount`; identity display never rewrites Claude's state file.
- A switch writes target credentials, clears CCR-managed settings through `SettingsManager::update_atomic`, and only then updates the registry. Settings failure restores the previously matched credentials snapshot; combined errors contain categories only, never token content.
- `SettingsManager::{update_atomic,update_atomic_async}` own local managed RMW. They use the guarded-write path lock, `secret: true`, centralized `settings` backups, and at most three deterministic conflict replays.
- `save_atomic` is reserved for validated complete replacement and restore paths. Production mutation flows must not use `load -> save_atomic`.
- Tauri Local mutations call the same `SettingsManager` API. SSH/WSL retain their environment-specific read/write protocol and are outside the local cross-process guarantee.
- `LocalEnvironment::write_config` uses the same secret/central-backup policy when performing a complete local Claude settings replacement; it must not leave `*.bak` beside `settings.json`.

### 3. Validation

- `cargo test -p ccr-cli claude_auth -- --test-threads=1`
- `cargo test -p ccr-cli managers::settings -- --test-threads=1`
- `cargo test -p ccr-desktop --manifest-path ccr-ui/src-tauri/Cargo.toml claude -- --test-threads=1`
- Assert A/B/A snapshot metadata follows credentials even while the state file remains stale.
- Inject a real CAS conflict between independent CLI and local Tauri mutations; both fields and unknown user JSON must survive.
- On Unix, assert auth durable files and settings replacements are owner-only. On Windows, verify inherited user-directory ACL behavior without claiming a Unix mode result.

## Scenario: Retired Platform Command Discovery Boundary

### 1. Scope / Trigger

- Trigger: changing `ccr platform`, root/platform help text, or migration
  behavior for the retired global platform-routing commands.
- Applies to `PlatformAction`, `dispatch_platform`, `help_config`, and command
  integration tests.

### 2. Signatures

- Supported discovery actions: `PlatformAction::{Help,List { json }}`.
- Compatibility-only actions:
  `PlatformAction::{Switch,Current,Info,Init,Profile}`.
- Compatibility errors:
  `legacy_platform_command_error(command: &str) -> CcrError` and the dedicated
  `legacy_platform_init_error() -> CcrError`.
- Current profile entry points: `ccr claude profile ...`,
  `ccr codex profile ...`, and `ccr grok profile ...`.

### 3. Contracts

- `ccr platform --help` and `ccr help platform` expose only `help` and `list`;
  they must not advertise retired actions in the Commands list or examples.
- Compatibility-only variants remain in the Clap tree with hidden help
  metadata. A syntactically valid old invocation must still parse and reach
  `dispatch_platform`, which returns the shared migration error.
- Do not delete the hidden variants or restore writes to legacy
  `current_platform` / `default_platform` state.
- Root and platform help direct status checks to `ccr current`, registry
  discovery to `ccr platform list`, and profile work to the explicit
  Claude/Codex/Grok command trees.

### 4. Validation & Error Matrix

- `ccr platform list [--json]` -> execute the supported registry view.
- `ccr platform --help` or `ccr help platform` -> success with identical,
  migration-safe help.
- Valid retired invocation such as `ccr platform init grok` -> non-zero
  `legacy command retired` error containing all three supported
  `ccr <platform> profile init` replacements.
- Retired action with malformed/missing arguments -> normal Clap argument
  error; do not initialize or mutate platform state.
- Unknown platform subcommand -> normal Clap unknown-subcommand error.

### 5. Good/Base/Bad Cases

- Good: an old script receives an actionable migration error while a new user
  cannot discover the retired action through help.
- Base: `ccr platform list` remains visible and unchanged.
- Bad: removing `PlatformAction::Init` makes old calls fail as unknown commands
  and loses the platform-specific migration guidance.
- Bad: leaving a hidden action in custom after-help examples still recommends
  an operation that always fails.

### 6. Tests Required

- `cargo test -p ccr --test commands -- --test-threads=1`
  - Assert root help contains `ccr current`, `ccr platform list`, and all three
    explicit profile help paths.
  - Assert direct and nested platform help are equal and omit every retired
    action from both Commands and examples.
  - Execute `ccr platform init grok`; assert non-zero status, the legacy error,
    and Grok profile migration guidance.
- `cargo test -p ccr-cli --test dispatch_routing -- --test-threads=1`
  keeps supported platform-list routing green.

### 7. Wrong vs Correct

#### Wrong

```rust
pub enum PlatformAction {
    List { json: bool },
}
```

This removes the compatibility parse surface and downgrades old calls to a
generic unknown-subcommand error.

#### Correct

```rust
pub enum PlatformAction {
    List { json: bool },
    #[command(hide = true)]
    Init { platform_name: String },
}
```

The dispatcher then returns `legacy_platform_init_error()` for init and the
shared compatibility error for the other retired actions.

## Testing

Use crate-local `test_support::TestHome` and `TestHostEnv` for env/path-sensitive command tests. These fixtures serialize process env mutation and restore variables on Drop.

Command integration behavior also has tests under `crates/ccr/tests/commands/`; update those when command output or compatibility surfaces change.

## Verification

For CLI command changes, run:

- `just fmt-check`
- `cargo test -p ccr-cli -- --test-threads=1`
- Relevant `cargo test -p ccr --test commands -- --test-threads=1`
- `just lint-strict`
