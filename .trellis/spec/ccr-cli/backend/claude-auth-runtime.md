# Claude Auth Runtime Diagnosis

> Executable contract for Claude Code credential-source ordering, secret-free diagnosis, and post-switch warnings. Source behavior was verified against the linked Claude Code documentation on 2026-07-29.

## Scenario: Observable Claude Auth Sources

### 1. Scope / Trigger

- Trigger: changing Claude profile/auth switching, runtime summaries, doctor checks, credential-source detection, or CLI/TUI/Tauri/UI auth-source presentation.
- Applies to `ccr_types::claude_auth`, `ClaudeAuthService`, `DoctorService`, profile off, Claude Auth CLI/TUI/Tauri commands, generated Claude Auth TypeScript, and `ClaudeAuthView.vue`.
- CCR diagnoses only the current process environment and the user-level files resolved by `ClaudeRuntimePaths`. It does not promise equality with `/status` in an independently launched Claude Code process.

### 2. Signatures

- `ClaudeAuthService::diagnose_auth_sources() -> Result<ClaudeAuthDiagnosis>` is the only source-ordering implementation.
- `ClaudeAuthService::action_outcome(cleared_managed_sources: Vec<String>) -> ClaudeAuthActionOutcome` performs post-action diagnosis without turning a successful write into a failed action.
- `ClaudeAuthService::switch_account(name) -> Result<ClaudeAuthActionOutcome>` and Claude `profile off` expose cleared CCR-owned keys plus remaining sources/warnings.
- `ClaudeRuntimeSummary::auth_diagnosis: ClaudeAuthDiagnosis` is additive and defaults during Rust deserialization for compatibility.
- `platform.claude.auth_sources` is the doctor check ID.
- Shared secret-free DTOs:
  - `ClaudeAuthSourceObservation { kind, location, confidence, evidence, ownership, suppresses_subscription }`
  - `ClaudeAuthDiagnosis { observations, presumed_effective_source, custom_api_key_responses_present, unobservable }`
  - `ClaudeAuthActionOutcome { cleared_managed_sources, remaining_suppressors, warnings }`

### 3. Contracts

#### Official priority table

The detector sorts observed sources in this order:

| Priority | Source | Normal confidence | Notes |
| --- | --- | --- | --- |
| 1 | `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY` | `confirmed` | Cloud-provider mode is an exception to the normal Anthropic credential flow and outranks it. Multiple sources at this level are ambiguous. |
| 2 | `ANTHROPIC_AUTH_TOKEN` | `confirmed` | Includes documented LLM-gateway bearer-token use; do not infer subscription identity or provider from the token shape. |
| 3 | `ANTHROPIC_API_KEY` | `potential` | Interactive takeover may depend on Claude Code's approval state. |
| 4 | top-level `settings.json.apiKeyHelper` | `potential` | CCR detects presence but never executes the helper or reads its external secret store. |
| 5 | `CLAUDE_CODE_OAUTH_TOKEN` | `confirmed` | Explicit OAuth token, distinct from `/login` subscription storage. |
| 6 | `/login` subscription OAuth | `confirmed` when readable | Windows/Linux use resolved `.credentials.json`; macOS Keychain content is unobservable and auth snapshot/switch remains unsupported there. |

`ANTHROPIC_AUTH_TOKEN` paired with gateway settings such as `ANTHROPIC_BASE_URL` is still priority 2. It must not be described as the user's Anthropic subscription token. `customApiKeyResponses` is approval context only: set `custom_api_key_responses_present`, but never create an observation or increment a competing-source count.

