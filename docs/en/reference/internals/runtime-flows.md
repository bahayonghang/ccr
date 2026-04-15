# Runtime Flows

This page records the most important current execution paths so the command surface stays aligned with the implementation.

## 1. CLI entry and no-subcommand behavior

```mermaid
sequenceDiagram
  participant User
  participant Main as main.rs
  participant Cli as cli/definitions.rs
  participant Dispatch as cli/dispatch.rs
  participant Tui as tui

  User->>Main: ccr ...
  Main->>Cli: parse args
  Main->>Dispatch: dispatch(&cli)
  alt command present
    Dispatch-->>User: route to command handler
  else no subcommand
    Dispatch->>Tui: run_tui()
  end
```

Current facts:

- the default build enables the `tui` feature
- `ccr` with no subcommand and no `config_name` enters TUI
- `ccr codex` with no action is also treated as TUI mode
- `ccr opencode` with no action opens the OpenCode Auth tab

## 2. Profile switching

```mermaid
sequenceDiagram
  participant User
  participant Cmd as commands/profile
  participant Config as ConfigService
  participant Settings as SettingsService
  participant History as HistoryService

  User->>Cmd: ccr switch <name>
  Cmd->>Config: load registry and profiles
  Config-->>Cmd: target profile
  Cmd->>Settings: apply settings / back up / write
  Settings-->>Cmd: write complete
  Cmd->>History: record masked history
```

Relevant components:

- `ConfigService::get_config` and `set_current`
- `SettingsService::apply_config`
- `HistoryService`

## 3. `ccr ui`

```mermaid
flowchart TD
  A[ccr ui] --> B[dispatch_ui]
  B --> C[UiService]
  C --> D{local ccr-ui/ exists?}
  D -- yes --> E[start_dev_mode]
  D -- no --> F{~/.ccr/ccr-ui/ exists?}
  F -- yes --> G[start_local]
  F -- no --> H[prompt download / sync from GitHub]
```

Current priority order:

1. a nearby `ccr-ui/` checkout
2. `~/.ccr/ccr-ui/`
3. download or update flow

## 4. Session indexing

```mermaid
flowchart LR
  A[ccr sessions *] --> B[SessionIndexer]
  B --> C[scan platform session files]
  C --> D[parse summaries / time / cwd / token stats]
  D --> E[SessionStore]
  E --> F[(local session storage)]
```

Code boundaries:

- `sessions/indexer.rs`: scanning and rebuild orchestration
- `storage/session_store.rs`: upsert, list, search, stats, prune

## 5. Codex multi-account auth

```mermaid
flowchart TD
  A[ccr codex auth ...] --> B[CodexAuthService]
  B --> C[read ~/.codex/auth.json]
  B --> D[read CCR-managed registry and saved account copies]
  B --> E[save / switch / delete / import / export]
  E --> F[sync current runtime config when needed]
  E --> G[create auth backups]
```

This service owns:

- current login state detection
- account inventory and expiry state
- backup rotation for auth documents
- import/export and switching

## 6. OpenCode auth migration

```mermaid
flowchart TD
  A[ccr opencode auth import-codex] --> B[OpenCodeAuthService]
  B --> C[read CCR-managed Codex registry and auth snapshots]
  C --> D[filter compatible ChatGPT OAuth accounts]
  D --> E[check OpenCode name and accountId conflicts]
  E --> F[write OpenCode snapshots and registry entries only for new accounts]
  F --> G[leave the current OpenCode runtime auth.json untouched]
```

This flow owns:

- reading only Codex accounts already saved in CCR, not an unsaved runtime login
- mapping compatible Codex token payloads into OpenCode `openai` OAuth snapshots
- skipping API-key entries, missing snapshots, invalid snapshots, and conflicts
- producing a structured migration report shared by CLI and TUI

## 7. WebDAV sync

```mermaid
flowchart TD
  A[sync config] --> B[write connection config]
  C[sync folder add/enable] --> D[register sync folder]
  E[sync <platform> push/pull] --> F[SyncService]
  F --> G[filter backups / history / lock files]
  F --> H[perform remote sync]
```

Relevant modules:

- `sync/config.rs`
- `sync/folder.rs`
- `sync/folder_manager.rs`
- `services/sync_service.rs`
