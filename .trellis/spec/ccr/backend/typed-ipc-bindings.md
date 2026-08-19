# Typed IPC Bindings (ts-rs)

> Executable contract for typed IPC domains: Rust DTO → generated TypeScript, committed bindings, drift guard. Extend per-domain; do not bypass.

## Scenario: typed Tauri command payloads with generated TS bindings

### 1. Scope / Trigger

- Trigger: adding/changing any wire DTO returned by (or accepted as input to) a typed Tauri command; typing a new command domain; upgrading `ts-rs`; converting `OpenJsonValueDto` / `JsonValueDto` numbers to `serde_json::Value`.
- Applies to `ccr-ui/src-tauri/src/services/*`, `ccr-ui/src-tauri/src/commands/{wire,system,grok}.rs`, `ccr-ui/src-tauri/src/llmusage_adapter/{queries,capabilities}.rs`, `ccr-ui/src-tauri/src/{usage_jobs,session_index_jobs}.rs`, `ccr-ui/src-tauri/src/claude_observer/subscription.rs`, `crates/ccr-usage/src/{queries,capabilities}.rs`, and the generated dir `ccr-ui/src/types/generated/`.
- Typed coverage is generated from the command manifest: 265/328 base commands (80.79%), with exact registry-owned input/output declarations for 265/265 typed commands. This includes Usage V2 (17), Claude Observer (9), install (8), config, system prompts, sync, Claude, Codex, auth/provider, Gemini, Grok, OpenCode, SSH, command execution, and the smaller system/UI/environment/event/shell domains. All typed commands expose a concrete generated return type; `Result<Value, String>` is banned at the command boundary.

### 2. Signatures

- src-tauri types (direct dependency, no feature gate):

  ```rust
  use ts_rs::TS;

  #[derive(Debug, Clone, Serialize, TS)]
  #[ts(export, export_to = "../../src/types/generated/usage/")]
  pub struct SomeDto { ... }
  ```

- Workspace crate types (`ccr-usage`) are feature-gated so the CLI graph stays ts-rs-free:

  ```rust
  #[derive(Debug, Clone, Serialize, PartialEq)]
  #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts", ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/"))]
  pub struct DailyTrendDto { ... }
  ```

  `ccr-usage` features: `ts = ["dep:ts-rs"]`（src-tauri 主依赖启用）、`test-fixtures = []`（src-tauri dev-dependency 启用，暴露 `ccr_usage::fixtures` 投影库 fixture）。

- Command shape: `#[tauri::command]` is a thin adapter (timing/State extraction/spawn_blocking/cache); business logic lives in State-free sync functions in `services/usage.rs` taking `&LlmusageRuntime` / `&DbPool` + plain args, returning `Result<NamedDto, String>`. `LlmusageRuntime::from_paths(AppPaths)` exists for tests.
- Regeneration: `just tauri-bindings` (root) → deletes `ccr-ui/src/types/generated/`, runs `cargo test -p ccr-cli --features ts export_bindings` + `cargo test -p ccr-usage --features ts export_bindings` + `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml export_bindings`.
- Drift guard: `just tauri-bindings-check` first runs the existing generated-file normalizer to repair deterministic whitespace, then snapshots `src/types/generated/`, regenerates, and compares the result with the normalized pre-generation worktree snapshot. Formatting-only drift is repaired automatically; DTO/file-shape drift still fails. Wired into `just ci` before `frontend-check`. Independent of the api-facade-boundary smoke (per that spec).

### 3. Contracts

