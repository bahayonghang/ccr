# Dependency Governance

> Root/Tauri dependency drift checks for the independent desktop manifest.

---

## Scenario: Version sync target registry

### 1. Scope / Trigger
- Trigger: changing version surfaces, UI shell version labels, `scripts/version/version-sync.ps1`, `scripts/version/version-sync.sh`, or paths listed in either script's `SYNC_TARGETS`.
- Applies because `just ci` starts with `just version-sync`; stale required paths fail the full gate before Rust or frontend checks run.

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
- Target path listed but missing -> fail in `just version-sync`.
- Target path exists but has neither `CCR UI v...` nor package-driven version marker -> fail while extracting UI version.
- Bash and PowerShell target lists differ -> cross-platform CI drift risk; update both before accepting the change.
- Tests or README still create/document a removed target -> stale contract; update them with the script change.

### 5. Good/Base/Bad Cases
- Good: `ccr-ui/src/components/MainLayout.vue` is the only MainLayout version target after the legacy `src/layouts/MainLayout.vue` file is removed.
- Base: a Vue component uses `APP_VERSION_LABEL`; the script accepts it as package-version-backed and does not rewrite the component.
- Bad: leaving `ccr-ui/src/layouts/MainLayout.vue` in `SYNC_TARGETS` after the file is deleted.

### 6. Tests Required
- Run `./scripts/version/version-sync.ps1 -Check -Verbose` after editing Windows sync behavior.
- Run `bash -n scripts/version/version-sync.sh` after editing Bash sync behavior.
- Run `just version-sync` to prove the first `just ci` step no longer fails.
- Run final `just ci` for release-ready version-sync changes.

### 7. Wrong vs Correct
#### Wrong
```powershell
@{ Name = "ui-legacy"; Path = "ccr-ui\src\layouts\MainLayout.vue"; Type = "vue" }
Test-RequiredFile $LEGACY_MAIN_LAYOUT
```

#### Correct
```powershell
@{ Name = "ui-component"; Path = "ccr-ui\src\components\MainLayout.vue"; Type = "vue" }
Test-RequiredFile $COMPONENT_MAIN_LAYOUT
```

## Scenario: Root/Tauri dependency drift gate

### 1. Scope / Trigger
- Trigger: changing root `[workspace.dependencies]`, `ccr-ui/src-tauri/Cargo.toml`, or dependency-governance scripts.
- Applies because the Tauri app currently remains an independent workspace/manifest while depending on the same Rust ecosystem as the root workspace.
- The gate detects new repeated dependency version drift before CI or release builds silently diverge.

### 2. Signatures
- Canonical validator: `python scripts/drift/check_dependency_drift.py [--verbose]`
- Metadata: `scripts/drift/dependency-drift-allowlist.json`
- Toolchain source: `rust-toolchain.toml` with channel `1.95.0`; crate manifests use `rust-version = "1.95"`.
- Root gate: `just version-check` must invoke the Python dependency drift checker for Windows, Linux, and macOS recipe variants.

### 3. Contracts
- Parse root `Cargo.toml` `[workspace.dependencies]`.
- Parse `ccr-ui/src-tauri/Cargo.toml` `[dependencies]`.
- Compare only dependencies repeated in both manifests.
- Matching versions pass.
- Non-matching versions pass only if the JSON allowlist supplies non-empty `owner`, `rationale`, and ISO `expires` fields.
- Expired, duplicate, stale, or ownerless exceptions fail; active exceptions must not exceed `max_active_exceptions` (currently 3).
- A stale allowlist entry fails when the dependency disappears from either manifest or the two versions become equal.
- Every crate plus the independent Tauri manifest must declare MSRV 1.95, matching the pinned 1.95.0 toolchain patch.
- Windows recipes call `python`; Linux and macOS recipes call `python3`. Both invoke the same Python validator.

### 4. Validation & Error Matrix
- Root Cargo manifest missing -> fail.
- Tauri Cargo manifest missing -> fail.
- `[workspace.dependencies]` cannot be parsed -> fail.
- Tauri `[dependencies]` cannot be parsed -> fail.
- Repeated dependency versions differ and dependency is not allowlisted -> fail.
- Allowlisted dependency is no longer repeated -> fail.
- Allowlisted dependency versions now match -> fail until the allowlist entry is removed.
- Exception owner/rationale missing, expiry invalid/past, or active count above 3 -> fail.
- Crate MSRV differs from 1.95 or toolchain differs from 1.95.0 -> fail.

