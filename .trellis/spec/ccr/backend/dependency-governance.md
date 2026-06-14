# Dependency Governance

> Root/Tauri dependency drift checks for the independent desktop manifest.

---

## Scenario: Version sync target registry

### 1. Scope / Trigger
- Trigger: changing version surfaces, UI shell version labels, `scripts/version-sync.ps1`, `scripts/version-sync.sh`, or paths listed in either script's `SYNC_TARGETS`.
- Applies because `just ci` starts with `just version-sync`; stale required paths fail the full gate before Rust or frontend checks run.

### 2. Signatures
- Windows sync: `scripts/version-sync.ps1 [-Check] [-Verbose]`
- Unix sync: `bash scripts/version-sync.sh [--check|-c] [--verbose|-v]`
- Root gate: `just version-sync` and `just version-check`

### 3. Contracts
- `SYNC_TARGETS` must list only current canonical version targets that exist in the working tree.
- If a UI file that used to carry a version label is deleted, moved, or replaced by a package-driven label such as `APP_VERSION_LABEL`, remove the stale target instead of recreating an unused compatibility file.
- Keep PowerShell and Bash target lists behaviorally aligned.
- Keep `scripts/version-sync.Tests.ps1`, `scripts/version-sync.bats`, and `scripts/README.md` aligned with the active target list.

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
- Run `./scripts/version-sync.ps1 -Check -Verbose` after editing Windows sync behavior.
- Run `bash -n scripts/version-sync.sh` after editing Bash sync behavior.
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
- Windows: `scripts/check-dependency-drift.ps1 [-Verbose]`
- Unix: `bash scripts/check-dependency-drift.sh [--verbose|-v]`
- Root gate: `just version-check` must invoke the dependency drift script for Windows, Linux, and macOS recipe variants.

### 3. Contracts
- Parse root `Cargo.toml` `[workspace.dependencies]`.
- Parse `ccr-ui/src-tauri/Cargo.toml` `[dependencies]`.
- Compare only dependencies repeated in both manifests.
- Matching versions pass.
- Non-matching versions pass only if explicitly allowlisted in the script with a human-readable reason.
- A stale allowlist entry fails when the dependency disappears from either manifest or the two versions become equal.
- Bash implementation must handle CRLF Cargo manifests and avoid Bash 4-only features so macOS default Bash can run the check.

### 4. Validation & Error Matrix
- Root Cargo manifest missing -> fail.
- Tauri Cargo manifest missing -> fail.
- `[workspace.dependencies]` cannot be parsed -> fail.
- Tauri `[dependencies]` cannot be parsed -> fail.
- Repeated dependency versions differ and dependency is not allowlisted -> fail.
- Allowlisted dependency is no longer repeated -> fail.
- Allowlisted dependency versions now match -> fail until the allowlist entry is removed.

### 5. Good/Base/Bad Cases
- Good: `serde` repeats with the same version in both manifests.
- Base: `tokio` differs but remains allowlisted with a reason while desktop runtime compatibility is being evaluated.
- Bad: adding `anyhow = "1.0.90"` to Tauri while root workspace uses `1.0.102` without an allowlist reason.
- Bad: leaving an allowlist entry after the Tauri version is aligned with root.
- Bad: using Bash associative arrays or `mapfile`, which breaks macOS default Bash.

### 6. Tests Required
- Run `bash -n scripts/check-dependency-drift.sh` after editing the Bash script.
- Run `bash scripts/check-dependency-drift.sh --verbose` in a Windows working tree to prove CRLF manifest parsing.
- Run `./scripts/check-dependency-drift.ps1 -Verbose` for the Windows path.
- Run `just version-check` to prove the root gate includes version, doc, and dependency drift checks.
- Run `git diff --check` before commit.

### 7. Wrong vs Correct
#### Wrong
```bash
awk '$0 == "[workspace.dependencies]" { in_section=1 }' Cargo.toml
mapfile -t deps < <(extract_deps)
declare -A versions=()
```

#### Correct
```bash
awk '{ line=$0; sub(/\015$/, "", line) } ...' Cargo.toml
while IFS=$'\t' read -r name version; do
  # Bash 3-compatible processing
  :
done <<EOF
$DEPS
EOF
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
