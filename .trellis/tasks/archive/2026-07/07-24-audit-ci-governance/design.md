# CI and contract governance design

## Single implementation boundary

Hosted workflows call repository-owned `just` recipes and scripts. Workflow
YAML selects triggers, runners, permissions, caching, and artifact upload; it
does not reimplement lint/test rules that can drift from local `just ci`.

## Product-surface matrix

- Root Rust fast lane: workspace formatting, strict clippy, tests, build,
  version/dependency governance.
- Tauri Rust lane: independent manifest format/check/clippy/test/binding drift
  on Linux for every relevant PR; Windows/macOS compile and focused platform
  smoke coverage.
- Vue/docs lane: existing frontend checks on `main`, `develop`, and `dev`.
- VS Code lane: clean install, type/lint, tests, package build, and artifact
  inspection for relevant PRs.

Path filters include shared manifests/scripts/specs that can affect each lane,
not only the surface directory.

## Stable required contexts

Pull-request workflow triggers do not use native `paths` filters because a
branch-protected context that is never created remains pending forever. A
repository-owned `ci_surface_policy.py` instead evaluates the merge-base diff.
Each workflow always creates a lightweight change detector and a stable final
context; heavy product jobs run only when that surface is relevant. The final
context fails when detection fails or any relevant validation, coverage,
audit, or platform matrix result is not successful.

The exact protected contexts are `Root Workspace Required`, `Vue and Docs
Required`, `Tauri Linux Required`, and `VS Code Required`. Changes to the
shared relevance policy make all four surfaces relevant so the router cannot
change without exercising every governed lane.

## Reproducible tooling

Rust/MSRV is declared in one checked-in toolchain source and validated against
all crate manifests. Bun is fixed at 1.3.10, Node at a reviewed LTS patch, and
third-party actions are pinned to full commit SHAs with version comments.
Dependabot owns scheduled updates for actions and package ecosystems.

The dependency drift allowlist becomes structured data with owner, rationale,
and expiry; expired or ownerless entries fail. The target is at most three
active exceptions.

## Generated contract checks

The handler registry produces machine-readable command inventory. Generated
docs and typed binding counts consume that inventory; handwritten frozen counts
are removed. A check regenerates into the working tree and fails on any diff.

## Parallel tests and coverage

Tests default to parallel operation with isolated CCR roots, home directories,
databases, and ports. Only documented environment-mutating cases use the
project's process-wide guard/serial annotation, and their count is reported.

Coverage jobs use deterministic fixtures. The root Rust workspace, Vue, and VS
Code line-coverage gates fail below 70 percent; designated root and Tauri
security gateways fail below 85 percent. Tauri also uploads its full baseline
report, but its broad command-wrapper surface is not substituted for the
explicit gateway threshold. Reports stay separate so a strong surface cannot
hide an uncovered one.

## External repository settings

Workflow files can create check runs but cannot prove branch protection. Final
acceptance requires current GitHub evidence showing Tauri and VS Code checks are
required on protected branches. If credentials cannot read or update that
setting, repository-side work may complete but the child remains unverified.

## Rollback

New expensive platform smoke jobs may begin non-blocking for one measured week,
but Linux security/contract gates remain required. Pinning, drift checks, and
branch trigger corrections are not rolled back to mutable behavior.