### 5. Good/Base/Bad Cases
- Good: `serde` repeats with the same version in both manifests.
- Base: `toml` differs with owner `desktop-platform`, a migration rationale, and a future expiry while parser compatibility is being evaluated.
- Bad: adding `anyhow = "1.0.90"` to Tauri while root workspace uses `1.0.102` without an allowlist reason.
- Bad: leaving an allowlist entry after the Tauri version is aligned with root.
- Bad: duplicating parsing logic outside the Python validator or leaving an exception without an accountable owner and expiry.

### 6. Tests Required
- Run `python scripts/drift/check_dependency_drift.py --verbose` after validator, manifest, toolchain, or exception changes.
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

## Scenario: SQLite native link compatibility

### 1. Scope / Trigger
- Trigger: changing `rusqlite`, `r2d2_sqlite`, `ccr-core`, `ccr-db`, `ccr-checkin`, or the Tauri manifest's direct SQLite dependency.
- Applies because `rusqlite` and `r2d2_sqlite` both resolve through `libsqlite3-sys`, whose `links = "sqlite3"` contract allows only one version in a Cargo dependency graph.

### 2. Signatures
- Root workspace dependency: `Cargo.toml` `[workspace.dependencies]` has `rusqlite = { version = "...", features = ["bundled"] }` and `r2d2_sqlite = "..."`.
- Tauri dependency: `ccr-ui/src-tauri/Cargo.toml` has a direct `rusqlite = { version = "...", features = ["bundled"] }`.
- Current compatibility pair: `r2d2_sqlite = "0.34.0"` pairs with `rusqlite = "0.39.0"` / `libsqlite3-sys = "0.37.0"`.

### 3. Contracts
- Keep every direct `rusqlite` requirement compatible with the `rusqlite` version required by `r2d2_sqlite`.
- Do not bump root or Tauri `rusqlite` to `0.40.x` while `r2d2_sqlite = "0.34.0"` remains in the workspace.
- If `rusqlite` must move to a newer minor version, first replace or remove `r2d2_sqlite`, or verify that a new `r2d2_sqlite` release depends on the same `rusqlite` minor line.

### 4. Validation & Error Matrix
- Root/Tauri `rusqlite` differs -> dependency drift script fails unless explicitly reviewed and allowlisted.
- Root `rusqlite = 0.40.x` with `r2d2_sqlite = 0.34.0` -> Cargo fails dependency resolution with duplicate `links = "sqlite3"`.
- Tauri `rusqlite = 0.40.x` alone can still break full desktop resolution when path crates and desktop dependencies share the same graph.

### 5. Good/Base/Bad Cases
- Good: root and Tauri both use `rusqlite = "0.39.0"` while workspace keeps `r2d2_sqlite = "0.34.0"`.
- Base: root and Tauri both bump `rusqlite` only after a compatible pool dependency is selected and verified.
- Bad: bumping only direct `rusqlite` to `0.40.1` because the drift check passes, while `r2d2_sqlite` still pulls `rusqlite 0.39`.

### 6. Tests Required
- Run `cargo metadata --no-deps --format-version 1` after dependency edits to catch immediate resolver failures.
- Run `just lint-strict` to exercise the strict workspace Cargo graph.
- Run `just version-check` when root/Tauri dependency versions changed.
- Run final `just ci` for release-ready dependency edits.

### 7. Wrong vs Correct
#### Wrong
```toml
rusqlite = { version = "0.40.1", features = ["bundled"] }
r2d2_sqlite = "0.34.0"
```

#### Correct
```toml
rusqlite = { version = "0.39.0", features = ["bundled"] }
r2d2_sqlite = "0.34.0"
```

## Scenario: Tauri JavaScript and Rust version alignment

### 1. Scope / Trigger
- Trigger: changing `ccr-ui/package.json`, `ccr-ui/bun.lock`, `ccr-ui/src-tauri/Cargo.toml`, or any recipe that runs `tauri build`.
- Applies because Tauri's build-time package check compares the Rust `tauri` crate with installed JavaScript Tauri packages before bundling installers.

### 2. Signatures
- JavaScript runtime dependency: `ccr-ui/package.json` `dependencies["@tauri-apps/api"]`.
- JavaScript CLI dependency: `ccr-ui/package.json` `devDependencies["@tauri-apps/cli"]`.
- Rust runtime dependency: `ccr-ui/src-tauri/Cargo.toml` `tauri = { version = "=<major>.<minor>.<patch>", ... }`.
- Verification command: `cd ccr-ui && bun run tauri info`.

