# Approved governance context excerpt

The canonical source remains authoritative. Verify its SHA-256 before context injection. Reload the affected source sections when the hash changes.
Source: .trellis/spec/ccr/backend/dependency-governance.md
Source SHA-256: a30515eb1d2e75b97fd3feb2f76650bd62385fb4bd053b3f9f2cf93ca275dafe

## Source lines L7-L53

## Scenario: Version sync target registry

### 1. Scope / Trigger
- Trigger: changing version surfaces, UI shell version labels, `scripts/version/version-sync.ps1`, `scripts/version/version-sync.sh`, or paths listed in either script's `SYNC_TARGETS`.
- Applies because `just ci` starts with read-only version validation; stale required paths fail the full gate before Rust or frontend checks run. `just version-sync` is an explicit maintenance command.

### 2. Signatures
- Windows sync: `scripts/version/version-sync.ps1 [-Check] [-Verbose]`
- Unix sync: `bash scripts/version/version-sync.sh [--check|-c] [--verbose|-v]`
- Root gate: `just version-sync` and `just version-check`

### 3. Contracts
- `SYNC_TARGETS` must list only current canonical version targets that exist in the working tree.
- If a UI file that used to carry a version label is deleted, moved, or replaced by a package-driven label such as `APP_VERSION_LABEL`, remove the stale target instead of recreating an unused compatibility file.
- Keep PowerShell and Bash target lists behaviorally aligned.
- Keep `scripts/version/version-sync.Tests.ps1`, `scripts/version/version-sync.bats`, and `scripts/README.md` aligned with the active target list.

### 4. Validation & Error Matrix
- Target path listed but missing -> fail in `just version-check` and `just version-sync`.
- Target path exists but has neither `CCR UI v...` nor package-driven version marker -> fail while extracting UI version.
- Bash and PowerShell target lists differ -> cross-platform CI drift risk; update both before accepting the change.
- Tests or README still create/document a removed target -> stale contract; update them with the script change.

### 5. Good/Base/Bad Cases
- Good: `ccr-ui/src/config/appMeta.ts` is the package-backed UI version target; `src/shell/MainLayoutChrome.tsx` and `Titlebar.tsx` consume its version.
- Base: `appMeta.ts` uses `APP_VERSION_LABEL`; both sync scripts retain the legacy parser tag `vue` for that source target and do not rewrite the package-backed value.
- Bad (historical migration example): leaving `ccr-ui/src/layouts/MainLayout.vue` in `SYNC_TARGETS` after the file is deleted.

### 6. Tests Required
- Run `./scripts/version/version-sync.ps1 -Check -Verbose` after editing Windows sync behavior.
- Run `bash -n scripts/version/version-sync.sh` after editing Bash sync behavior.
- Run `just version-check` to prove the first `just ci` step succeeds without rewriting version files. Run `just version-sync` only when a version update is authorized.
- Run final `just ci` for release-ready version-sync changes.

### 7. Wrong vs Correct
#### Wrong
```powershell
@{ Name = "ui-legacy"; Path = "ccr-ui\src\layouts\MainLayout.vue"; Type = "vue" }
Test-RequiredFile $LEGACY_MAIN_LAYOUT
```

#### Correct
```powershell
@{ Name = "ui-component"; Path = "ccr-ui\src\config\appMeta.ts"; Type = "vue" }
Test-RequiredFile $COMPONENT_MAIN_LAYOUT
```

## Source lines L54-L124

## Scenario: Root/Tauri dependency drift gate

### 1. Scope / Trigger
- Trigger: changing root `[workspace.dependencies]`, `ccr-ui/src-tauri/Cargo.toml`, or dependency-governance scripts.
- Security updates to either `Cargo.lock` also require verification of both independent lockfiles.
- Applies because the Tauri app currently remains an independent workspace/manifest while depending on the same Rust ecosystem as the root workspace.
- The gate detects new repeated dependency version drift before CI or release builds silently diverge.