- **64-bit integers**: released ts-rs (v10/v11) hardcodes `i64/u64 → bigint`. Wire is serde_json `number`, so every `i64`/`u64` field MUST carry `#[ts(as = "f64")]` (`Option<i64>` → `#[ts(as = "Option<f64>")]`, maps → `#[ts(as = "std::collections::HashMap<String, f64>")]`). `usize/f64/bool/String` need nothing. Upgrade path: ts-rs v12 ships `TS_RS_LARGE_INT`; set it to `number` globally and delete the per-field `as` annotations.
- **Input DTOs** (command args deserialized by serde): `Option` fields take `#[ts(optional)]` so TS gets `field?: T` (missing key == None). Without it ts-rs emits required `T | null`, breaking object-literal call sites.
- **`skip_serializing_if = "Option::is_none"` output fields**: wire is _absent key_, not `null` → also `#[ts(optional)]`.
- **`export_to` paths resolve relative to `<manifest>/bindings/`**, not the manifest dir (one level deeper than intuition).
- Generated files are **committed** (reviewers see contract diffs), `linguist-generated` + `eol=lf` via root `.gitattributes`, excluded from eslint (`ccr-ui/eslint.config.js` ignores `src/types/generated/**`), covered by `bun run type-check`.
- TS consumption: domain wrappers re-export registry-generated clients and expose concrete return types -- no direct typed `invoke()` or `<T = UnknownRecord>` generics in a typed domain. `src/types/usage.ts` is a compat shim re-exporting generated types under legacy names plus hand-written view-only types (`UsagePlatform`, `HomeOverviewViewMode`, event payloads). Event payloads (`app_handle.emit`) are not command returns and stay hand-written until events join the pilot.
- Structurally open configuration payloads use the generated recursive `OpenJsonValueDto` union at the command boundary. Handwritten wrappers must convert unknown inputs with `toOpenJsonValue`; unchecked `as OpenJsonValueDto` casts are forbidden because they admit bigint, non-finite numbers, symbols, and other non-JSON values.
- **Open JSON numbers**: JS/Tauri collapse every JSON number into `f64` (`OpenJsonValueDto::Number` / `JsonValueDto::Number`). `From` conversion MUST go through `json_number_from_f64` in `commands/wire.rs` (shared by `system.rs`). Values that round-trip as `u64`/`i64` become integer `serde_json::Number`; fractions stay Float; non-finite become Null. Do not call `Number::from_f64` alone — serde_json 1.0.x keeps that as `N::Float`, so `as_u64()` / `as_i64()` are `None` even for `500000.0`.
- Typed commands are invoked only by registry-generated clients under `src/api/generated/`. `stats.ts`, `claudeObserver.ts`, and `install.ts` are compatibility re-export/projection surfaces, not direct invoke owners; the manifest-aware smoke guard has no typed-client exception list.
- Name uniqueness: one exported type name per generated dir. If a workspace crate and src-tauri both define a same-named type (e.g. `HomeOverview*`), only the wire-facing one gets `ts(export)`.
- **Repository types never go on the wire directly**: when a domain returns rows owned by a ccr-db repository (e.g. `claude_tool_calls_repo::{HeatmapCell,TopToolRow}`), the service layer defines a same-shaped wire DTO with the `TS` derive and maps via `From`. ccr-db stays free of ts-rs/frontend-binding concerns, and the bindings recipe never needs a ccr-db export step.

### 4. Validation & Error Matrix

- i64/u64 field missing `ts(as)` → `bigint` appears in generated file → consumer `bun run type-check` fails + drift diff shows `bigint`.
- `OpenJsonValueDto::Number(500_000.0)` converted with `Number::from_f64` → `as_u64()` is `None` → domain validators that require a positive integer reject a legal whole number. Conversion via `json_number_from_f64` must yield `as_u64() == Some(500_000)`. Fractional `1.5` stays Float; negative whole `-8.0` yields `as_i64() == Some(-8)`.
- Rust DTO changed without regeneration → `just tauri-bindings-check` exits 1 listing the generated paths changed by regeneration.
- Hand-edited generated file that changes its generated shape → same guard failure; deterministic whitespace is repaired by the normalizer.
- New typed command added → handler-registry contract still applies unchanged (`define_command_registry!`, frozen counts 328 base / 336 Windows).
- `serde(alias)` on input DTOs is ignored by ts-rs but remains active for deserialization; keep the desktop dependency's `no-serde-warnings` feature enabled so this intentional compatibility alias does not emit macro warnings. The generated shape remains canonical snake_case.
- Plain `cargo test` in src-tauri reruns export tests and rewrites generated files idempotently — `just tauri-bindings-check` normalizes those side effects before comparing the regenerated output with the current worktree baseline.

### 5. Good / Base / Bad Cases