### 3. Contracts
- Keep `@tauri-apps/api` on the same major/minor line as the Rust `tauri` crate.
- Keep `@tauri-apps/cli` on the same major/minor line as the Rust `tauri` crate.
- Patch versions do not have to match exactly; npm and crates.io may publish different patch sets for the same minor line.
- Refresh `ccr-ui/bun.lock` with `bun install` after changing package versions.

### 4. Validation & Error Matrix
- Rust `tauri = 2.11.x` with `@tauri-apps/api = 2.10.x` -> `tauri build` fails with "Found version mismatched Tauri packages".
- Pinning `@tauri-apps/api` to a non-existent patch such as `2.11.2` -> `bun install` fails because that package version is unavailable.
- Updating `package.json` without `bun.lock` -> install/build can continue resolving the old JavaScript package version.
- Matching major/minor lines, for example Rust `tauri = 2.11.2` and `@tauri-apps/api = 2.11.0` -> build-time version check passes.

### 5. Good/Base/Bad Cases
- Good: `tauri = "=2.11.2"`, `@tauri-apps/api = "2.11.0"`, and `@tauri-apps/cli = "2.11.2"`.
- Base: CLI patch equals Rust patch while API uses the latest published patch on the same minor line.
- Bad: leaving `@tauri-apps/api = "2.10.1"` after bumping Rust `tauri` to `2.11.x`.
- Bad: guessing that every Rust Tauri patch has a matching `@tauri-apps/api` patch without checking npm availability.

### 6. Tests Required
- Run `cd ccr-ui && bun install` after package version edits.
- Run `cd ccr-ui && bun run tauri info` to inspect resolved Rust and JavaScript Tauri package versions.
- Run `cd ccr-ui && bun run tauri:build` or root `just tauri-build` to prove the bundling path passes the mismatch check.
- Run `git diff --check` before commit.

### 7. Wrong vs Correct
#### Wrong
```json
"@tauri-apps/api": "2.10.1"
```

#### Correct
```json
"@tauri-apps/api": "2.11.0"
```

## Scenario: Unsigned release accepted-risk boundary

### 1. Scope / Trigger
- Trigger: changing `.github/workflows/release.yml`, checksum publication,
  Tauri/VSIX signing configuration, release copy, or updater dependencies.
- Applies because the user explicitly accepts unsigned release artifacts. This
  is a residual risk decision, not proof that publisher identity is verified.

### 2. Signatures
- Release workflow: `.github/workflows/release.yml`, triggered by `v*` tags.
- Integrity artifacts: per-artifact `*.sha256` files generated by `sha256sum`
  or `shasum -a 256`.
- Disabled updater indicators: no `tauri-plugin-updater`,
  `@tauri-apps/plugin-updater`, or Tauri `plugins.updater` configuration.
- Local validation: `actionlint .github/workflows/release.yml` and
  `python scripts/ci/check_workflow_governance.py`.

### 3. Contracts
- macOS, Windows, and VSIX release artifacts may remain unsigned. The workflow
  must not require signing identities, certificate secrets, publisher tools,
  notarization, or attestation as a publication precondition.
- SHA-256 proves file integrity only. Release copy must state that it does not
  authenticate the publisher and must not describe artifacts as signed,
  notarized, attested, or publisher-verified.
- The updater remains disabled while artifacts have no authenticated update
  manifest. Enabling an updater requires a new explicit security decision and
  a verifier whose failure leaves the installed version unchanged.
- Audit finding P2-14 remains `ACCEPTED_RISK`, never `PASS`, while real platform
  and publisher signatures are absent.

### 4. Validation & Error Matrix
- Certificate/sign-tool requirement in the workflow -> reject unless the user
  supplies a new identity model and explicitly reopens signing work.
- Release copy claims publisher identity from a checksum -> reject as false
  authentication.
- Updater dependency or config appears without an authenticated manifest
  verifier -> reject; keep the updater disabled.
- Missing checksum generation or publication -> fail the unsigned release
  integrity baseline.
- Signing identities remain absent -> expected under this accepted-risk model;
  do not convert P2-14 to `PASS`.

### 5. Good/Base/Bad Cases
- Good: unsigned archives and VSIX files ship with SHA-256 files and an explicit
  warning that checksums do not prove publisher identity.
- Base: local development packages remain unsigned and use no updater metadata.
- Bad: calling a checksum-only artifact "signed" or "verified publisher".
- Bad: enabling Tauri updater endpoints because transport checksums exist.