### 2. Signatures
- Canonical validator: `python scripts/drift/check_dependency_drift.py [--verbose]`
- Metadata: `scripts/drift/dependency-drift-allowlist.json`
- Development toolchain source: `rust-toolchain.toml` with channel `1.98.0`; crate manifests independently keep `rust-version = "1.95"`.
- Hosted MSRV gate: `.github/workflows/ci.yml` job `workspace-msrv` explicitly uses Rust `1.95.0` and runs `cargo check --workspace --all-targets --all-features`.
- Root gate: `just version-check` must invoke the Python dependency drift checker for Windows, Linux, and macOS recipe variants.
- Lockfile security checks: `cargo audit --file Cargo.lock` and `cargo audit --file ccr-ui/src-tauri/Cargo.lock`. Record the advisory database revision and whether fetching succeeded.

### 3. Contracts
- Parse root `Cargo.toml` `[workspace.dependencies]`.
- Parse `ccr-ui/src-tauri/Cargo.toml` `[dependencies]`.
- Compare only dependencies repeated in both manifests.
- Matching versions pass.
- Non-matching versions pass only if the JSON allowlist supplies non-empty `owner`, `rationale`, and ISO `expires` fields.
- Expired, duplicate, stale, or ownerless exceptions fail; active exceptions must not exceed `max_active_exceptions` (currently 3).
- A stale allowlist entry fails when the dependency disappears from either manifest or the two versions become equal.
- Every crate plus the independent Tauri manifest must declare MSRV 1.95; upgrading the development/ordinary CI compiler must not rewrite these declarations.
- Ordinary local, hosted, and release Rust jobs use the `1.98.0` development pin. The dedicated root `workspace-msrv` job stays on `1.95.0`, and `root-required` must fail closed when it fails.
- Windows recipes call `python`; Linux and macOS recipes call `python3`. Both invoke the same Python validator.
- For a narrow security update, record each lockfile's before/after package versions, checksums, dependency lists, and package count. Verify checksums against the registry. Preserve existing audit warnings and failed attempts; do not add ignores to obtain a passing result.
- A vulnerable package in a lockfile does not establish that the package is active in the built feature/target graph. Record that boundary separately from the audit result. A successful development-toolchain check does not replace the MSRV check.

### 4. Validation & Error Matrix
- Root Cargo manifest missing -> fail.
- Tauri Cargo manifest missing -> fail.
- `[workspace.dependencies]` cannot be parsed -> fail.
- Tauri `[dependencies]` cannot be parsed -> fail.
- Repeated dependency versions differ and dependency is not allowlisted -> fail.
- Allowlisted dependency is no longer repeated -> fail.
- Allowlisted dependency versions now match -> fail until the allowlist entry is removed.
- Exception owner/rationale missing, expiry invalid/past, or active count above 3 -> fail.
- Crate MSRV differs from 1.95 or development toolchain differs from 1.98.0 -> fail.
- An ordinary workflow uses a Rust version other than 1.98.0, the explicit MSRV job is missing/not 1.95.0, or `root-required` omits it -> fail.
- Only one independent lockfile audited, fetch failed, or the checked lockfile differs from the delivered file -> incomplete security evidence.

### 5. Good/Base/Bad Cases
- Good: `serde` repeats with the same version in both manifests.
- Good: `rust-toolchain.toml` and ordinary CI use 1.98.0 while crate `rust-version` remains 1.95 and the dedicated MSRV job compiles all workspace targets on 1.95.0.
- Base: `toml` differs with owner `desktop-platform`, a migration rationale, and a future expiry while parser compatibility is being evaluated.
- Bad: adding `anyhow = "1.0.90"` to Tauri while root workspace uses `1.0.102` without an allowlist reason.
- Bad: leaving an allowlist entry after the Tauri version is aligned with root.
- Bad: duplicating parsing logic outside the Python validator or leaving an exception without an accountable owner and expiry.
- Bad: raising `rust-version` because the development compiler was upgraded, or changing every hosted job to 1.98.0 without retaining an executable 1.95.0 MSRV gate.

### 6. Tests Required
- Run `python scripts/drift/check_dependency_drift.py --verbose` after validator, manifest, toolchain, or exception changes.
- Run `python -m unittest scripts.drift.test_check_dependency_drift scripts.ci.test_check_workflow_governance` after changing the development/MSRV split.
- Run focused Rust 1.98 fmt/clippy/test plus Rust 1.95 check/test for every touched Rust surface; a successful 1.98 build is not MSRV evidence.
- Run `just version-check` to prove the root gate includes version, doc, and dependency drift checks.
- Run `git diff --check` before commit.