- Good: add field `pub cache_hits: i64` with `#[ts(as = "f64")]`, run `just tauri-bindings`, commit code + regenerated `.ts` together.
- Good: force `OpenJsonValueDto::Number(500_000.0)` in a unit test; `json!(500_000)` is already an integer Number and does not exercise the IPC f64 path.
- Bad: `serde_json::Number::from_f64(n)` then `as_u64()` for integers that arrived from the UI.
- Good: new service fn in `services/usage.rs` + unit test against `ccr_usage::fixtures::create_projection_db`.
- Base: legacy manifest domains keep `Result<Value, String>` until their own migration task; every domain marked `Generated` has exact registry-owned types and client declarations.
- Bad: `serde_json::to_value(dto)` at the end of a typed command (reintroduces erasure).
- Bad: editing files under `src/types/generated/` by hand, or adding new interfaces to `src/types/usage.ts` that mirror Rust structs.
- Bad: giving frontend wrappers back their `<T = UnknownRecord>` escape hatch inside a typed domain.

### 6. Tests Required

- `cargo test -p ccr-usage --features ts export_bindings -- --test-threads=1` and src-tauri `cargo test export_bindings` (regeneration suites).
- `just tauri-bindings-check` (drift; also part of `just ci`).
- Service unit tests without a Tauri app: src-tauri `cargo test services -- --test-threads=1` (fixture DB via `ccr_usage::fixtures`, temp ccr-db pool via `create_pool` + `run_all_migrations`; no real home-dir access — FS probes like `has_any_raw_sessions` are passed in as booleans by the command layer).
- `cd ccr-ui && bun run type-check` (generated types + consumers).
- `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts tests/typed-json-boundary.smoke.test.ts tests/typed-command-boundary.smoke.test.ts` (generated-client ownership + JSON input boundary + zero raw-`Value` command returns).
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture` (counts unchanged).
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::wire::tests -- --test-threads=1` (whole f64 → integer Number; fraction stays float).

### 7. Wrong vs Correct

#### Wrong

```rust
#[derive(Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct StatsDto {
    pub total_tokens: i64,          // 生成 bigint，与 serde_json number 运行时不符
}

#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(compute(&state)) // 类型擦除回潮
        .map_err(|e| format!("Serialize error: {e}"))
}
```

#### Correct

```rust
#[derive(Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct StatsDto {
    #[ts(as = "f64")]
    pub total_tokens: i64,
}

#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<StatsDto, String> {
    let llmusage = state.llmusage.clone();
    tokio::task::spawn_blocking(move || services::usage::stats(&llmusage))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}
```

#### Wrong (open JSON numbers)

```rust
OpenJsonValueDto::Number(value) => Number::from_f64(value)
    .map(Value::Number)
    .unwrap_or(Value::Null),
```

`500000.0` becomes Float; later `as_u64()` is `None`.

#### Correct (open JSON numbers)

```rust
OpenJsonValueDto::Number(value) => json_number_from_f64(value),
```

Whole f64 values become integer JSON numbers so `as_u64()` / `as_i64()` work.

## Scenario: llmusage install opaque plan handle

### 1. Scope / Trigger

- Trigger: changing llmusage detection, planning, automatic execution, install events, cancellation, or generated install DTOs.
- Applies to `crates/ccr-cli/src/services/install_{types,plan,service,exec,ring_buffer}.rs`, `ccr-ui/src-tauri/src/commands/install.rs`, `ccr-ui/src/api/domains/install.ts`, `ccr-ui/src/components/usage/LlmusageInstallDialog.vue`, and `ccr-ui/src/types/generated/install/`.
- The renderer is a low-trust request layer. It may display a plan and return its opaque ID, but it never owns an executable capability.

### 2. Signatures

- `PlanId` and `AttemptId` are `#[serde(transparent)]` UUID newtypes and generate as TypeScript `string` aliases.
- `InstallService::plan(&DetectionResult, &HostCapabilities) -> Result<PlanOutcome, InstallFlowError>` rechecks renderer hints against fresh backend probes before registering a plan.
- `llmusage_install_plan(detection, capabilities) -> Result<PlanOutcome, String>` returns either `InstallPlanView` or `UnsupportedReason`.
- `llmusage_install_execute(plan_id: PlanId) -> Result<AttemptId, String>` consumes a backend plan exactly once.
- `install_exec::run_attempt(InstallAction, InstallPlanView, AttemptId, ...)` is crate-private; only its closed `InstallAction` produces a `ProcessSpec`.

