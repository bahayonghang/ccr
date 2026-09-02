# Agent Session Observability Contracts

> Cross-layer contracts for the local-only `/agent-sessions` browser. These rules apply to provider discovery, the usage archive, typed Tauri IPC, and the React surface.

---

## Scenario: add or change an Agent session provider

### 1. Scope / Trigger

- Trigger: adding a session family or source variant, changing discovery/parsing, changing the archive schema, changing any `agent_sessions_*` command/DTO, or changing the `/agent-sessions` list/detail/refresh flow.
- The v1 registry is independent of `ccr_config::Platform` and contains exactly Grok, Claude, Codex, OpenCode, Pi, OMP, Antigravity, and Kimi.
- Source kinds are `file`, `bundle`, and `sqlite_member`. Antigravity IDE/CLI share one family; Kimi includes `kimi-legacy` and `kimi-code`, not Kimi Work.
- External Agent files and databases are read-only. CCR persists only normalized summaries, fingerprints, source identity, and state in the usage archive; message bodies remain provider-owned and are loaded on demand.

### 2. Signatures

Typed commands and generated frontend functions:

```ts
agentSessionsList(request: AgentSessionListRequestDto): Promise<AgentSessionPageDto>
agentSessionsGetDetail(request: AgentSessionDetailRequestDto): Promise<AgentSessionDetailDto>
agentSessionsGetProviderStatus(): Promise<AgentSessionProviderStatusDto[]>
agentSessionsStartRefresh(): Promise<StartSessionIndexJobResponse>
agentSessionsGetRefreshStatus(jobId: string): Promise<SessionIndexJobSnapshot>
```

Repository/provider boundaries:

```rust
get_agent_session_archive_page(conn, &AgentSessionArchiveQuery) -> Result<Vec<AgentSessionArchiveRow>>
get_agent_session_archive_source(conn, archive_id) -> Result<Option<AgentSessionArchiveSource>>
AgentSessionProviderRegistry::discover(agent) -> Result<Vec<AgentSessionSourceRef>>
AgentSessionProviderRegistry::read_message_page(source, before, limit) -> Result<AgentSessionMessagePage>
```

Database ownership:

- `usage_session_archive` is keyed by opaque `archive_id`; physical identity is unique on `(platform, file_path, source_member_id)`.
- `usage_session_source_state` is keyed by `(platform, source_path, source_kind)` and owns incremental source fingerprints.
- Migration v17 must preserve fresh-schema and upgraded-schema parity and must replace legacy path-derived renderer IDs with stable `as-<sha256>` IDs.

### 3. Contracts