### 7. Wrong vs Correct
#### Wrong
```bash
# Independent platform parser with a separate inline allowlist.
declare -A ALLOWED_DRIFT=([toml]="temporary")
```

#### Correct
```bash
python3 scripts/drift/check_dependency_drift.py "$@"
```

## Source lines L125-L178

## Scenario: Docs package manager and lock authority

### 1. Scope / Trigger
- Trigger: changing `docs/package.json`, docs dependency installation/build recipes, `docs/README.md`, or `scripts/drift/check_doc_drift.py`.
- Applies because the VitePress site uses Bun in CI and `docs/bun.lock` is its only maintained dependency resolution.

### 2. Signatures
- Manifest: `docs/package.json`.
- Canonical Bun toolchain pin: `ccr-ui/package.json#packageManager`; the docs manifest mirrors it exactly.
- Lock authority: `docs/bun.lock`.
- Focused build: `cd docs && bun install --frozen-lockfile && bun run audit && bun run build`.
- Root recipes: `just docs` and `just docs-check`.
- Drift gate: `python scripts/drift/check_doc_drift.py --verbose`, included by `just version-check`.

### 3. Contracts
- Do not create or maintain `docs/package-lock.json`; npm is not a second docs dependency authority.
- `docs/package.json#packageManager` must exactly match the canonical exact-semver Bun pin in `ccr-ui/package.json#packageManager`.
- Root docs recipes install from `docs/bun.lock` with `bun install --frozen-lockfile` before building or auditing.
- `docs/README.md` must state that `docs/bun.lock` is the only maintained docs dependency lockfile.
- Keep npm usage under `ccr-vscode/` independent; the docs lock policy must not rewrite the extension's npm workflow.

### 4. Validation & Error Matrix
- `docs/bun.lock` missing -> docs drift gate fails.
- `docs/package-lock.json` exists, including as ignored local residue -> docs drift gate fails.
- README omits the lock authority statement -> docs drift gate fails.
- Either package manager field is not exact `bun@x.y.z`, or the docs mirror differs from the UI canonical pin -> docs drift gate fails.
- Manifest and lock disagree -> frozen Bun install fails without rewriting the lock.
- A docs recipe invokes npm -> reject as a second resolver path even if the current build succeeds.

### 5. Good/Base/Bad Cases
- Good: `just docs` runs a frozen Bun install followed by `bun run build`.
- Base: `docs/.gitignore` keeps `package-lock.json` ignored while the drift gate also rejects a locally generated copy.
- Bad: `npm install` in a root docs recipe, or keeping both `docs/bun.lock` and `docs/package-lock.json` as dependency authorities.
- Bad: removing npm commands from `ccr-vscode/`; that package intentionally keeps its own npm lock authority.

### 6. Tests Required
- `python -m unittest scripts.drift.test_check_doc_drift` covers missing docs files, forbidden docs npm lock, missing README authority, and the happy path.
- `python scripts/drift/check_doc_drift.py --verbose` and `just version-check` prove the repository state follows the lock policy.
- `cd docs && bun install --frozen-lockfile && bun run audit && bun run build` and `just docs-check` validate the focused docs path.
- Run final `just ci` for release-ready governance changes.

### 7. Wrong vs Correct
#### Wrong
```just
docs:
    cd docs && npm install && npm run build
```

#### Correct
```just
docs:
    cd docs && bun install --frozen-lockfile && bun run build
```

## Source lines L342-L462

## Scenario: Hosted workflow, tool pin, and coverage governance

### 1. Scope / Trigger
- Trigger: changing `.github/workflows/**`, `justfile`, coverage recipes, action pins, test parallelism, or the generated Tauri command inventory.
- Applies because hosted checks must call repository-owned gates and must not silently weaken local acceptance.

