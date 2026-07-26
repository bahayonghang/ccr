# CI and contract governance implementation plan

## Ordered work

- [x] Add/reuse root `just` recipes for Tauri Rust and VS Code clean CI; make
  local and hosted entry points identical.
- [x] Add Tauri and VS Code PR workflows with complete relevant path filters;
  normalize frontend branches to `main`/`develop`/`dev`.
- [x] Pin Rust/MSRV, Bun 1.3.10, Node LTS patch, and all third-party actions;
  add Dependabot update ownership.
- [x] Convert dependency exceptions to owner/rationale/expiry records and wire
  drift checks into hosted CI.
- [x] Generate handler counts/docs from the registry and make bindings/docs
  regeneration drift a required check.
- [x] Inventory shared-state tests, isolate fixtures, remove global
  `--test-threads=1`, and retain serial execution only for documented cases.
- [x] Add root Rust/Vue/VS Code line-coverage collection and 70% thresholds,
  root/Tauri security-gateway 85% thresholds, and uploaded reports; add tests
  until thresholds are genuinely met rather than lowering targets.
- [x] Run workflow syntax/action pin/path-filter tests and compare each hosted
  job command to its local recipe.
- [ ] Query/apply branch-protection required checks only with explicit
  repository permission; capture the resulting protected-branch evidence.
- [x] Update dependency, handler-registry, typed-binding, and test-fixture specs.

## Focused validation

```powershell
just version-check
just fmt-check
just ui-check
just vscode-ci
just frontend-check
just ci
```

Also validate workflow syntax and inspect an actual PR check matrix. Local YAML
parsing alone cannot prove trigger or required-check behavior.

## Rollback checks

- A workflow-only rollback must not remove pinning or required security gates.
- Coverage exceptions require an owner and expiry; no blanket ignore.
- Unavailable GitHub branch-protection evidence is `UNVERIFIED`, not `PASS`.
