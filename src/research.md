# CCR Source Code Deep Research Report

**Date:** 2026-02-23
**Scope:** `src/` directory — 171 Rust source files
**Method:** Parallel 7-stage deep-read analysis

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Entry Points & Core Infrastructure](#3-entry-points--core-infrastructure)
4. [CLI Definitions & Command Implementations](#4-cli-definitions--command-implementations)
5. [Service Layer & Manager Layer](#5-service-layer--manager-layer)
6. [Platform Abstraction Layer](#6-platform-abstraction-layer)
7. [Sessions, Storage & Sync Layers](#7-sessions-storage--sync-layers)
8. [Web API & TUI Layers](#8-web-api--tui-layers)
9. [Models, Utils & Cross-Cutting Concerns](#9-models-utils--cross-cutting-concerns)
10. [Limitations & Technical Debt](#10-limitations--technical-debt)
11. [Recommendations](#11-recommendations)

---

## 1. Executive Summary

CCR (Claude Code Configuration Switcher) is a **171-file Rust CLI application** providing unified configuration management for 6 AI CLI platforms (Claude, Codex, Gemini, Qwen, IFlow, Droid). It follows a strict **4-layer architecture** (CLI → Services → Managers → Core) with atomic file operations, cross-process file locking, and a full audit trail.

Key metrics:
- **38 REST API endpoints** (Axum, port 19527)
- **30+ CLI commands** with nested subcommands
- **6 platform implementations** (4 active, 2 stubs)
- **3 SQLite tables** (sessions, search_history, history)
- **2 TUI applications** (profile switcher + Codex auth manager)
- **WebDAV cloud sync** with multi-folder support

---

## 2. Architecture Overview

### Layered Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   CLI / Web / TUI Layer                  │
│  main.rs → cli/ (Clap) → commands/ (30+ implementations)│
│  web/ (Axum, 38 endpoints)  │  tui/ (Ratatui, 2 apps)  │
├─────────────────────────────────────────────────────────┤
│                     Service Layer                        │
│  config_service  │ settings_service │ history_service    │
│  backup_service  │ validate_service │ sync_service       │
│  ui_service │ health_check │ codex_auth/usage_service    │
├─────────────────────────────────────────────────────────┤
│                     Manager Layer                        │
│  ConfigManager │ SettingsManager │ HistoryManager        │
│  CostTracker │ BudgetManager │ PricingManager            │
│  PlatformConfigManager │ TempOverrideManager             │
├─────────────────────────────────────────────────────────┤
│                  Platform Layer                          │
│  claude │ codex │ gemini │ droid │ qwen(stub) │ iflow(stub)│
├─────────────────────────────────────────────────────────┤
│                     Core Layer                           │
│  error │ lock │ atomic_writer │ logging │ cache │ fileio │
│  file_manager │ http                                     │
├─────────────────────────────────────────────────────────┤
│              Sessions / Storage / Sync                   │
│  parser │ indexer │ database(SQLite) │ session_store     │
│  sync/service(WebDAV) │ folder_manager │ content_selector│
└─────────────────────────────────────────────────────────┘
```

### File System Layout (Runtime)

```
~/.ccr/                              # CCR root (overridable via CCR_ROOT)
├── config.toml                      # Unified platform registry
├── data.db                          # SQLite database
├── sync.toml                        # Legacy single-folder sync config
├── sync_folders.toml                # Multi-folder sync config
├── logs/ccr.log                     # Rotating daily log (14-day retention)
├── platforms/{name}/
│   ├── profiles.toml                # Profile store per platform
│   └── settings.json                # CCR-side settings
├── history/{name}.json              # Audit trail per platform
└── backups/{name}/                  # Auto-backups per platform
```

### Feature Flags

| Feature | Default | Gates |
|---------|---------|-------|
| `tui` | ✅ | TUI app, crossterm, ratatui |
| `web` | ✅ | Web API, Axum, sync commands, stats/budget/pricing |

---

## 3. Entry Points & Core Infrastructure

### 3.1 `main.rs` — CLI Entry Point

Minimal async entry (`#[tokio::main]`). Parses CLI via Clap, initializes logging (TUI-aware: `init_file_only_logger()` for TUI mode to avoid corrupting the terminal, `init_logger()` otherwise), then delegates to `CommandDispatcher::dispatch()`. Errors are mapped to exit codes via `handle_error()`.

### 3.2 `lib.rs` — Library Facade

Re-exports the entire public API surface, making CCR usable as both a binary and a Rust library. Feature-gated modules: `tui`, `web`.

### 3.3 `help.rs` — Custom Help Renderer

Replaces Clap's default help with colorized, table-formatted, terminal-width-responsive output. `term_width()` uses `crossterm::terminal::size()` (feature-gated) or `$COLUMNS` env var (fallback 80). Wide terminals (≥100 cols) show 3-column tables with examples; narrow shows 2-column.

### 3.4 Core Modules

#### `core/error.rs` — Error Taxonomy

`CcrError` enum with **24 variants** using `thiserror`. Structured exit codes (10–99 by category). Key methods:
- `exit_code()` → maps variant to named constant
- `is_fatal()` → true for `ConfigMissing`, `SettingsMissing`, `IoError`
- `user_message()` → human-readable with actionable suggestions (e.g., "run `ccr list`")

Auto-conversions via `#[from]`: `serde_json::Error`, `toml::de::Error`, `std::io::Error`.

#### `core/lock.rs` — Dual-Layer Concurrency Protection

Two mechanisms:
1. **In-process**: `CONFIG_LOCK: LazyLock<Mutex<()>>` — serializes config operations within a single process
2. **Cross-process**: `FileLock` — RAII exclusive file lock via `fs4::FileExt::try_lock_exclusive()` with exponential backoff (50ms × min(2^retry, 8), max 400ms). `Drop` releases the lock automatically.

`LockManager` factory: default lock dir `~/.claude/.locks/` (overridable via `$CCR_LOCK_DIR`). Named locks: `lock_settings()`, `lock_history()`, `lock_resource(name)`.

#### `core/atomic_writer.rs` — Atomic File Writes

Two variants:
- **Sync** (`AtomicWriter`): `NamedTempFile::new_in(same_dir)` → write → `persist()` (atomic rename)
- **Async** (`AsyncAtomicWriter`): UUID-named temp file → `tokio::fs::write` → `tokio::fs::rename`, with cleanup on failure

Same-directory temp file guarantees single-filesystem atomic rename.

#### `core/logging.rs` — Dual-Output Logging

Bridges `log` crate to `tracing`. Two layers: stdout (ANSI color) + daily rotating file (`~/.ccr/logs/ccr.log`, 14-day retention). `ColorOutput` provides static methods: `success`, `info`, `warning`, `error`, `step`, `title`, `banner`, `separator`, `key_value`, `mask_sensitive`, `ask_confirmation`.

#### `core/cache.rs` — Generic TTL Cache

`ConfigCache<T>` with `RwLock` thread safety. Default TTL: 30s. `get_or_load(loader)` checks TTL before calling loader. Poisoning recovery via `unwrap_or_else(|p| p.into_inner())`. Note: TOCTOU window between read-lock release and write-lock acquisition.

#### `core/fileio.rs` — Generic I/O Layer

`read_toml`/`write_toml`/`read_json`/`write_json` in sync and async variants. All generic over `T: Serialize + Deserialize`. Auto-creates parent directories. Note: `write_toml`/`write_json` use `fs::write` directly (not `AtomicWriter`).

#### `core/http.rs` — Global HTTP Client

`HTTP_CLIENT: LazyLock<reqwest::Client>` — 30s timeout, 5 idle connections per host, versioned user-agent.

#### `core/file_manager.rs` — Manager Trait

```rust
pub trait FileManager<T> {
    fn load(&self) -> Result<T>;
    fn save(&self, data: &T) -> Result<()>;
    fn path(&self) -> &Path;
}
```

---

## 4. CLI Definitions & Command Implementations

### 4.1 CLI Structure (`cli/`)

- `definitions.rs` — Top-level `Cli` struct (Clap `Parser`) and `Commands` enum (Clap `Subcommand`). Custom help rendering (`disable_help_flag = true`). Global `--yes/-y` flag. Positional `config_name` enables shortcut `ccr <name>` = `ccr switch <name>`.
- `dispatch.rs` — `CommandDispatcher::dispatch_async()` matches on `Commands` enum. Sub-dispatchers for nested groups: `dispatch_ui`, `dispatch_sync`, `dispatch_codex`, `dispatch_platform`, `dispatch_stats`, `dispatch_budget`, `dispatch_pricing`, etc.
- `subcommands/` — Nested action enums: `CheckAction`, `CodexAction`, `CodexAuthAction`, `PlatformAction`, `SyncAction`, `FolderAction`, `AllSyncAction`, `UiAction`, `TempTokenAction`.

### 4.2 Complete Command Tree

```
ccr [--yes/-y] [COMMAND | <config_name>]
├── (no args)                    → TUI or current_command
├── <config_name>                → switch (shortcut)
├── help [subcmd]                → custom rich help
├── list (ls)                    → list profiles
├── current (status, show)       → show active profile
├── switch <name>                → apply profile
├── add                          → interactive profile creation
├── delete <name> [--force]      → remove profile
├── enable <name>                → enable profile
├── disable <name> [--force]     → disable profile
├── validate                     → validate all profiles
├── history [--limit N] [--filter-type T]
├── init [--force]               → initialize CCR
├── export [--output P] [--no-secrets]
├── import <input> [--merge] [--backup] [--force]
├── clean [--days N] [--dry-run] [--force]
├── clear [--force]              → clear settings
├── optimize                     → sort profiles
├── version (ver)
├── update [branch] [--check]
├── temp                         → interactive temp config
├── temp-token {set|show|clear}
├── platform {list|switch|current|info|init}
├── check {conflicts}
├── codex                        → TUI or auth subcommands
│   └── auth {save|list|switch|delete|current|export|import}
├── web [--host] [--port] [--no-browser]
├── ui [--port] [--backend-port] | ui update
├── sync {config|status|push|pull}
│   ├── folder {list|add|remove|info|enable|disable}
│   ├── all {push|pull|status}
│   └── <folder_name> {push|pull|status}
├── stats {summary|import|export|clear|cost}
├── budget {status|set|reset}
├── pricing {list|set|remove|reset}
├── skills                       → skill management
├── prompts                      → prompt management
├── sessions                     → session management
└── provider                     → health check
```

### 4.3 Command Categories

| Category | Module | Commands |
|----------|--------|----------|
| Profile CRUD | `commands/profile/` | add, delete, list, current, switch, enable, disable |
| Lifecycle | `commands/lifecycle/` | init, clean, clear, validate, optimize |
| Data I/O | `commands/data/` | export, import, history, stats, budget, pricing |
| Platform | `commands/platform/` | list, switch, current, info, init |
| Codex Auth | `commands/codex/auth/` | save, list, switch, delete, current, export, import |
| Common | `commands/common/` | mode detection, prompt helpers, table builders |

### 4.4 Key Dispatch Patterns

- **Confirmation flow**: Destructive commands check `force || settings.skip_confirmation`. Global `--yes/-y` is OR'd with per-command `--force`.
- **Async blocking**: Interactive stdin reads use `tokio::task::spawn_blocking()`.
- **Platform abstraction**: `switch_command` calls `create_platform(platform)` returning `dyn PlatformConfig` trait object.
- **Output formatting**: All commands use `ColorOutput` for colored terminal output. Tables use `comfy_table` with `UTF8_FULL` preset. `--json` flag available on platform commands.

---

## 5. Service Layer & Manager Layer

### 5.1 Service Layer (12 files)

Services orchestrate managers and enforce business rules. All destructive operations follow **backup-before-mutate**.

| Service | Responsibility |
|---------|---------------|
| `ConfigService` | Primary orchestrator: switch_config coordinates ConfigManager + SettingsManager + HistoryManager in validate → backup → mutate → audit sequence |
| `SettingsService` | Settings.json read/write with atomic saves |
| `HistoryService` | Audit log queries (recent, filtered, async wrappers) |
| `BackupService` | Backup creation and rotation (max 10 per platform) |
| `ValidateService` | Profile validation across all platforms |
| `SyncService` | WebDAV sync orchestration |
| `UIService` | Launches ccr-ui desktop app (Tauri) |
| `HealthCheckService` | Provider API health checks with latency measurement |
| `CodexAuthService` | Codex multi-account lifecycle: JWT email extraction, token freshness (Fresh/Stale/Old), import/export with Merge/Replace modes |
| `CodexUsageService` | Codex API usage statistics (5h/7d rolling windows) |
| `MultiBackupService` | Cross-platform incremental backups using blake3 content hashing + rayon parallel digest |

**Key pattern**: Every `*_async()` method is a thin wrapper calling its sync counterpart. Blocking I/O runs directly on the async executor (no `spawn_blocking` in services).

### 5.2 Manager Layer

Managers handle data access and persistence. Each manager owns a file path and provides CRUD operations.

| Manager | File | Format | Key Details |
|---------|------|--------|-------------|
| `ConfigManager` | `platforms/{name}/profiles.toml` | TOML | Atomic writes via `AtomicWriter`. Supports both `CcsConfig` (full) and bare `IndexMap` (legacy) formats |
| `PlatformConfigManager` | `~/.ccr/config.toml` | TOML | Unified registry: tracks current platform, registered platforms, last_used timestamps |
| `SettingsManager` | `~/.claude/settings.json` | JSON | Atomic save via `save_atomic()`. Merges profile fields into existing settings |
| `HistoryManager` | `history/{name}.json` | JSON | Append-only audit entries with UUID, timestamp, actor, operation, env_changes |
| `CostTracker` | `~/.ccr/costs/YYYYMM.json` | JSON | Monthly cost files. Tracks per-session: model, tokens (input/output/cache), cost in USD |
| `BudgetManager` | `~/.ccr/budget.toml` | TOML | Daily/weekly/monthly limits. `LimitAction`: Warn, Log, None. **Uses plain `fs::write` (not atomic)** |
| `PricingManager` | `~/.ccr/pricing.toml` | TOML | Per-model pricing (input/output/cache_read/cache_write per million tokens) |
| `TempOverrideManager` | `~/.ccr/temp_override.json` | JSON | Temporary token/model/base_url overrides |
| `BuiltinPrompts` | embedded | — | Hardcoded prompt templates for Claude system prompts |

### 5.3 Atomic Write Pattern (Universal)

```
NamedTempFile::new_in(same_directory)
  → write content to temp file
  → temp_file.persist(target_path)   // atomic rename
```

Same-directory temp file guarantees single-filesystem atomic rename. Used by ConfigManager, SettingsManager, HistoryManager, and sync FolderManager.

---

## 6. Platform Abstraction Layer

### 6.1 `PlatformConfig` Trait

The core abstraction enabling multi-platform support:

```rust
pub trait PlatformConfig: Send + Sync {
    fn platform_name(&self) -> &str;
    fn platform_type(&self) -> Platform;
    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>>;
    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()>;
    fn delete_profile(&self, name: &str) -> Result<()>;
    fn get_settings_path(&self) -> PathBuf;
    fn apply_profile(&self, name: &str) -> Result<()>;
    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()>;
    fn get_current_profile(&self) -> Result<Option<String>>;
    fn list_profile_names(&self) -> Result<Vec<String>> { /* default */ }
    fn get_env_var_names(&self) -> Vec<&'static str> { vec![] }
}
```

Factory: `create_platform(Platform) -> Result<Arc<dyn PlatformConfig>>`.

### 6.2 Platform Implementations

| Platform | Status | Native Config Target | Token Validation | Env Vars |
|----------|--------|---------------------|-----------------|----------|
| **Claude** | Full | `~/.claude/settings.json` | Via `ConfigSection::validate()` | `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_MODEL`, `ANTHROPIC_SMALL_FAST_MODEL` |
| **Codex** | Full | `~/.codex/config.toml` + `auth.json` | GitHub token prefix | `OPENAI_API_KEY` |
| **Gemini** | Full | `~/.ccr/platforms/gemini/settings.json` | `AIza` prefix + length ≥ 30 | `GEMINI_API_KEY` |
| **Droid** | Full | `~/.factory/settings.json` | Non-empty only | None |
| **Qwen** | Stub | N/A | N/A | N/A |
| **IFlow** | Stub | N/A | N/A | N/A |

### 6.3 Codex Three-Mode Apply

Codex has the most complex `apply_profile` with three branches:

| Mode | Condition | Action |
|------|-----------|--------|
| Official | `base_url` is None/empty | Remove `model_provider`/`model_providers` from config.toml |
| GitHub Copilot | `api_mode == "github"` or URL contains `github.com` | Write `~/.codex/settings.json` with `GitHubConfig` |
| Custom API | Has `base_url` + `wire_api` | Write `config.toml` with `[model_providers.{id}]` section + `auth.json` |

### 6.4 Profile Switching Flow (All Platforms)

```
1. load_profiles() from ~/.ccr/platforms/{name}/profiles.toml
2. validate_profile() on target profile
3. Write to platform-native config file (platform-specific)
4. Dual registry update:
   a. update_current_config() → profiles.toml current_config field
   b. update_registry_current_profile() → ~/.ccr/config.toml unified registry
```

### 6.5 Shared Base Module (`platforms/base.rs`)

Eliminates ~150 lines of duplication. Provides: `section_to_profile`, `profile_to_section`, `load_profiles_from_toml` (supports both `CcsConfig` and bare `IndexMap` formats), `save_profiles_to_toml` (3-step `current_config` resolution), `update_current_config`, `update_registry_current_profile`.

---

## 7. Sessions, Storage & Sync Layers

### 7.1 Session Models

- `Session` — Full record: id, platform, title (first 50 chars of first user message), cwd, file_path, file_hash (blake3), timestamps, message counts (total/user/assistant/tool_use)
- `SessionEvent` — One JSONL line: event_type, role, message, timestamp, tool_name, session_id, cwd, raw_json
- `SessionFilter` — Query params: platform, date range, cwd_prefix, limit/offset, today_only
- `SessionSummary` — List view subset

### 7.2 JSONL Parser (`sessions/parser.rs`)

Stateless parser using **rayon** for parallel file scanning and parsing.

| Platform | Session Dir | Extensions | ID Source |
|----------|------------|------------|-----------|
| Claude | `~/.claude/projects` | `.jsonl` | Event field → filename stem → UUID |
| Codex | `~/.codex/sessions` | `.jsonl` | Event field → filename stem → UUID |
| Gemini | `~/.gemini/tmp` | `.jsonl`, `.json` | Filename stem only |
| Qwen | `~/.qwen/sessions` | `.jsonl` | Filename stem only |
| IFlow | `~/.iflow/sessions` | `.jsonl` | Filename stem only |
| Droid | `~/.factory/sessions` | `.jsonl` | Filename stem only |

Message classification: `is_user_message()` checks `role == "user"` OR `event_type in ["user", "human"]`. `is_tool_use()` checks `event_type in ["tool_use", "tool_call"]` OR `tool_name.is_some()`. Timestamps: collect all RFC3339 from events, use first/last; fallback to filesystem mtime/ctime.

### 7.3 Session Indexer (`sessions/indexer.rs`)

Incremental index pipeline:
1. Scan platform session dirs recursively (rayon parallel)
2. Compute blake3 hash per file (rayon parallel)
3. Compare against stored hash via `SessionStore::get_file_hash` — skip unchanged
4. Parse changed files (rayon parallel)
5. Upsert into SQLite via `SessionStore::upsert_sessions`

Operations: `index_all`, `index_platform`, `list`, `search`, `get`, `prune_stale`, `stats`, `rebuild`.

### 7.4 SQLite Storage (`storage/`)

**Pool**: r2d2 + r2d2_sqlite, max_size=10, min_idle=1. PRAGMA: `journal_mode=WAL`, `synchronous=NORMAL`.

**Migration system**: `migrations` table tracks applied migrations by name. Three migrations:

| Migration | Table | Key Columns |
|-----------|-------|-------------|
| 001 | `sessions` | id (PK), platform, title, cwd, file_path (UNIQUE), file_hash, timestamps, message counts. Indexes: platform, created_at, updated_at, cwd |
| 002 | `search_history` | id (auto), query, scope, result_count, searched_at |
| 003 | `history` | id (PK), timestamp, actor, operation, result_message, from/to_config, backup_path, extra, notes, env_changes |

**Upsert**: `INSERT ... ON CONFLICT(file_path) DO UPDATE` — updates title, hash, counts, indexed_at; preserves id, platform, cwd, created_at.

### 7.5 WebDAV Sync (`sync/`)

**Architecture**: Legacy single-folder (`sync.toml`) + multi-folder (`sync_folders.toml`) with migration support.

**Multi-folder config** (`SyncFoldersConfig`):
```toml
version = "1.0"
[webdav]
url = "https://dav.jianguoyun.com/dav/"
username = "user@example.com"
password = "app_password"
base_remote_path = "/ccr-sync"

[[folders]]
name = "claude"
local_path = "~/.claude"
remote_path = "/ccr-sync/claude"
enabled = true
exclude_patterns = ["*.log", ".locks/", "cache/"]
```

**Sync service** (`sync/service.rs`): HTTP Basic auth. Push: recursive directory upload with `MKCOL` for directories. Pull: `LIST Depth(1)` + recursive download. `ensure_remote_directory`: recursive parent creation on 409 Conflict.

**Exclusion rules**: `.tmp`, `.lock`, `.bak`, `.DS_Store`, `.git`, `.locks`, `backups`, `history`, `ccr-ui`, hidden files (except `.ccs_config.toml` and `*.toml`).

**Conflict resolution**: Last-write-wins. No automatic merge. Mandatory local backup before pull. Push skips if remote exists (unless `--force`).

**Concurrency safety**: `FolderManager::save_config` acquires `LockManager` file lock (5s timeout) before writing.

---

## 8. Web API & TUI Layers

### 8.1 Web API (Axum)

**Server**: Default port 19527, binds `0.0.0.0` (fallback `127.0.0.1` on Windows). Port auto-increments up to 10 times on conflict. Static assets embedded via `include_str!`. Only middleware: `CorsLayer::permissive()` — **no authentication**.

**Shared state** (`AppState`): `Arc<ConfigService>`, `Arc<SettingsService>`, `Arc<HistoryService>`, `Arc<BackupService>`, `Arc<ValidateService>`, `Arc<SystemInfoCache>` (2s background refresh), `Arc<RwLock<CcsConfig>>` (reloadable cache).

**Response envelope**: `{ "success": bool, "data": T | null, "message": string | null }` — used by 36 of 38 endpoints. Two exceptions: `enable/disable` return raw JSON.

**All 38 endpoints**:

| Group | Method | Path | Handler |
|-------|--------|------|---------|
| Config (9) | GET | `/api/configs` | List (tokens masked) |
| | GET | `/api/config/{name}` | Get (unmasked for edit UI) |
| | POST | `/api/config` | Add |
| | PUT | `/api/config/{name}` | Update (rename supported) |
| | DELETE | `/api/config/{name}` | Delete |
| | PATCH | `/api/config/{name}/enable` | Enable |
| | PATCH | `/api/config/{name}/disable` | Disable |
| | POST | `/api/export` | Export TOML |
| | POST | `/api/import` | Import (merge/replace) |
| Codex (4) | GET | `/api/codex/profiles` | List with `is_current` |
| | POST | `/api/codex/profiles` | Add |
| | PUT | `/api/codex/profiles/{name}` | Update |
| | DELETE | `/api/codex/profiles/{name}` | Delete |
| System (8) | GET | `/api/history` | Last 50 entries |
| | POST | `/api/validate` | Validate all |
| | POST | `/api/clean` | Clean backups |
| | GET | `/api/settings` | Raw settings |
| | GET | `/api/settings/backups` | Backup list |
| | POST | `/api/settings/restore` | Restore backup |
| | GET | `/api/system` | System info (cached) |
| | POST | `/api/reload` | Reload config cache |
| Stats (12) | GET | `/api/stats/provider-usage` | Provider counts |
| | GET | `/api/stats/cost/summary` | Cost summary |
| | GET | `/api/stats/cost/details` | Monthly details |
| | GET | `/api/stats/cost/export` | CSV export |
| | GET | `/api/stats/cost/by-model` | Per-model usage |
| | GET | `/api/budget/status` | Budget status |
| | POST | `/api/budget/set` | Set limits |
| | POST | `/api/budget/reset` | Reset limits |
| | GET | `/api/pricing/list` | List pricing |
| | POST | `/api/pricing/set` | Set model price |
| | DELETE | `/api/pricing/remove/{model}` | Remove pricing |
| | POST | `/api/pricing/reset` | Reset to defaults |
| Platform (2) | GET | `/api/platforms` | Platform info |
| | POST | `/api/platforms/switch` | Switch platform |
| Sync (4) | GET | `/api/sync/status` | Live WebDAV check |
| | POST | `/api/sync/config` | Save sync config |
| | POST | `/api/sync/push` | Upload |
| | POST | `/api/sync/pull` | Download |

### 8.2 TUI — Main App (Profile Switcher)

**Runtime**: Trait-based (`TuiApp`) with RAII `TerminalGuard` (raw mode + alternate screen + mouse capture). Event loop: 250ms tick rate. Windows: filters `KeyEventKind::Release` to prevent double-firing.

**Layout** (responsive):
- Normal (height ≥ 20): Header (3 rows, tabs) + Content (profile list) + Footer (5 rows, shortcuts + toast)
- Compact (height < 20): Header + Content + Toast only (2 rows)
- Width < 60: name column only; ≥ 60: name (30%) + description

**Two tabs**: Claude (profile switcher) and Codex (embedded `CodexAuthApp`).

**Keyboard**: `q`/`Esc` quit, `Tab` switch tab, `↑↓`/`jk` navigate, `←→`/`hl` paginate, `Enter` apply+quit, `Space` apply+stay, `r` reload. Mouse: click tabs, click list items, scroll.

**Pagination**: 20 profiles per page.

### 8.3 TUI — Codex Auth App

Manages Codex multi-account switching with modal overlays.

**Modes**: Normal and Overlay (Confirm delete / Input save name).

**Keyboard**: `↑↓`/`jk` navigate, `Enter` switch account, `s` save overlay (if unsaved login), `d`/`Delete` delete overlay, `r` reload. Overlay: `y`/`n` confirm, text input (max 32 chars, alphanumeric/`_`/`-`).

**Switch guards** (4 checks): virtual accounts blocked, already-current shows info, expired blocked, running Codex processes warned.

**Pagination**: 10 accounts per page.

### 8.4 Theme & Toast

**Theme**: Platform-aware accent colors — Claude (#f59e0b amber), Codex (#6366f1 indigo), Gemini (#4285f4 blue), Droid (#10b981 emerald). 3 foreground levels, 3 semantic colors.

**Toast**: Queue-based with TTLs — Success/Info: 3s, Warning: 4s, Error: 5s. Garbage-collected every 250ms tick.

---

## 9. Models, Utils & Cross-Cutting Concerns

### 9.1 Models (`models/`)

- `Platform` enum: 6 variants (Claude, Codex, Gemini, Qwen, IFlow, Droid). `FromStr` is case-insensitive; "factory" aliases "droid". `all()` returns 6, `implemented()` returns 4.
- `ProfileConfig`: Universal profile with `#[serde(flatten)]` on `platform_data: IndexMap<String, Value>` for forward-compatible platform extensions. All named fields are `Option<T>`. Implements `AutoCompletable` for schema migration.
- `PlatformPaths`: Resolves CCR root via `$CCR_ROOT` → `~/.ccr/`. Per-platform: `platforms/{name}/profiles.toml`, `settings.json`, `history/{name}.json`, `backups/{name}/`.
- `ModelPricing`: Calculates cost (input/output/cache_write/cache_read) in USD per million tokens.
- `BudgetConfig`: Validation: `warn_at_percent ≤ 100`, limits ≥ 0.0. `LimitAction`: Warn, Log, None.
- `PricingConfig`: Validates non-empty version, non-empty model names, prices ≥ 0.0. `merge()` overwrites entries.

### 9.2 Utils (`utils/`)

- `mask.rs`: `mask_sensitive(value)` — len ≤ 10 → all `*`; len > 10 → first 4 + `...` + last 4. `mask_if_sensitive(var_name, value)` — applies only if var_name contains "TOKEN", "KEY", or "SECRET" (case-sensitive).
- `validation.rs`: `Validatable` trait — `fn validate(&self) -> Result<()>`. `AutoCompletable` trait — fills missing optional fields with defaults, returns `true` if modified (schema migration mechanism).
- `auto_complete.rs`: `AutoCompletable` trait definition and contract.
- `toml_json.rs`: Bidirectional TOML↔JSON conversion on `IndexMap<String, Value>`. JSON `null` dropped (TOML has no null). Non-finite floats become JSON `null`.

### 9.3 Remaining Commands

- `skills_cmd.rs`: Install searches repositories sequentially, fetches `SKILL.md` with fallback to `README.md`.
- `prompts_cmd.rs`: `add` supports `@file` syntax. `apply` creates `.backup` before overwriting.
- `temp_cmd.rs`: Standalone interactive config. `smart_parse_model()` maps aliases: `sonnet`→`claude-sonnet-4-*`, `opus`→`claude-opus-4-*`, `haiku`→`claude-3-5-haiku-*`, `gpt4`→`gpt-4o`, `gemini`→`gemini-2.0-flash`.
- `update.rs`: Self-update via `cargo install --git`. `--check` prints command without executing.
- `sessions_cmd.rs`: 7 subcommands. `list` triggers full incremental index scan on every call.
- `provider_cmd.rs`: Health check with latency. `verify` validates API keys.

---

## 10. Limitations & Technical Debt

### Critical

1. **No Web API authentication** — All 38 endpoints accessible to anyone on the network. Default binding `0.0.0.0:19527`.
2. **`temp_token` set/show incoherence** — `set` writes to `settings.json`, `show` reads from `TempOverrideManager` file. They operate on different storage locations.
3. **BudgetManager non-atomic writes** — Uses plain `fs::write`, not `AtomicWriter`. Crash mid-write could corrupt `budget.toml`.

### Moderate

4. **Two divergent pricing tables** — `ModelPricing::default_pricing()` and `PricingConfig::with_claude_defaults()` hardcode different values for the same models.
5. **Async methods block executor** — Service `*_async()` methods are thin sync wrappers with no `spawn_blocking`. Web API handlers use `spawn_blocking_string()` but services don't.
6. **Two validation patterns** — `Validatable` trait (returns `CcrError`) vs standalone `validate() -> Result<(), String>`. Not unified.
7. **Masking threshold inconsistency** — `utils/mask.rs` uses threshold 10; `codex/auth/current.rs` has local duplicate with threshold 8.
8. **SyncConfig password obfuscation** — XOR + base64, not cryptographic security.
9. **`sessions list` full re-index** — Calls `indexer.index_all()` on every invocation, discarding the result.

### Minor

10. **`fileio::write_toml/write_json` not atomic** — Callers must use `AtomicWriter` explicitly.
11. **`ConfigCache::get_or_load` TOCTOU** — Read lock released before write lock acquired; loader may run multiple times under concurrency.
12. **Web API response inconsistency** — `enable/disable` endpoints return different JSON shape than other endpoints. `provider-usage` returns raw HashMap with no envelope.
13. **Dead code** — `routes.rs` `Route` enum is `#[allow(dead_code)]`. `PlatformStatus::Configured/Available` unused. `CodexAuthApp::refresh_usage()` never called.
14. **Hardcoded model IDs** — `smart_parse_model()` aliases will become stale as new model versions release.
15. **`sessions_cmd::cmd_resume`** — Non-dry-run mode prints resume command but does not execute it (not yet implemented).

---

## 11. Recommendations

### Security

1. Add authentication middleware to the Web API (at minimum, a bearer token or localhost-only binding).
2. Replace XOR password obfuscation with OS keychain integration or proper encryption.

### Consistency

3. Unify the two pricing tables into a single source of truth.
4. Unify validation patterns: make `BudgetConfig` and `PricingConfig` implement the `Validatable` trait.
5. Consolidate masking functions: remove the local duplicate in `codex/auth/current.rs`.
6. Standardize Web API response envelope across all 38 endpoints.

### Reliability

7. Use `AtomicWriter` in `BudgetManager::save()` to prevent corruption.
8. Fix `temp_token` set/show to use the same storage location.
9. Add `spawn_blocking` to service async methods for proper non-blocking behavior.
10. Cache session index results to avoid full re-scan on every `sessions list`.

### Performance

11. Consider lazy-loading the `CodexAuthApp` (currently eagerly initialized even when Claude tab is active).
12. Use the existing `PLATFORM_MODE` static in `handle_list_configs` instead of re-detecting on every request.

---

*Report generated by parallel 7-stage SCIOMC research workflow. 171 files analyzed across 7 scientist agents.*
