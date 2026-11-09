# Checkin existing-key preflight — 2026-11-06

Stage: **ReadyForReview**. Implementation and focused checks complete; independent review and final integrated CI remain owned by the main session. Native GUI verification remains user-deferred.

## Delivered boundary

The native AddAccount, configuration Export and Import paths now ask the captured AccountManager whether account rows exist. Existing rows require `CryptoManager::load_existing`; empty storage can retain first-use initialization. Import performs the check before invoking the original version-bound preview/backup/apply owner. The existing root, input version and database revision checks are preserved.

Native selected-account check-in, balance, connection/WAF inspection and CDK recharge require the existing key before constructing a request. Cookie replacement also returns typed `CryptoKeyUnavailable`. Metadata-only account updates, CDK extra-config updates and nondecrypting stats remain usable. No TLS scope spans an await.

Two domain read methods, `AccountManager::get_info` and `get_cookies_json`, now load existing keys instead of invoking the initializer. This also protects the CDK worker's credential read. The committed metadata-only/update fix is preserved. Default CryptoManager construction, domain create and legacy CLI/Tauri import/export are not redesigned.

The selected editor now retains an `Option<OperationError>` for failures and renders a dedicated original-key recovery notice. Missing-key replacement is refused; clearing the replacement allows a name-only save without a key. Existing Session-only editing, advanced full JSON and unknown-cookie preservation are unchanged. Credential DTOs remain without Debug/Serialize and do not enter retained management results.

Missing and malformed keys are distinct: a missing key previously caused `CryptoManager::new` to initialize one; malformed existing bytes already returned an error. This change is a native preflight and pure-read correction, not key rotation or an atomic guarantee against external key removal after the check.

## Files

- `ccr-desktop/src/runtime/checkin/operations/crypto_guard.rs` and `crypto_guard/tests.rs`: captured existing-account preflight, key validation and controlled fixtures.
- `runtime/checkin/operations.rs`, `extended.rs`, `operations/credentials.rs`: mutation/inspection/transfer gates and typed read failure.
- `features/checkin/operations.rs`, `credentials.rs`: static recovery message, typed editor failure and actual control regression.
- `features/environments/tests.rs`: one exact-name addition to the existing native-stack guard.
- `crates/ccr-checkin/src/managers/checkin/account_manager.rs`: two pure reads plus domain regression.
- Owning backend spec: existing-key contract and validation boundary.

## Verification

- `cargo test --locked --offline --lib checkin -- --skip export_bindings` in `ccr-desktop`: **17 passed**, default parallelism, allowed controlled loopback. Includes existing CRUD/transfer/batch/session-editor coverage and three new missing-key tests.
- `cargo test --locked --offline -p ccr-checkin account_manager -- --skip export_bindings`: **10 passed**, default parallelism.
- Desktop `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- Root `cargo clippy --locked --offline -p ccr-checkin --all-targets --all-features -- -D warnings`: passed.
- Same compiled desktop artifact `ccr_gpui-4a39d1c098e6c17e.exe`, child-only `RUST_MIN_STACK=1048576` and `CCR_LOCK_DIR` pointing at an owned temporary file: **3 exact tests passed**: missing-key editor recovery/metadata save, existing selected-credential/stale-response editor, and empty fixture transfer/captured locks. The new exact name is registered in the existing aggregate guard; final full guard is pending main-session integration.

The controlled matrix covers missing and malformed keys across AddAccount, cookie update, Export, Import, check-in, balance, connection inspection, WAF inspection and recharge. It checks unchanged ciphertext/key bytes, no output/backup, no accepted request, permitted metadata/CDK-only writes, original import revision conflict, and successful fresh preview/backup after restoring the original fixture key. Empty storage still initializes on first account creation and existing empty-fixture transfer remains green. Domain read tests recover normally after restoring the original fixture key.

All new behavior tests use owned temporary data or synthetic headless controls. No real user credentials/keys, provider network or native window were used. This statement concerns this batch's tests, not every existing test in final CI.
