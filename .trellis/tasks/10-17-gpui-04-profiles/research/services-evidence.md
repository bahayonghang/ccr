# 04 shared configuration services evidence

Date: 2026-10-18. Owner: profiles_services. Shared workspace; prior02/03 and other workers' edits preserved. No commit, push, account access or native launch performed by this worker.

## Boundary

- `ccr-app/config/source.rs`: canonical read/export/raw-save validation, active-removal confirmation and original-revision CAS composition removed from Tauri. Typed results; plaintext source has no Debug. Catalog SQL/persistence/private writes remain ccr-config.
- `config/mutations.rs`: generic Claude configuration add/update/duplicate/delete policy removed from Tauri, consuming existing manager/platform validators. Active guards require the caller's application FileLock before active resolution.
- `config/backups.rs`: existing same-directory relative leaf restriction, safe domain parser then existing locked preimage-backup/restore; existing depth/extensions/success-count cleanup receives an explicit root. Tauri retains its historical HOME/.ccr cleanup root and deletion set; native callers inject the approved managed fixture/root. Tauri cleanup is never invoked against the real home for verification.
- `config/profile_patch/`: existing pure Claude/Codex/Grok field decoding and composition removed from Tauri and consumed natively. No TS DTO/export ownership moved. Omitted/null/whole-platform_data replacement, Codex explicit->extra precedence and Grok credential_action behavior retained. Generic ConfigPatch and dedicated platform patch remain distinct.
- Manifests/locks/dependencies.md are single-owner writes here. Desktop adds existing domain crates; ccr-app never depends on ccr/ccr-cli/Tauri/GPUI/WebView. No alternate database/SQL.
- Grok staged update belongs in the existing Grok domain owner: main review found operation->catalog ordering would be inverted by a shared CatalogGuard followed by public Grok apply. That temporary shared-stage draft was removed before acceptance. Lifecycle worker supplies private-operation lock + already-locked stages, both partial outcomes and expected revision. Tauri maps typed owner outcomes into unchanged response variants.

## Actual checks so far

| Command | Exit | Actual result |
| --- | --- | --- |
| official crates.io toml/walkdir/serde_json fetch | 0 | Current versions/features/MSRV raw responses under outputs/desktop/profiles/services; restricted first socket attempts failed, permitted escalation succeeded |
| cargo check -p ccr-app --lib | 0 | New shared modules compile; exactly six required serde/TOML package updates |
| cargo test -p ccr-app --locked --test profile_services -- --skip export_bindings | 0 | 7 passed; validation/secret safety, canonical source/force/CAS, two clients exactly one winner, managed restore preimage, cleanup and dedicated patches |
| cargo clippy -p ccr-app --locked --all-targets --all-features -- -D warnings | 0 | Paired real Cargo/rustc1.98; no fmt repair |
| cargo test -p ccr-app --locked -- --skip export_bindings | 0 | Runtime9 + config-read5 + profile-services7 =21 passed; one explicit fixture generator ignored; default parallel |
| cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml --lib | 101 | Incorrect target: Tauri package has binary only; lock resolved, no compile claim |
| cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::config:: --locked --no-default-features -- --skip export_bindings | 101 | Existing Grok tests directly used moved credential helper; fixed cfg(test) shared alias, no test removed |
| cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::config:: --locked -- --skip export_bindings | 0 | Actual default-feature Tauri config18 passed, one child-only case ignored and invoked by parent; original wire types/command IDs retained |
| python scripts/desktop.py cargo metadata --format-version 1 --no-deps | 0 | Own cwd, no lock-resolution claim |
| python scripts/desktop.py cargo metadata --format-version 1 | 0 | Own cwd + verified resolver;44 necessary new package records, no existing desktop versions removed/upgraded |
| git diff --check -- owned Rust files | 0 | CRLF advisory only; no whitespace errors |

The first root/Tauri commands used the default MBX shim. Literal informational cache-savings text is retained in `profile-services.log` and `tauri-check.log`: “your deleted worktrees left 284.6 GiB logical behind. left. past tense.” / “284.6 GiB logical of target/ had outlived its checkouts. it has been dealt with.” These messages alone do not establish deletion at this command's time or source loss. No manual cache cleanup/global configuration change occurred. Subsequent root/Tauri commands use rustup-resolved real Cargo/rustc with child-only PATH pairing (toolchain.json). Desktop remains a real local target directory and uses its normal helper.

`outputs/desktop/profiles/services/lock-delta.json` retains all three lock comparisons. Root/Tauri versions change only serde_json and toml plus four required TOML-family packages; other Tauri record changes are dependency version-qualifier references. Desktop adds44 required existing-domain dependency records without removing/upgrading old versions. Direct dependency candidate/selected/MSRV/features/source evidence is in ccr-desktop/dependencies.md.

## Open checks

Grok actual staged outcomes/direct-apply concurrency, full platform Tauri regressions, final frozen lint/fmt/bindings/inventory/secret-write checks, MSRV and native acceptance remain open until logged. No native screenshots, resize/DPI, performance sampling or parent acceptance performed here. Main owns inventory; no command/route is marked complete by this worker.


## Frozen shared/adapter state