### 6. Tests Required
- `go run github.com/rhysd/actionlint/cmd/actionlint@v1.7.7 .github/workflows/release.yml`
  validates the release workflow when `actionlint` is not installed directly.
- `python -m unittest scripts.ci.test_check_workflow_governance` and
  `python scripts/ci/check_workflow_governance.py` validate pinned actions and
  workflow governance.
- Search manifests/config for `tauri-plugin-updater`,
  `@tauri-apps/plugin-updater`, and `plugins.updater`; all must be absent.
- Run `just ci-governance-check` and final `just ci` before delivery.

### 7. Wrong vs Correct
#### Wrong
```markdown
SHA256 verified: this artifact is signed by the CCR publisher.
```

#### Correct
```markdown
This artifact is unsigned. SHA256 verifies file integrity only and does not
authenticate the publisher; automatic updates remain disabled.
```

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
- Rust is pinned to 1.95.0, Bun to 1.3.10, Node to 24.18.0, just to 1.57.0, and cargo-llvm-cov to 0.8.7.
- Root Rust, Vue, and VS Code line coverage must be at least 70%; root and Tauri process gateways must be at least 85%.
- Tauri uploads its full coverage baseline while the hard security threshold remains the gateway; a broad command-wrapper percentage cannot hide a gateway regression.
- Root workspace tests use default parallelism. `scripts/ci/check_workflow_governance.py` counts `#[serial]` / `#[serial_test::serial]`; current and target counts are both 0.
- Tauri command inventory is generated from the handler registry and freezes 315 base / 323 Windows commands across 30 base modules.
- The Tauri Rust gate runs direct Cargo fmt/check/clippy/test plus repository governance recipes. Its Linux job installs pinned Bun 1.3.10 because `tauri-bindings-check` formats and compares generated TypeScript; it does not install the Vue dependency graph.
- Fresh checkouts run Tauri Rust compile/test/coverage commands with `.cargo/tauri-ci.toml`, which overrides `frontendDist` to the tracked `ccr-ui/src-tauri/ci-dist/index.html` fixture. Production Tauri builds keep using `ccr-ui/dist`; the fixture must never replace the real `beforeBuildCommand` output in release packaging.
- Hosted frontend dependency audit calls the repository-owned `frontend-audit` recipe and parses Bun's JSON report. Unexpected, expired, duplicate, package-mismatched, or stale advisory exceptions fail closed.
- Frontend advisory exceptions require non-empty owner/rationale, ISO expiry, explicit patched versions, and must stay within `maxActiveExceptions` (currently 0).
- Prefer upstream patched releases. Current frontend overrides pin `fast-uri` 3.1.5, `js-yaml` 4.3.1, and `nanoid` 3.3.17; `dompurify` tracks `^3.4.13`. Nested `brace-expansion` copies are lockfile-pinned per major: `1.1.18`, `2.1.4`, and `5.0.9`.
- Bun 1.3.10 supports only top-level overrides. Do not force one `brace-expansion` major across `minimatch` 3.x/9.x/10.x: 5.x exports `{ expand }`, while the legacy consumers require the module itself as a function. When a nested copy needs a patched release, bump that lockfile path inside its existing major instead of adding a Bun patch or alias.

### 4. Validation & Error Matrix
- Mutable action tag, duplicate YAML key, missing workflow, missing branch/relevance policy, PR-level `paths` filter, or missing local recipe -> governance check fails.
- Relevant product job fails/skips/cancels -> its stable required aggregator fails.
- Irrelevant product change -> heavy jobs skip, but the stable required aggregator completes successfully so branch protection never waits for a missing context.
- Root/Vue/VS Code line coverage below 70 -> corresponding coverage recipe fails.
- Root/Tauri gateway below 85 or gateway path not found -> coverage validator fails closed.
- Global `--test-threads=1` or serial annotation count above 0 -> governance check fails.
- Handler inventory differs from registry -> `command_inventory_document_matches_registry` fails.
- Tauri Rust gate omits `.cargo/tauri-ci.toml`, or the tracked `ci-dist/index.html` fixture is missing -> a fresh checkout may fail in `tauri::generate_context!()` before tests run.
- Tauri Linux validation omits pinned Bun 1.3.10 -> `tauri-bindings-check` fails at the TypeScript formatting step even when every Rust test passes.
- Unexpected high advisory, leftover `patchedDependencies` while the allowlist is empty, or stale/expired frontend exception -> `bun run audit:dependencies` fails.
- Required branch protection not readable/configured -> local files may pass, but repository-setting evidence remains `UNVERIFIED`.