`.claude.json.primaryApiKey` is outside the official priority contract. Issue [anthropics/claude-code#80713](https://github.com/anthropics/claude-code/issues/80713), verified open on 2026-07-29, reports that it may suppress an active subscription. Always emit it as `issue_report` + `potential`, ordered after `CLAUDE_CODE_OAUTH_TOKEN` and before subscription OAuth; never call it officially confirmed behavior.

#### Confidence and effective-source rules

- `confirmed`: CCR observed a non-empty source whose use follows the official contract. This confirms only the current CCR scope.
- `potential`: the source exists, but approval, helper execution, or version-specific issue behavior is unknown. Presentation must say "potential", "competing", or "may suppress", not that it is currently active.
- `unobservable`: a capability boundary, not evidence of absence. Always list other-shell env, unknown-cwd project settings, external CLI args, managed dynamic policy, helper/external secret-store results, and macOS Keychain content.
- `presumed_effective_source` is the sole observation at the highest observed priority. If that priority has multiple observations, return all observations and leave the presumed source unset rather than choosing arbitrarily.

#### Secret and ownership rules

- Detection may test presence/non-emptiness only. DTOs, doctor detail, errors, logs, CLI/TUI output, Tauri JSON, and UI text must never contain credential values or credential hashes.
- `CCR_MANAGED_KEYS` is the automatic cleanup boundary. Anything outside it, including user `ANTHROPIC_API_KEY`, `apiKeyHelper`, cloud-provider env, and `primaryApiKey`, is warning-only.
- Post-switch/profile-off output distinguishes `cleared_managed_sources` from remaining `user_owned`/`external_runtime` observations. If diagnosis fails after a successful action, retain the success and surface `warnings`; consumers must not replace that state with "no source observed".
- Settings, credentials, and state paths come only from [`ClaudeRuntimePaths`](../../ccr-config/backend/backend-guidelines.md). Credential snapshots, secret writes, and settings CAS are owned by [Backend Guidelines: Claude Credential Snapshots And Settings CAS](./backend-guidelines.md#scenario-claude-credential-snapshots-and-settings-cas). State-file write ownership and MCP CAS are owned by [Backend Guidelines: Claude API-Key Profile Runtime Env](./backend-guidelines.md#scenario-claude-api-key-profile-runtime-env). Diagnosis is read-only.

### 4. Validation & Error Matrix

- Missing settings/state/credentials file -> treat that file as having no observation; do not create it.
- Existing unreadable, empty, malformed, or non-object state JSON -> diagnosis error; doctor `fail`; post-action outcome keeps action success and emits a warning.
- Existing malformed/unreadable `.credentials.json` on Windows/Linux -> diagnosis error; never fall back to the CCR auth registry to claim subscription OAuth.
- No competing source + confirmed subscription OAuth -> doctor `ok`.
- No confirmed source -> doctor `warn` and list unobservable layers.
- One or more confirmed/potential competing sources -> doctor `warn`; summary distinguishes confirmed suppressors from potential competing sources.
- Multiple observations at the highest priority -> `presumed_effective_source = None`.
- `customApiKeyResponses` alone -> context flag only; zero competing observations.
- Post-action diagnosis failure -> return empty `remaining_suppressors` plus non-empty `warnings`; CLI/TUI/UI must display the warning.

### 5. Good/Base/Bad Cases

- Good: subscription credentials plus user `ANTHROPIC_API_KEY` produce ordered `potential` API-key and `confirmed` subscription observations; switching preserves the user key and warns.
- Good: Bedrock and Vertex are both present, so both observations remain visible and the presumed source is unresolved.
- Good: `primaryApiKey` and `customApiKeyResponses` produce one issue-backed potential observation plus one context flag.
- Base: valid subscription credentials with no higher source produce an OK doctor check.
- Base: another shell may differ; the UI lists that layer as unobservable instead of claiming system-wide state.
- Bad: execute `apiKeyHelper`, print a masked token/hash, or inspect arbitrary project directories to strengthen confidence.
- Bad: delete a user-owned source during auth switch/profile off.
- Bad: call a potential source "currently active" or label `primaryApiKey` as an official contract.

### 6. Tests Required

- `cargo test -p ccr-types claude_auth -- --test-threads=1`
  - Assert additive serde compatibility and that diagnosis JSON contains no injected secret.
- `cargo test -p ccr-cli claude_auth -- --test-threads=1`
  - Cover every supported process/settings env source, priority ordering, ambiguity, ownership, API-key/helper confidence, `primaryApiKey` evidence, context-only responses, fixed unobservable layers, strict credentials/state parsing, and post-action fallback warnings.
- `cargo test -p ccr-cli doctor -- --test-threads=1` and `cargo test -p ccr --test commands doctor -- --test-threads=1`
  - Assert auth-source OK/warn/fail states and secret-free detail.
- `cargo test -p ccr-tui claude_auth -- --test-threads=1`
  - Assert diagnosis rendering and both structured/fallback action warnings.
- Tauri Claude Auth DTO/export tests plus `just tauri-bindings-check` after the generated baseline is committed.
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/platforms/claude-auth-view.smoke.test.tsx`
  - Assert confidence/evidence/ownership/boundary rendering and post-switch warnings.
- Final cross-layer gates: `just lint-strict`, `just test`, `just frontend-check-quick`, `cargo fmt --all -- --check`, and `git diff --check`.

### 7. Wrong vs Correct

#### Wrong

```rust
if settings.env.contains_key("ANTHROPIC_API_KEY") {
    println!("Claude is using API key {}", settings.env["ANTHROPIC_API_KEY"]);
    settings.env.remove("ANTHROPIC_API_KEY");
}
```

This leaks a secret, upgrades a potential source to certainty, and deletes user-owned configuration.

#### Correct

```rust
let diagnosis = service.diagnose_auth_sources()?;
for source in diagnosis.suppressors() {
    println!(
        "{} @ {} ({})",
        source.kind.as_str(),
        source.location.as_str(),
        source.confidence.as_str(),
    );
}
```

The shared service owns ordering and confidence, while presentation consumes only secret-free identifiers.

## References

- https://code.claude.com/docs/en/authentication
- https://code.claude.com/docs/en/settings
- https://code.claude.com/docs/en/llm-gateway
- https://support.claude.com/en/articles/12304248
- https://github.com/anthropics/claude-code/issues/80713
