# Persistence and migration implementation plan

## Ordered work

- [x] Add async writer options and platform-specific temporary-file permission
  creation, metadata preservation, and parent-directory fsync.
- [x] Add Unix umask/mode tests and Windows ACL integration tests; prove failed
  replacement preserves old bytes and metadata.
- [x] Inventory credential/profile/settings writers and migrate every sensitive
  caller to the secret policy; add a repository guard against direct async
  writes to known secret paths.
- [x] Introduce the migration descriptor and `apply_migration` transaction/
  postcondition/marker helper; convert v3-v5 first, then remaining compatible
  migrations.
- [x] Add `migration_rejections` and the next-version v3 repair migration with
  full processed/repaired/rejected accounting.
- [x] Add durable backup/restore verification and historical release fixture
  upgrades, including two-run idempotence and integrity checks.
- [x] Remove all silent row/update error patterns and repair the migration
  logging so success is emitted only after commit.
- [x] Update atomic-writer and ccr-db migration specs with proven contracts.

## Verification evidence (2026-07-26)

- Windows: `cargo test -p ccr-core atomic_writer -- --test-threads=1` (9 passed,
  including real-filesystem DACL preservation).
- WSL2 Linux: `cargo test -p ccr-core async_secret -- --test-threads=1`
  (2 passed: umask and stricter-mode preservation).
- `cargo test -p ccr-db migration -- --test-threads=1` (16 passed) and full
  `cargo test -p ccr-db -- --test-threads=1` (118 passed).
- CLI settings focused tests (10 passed), Codex quota focused tests (12 passed),
  `just lint-strict`, and final `just test` all passed.
- `python scripts/check-secret-writes.py`, `cargo fmt --all -- --check`, and
  `git diff --check` passed; silent-error search returned no matches.

## Focused validation

```powershell
cargo test -p ccr-core atomic_writer -- --test-threads=1
cargo test -p ccr-db migration -- --test-threads=1
cargo test -p ccr-cli settings -- --test-threads=1
just lint-strict
just test
```

Windows ACL evidence must come from a Windows filesystem test, not a mocked
mode assertion. Unsupported parent-directory fsync must be recorded explicitly.

## Rollback checks

- Verify backup identity before restore.
- Never insert a migration marker for a rolled-back or partially accounted run.
- Never loosen an existing secret file's permissions.
- Do not log malformed raw JSON or secret values in rejection detail.
