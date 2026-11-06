# 02 shared configuration domain implementation evidence

Date: 2026-10-18. Host: Windows x86_64, PowerShell 7, repository Rust toolchain
1.98.0, Python 3.14. This report covers the domain implementer's actual work;
runtime, native screenshots, root integration and independent acceptance belong
to their respective owners. No claim that all of 02 or the parent is complete.

## Change boundary and delivered behavior

- Added root workspace member `crates/ccr-app`; root CLI default-members remain
  `crates/ccr`. The independent GPUI workspace remains independent.
- `ccr-app::config::ConfigReadService::{for_claude,from_paths,list}` contains the
  first real shared list orchestration. Constructors hold paths; `list` runs on
  a blocking worker. It reads the domain `ConfigManager::snapshot`, retains its
  revision on every row, and calls the domain current-marker resolver.
- `ConfigRow` preserves the existing Tauri list's order, empty description/base
  URL/token defaults, provider/account/tags/model/usage/enabled fields, default
  marker and registry fallback. `auth_token` uses existing `Secret::Display`;
  plaintext credential exposure and a second masking implementation were not added.
  `ConfigReadError` is the original domain error type, preserving classifications;
  Tauri error conversion remains at its adapter boundary.
- Extracted only Claude's existing pure current-read orchestration into
  `ccr-config::platforms::base::read_file_current_profile_with_paths`.
  It reuses existing profile/marker loading, explicit registry manager and the
  unchanged `resolve_file_current_profile` precedence. CLI delegates to it;
  reconciliation and all profile persistence/CAS rules stay with their owners.
- Tauri `commands/config.rs::list_configs` now calls the shared service inside
  its existing `spawn_blocking`. `adapter.rs` maps rows back to the existing
  `ConfigInfo`. DTO declaration, TS export location, command name/signature,
  explicit-Claude validation, and join/domain error boundaries stay in Tauri.
- Added path dependencies from Tauri and GPUI to `ccr-app`; no umbrella `ccr`,
  GPUI, Tauri or WebView dependency was added to `ccr-app`.
- `lib.rs` exposes the separately owned runtime module. Runtime implementation
  and GPUI consumption are documented by the runtime implementer, not claimed here.

## Existing contract fixtures, HOME, and safe reads

The original Tauri config fixture called `profile_contract::isolated_fixture`,
whose legacy `TestHome::new_with_home_env` changed process-global HOME/USERPROFILE.
Running it unchanged would violate this task's test restriction.

The actual Tauri contract tests now dispatch each case into its own exact-name
test subprocess with `Command::env` overrides for CCR_ROOT, CCR_DATA_DIR,
CCR_LOCK_DIR, Claude runtime/state/backup paths, CCR_CODEX_DIR and GROK_HOME.
HOME and USERPROFILE are not overridden by this Tauri dispatcher. Tests retain
normal parallelism. Its explicit `CCR_PROFILE_CONTRACT_ROOT` seam is test-support
code only: it checks an absolute canonical parent, ownership marker, and every
expected child path before seeding. A common domain seed preserves the old
cross-platform lifecycle scenarios. Cleanup only removes the explicitly owned
fixture children; it never cleans a real home.

`profile_contract.rs` actual unit-test entries and Claude's one direct
`new_with_home_env` test also received the independent checker-owned
`home_process::run_current_test` guard. The pure source assertion remains direct.
The shared helper and other root test callers are owned by the checker; their
global-HOME audit and full-root verification must be consumed separately.

The configuration fixtures are domain-created `profiles.db` catalogs, using
`ConfigManager::replace_toml`, not copied SQL/schema or a second database.
Tests cover revision/order/masking/empty defaults, valid-file precedence,
registry fallback, absent registry, missing catalog, existing empty catalog,
legacy-not-migrated catalog, corrupt database and corrupt registry.

Important measured existing SQLite behavior: opening a live WAL database with
the existing read-only domain connection can create exactly `profiles.db-wal`
(0 bytes) and `profiles.db-shm` (32,768 bytes). On Windows, a repeated read can
update the SHM modification time while its bytes remain identical. The tests
strictly preserve catalog and registry bytes/mtime/revision and all other
business files, allow only those measured SQLite auxiliary-file effects, and
check repeated query stability under that same rule. This is **no business
write**, not a claim of physically zero filesystem changes. No immutable SQLite
flag, connection-mode change, migration, marker repair, backup or repository
write lock was introduced to suppress live-WAL visibility.

## Dependencies and lockfiles

The official crates.io API was rechecked on 2026-10-18. Successful public metadata
responses are retained as selected metadata records:

- `outputs/desktop/services/tokio-registry.json`: candidate 1.53.3, MSRV 1.71.
- `outputs/desktop/services/tokio-util-registry.json`: candidate 0.7.20, MSRV 1.85.
- `outputs/desktop/services/async-channel-registry.json`: candidate 2.5.0, MSRV 1.60.

Sources: <https://crates.io/api/v1/crates/tokio>,
<https://crates.io/api/v1/crates/tokio-util>,
<https://crates.io/api/v1/crates/async-channel>.
`ccr-desktop/dependencies.md` records the full selected versions/features and
exceptions. The directly selected runtime dependencies are:

| Dependency | Selection and feature policy | MSRV/source |
| --- | --- | --- |
| ccr-config, ccr-core | Local path, workspace 7.5.0, existing domain features | Existing manifests, 1.95 |
| async-channel | 2.5.0 in all locks; default std | Official registry, 1.60 |
| tokio | Workspace requirement 1.53.1; root/Tauri existing locks 1.53.1; desktop existing lock 1.53.3; workspace runtime/fs/io/net/process/signal/sync/time/macros features, dev adds test-util | Both selected manifests 1.71 |
| tokio-util | Workspace requirement 0.7.19; root/Tauri existing locks 0.7.19; desktop existing lock 0.7.20; rt feature for cancellation/task ownership | Locked manifests 1.71 for 0.7.19, 1.85 for 0.7.20 |
| tempfile | Reused root dev-only requirement/lock 3.27.0, defaults; no new production dependency | Locked manifest 1.63; latest external candidate not independently queried for this already-established fixture policy |

The root/Tauri Tokio versions deliberately retain their existing tested
resolution instead of expanding this work into a shared dependency upgrade.
The independent desktop lock already selected the latest candidate family.

Root Cargo.lock changes are 59 added lines: new ccr-app, async-channel,
concurrent-queue, event-listener, event-listener-strategy and parking package
records. Tauri lock changes are 33 added lines, reusing its existing transitive
event/queue packages. No prior root/Tauri package version was changed.

During desktop path-dependency resolution Cargo's MSRV fallback initially
downgraded ten unrelated existing choices. They were restored precisely:
font-types 0.12.7, h2 0.4.20, libredox 0.1.25, open 5.4.4,
redox_users 0.5.3, rustls-native-certs 0.8.5, schannel 0.1.30,
zbus_names 4.3.5, zvariant 5.15.1, zvariant_derive 5.15.1.
No permanent resolver setting or manifest pin was introduced. Newly required
domain graph packages were added to the independent lock; the final locked
Windows metadata command passed. New actual Windows package downloads were
public h2/schannel/rustls-native-certs cache additions, not global tool installs.

## Actual commands and results

All Cargo behavior tests retained default Rust parallelism and used
`--skip export_bindings`. Explicit formatting repair ran separately, never as
part of a validation command. Cargo execution required existing mbx/cache access
outside the sandbox; automatic review permitted that access.

| Command | Final exit/result | Evidence / limits |
| --- | --- | --- |
| `cargo check -p ccr-app --locked --offline` | 0 | Shared crate and existing domain owners compile |
| `cargo test -p ccr-app --locked --offline --test config_read -- --skip export_bindings` | 0; 5 passed, 1 evidence generator ignored | `outputs/desktop/services/config-read-final.log` |
| `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked --offline commands::config -- --skip export_bindings` | 0; 18 passed, 1 subprocess-entry ignored | `outputs/desktop/services/tauri-config-final.log`; the ignored internal subprocess entry is actually invoked by the 3-round concurrency parent test |
| `cargo tree -p ccr-app --locked --offline --edges normal --prefix none` plus package-name forbidden-graph assertion | 0 | `outputs/desktop/services/ccr-app-dependency-tree.txt`; no ccr umbrella/Tauri/GPUI/wry/WebView |
| `cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc` in desktop cwd | 0 | Independent final lock parsed after original choices restored |
| Toolchain 1.98 rustfmt `--check --config skip_children=true` on owned Rust files | 0 | Final read-only format check after explicit formatting |

The Tauri suite includes an actual `list_configs` handler vs shared-native-service
projection test, comparing every JSON wire field and row order, optional-field
omission, revision and masking. It also retains existing platform rejection,
mutation validation/CAS, registry fallback, lifecycle/enable fault compensation,
replay, application-lock ordering and multi-process lost-update coverage.

Initial failures and recovery were not treated as passes:

- Sandbox public socket/cache denial: registry/cache access retried under allowed
  escalation; no product behavior changed to bypass it.
- Full offline metadata attempted unneeded foreign-target cache downloads and
  changed unrelated versions: narrowed metadata to the Windows target and restored
  unrelated version choices before locked validation.
- Initial fixture fingerprint assertions rejected measured SQLite sidecars/SHM
  metadata: diagnosed and explicitly bounded those pre-existing effects as above;
  never weakened catalog/registry business-file invariants.
- Initial Tauri compilation found the fixture enum needed an explicit Drop trait
  to retain its public `impl Drop` fixture return contract: fixed and reran.
- An accidentally duplicated evidence-generator test failed compilation: removed
  the duplicate, then reran all five configuration assertions successfully.
- A dependency-note append attempted Python's platform-default encoding and failed
  before writing; the note was written with explicit UTF-8.

## Native fixture handoff and outstanding verification

An explicit ignored **data-generation** test `seed_native_fixture` is available;
it is not a skipped acceptance test. Main executed it with child-only
CCR_APP_FIXTURE_OUTPUT/CCR_LOCK_DIR overrides. Main reports exit 0, one data seed
executed; its receipt is `outputs/desktop/services/native-seed-receipt.json`.
The generator rejects existing destinations and destinations outside a new direct
child of repository `outputs/desktop/services`, creates catalog/registry through
domain APIs, and emits `fixture-manifest.toml` with schema/revision/BLAKE3 hashes
and synthetic-only credential identification. Logical profiles.toml is a domain
input path; physical storage remains profiles.db. Native screenshots belong to
main; generation alone is not native verification.

Not run by this domain implementer: final bindings/inventory gate, full ccr-config
and CLI/root regression, root lint/test/CI, final independent desktop runtime/UI
gates, actual native slow-read interaction, cross-platform native/DPI/IME matrix
and performance comparison. Check/main own those gates and must attach actual
results; their absence here is not a pass or waiver.

All product files were frozen and handed to the independent checker before this
report was written. No commit, push, publish, user-config write or task archive
was performed.
