# P3 cleanup implementation plan

## Ordered work

- [x] Inventory facade re-exports and internal umbrella-crate dependencies;
  define replacement path/removal window for each broad legacy surface.
- [x] Add 7.x deprecation attributes/docs and compatibility compile tests; add
  the internal dependency-policy guard.
- [x] Write the responsibility-based decomposition architecture note and link
  each move to its owning gateway child/spec.
- [x] Review module-move eligibility after completed install/process/sync/
  migration work. No move had a separately owned interface that justified
  churn in this P3 task; retain paths and record the extraction gates instead.
- [x] Replace all verified mojibake comments in the migration region; search the
  affected source tree for remaining corruption markers.
- [x] Add deterministic JSON format/check script, explicit include/exclude
  inventory, and tests for malformed/noncanonical/excluded files.
- [x] Format `tauri.conf.json` and other selected human-authored JSON; wire check
  mode into `just fmt-check` and hosted CI.
- [x] Update public API, module ownership, and formatting specs.

## Focused validation

```powershell
just version-check
just fmt-check
cargo test -p ccr -- --test-threads=1
cargo test -p ccr-db migration -- --test-threads=1
just lint-strict
just test
```

Use `rg` to prove mojibake markers are absent and inspect formatting diffs to
ensure JSON values are byte-for-byte semantically equivalent after parsing.

## Rollback checks

- Deprecation rollback must not remove the documented narrow replacement path.
- Module move rollback must preserve the post-gateway behavior/tests.
- Generated/lock/semantic-whitespace fixtures remain outside format rewrites.