### 2. Signatures
- Governance validator: `python scripts/ci/check_workflow_governance.py`.
- Relevance resolver: `python scripts/ci/ci_surface_policy.py --surface <root|frontend|tauri|vscode> --base <sha> --head <sha>`; writes `relevant=true|false` to `$GITHUB_OUTPUT`.
- Coverage validator: `python scripts/quality/check_coverage_thresholds.py <report.json> [--overall 70] --gateway 85 --gateway-pattern <path>`.
- Frontend audit validator: `cd ccr-ui && bun run audit:dependencies`; policy: `ccr-ui/scripts/frontend-audit-allowlist.json`.
- Local recipes: `workflow-governance-check`, `dependency-governance-check`, `frontend-audit`, `ci-governance-check`, `coverage-rust`, `coverage-tauri`, `frontend-coverage`, `vscode-coverage`, `tauri-ci`, and `vscode-ci`.
- Hosted files: `ci.yml`, `frontend-ci.yml`, `tauri-rust-ci.yml`, and `vscode-ci.yml`.

### 3. Contracts
- Third-party `uses:` references are immutable 40-character commit SHAs; version comments are review hints, not executable refs.
- Workflow YAML must reject duplicate mapping keys. Pull requests to `main`, `develop`, and `dev` always instantiate the four stable required contexts; product path filters live only in `SURFACE_PATHS`. Quality workflows (`ci.yml`, `frontend-ci.yml`, `tauri-rust-ci.yml`, `vscode-ci.yml`) are `pull_request`-only. `release.yml` remains tag-push only.
- Stable branch-protection contexts are `Root Workspace Required`, `Vue and Docs Required`, `Tauri Linux Required`, and `VS Code Required`. Each is a final aggregator: irrelevant changes pass after change detection, while relevant changes pass only when every heavy validation, coverage, audit, and platform matrix dependency succeeds.
- Change detection checks out full history and uses the pull request's merge-base diff (`base...head`). Changing `scripts/ci/ci_surface_policy.py` makes all four surfaces relevant. Detection failure must fail the aggregator; an empty or failed relevance output must never silently skip a required validation.
- Rust development, ordinary CI, and release jobs are pinned to 1.98.0; crate MSRV stays at 1.95 and `ci.yml` retains a required Rust 1.95.0 workspace check. Bun is pinned to 1.4.0, Node to 24.20.0, just to 1.58.0, and cargo-llvm-cov to 0.9.0.
- `ccr-ui/package.json#packageManager` is the canonical Bun version source. `docs/package.json` and the root, frontend, Tauri, and release workflow `bun-version` inputs must mirror that exact pin; governance rejects missing, duplicate, or divergent workflow inputs. The root quality job installs Bun for the OMP contract and Node for Copilot checks.
- Dependabot uses the `bun` ecosystem for `/ccr-ui` and `/docs`, matching their Bun manifests and `bun.lock` files. `/ccr-vscode` keeps `npm` and `package-lock.json`. The existing workflow governance gate checks the directory, declared package manager, lockfile, and update ecosystem together. Missing, duplicate, or mismatched JavaScript mappings fail. Preserve the schedules and Cargo entries. Claude Code, Codex, Grok Build, Kimi Code, and OMP use this same contract. A local configuration check does not prove that a hosted Dependabot update succeeded.
- Root Rust, React frontend, and VS Code line coverage must be at least 70%; root and Tauri process gateways must be at least 85%. The stable hosted context name `Vue and Docs Required` remains unchanged for branch-protection compatibility.
- Tauri uploads its full coverage baseline while the hard security threshold remains the gateway; a broad command-wrapper percentage cannot hide a gateway regression.
- Root workspace tests use default parallelism. `scripts/ci/check_workflow_governance.py` counts `#[serial]` / `#[serial_test::serial]`; current and target counts are both 0.
- Tauri command inventory is generated from `commands/handler_registry.rs`: 340 base / 348 Windows commands across 38 base modules. `ccr-ui/src/api/generated/command-manifest.json` records 278 typed commands and 278 exact wire contracts. Registry count tests intentionally freeze that surface; regenerate and verify the manifest when the surface changes.
- The Tauri Rust gate runs direct Cargo fmt/check/clippy/test plus repository governance recipes. Its Linux job installs canonical Bun 1.4.0 because `tauri-bindings-check` formats and compares generated TypeScript; it does not install the frontend dependency graph.
- Windows/Linux/macOS `just ci` runs `tauri-ci` once: strict binary Clippy, all behavior tests, bindings drift and inventory. Desktop test failure must make the aggregate nonzero.
- Root/Tauri tests and coverage remain parallel with only `--skip export_bindings`; the transactional generator owns exports. Existing thresholds and hosted lanes remain unchanged.
- Windows/Linux/macOS `ci` validates source without running `version-sync` or `fmt`. The aggregate uses `version-check` and `fmt-check`; the repair recipes remain separate, explicit commands. Preserve the same ordered checks and first failure exit on all three platforms.
- The aggregate includes OMP injection tests, shared harness contract tests and validation, Copilot parser tests and validation, frontend advisory checks, and final VSIX inventory validation through `vscode-ci`. `frontend-build` sets `CCR_SKIP_ICON_GENERATION=1` at recipe scope, so validation uses committed icons. Explicit asset generation remains separate.
- `audit` scans both `Cargo.lock` and `ccr-ui/src-tauri/Cargo.lock`. A missing cargo-audit prerequisite fails with installation guidance; the check must not install a global tool. Preserve advisory warnings and database-fetch failures.
- Binding checks restore caller bytes on every result; direct generation retains output only on success. A full `just ci` receipt records command, tool versions, source hashes before/after, and actual executed gates. Rust/Tauri coverage, MSRV, hosted OS lanes, and native client trust remain separate evidence unless the recorded command executed them.
- Fresh checkouts run Tauri Rust compile/test/coverage commands with `.cargo/tauri-ci.toml`, which overrides `frontendDist` to the tracked `ccr-ui/src-tauri/ci-dist/index.html` fixture. Production Tauri builds keep using `ccr-ui/dist`; the fixture must never replace the real `beforeBuildCommand` output in release packaging.
- Hosted frontend dependency audit calls the repository-owned `frontend-audit` recipe and parses Bun's JSON report. Unexpected, expired, duplicate, package-mismatched, or stale advisory exceptions fail closed.
- Frontend advisory exceptions require non-empty owner/rationale, ISO expiry, explicit patched versions, and must stay within `maxActiveExceptions` (currently 0).
- Prefer upstream patched releases within the existing dependency ranges. `ccr-ui/bun.lock` owns resolved transitive versions; `ccr-ui/package.json` currently has no overrides. Change the manifest only when its existing ranges cannot select a compatible fix.
- Claude Code, Codex, Grok Build, Kimi Code, and OMP use the same audit policy. Each security update records advisory IDs, old/new versions, dependency chains, development/runtime scope, and verified registry integrity. Keep production reachability and native client validation as separate evidence fields.
- Audit results are time-specific evidence. If a later audit fails with unchanged lockfile hashes, preserve both receipts and capture advisory publication times. Do not infer that the advisories were newly published. Recheck all affected ranges, parent compatibility, and archive integrity before a patch; rerun the failed aggregate after the scoped checks pass.
- Bun manifests use only top-level overrides. Do not force one `brace-expansion` major across `minimatch` 3.x/9.x/10.x: 5.x exports `{ expand }`, while the legacy consumers require the module itself as a function. When a nested copy needs a patched release, bump that lockfile path inside its existing major instead of adding a Bun patch or alias.