### 5. Good/Base/Bad Cases
- Good: hosted workflow calls `just coverage-rust`; local and hosted execute the same threshold script.
- Good: `just tauri-ci` succeeds in a clean worktree with no ignored `ccr-ui/dist` directory because Cargo receives the CI-only frontend fixture config.
- Good: `tauri-linux-required` installs pinned Bun before `just tauri-ci`, so the bindings drift gate runs in a fresh hosted checkout without installing frontend packages.
- Good: a docs-only PR creates all four required contexts but runs only the frontend heavy gate; a `ccr-vscode/**` PR runs VS Code validation/coverage before `VS Code Required` succeeds.
- Base: Tauri overall coverage is reported separately while its security gateway remains above 85%.
- Base: nested `brace-expansion` stays on patched majors `1.1.18` / `2.1.4` / `5.0.9`, `bun audit --json` is empty, and `maxActiveExceptions` remains 0.
- Bad: keeping a PR-level `paths` filter on a branch-protected workflow, because an unrelated PR never creates the required context and remains pending forever.
- Bad: copying lint/test commands into workflow YAML, pinning `actions/checkout@v6`, lowering the gateway threshold, or globally overriding all `brace-expansion` consumers to 5.x.
- Bad: creating an ignored `ccr-ui/dist` locally before testing and treating that residue-dependent pass as fresh-checkout evidence.

### 6. Tests Required
- `python -m unittest scripts.ci.test_check_workflow_governance` -> path matching, event parsing, and duplicate-key cases pass.
- The workflow-governance unit suite asserts that root/UI Tauri Cargo recipes use `.cargo/tauri-ci.toml`, the tracked CI frontend fixture exists, and the Tauri Linux job installs pinned Bun 1.3.10.
- `python scripts/ci/check_workflow_governance.py` -> 52 immutable action references, stable relevance routing, Tauri Linux Bun setup, and serial-only count 0.
- `just ci-governance-check` -> dependency, workflow, and handler inventory gates pass.
- `cd ccr-ui && bun install --frozen-lockfile && bun run audit:dependencies` -> nested `brace-expansion` is 1.1.18/2.1.4/5.0.9, the audit JSON is empty, and the allowlist has 0/0 active exceptions.
- `cd ccr-ui && bun run test:smoke -- tests/frontend-dependency-audit.smoke.test.ts` -> exception limit, expiry, package match, stale detection, and GHSA extraction pass.
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

```json
{
  "dependencies": {"dompurify":"^3.4.13"},
  "overrides": {
    "fast-uri":"3.1.5",
    "js-yaml":"4.3.1",
    "nanoid":"3.3.17"
  }
}
```

## Scenario: internal crates do not depend on the umbrella facade

### 1. Scope / Trigger

- Trigger: adding or changing a dependency in `crates/*/Cargo.toml` or changing
  `scripts/drift/check_dependency_drift.py`.

### 2. Signatures

- Validator: `internal_umbrella_dependents(root) -> list[str]`.
- Gate: `just dependency-governance-check`.

### 3. Contracts

- Recursively inspect normal, development, build, and target-specific
  dependency tables in every internal crate manifest.
- `crates/ccr/Cargo.toml` is the umbrella package itself and is excluded.
- Internal crates import the owning domain crate (`ccr-cli`, `ccr-core`,
  `ccr-db`, and so on), never `ccr`. Any exception requires an explicit path in
  `INTERNAL_UMBRELLA_ALLOWLIST` and a reviewed compatibility rationale.

### 4. Validation & Error Matrix

- Internal manifest declares dependency key `ccr` -> fail with the manifest
  path and narrow-crate instruction.
- Dependency named `ccr-core` or another prefix match -> pass.
- Target-specific `ccr` dev dependency -> fail like a top-level dependency.

### 5. Good/Base/Bad Cases

- Good: an integration test imports `ccr_cli::services` directly.
- Base: the root `ccr` facade depends on its domain crates.
- Bad: an internal crate adds `ccr = { path = "../ccr" }` for convenience.

### 6. Tests Required

- `python -m unittest scripts.drift.test_check_dependency_drift` covers direct,
  target-specific, prefix, and root-facade cases.
- Run `just dependency-governance-check` and `just version-check`.

### 7. Wrong vs Correct

#### Wrong

```toml
[dev-dependencies]
ccr = { path = "../ccr" }
```

#### Correct

```toml
[dev-dependencies]
ccr-cli = { workspace = true }
```
