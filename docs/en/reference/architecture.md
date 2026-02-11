# Architecture (v3.20.11)

CCR is a Rust 2024 workspace with a strict layered design and dual config modes (Unified by default, Legacy compatible). It ships CLI/TUI/legacy Web API and powers the full CCR UI app.

## Workspace & Modes

```
ccr/                  # workspace root
├─ Cargo.toml         # workspace deps (clap/serde/tokio/axum/...)
├─ src/               # core CLI + library
├─ ccr-ui/
│  ├─ backend/        # Axum REST server, reuses ccr crate (default-features = false)
│  └─ frontend/       # Vue3 + Vite + Pinia + Tailwind + Tauri
├─ docs/              # VitePress docs (zh/en)
└─ tests/             # integration tests
```

- **Unified mode (default)**: `~/.ccr/config.toml` + `platforms/<name>/profiles.toml`, history/backups per platform.
- **Legacy mode**: `~/.ccs_config.toml` (CCS compatible).
- **Settings target**: writes directly to `~/.claude/settings.json` (atomic + locked).

## Layered Architecture

```
CLI / Web API / TUI
        │
        ▼
   Services (orchestration)
        │
        ▼
  Managers (data access)
        │
        ▼
 Core & Utils (infrastructure)
```

- **CLI (`src/commands/`)**: domain submodules `platform/`, `profile/`, `lifecycle/`, `data/`, `common/` plus `sync_cmd`, `ui`, `skills_cmd`, `prompts_cmd`, `check_cmd`, `update`. Clap routing lives in `main.rs`, supports shortcut `ccr <profile>`.
- **Services (`src/services/`)**: `ConfigService`, `SettingsService`, `HistoryService`, `BackupService` & `MultiBackupService`, `SyncService` (WebDAV), `UiService` (CCR UI bootstrap).
- **Managers (`src/managers/`)**: `ConfigManager` (Legacy), `PlatformConfigManager` (Unified registry), `SyncConfigManager`/`SyncFolderManager`, `TempOverrideManager`, `SettingsManager`, `HistoryManager`, `CostTracker`, `PromptsManager`, `SkillsManager`, `ConflictChecker`.
- **Core (`src/core/`)**: unified `CcrError`, file locks + in-process mutex, `atomic_writer`, `fileio`, `file_manager`, `logging` (tracing + colored output).
- **Platforms/Models**: `Platform`/`PlatformPaths`/`ProfileConfig` traits and impls for Claude, Codex, Gemini (Qwen/iFlow stubs), plus `PlatformRegistry`/`PlatformDetector`.
- **Sync (`src/sync/`)**: `SyncService` on `reqwest_dav` with recursion, filtering, allow-list, and remote directory guards; `content_selector` for interactive choices; `commands` for folder/all/dynamic subcommands.
- **Interfaces**: `web/` (Axum legacy API, cached system info, JSON/error helpers); `tui/` (Ratatui views/themes, feature gated).

### Dependency Direction

Interfaces → Services → Managers → Core/Utils. Models/Platforms/Utils are shared; `ccr-ui/backend` consumes the `ccr` crate directly to reuse service/manager logic.

## Key Flows

- **Profile switch (Unified)**: CLI → `ConfigService` reads `config.toml` & `profiles.toml` → `SettingsService` locks + backups + atomic write to `settings.json` → `HistoryService` records masked diffs; optional `TempOverrideManager` applies ephemeral token/base_url/model.
- **Platform management**: `platform list/current/info/init/switch` operate on `config.toml` via `PlatformConfigManager`; platform impls satisfy `PlatformConfig`.
- **WebDAV sync**: `sync config` stores connection; `sync folder ...` registers/enables directories; `sync push/pull/all` uses `SyncService` recursion with filters (`backups/`, `history/`, `.locks/`, `ccr-ui/` etc.), `--force`, interactive allow-list, single folder or batch.
- **CCR UI launch**: `UiService` probes `./ccr-ui` → `~/.ccr/ccr-ui` → GitHub download (prompted) before starting frontend/backend; ports override via `-p/--backend-port`.

## Reliability & Performance

- File locks + in-process mutex; atomic writes.
- Automatic backups before destructive operations; `MultiBackupService` cleans per-platform.
- Tracing logger with `CCR_LOG_LEVEL`; colored console + daily logs under `~/.ccr/logs/`.
- Rayon parallel validation; shared `fileio`; dev profile `opt-level=1` (deps at `opt-level=2`); Axum caches system info.

## Testing & Contribution

- Unit tests for platforms/managers/locks; integration tests under `tests/` use temp dirs.
- No `panic!` in production paths; errors centralized in `CcrError` with exit codes.
- Add commands under `src/commands/<domain>/`, export in `mod.rs`, wire in `main.rs`.
- Add platforms by implementing `PlatformConfig` and registering in `platforms::create_platform`.
