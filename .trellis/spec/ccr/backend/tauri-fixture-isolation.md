# Explicit Tauri fixture storage

## Scope and entry point

The desktop benchmark foundation uses `--fixture-root <absolute-existing-root>`.
The root must contain `ccr-tauri-fixture.json` with version 1 and a nonempty
ASCII fixture ID. This is opt-in synthetic storage, not a performance result.
No flag preserves ordinary production resolution. Never change process-global
HOME/USERPROFILE to implement this mode.

## Startup and ownership

- Parse and validate the marker and existing tree before logger, migrations,
  Tauri context or WebView creation. Reject traversal, symlinks/reparse points,
  unsupported marker versions and unreadable roots; fail instead of falling
  back to host data. Keep the installed layout immutable for the process.
- Logger uses `ccr_core::init_logger_at`. Main DB uses
  `initialize_app_pool_at(path, migration_home)` and the returned pool is the
  same global pool used by managers. The explicit initializer refuses repeat
  registration. Default initializer behavior remains compatible.
- Archive uses `create_usage_archive_pool_at(path, migration_home,
  legacy_ui_path)`. Both migrations consume only the captured legacy home and
  UI DB. Current main, legacy CLI DB and archive remain separate databases.
- Catalog migration uses `migrate_catalogs_at(&[PlatformPaths])`, preserving
  journal, locking and source-retirement semantics. Captured source paths do
  not inherit platform HOME/config environment overrides.
- Preferences load/quarantine and detached saves keep the same captured
  `preferences/desktop-shell.json`; background workers must not resolve HOME.
- Tauri creates configured windows before project setup. Fixture context
  disables automatic creation and explicitly builds the original main window
  after captured State installation. Initial, tray and rebuilt windows all use
  the captured WebView data directory. Do not replace the app with another page.
- macOS fixture boot currently refuses before side effects: locked WebKit
  dataDirectory cannot provide the same root-contained profile contract for
  the supported macOS floor. Windows/Linux configuration is not native proof.

## Capability and data-flow contract

Registry dispatch and Rust command runtime admission share one explicit
captured-command allowlist. Missing IDs fail before handler-body execution.
Read-only risk metadata is not isolation evidence. Preserve normal ACL,
confirmation, timeout, concurrency and audit boundaries.

Admitted graphs must consume captured owners end to end: profile lifecycle and
CAS, Claude settings/MCP/collections, read-only usage projections, session
provider registry/archive/detail, UI state/preferences and monitoring. Registry
refresh keeps only captured Local in fixture mode. Background maintenance and
log bridge retain their real lifecycle; their file/DB roots are captured too.

Startup must not restore OAuth, discover WSL/SSH, probe installed CLI tools or
query remote quota. Tray snapshots consume captured account/runtime facts and
do not fetch quota. Unreviewed and external command graphs remain explicitly
unavailable rather than returning synthetic success.

Auth source diagnosis defaults to observing the normal process environment.
Synthetic `PlatformRoots::claude_auth` and `ProfileLifecycleContext` explicitly
carry `with_process_environment(false)` through profile apply/off. File-based
evidence remains visible; host process credentials must not become fixture facts.

Auth summary also needs `with_profile_paths(PlatformPaths)`: constructing an auth
service with captured credential paths does not capture a nested
`ClaudePlatform::new`. `PlatformRoots::claude_auth` binds catalog/registry paths,
runtime settings, backups and locks for every consumer, including internal
`list_accounts -> get_runtime_summary`. Profile-off binds its context paths too.
An environment-disabled service without profile paths refuses summary before
any ambient profile constructor. The default service without an explicit capture
retains the legacy wrapper behavior.

Direct Claude profile add/delete and Settings/collection mutation must enter the
captured lock scope on the actual blocking thread. A SettingsManager containing
an explicit lock manager does not prove that its nested guarded writer uses it;
its async helper may spawn another worker. Preserve the normal async path and
bind fixture scope around the synchronous owner inside its real blocking closure.
Poison the default lock directory and exercise those actual commands in isolated
children; generic-profile tests alone do not cover dedicated profile writes.

## Verification and comparison qualification

Run Tauri `fixture::tests`, affected registry/runtime-policy and default-owner
regressions, plus strict lint. The core graph test executes a fresh subprocess:
same global/State pool, duplicate registration refusal, legacy catalog migration,
profile CAS, guarded settings backup, real usage/session projections, monitoring
flush and pre-body external-capability rejection. Root auth and logging tests
cover default/explicit behavior without parallel HOME mutation.

Headless storage configuration cannot prove WebView localStorage/cookies/cache
isolation. Record native loading separately. All required benchmark actions must
be supported and both clients must load the same logical S0/S1/S2 data before a
baseline is comparable. An error-only app, blocked workflow or disabled
maintenance is not a valid whole-app performance baseline. No main-entry,
JavaScript paint or layout callback timestamp is first-present evidence.

The shared ignored `ccr-app::benchmark_fixture::generate_s0_s1_s2_pairs` generator
is test-only and accepts only a new confined outputs directory. Validate
ancestors/links before creating even the allowed output-directory prefix;
rejecting a redirected prefix after creation is too late. It uses existing
catalog, provider parser/archive and usage fixture owners, then verifies actual
projection counts and logical hashes for both physical roots. Keep root-bound
opaque IDs unchanged and separately hash them. Each manifest fixes as_of/UTC;
future clients must consume that definition. S2's descriptor is `not_emitted`
until real runtime injection executes. Seed success never implies application
loading, first-present measurement or benchmark comparability.
