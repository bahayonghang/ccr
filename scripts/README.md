# CCR Scripts

Root `scripts/` holds repository-owned maintenance gates. Duty scripts live in
`version/`, `drift/`, `ci/`, and `quality/`. The root layer only has this
catalog, `scripts/__init__.py`, and `scripts/common.py` (`REPO_ROOT`).

Workspace layout used by these scripts:

- Installable CLI crate: `crates/ccr`
- Database crate: `crates/ccr-db`
- Shared types crate: `crates/ccr-types`
- UI project: `ccr-ui`
- VS Code extension: `ccr-vscode`
- Docs, scripts, and examples: root `docs/`, `scripts/`, `examples/`
- Collected artifacts: root `outputs/` when present

> The root `Cargo.toml` is the workspace manifest, not a `cargo install --path`
> target. Install from source with `cargo install --path crates/ccr`.

## version/

Keeps workspace crate and UI version labels aligned with the root
`Cargo.toml`.

`SYNC_TARGETS` currently covers:

- `crates/ccr-types/Cargo.toml`
- `crates/ccr-db/Cargo.toml`
- `ccr-ui/package.json`
- `ccr-ui/src-tauri/Cargo.toml`
- `ccr-ui/src-tauri/tauri.conf.json`
- `ccr-ui/src/config/appMeta.ts`
- `ccr-vscode/package.json`

```bash
just version-sync
just version-check
bash scripts/version/version-sync.sh --check --verbose
```

```powershell
.\scripts\version\version-sync.ps1 -Check -Verbose
```

`version-sync.bats` and `version-sync.Tests.ps1` live beside the scripts.
Run them with `just test-scripts` when Bats or Pester is installed.

## drift/

Single Python implementations for dependency drift and documentation drift.

- `check_dependency_drift.py` compares root `[workspace.dependencies]` with
  `ccr-ui/src-tauri/Cargo.toml`, validates
  `scripts/drift/dependency-drift-allowlist.json`, checks MSRV, and rejects
  internal crate dependencies on the umbrella `ccr` facade.
- `check_doc_drift.py` checks `ccr-ui/README.md`, Bun-only lock policy, and
  Tauri MSRV / edition facts.

```powershell
python scripts/drift/check_dependency_drift.py --verbose
python scripts/drift/check_doc_drift.py --verbose
python -m unittest scripts.drift.test_check_dependency_drift
python -m unittest scripts.drift.test_check_doc_drift
just version-check
just dependency-governance-check
```

## ci/

Hosted workflow governance and product-surface relevance.

- `ci_surface_policy.py` decides whether a pull request is relevant for
  `root`, `frontend`, `tauri`, or `vscode`. Changing
  `scripts/ci/ci_surface_policy.py` makes all four surfaces relevant.
  `frontend` and `vscode` match that exact path; they do not use `scripts/**`.
- `check_workflow_governance.py` checks action pins, pull-request-only quality
  workflows, required aggregators, just recipes, coverage policy, and serial
  test annotations.

```powershell
python scripts/ci/ci_surface_policy.py --surface root --base <sha> --head <sha>
python scripts/ci/check_workflow_governance.py
python -m unittest scripts.ci.test_check_workflow_governance
just workflow-governance-check
```

## quality/

Formatting, secret-write policy, coverage thresholds, and Copilot assets.

- `check_json_format.py` formats the explicit human-authored JSON inventory.
- `check_secret_writes.py` rejects direct async writes and
  `AtomicWriter` chains without `.secret(true)` in credential modules.
- `check_coverage_thresholds.py` enforces llvm-cov overall / gateway lines.
- `check-copilot-assets.mjs` checks Copilot workspace files and naming.

```powershell
python scripts/quality/check_json_format.py
python scripts/quality/check_json_format.py --write
python -m unittest scripts.quality.test_check_json_format
python scripts/quality/check_secret_writes.py
node scripts/quality/check-copilot-assets.mjs
just json-format-check
just secret-write-check
just copilot-check
```

## Recommended flow

```bash
# 1. Change the version source
vim Cargo.toml

# 2. Check sync targets
just version-check

# 3. Repair versions when needed
just version-sync
```

## Maintenance

- Keep Bash and PowerShell `version-sync` target lists aligned.
- When a directory moves, update this catalog before changing callers.
- Collect pipeline artifacts under root `outputs/`.
- Python checkers import `REPO_ROOT` from `scripts.common`.