Product files and all manifests/locks frozen after final checks. The main session
owns subsequent independent verification and inventory/native acceptance.

- `config::mutations::parse_patch(&Value)` is the single strict generic ten-field
  policy, consumed by both Tauri's unchanged ConfigPatchInput wire DTO and native.
  Unknown fields/wrong types are rejected without showing rejected values;
  optional null removes, omitted retains, enabled:null is invalid.
- Dedicated `profile_patch::{claude,codex,grok}::validate_fields` describes the actual
  desktop patch fields for native advanced editing. CLI editable descriptors are
  not equivalent (notably Codex bearer and Claude model mappings). Legacy Tauri
  dedicated patch unknown-field behavior remains unchanged.
- Grok update/delete now delegate existing-domain typed owners:
  `GrokPlatform::update_profile_versioned` / `delete_profile_guarded`. Their domain
  lock order, already-locked stages and original revision remain lifecycle-worker
  owned. Tauri keeps original wire variants/messages and uses the domain's pure
  finish/delete decision helpers in existing callback regressions. No second
  staged state machine remains in ccr-app or native adapters.

| Final command / raw log | Exit | Result |
| --- | --- | --- |
| cargo test -p ccr-app --locked -- --skip export_bindings / shared-frozen-tests.log | 0 | 23 passed: runtime9/read5/profile9;2 ignored explicit fixture tools; default parallel |
| cargo clippy -p ccr-app --locked --all-targets --all-features -- -D warnings / shared-frozen-clippy.log | 0 | Final code including CU helper |
| Rust1.95 cargo check -p ccr-app --locked --all-targets / shared-final-msrv.log | 0 | Actual cargo1.95.0/rustc1.95.0; child-only matching bin/RUSTUP_TOOLCHAIN |
| cargo fmt -p ccr-app -- --check / shared-final-fmt.log | 0 | No repair invoked by validation |
| cargo fmt --manifest-path ccr-ui/src-tauri/Cargo.toml --all -- --check / tauri-final-fmt.log | 0 | No repair invoked by validation |
| git diff --check -- owned Rust files | 0 | CRLF advisory only |
| cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml profile --locked -- --skip export_bindings / tauri-profiles-final.log | 101 | 43 passed,2 exact dedicated-redacted-export error string failures: CcrError added prefix; adapter restored historical plain wire error, tests unchanged |
| same profile command / tauri-profiles-verified.log | 0 | 45 passed after compatibility fix |
| cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests --locked -- --skip export_bindings / tauri-grok-final.log | 0 |16 passed, including original partial rename/force/unsafe/credential DTO tests against domain-backed adapters |
| prebuilt profile_services exe --skip export_bindings / profile-final-frozen-tests-sandbox.log | 101 | Sandbox Temp path redirected into nonexistent Codex package path;5 temp-fixture setup failures,4 pure tests passed,1 helper ignored |
| identical exe/arguments with required filesystem access / profile-final-frozen-tests.log | 0 |9 passed,1 explicit helper ignored; environment issue, no product/test change |

The final Tauri config18 test run predates the last strict-parser delegation;
its frozen-tree rerun is left to the independent checker. Final bindings,
command-inventory, secret-write, root aggregate gates, desktop MSRV and native
verification are NOT run/passed by this worker. Domain failure-injection coverage
belongs to lifecycle-evidence.md; checker must cover actual invalid, backup,
write/apply and staged partial outcomes, not infer them from compiler success.

### Precompiled second-client evidence helper

`ccr-app/tests/profile_services.rs::mutate_native_profile_fixture` is ignored in
routine suites and was compiled with real Cargo before native interaction:
`cargo test -p ccr-app --locked --test profile_services --no-run --message-format=json`
exit0, raw `helper-build.jsonl`; exact compiler artifact `helper-artifact.json`.
The helper itself has not been executed against the CU fixture by this worker.
Main invokes the test executable directly with `--exact mutate_native_profile_fixture
--ignored --nocapture` while the UI is running; no Cargo/build runs during CU.

Required child-only env:
`CCR_APP_PROFILE_MUTATION_ROOT`, `CCR_APP_PROFILE_MUTATION_PROFILES`,
`CCR_APP_PROFILE_MUTATION_NAME`. All paths must be absolute. Root must canonicalize
beneath this repository's outputs/desktop/profiles and contain ownership marker
`.ccr-gpui-profile-fixture` with exact bytes `ccr-gpui-profiles-fixture-v1`.
Logical profiles.toml is not required to exist; its canonical parent and actual
profiles.db must be under that root. The lock directory is derived exactly as
native ConfigSource: profilesParent/desktop-profile-fixture/locks. Existing
ancestors must stay under root, preventing an escaped symlink lock directory.
The domain mutation compares its own before revision and changes one description
on the chosen existing synthetic profile; it emits only name/status/before/after
revision, no contents or credential diagnostics. A no-op repeat is rejected so a
receipt cannot falsely claim a new competing save. HOME/USERPROFILE are untouched.

Raw root/Tauri/toolchain/registry logs are all under outputs/desktop/profiles/services.
No complete42-command/5-route claim is made; main/native owner must establish
actual discoverable actions and acceptance evidence for each mapping.