### 3. Contracts

- `InstallPlanView` contains `plan_id`, `platform`, `package_manager`, `action`, `expected_effects`, `elevation_required`, `duration_class`, and `expires_at_ms`. It contains no executable, arguments, or environment.
- `CanonicalInstallPlan` is backend-private and binds the closed `InstallAction` to the probed host snapshot for 120 seconds.
- Plan consumption removes the canonical entry while holding the registry mutex. A successful consume leaves a reused tombstone, so concurrent consumers have exactly one winner.
- `InstallAction::{Cargo,Homebrew,Scoop,Winget}` is the only source for executable/argument construction. `Command::new` receives the resulting private `ProcessSpec`, never a renderer DTO.
- `InstallEvent::Started` contains the renderer-safe plan view. Structured tracing records `action`, `plan_id`, and `attempt_id`; it must not record environment values.
- The manual install catalog may expose copyable `command_line` text for the user, but it is not accepted by the automatic execute command.

### 4. Validation & Error Matrix

- Renderer detection or capability hints differ from fresh backend probes -> `invalid_payload`; do not register a plan.
- Unknown UUID -> error string contains stable code `install_plan_unknown`.
- Plan older than 120 seconds -> consume fails with `install_plan_expired`.
- Already consumed plan -> consume fails with `install_plan_reused`.
- Current host snapshot differs from the registered snapshot -> consume fails with `install_plan_host_mismatch` and the plan is not executable afterward.
- Another install attempt owns the slot -> `already_running`; do not consume the submitted plan.
- Malformed UUID input -> Tauri rejects deserialization before the service is called.

### 5. Good / Base / Bad Cases

- Good: the UI receives a plan view, later invokes execute with only `planId`, and retry obtains a fresh plan before executing again.
- Good: two concurrent registry consumers race on one `PlanId`; exactly one obtains the canonical action.
- Base: a plan expires while the dialog is open; execute returns the stable expiry code and retry replans from fresh host state.
- Bad: accepting `InstallPlanView` or any `{ command, args, envs }` object in `llmusage_install_execute`.
- Bad: rendering `Command::new(&plan.command)` or resolving a renderer-supplied package-manager path.

### 6. Tests Required