### 4. Validation & Error Matrix
- Mutable action tag, duplicate YAML key, missing workflow, missing branch/relevance policy, PR-level `paths` filter, or missing local recipe -> governance check fails.
- Relevant product job fails/skips/cancels -> its stable required aggregator fails.
- Irrelevant product change -> heavy jobs skip, but the stable required aggregator completes successfully so branch protection never waits for a missing context.
- Root/React/VS Code line coverage below 70 -> corresponding coverage recipe fails.
- Root/Tauri gateway below 85 or gateway path not found -> coverage validator fails closed.
- Global `--test-threads=1` or serial annotation count above 0 -> governance check fails.
- Handler inventory differs from registry -> `command_inventory_document_matches_registry` fails.
- Tauri Rust gate omits `.cargo/tauri-ci.toml`, or the tracked `ci-dist/index.html` fixture is missing -> a fresh checkout may fail in `tauri::generate_context!()` before tests run.
- Tauri Linux validation omits canonical Bun 1.4.0, or any governed workflow/docs manifest drifts from the canonical UI pin -> governance fails before accepting generated bindings or frontend builds.
- Unexpected high advisory, leftover `patchedDependencies` while the allowlist is empty, or stale/expired frontend exception -> `bun run audit:dependencies` fails.
- Required branch protection not readable/configured -> local files may pass, but repository-setting evidence remains `UNVERIFIED`.