- `AgentSessionListRequestDto` accepts optional `agents`, `query`, `cwd_prefix`, `started_at`, `ended_at`, `source_state`, `fidelity`, `cursor`, and `limit`. Empty `agents` means the eight v1 families only. List ordering and keyset pagination are `(updated_at DESC, archive_id DESC)`.
- `AgentSessionDetailRequestDto` accepts `archive_id`, optional `before_cursor`, and optional `limit`. List and detail limits clamp to `1..=200`; defaults are 80 and 100 respectively.
- Each message has a stable `key` and `ordinal`; `latest` followed by `before_cursor` must not duplicate messages. Provider readers keep only the requested window in memory, and each normalized content field is at most 256 KiB with `clipped=true` when truncated.
- Availability (`not_installed | no_data | available | error`) is independent from fidelity (`full | partial | locked`). Missing or encrypted data is not a parse error.
- `ANTIGRAVITY_KEY` is optional and process-scoped. Its presence may permit a partial status, but v1 does not claim `.pb` decryption or `full` fidelity from the key alone; plaintext brain/history fallback is `partial`, and inaccessible encrypted-only data is `locked`.
- Refresh fingerprints sources before parsing. An unchanged container must not enumerate SQLite members, parse messages, or upsert summaries; the refresh report must expose `discovered`, `unchanged`, `fingerprinted`, `parsed`, `upserted`, `partial`, `locked`, and `errors` counters.
- The renderer never receives raw source paths or secrets. Logs and user-facing error categories must not contain paths, keys, or transcript content. Detail empty states map `agent_session_*` codes to i18n; they must not render the raw code string.
- `/agent-sessions` is local-only, lives immediately above MCP Manager, and does not replace `/agents`. Non-local environments fail closed before provider/list/detail/refresh queries run; every query key includes the active environment ID.
- Local mount starts one incremental refresh (`agent_sessions_start_refresh`) so the archive can catch live discover counts. Auto-select skips `missing` / `deleted_by_user` rows and archive IDs whose detail returned `agent_session_source_unavailable` or `agent_session_source_validation_failed`, unless the user clicked that row.
- The generated client under `src/api/generated/agentSessions.ts` is the only `invoke()` owner. Feature code imports the domain facade, not Tauri directly. Both list rows and transcript rows use measured virtualization with stable keys.
- After `canonicalize`, Windows `\\?\` / `\\?\UNC\` prefixes and `ParentDir` components must not pass root containment. Compare path components, never string `starts_with`.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Query text over 200 characters, invalid date, reversed date range, bad cursor/filter | Reject with a stable `agent_session_*` validation error; do not query SQLite |
| Empty or longer-than-128 archive ID | `agent_session_invalid_archive_id` |
| Archive ID not found | `agent_session_not_found` |
| Stored source is outside the canonical provider root, or kind/variant/file/member shape is invalid | `agent_session_source_validation_failed`; never open the supplied path |
| Provider directory absent / installed with no sessions / usable / discovery fails | `not_installed` / `no_data` / `available` / `error` |
| Encrypted Antigravity data without usable plaintext | `locked`, not `error` and not synthetic transcript text |
| Source file missing, provider root unavailable, or source removed during detail read | `agent_session_source_unavailable`; retain the indexed summary for refresh reconciliation; do not reuse `agent_session_source_validation_failed` |
| Active environment is not local | Render local-only state and issue no Agent Session IPC |
| Generated command, permission, client, DTO, inventory, or docs drift | Binding/inventory checks fail |

### 5. Good/Base/Bad Cases

- Good: a 450-message SQLite session returns at most 200 messages, then an older non-overlapping page with stable ordinals and keys.
- Good: a second refresh of an unchanged OpenCode database reports `parsed=0` and `upserted=0` without opening or enumerating invalid members.
- Base: a provider with no source directory shows `not_installed`; an empty existing source shows `no_data`.
- Base: an Antigravity `.pb`-only installation without a usable key shows `locked`; plaintext brain/history fallback shows `partial`.
- Bad: returning `file_path` as `archive_id`, logging parser input, or trusting a renderer-supplied source path.
- Bad: mapping a deleted jsonl to `agent_session_source_validation_failed`, or rendering that raw code in the transcript empty state.
- Bad: reading every transcript message into a `Vec` and slicing after parsing, using offset pagination, or keying React Query without environment identity.

### 6. Tests Required

- `cargo test -p ccr-store --no-fail-fast`: discovery/parser fixtures for all eight families, source-shape tampering, UTF-8 clipping, stable pagination, and long JSONL/SQLite/bundle sessions.
- `cargo test -p ccr-db -- --test-threads=1`: fresh/upgrade migration parity, opaque ID backfill, shared-container members, keyset pagination, source-state fingerprints, and missing-state reconciliation.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml agent_sessions --no-fail-fast`: validation/error mapping, refresh counters, typed command registration, and source restoration guards. Missing jsonl → `agent_session_source_unavailable`; wrong extension / escaped path → `agent_session_source_validation_failed`; refresh marks unseen live rows `missing`.
- `just tauri-bindings-check` and `just tauri-command-inventory-check`: command/client/DTO/permission/inventory/docs drift.
- Agent Session smoke tests: route order, local-only fail-closed behavior, environment-scoped keys, provider/fidelity states, virtualized list/transcript, responsive stacking, i18n, loading/empty/error states, and no raw `agent_session_*` code in the transcript empty state.
- Final gates: `just ui-check`, `git diff --check`, and process-scoped UTF-8 `just ci` on Windows.
- Visual evidence: light/dark, zh/en, 1440×900 and sub-900px layouts, all eight family labels, no horizontal overflow, virtualization DOM bounds, and keyboard focus/activation. Native Tauri and real local datasets remain `UNVERIFIED` until exercised directly.

### 7. Wrong vs Correct

#### Wrong

```ts
// Renderer controls a physical path and bypasses environment isolation.
invoke('agent_sessions_get_detail', { path, offset: page * 200 })
useQuery({ queryKey: ['agent-sessions', archiveId] })
```

#### Correct

```ts
// Renderer supplies only opaque identity/cursor; the backend restores and validates the source.
agentSessionsApi.agentSessionsGetDetail({ archive_id: archiveId, before_cursor: before, limit: 200 })
useQuery({ queryKey: agentSessionKeys.detail(environmentId, archiveId), enabled: localEnvironment })
```