- `cargo test -p ccr-cli install -- --test-threads=1`: assert hostile/modified payload fields are absent, closed action mappings are fixed, TTL/unknown/reuse/host mismatch errors are stable, cleanup prunes expired entries, and concurrent consumption has one winner.
- `cargo test -p ccr-cli --features ts export_bindings -- --test-threads=1`: export every install DTO, including UUID string aliases.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml install -- --test-threads=1`: compile and exercise the desktop command surface.
- `cd ccr-ui && bun run test:smoke -- tests/install-opaque-handle.smoke.test.ts`: assert execute sends only `{ planId }` and the generated plan view has no `command`, `args`, or `envs`.
- Run `cd ccr-ui && bun run type-check`, `just frontend-check-quick`, `just lint-strict`, and `just test`.
- After the generated baseline is committed, run `just tauri-bindings-check`; intended new generated files make the HEAD-based drift guard red before that baseline exists.

### 7. Wrong vs Correct

#### Wrong

```rust
#[tauri::command]
async fn llmusage_install_execute(plan: InstallPlan) -> Result<AttemptId, String> {
    Command::new(&plan.command).args(&plan.args).envs(&plan.envs);
    // renderer data has become an executable capability
}
```

#### Correct

```rust
#[tauri::command]
async fn llmusage_install_execute(
    svc: State<'_, Arc<InstallService>>,
    plan_id: PlanId,
) -> Result<AttemptId, String> {
    svc.execute(plan_id)
        .await
        .map(|attempt| attempt.attempt_id)
        .map_err(|error| error.to_string())
}
```

The service atomically consumes the private canonical plan and passes only its closed `InstallAction` to the executor.

## Extending to a new domain (checklist)

1. Move/extract the domain's wire DTOs next to its State-free service functions; add `TS` derives with the annotation rules above (`export_to` still points at `src/types/generated/<domain>/`... keep one dir per domain).
2. Convert commands to `Result<NamedDto, String>`; delete trailing `to_value`; cache layers store `serde_json::to_value(&dto)` and decode hits with `from_value`.
3. Regenerate (`just tauri-bindings` — extend the recipe if a new crate joins), commit bindings.
4. Type the domain's wrappers in `src/api/domains/<domain>.ts`; turn the domain's hand-written mirror types into a shim; drop caller generics.
5. Add service unit tests (reuse `ccr_usage::fixtures` where the domain reads llmusage projections).
6. Registry counts unchanged unless commands were added/removed intentionally.

## Scenario: typed capability metadata at the invoke boundary

### 1. Scope / Trigger

- Trigger: changing command risk, timeout ownership, concurrency, confirmation, or generated manifest/client fields.
- Applies across the Rust registry, `command-manifest.json`, `commandCapabilities.ts`, generated clients, and the frontend runtime facade.

### 2. Signatures

```rust
pub(crate) enum CommandTimeoutEnforcement {
    Cooperative,
    CompletionAware,
    BusinessOwned,
}
```

```typescript
type CommandCapability = {
  timeout_ms: number
  timeout_enforcement: 'cooperative' | 'completion_aware' | 'business_owned'
  concurrency: 'parallel' | 'module_exclusive' | 'singleton'
  confirmation: 'none' | 'user_gesture' | 'opaque_capability'
}
```

### 3. Contracts

- `timeout_enforcement` is required for every generated descriptor and is serialized from the Rust enum; TypeScript must not infer it from risk.
- Risk defaults may select policy, but command-specific exceptions remain explicit backend rules and are covered by registry tests.
- `user_gesture` is transport confirmation, not authorization. The frontend derives its token from the generated manifest and the backend independently verifies the exact command binding.
- `opaque_capability` is backend-issued proof: install uses `planId`, SSH fingerprint confirmation uses `request.challenge_id`. The frontend must not synthesize these values from capability metadata.
- Manifest audit fields remain metadata-only or redacted; payload bodies, environment values, secrets, and raw DTO debug output are never logged by generic runtime policy.

### 4. Validation & Error Matrix

- Missing `timeout_enforcement` in generated manifest -> inventory/schema test fails.
- Unknown command at runtime -> reject before dispatch.
- `user_gesture` token for another command -> confirmation validation fails.
- Empty install `planId` or SSH `challenge_id` -> opaque capability validation fails.
- Binary payload for a gesture-confirmed command -> frontend throws `Command <id> requires a JSON confirmation payload` before invoke.
- `Cooperative` assigned to a command without cancellation proof -> policy review/test failure; use `CompletionAware` or `BusinessOwned`.

### 5. Good / Base / Bad Cases

- Good: regenerate both JSON and TypeScript capability artifacts from one descriptor after changing a command policy.
- Good: keep a process command typed while its business layer owns cancellation.
- Base: a read-only typed command has no confirmation and uses completion-aware admission.
- Bad: add a second handwritten timeout-policy map in TypeScript.
- Bad: treat `desktop-confirm:<command>` as a secret or durable authorization token.

### 6. Tests Required

- Registry tests assert complete timeout ownership and freeze the cooperative allowlist.
- Runtime tests assert queue/permit behavior against real futures.
- `just tauri-command-inventory` followed by `just tauri-command-inventory-check` proves deterministic cross-layer generation.
- `just tauri-bindings` and `just tauri-bindings-check` prove DTO/client drift remains closed.
- Frontend smoke tests assert gesture injection, opaque capability passthrough, and binary-payload rejection.

### 7. Wrong vs Correct

#### Wrong

```typescript
const timeoutMode = risk === 'process_execution' ? 'hard_timeout' : 'none'
```

#### Correct

```typescript
const capability = COMMAND_MANIFEST.commands.find(item => item.id === command)
// Consume the backend-generated policy; do not reinterpret risk locally.
```