### 5. Good/Base/Bad Cases
- Good: hosted workflow calls `just coverage-rust`; local and hosted execute the same threshold script.
- Good: `just tauri-ci` succeeds in a clean worktree with no ignored `ccr-ui/dist` directory because Cargo receives the CI-only frontend fixture config.
- Good: `tauri-linux-required` installs pinned Bun before `just tauri-ci`, so the bindings drift gate runs in a fresh hosted checkout without installing frontend packages.
- Good: a docs-only PR creates all four required contexts but runs only the frontend heavy gate; a `ccr-vscode/**` PR runs VS Code validation/coverage before `VS Code Required` succeeds.
- Base: Tauri overall coverage is reported separately while its security gateway remains above 85%.
- Base: each nested `brace-expansion` resolution keeps its existing major and satisfies every resolved parent range. Verify the patched release against the current advisory snapshot; `bun audit --json` is empty and `maxActiveExceptions` remains 0.
- Bad: keeping a PR-level `paths` filter on a branch-protected workflow, because an unrelated PR never creates the required context and remains pending forever.
- Bad: copying lint/test commands into workflow YAML, pinning `actions/checkout@v6`, lowering the gateway threshold, or globally overriding all `brace-expansion` consumers to 5.x.
- Bad: creating an ignored `ccr-ui/dist` locally before testing and treating that residue-dependent pass as fresh-checkout evidence.

### 6. Tests Required
- `python -m unittest scripts.ci.test_check_workflow_governance` -> path matching, event parsing, and duplicate-key cases pass.
- `python -m unittest scripts.ci.test_architecture_contract_gates` runs the actual aggregate graph with failing/successful Cargo fixtures and checks platform lists/export ownership. The fixture proves propagation, not real Rust suite success.
- The workflow-governance unit suite asserts that root/UI Tauri Cargo recipes use `.cargo/tauri-ci.toml`, the tracked CI frontend fixture exists, every governed manifest/workflow mirrors canonical Bun 1.4.0, and all four Node setup inputs are exactly Node 24.20.0. Controlled command stubs verify each OS aggregate order, first failure propagation, and omission guards without requiring a complete host toolchain.
- `python scripts/ci/check_workflow_governance.py` -> immutable action references, stable relevance routing, required Rust 1.95 MSRV lane, root/Tauri Bun setup, and serial-only count 0.
- `just ci-governance-check` -> dependency, workflow, and handler inventory gates pass.
- `cd ccr-ui && bun install --frozen-lockfile && bun run audit:dependencies` -> the lockfile bytes stay unchanged during installation, only approved lock nodes differ from the pre-fix snapshot, the audit JSON is empty, and the allowlist has 0/0 active exceptions.
- `cd ccr-ui && bun run test:smoke -- tests/quality/frontend-dependency-audit.smoke.test.ts` -> exception limit, expiry, package match, stale detection, and GHSA extraction pass.
- `just coverage-rust`, `just coverage-tauri`, `just frontend-coverage`, and `just vscode-coverage` -> configured line/gateway thresholds pass.
- `just tauri-ci`, `just vscode-ci`, and final `just ci` when unrelated workspace metadata is clean.
- Inspect an actual relevant PR across Linux, Windows, and macOS, then query `main` and `dev` protection for all four exact context names; do not infer remote configuration from workflow files.

### 7. Wrong vs Correct
#### Wrong
```yaml
on:
  pull_request:
    paths: ['ccr-vscode/**']
jobs:
  test:
    steps:
      - uses: actions/checkout@v6
      - run: cargo test --workspace -- --test-threads=1
```

```json
{"overrides":{"brace-expansion":"5.0.8"}}
```

#### Correct
```yaml
on:
  pull_request:
    branches: [main, develop, dev]
# Heavy-job relevance comes from scripts/ci/ci_surface_policy.py; the required
# aggregator is always created.
jobs:
  root-required:
    name: Root Workspace Required
    if: ${{ always() }}
    needs: [changes, workspace-quality]
    runs-on: ubuntu-24.04
    steps:
      - run: test "${{ needs.workspace-quality.result }}" = success
```

```sh
cd ccr-ui
bun pm why undici
bun install --frozen-lockfile
cd ..
just frontend-audit
just frontend-check
just frontend-coverage
```
