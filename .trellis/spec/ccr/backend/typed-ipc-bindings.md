# Typed IPC Bindings (ts-rs)

> Executable contract for typed IPC domains: Rust DTO → generated TypeScript, committed bindings, drift guard. Extend per-domain; do not bypass.

## Scenario: typed Tauri command payloads with generated TS bindings

### 1. Scope / Trigger

- Trigger: adding/changing any wire DTO returned by (or accepted as input to) a typed Tauri command; typing a new command domain; upgrading `ts-rs`.
- Applies to `ccr-ui/src-tauri/src/services/*`, `ccr-ui/src-tauri/src/llmusage_adapter/{queries,capabilities}.rs`, `ccr-ui/src-tauri/src/{usage_jobs,session_index_jobs}.rs`, `ccr-ui/src-tauri/src/claude_observer/subscription.rs`, `crates/ccr-usage/src/{queries,capabilities}.rs`, and the generated dir `ccr-ui/src/types/generated/`.
- Typed domains: handler_registry "Usage V2" group (17 commands, pilot), "Claude Observer" group (9 commands, `services/claude_observer.rs` + `generated/claude_observer/`), and the llmusage install flow (`ccr_cli::services::install_types` + `generated/install/`). All typed commands are `Result<NamedDto, String>` — `Result<Value, String>` is banned in typed domains.

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
- Drift guard: `just tauri-bindings-check` → regenerate then `git status --porcelain -- src/types/generated` must be empty. Wired into `just ci` before `frontend-check`. Independent of the api-facade-boundary smoke (per that spec).

### 3. Contracts

- **64-bit integers**: released ts-rs (v10/v11) hardcodes `i64/u64 → bigint`. Wire is serde_json `number`, so every `i64`/`u64` field MUST carry `#[ts(as = "f64")]` (`Option<i64>` → `#[ts(as = "Option<f64>")]`, maps → `#[ts(as = "std::collections::HashMap<String, f64>")]`). `usize/f64/bool/String` need nothing. Upgrade path: ts-rs v12 ships `TS_RS_LARGE_INT`; set it to `number` globally and delete the per-field `as` annotations.
- **Input DTOs** (command args deserialized by serde): `Option` fields take `#[ts(optional)]` so TS gets `field?: T` (missing key == None). Without it ts-rs emits required `T | null`, breaking object-literal call sites.
- **`skip_serializing_if = "Option::is_none"` output fields**: wire is _absent key_, not `null` → also `#[ts(optional)]`.
- **`export_to` paths resolve relative to `<manifest>/bindings/`**, not the manifest dir (one level deeper than intuition).
- Generated files are **committed** (reviewers see contract diffs), `linguist-generated` + `eol=lf` via root `.gitattributes`, excluded from eslint (`ccr-ui/eslint.config.js` ignores `src/types/generated/**`), covered by `bun run type-check`.
- TS consumption: domain wrappers (`src/api/domains/stats.ts`) import generated types directly and expose concrete return types — no `<T = UnknownRecord>` generics in a typed domain. `src/types/usage.ts` is a compat shim re-exporting generated types under legacy names plus hand-written view-only types (`UsagePlatform`, `HomeOverviewViewMode`, event payloads). Event payloads (`app_handle.emit`) are not command returns and stay hand-written until events join the pilot.
- Name uniqueness: one exported type name per generated dir. If a workspace crate and src-tauri both define a same-named type (e.g. `HomeOverview*`), only the wire-facing one gets `ts(export)`.
- **Repository types never go on the wire directly**: when a domain returns rows owned by a ccr-db repository (e.g. `claude_tool_calls_repo::{HeatmapCell,TopToolRow}`), the service layer defines a same-shaped wire DTO with the `TS` derive and maps via `From`. ccr-db stays free of ts-rs/frontend-binding concerns, and the bindings recipe never needs a ccr-db export step.

### 4. Validation & Error Matrix

- i64/u64 field missing `ts(as)` → `bigint` appears in generated file → consumer `bun run type-check` fails + drift diff shows `bigint`.
- Rust DTO changed without regeneration → `just tauri-bindings-check` exits 1 listing the dirty/untracked paths.
- Hand-edited generated file → same guard failure (regeneration restores canonical output).
- New typed command added → handler-registry contract still applies unchanged (`define_command_registry!`, frozen counts 315 base / 323 Windows).
- `serde(alias)` on input DTOs is ignored by ts-rs but remains active for deserialization; keep the desktop dependency's `no-serde-warnings` feature enabled so this intentional compatibility alias does not emit macro warnings. The generated shape remains canonical snake_case.
- Plain `cargo test` in src-tauri reruns export tests and rewrites generated files idempotently — a dirty tree afterwards means Rust and committed bindings genuinely diverged.

### 5. Good / Base / Bad Cases

- Good: add field `pub cache_hits: i64` with `#[ts(as = "f64")]`, run `just tauri-bindings`, commit code + regenerated `.ts` together.
- Good: new service fn in `services/usage.rs` + unit test against `ccr_usage::fixtures::create_projection_db`.
- Base: non-pilot domains keep `Result<Value, String>` until their own migration task (old/new styles coexist by design).
- Bad: `serde_json::to_value(dto)` at the end of a typed command (reintroduces erasure).
- Bad: editing files under `src/types/generated/` by hand, or adding new interfaces to `src/types/usage.ts` that mirror Rust structs.
- Bad: giving frontend wrappers back their `<T = UnknownRecord>` escape hatch inside a typed domain.

### 6. Tests Required

- `cargo test -p ccr-usage --features ts export_bindings -- --test-threads=1` and src-tauri `cargo test export_bindings` (regeneration suites).
- `just tauri-bindings-check` (drift; also part of `just ci`).
- Service unit tests without a Tauri app: src-tauri `cargo test services -- --test-threads=1` (fixture DB via `ccr_usage::fixtures`, temp ccr-db pool via `create_pool` + `run_all_migrations`; no real home-dir access — FS probes like `has_any_raw_sessions` are passed in as booleans by the command layer).
- `cd ccr-ui && bun run type-check` (generated types + consumers).
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture` (counts unchanged).

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
