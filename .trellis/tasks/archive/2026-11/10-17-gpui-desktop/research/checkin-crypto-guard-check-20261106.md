# Check-in crypto guard independent check — 2026-11-06

Reviewed the ReadyForReview `checkin-crypto-guard-20261106.md` receipt against
native Storage/crypto guards, mutation/inspection/transfer entry points, selected
credential getter/editor recovery UI, and Root AccountManager reads/tests. HEAD
at review was `57abc91f`; concurrent pending native work was preserved. Source
was frozen for the coordinator's integrated gate. This check changed no
production/test/spec files and started no Cargo process; only this report was
written. No commit was made.

## Findings

**No new defect found in the reviewed scope.** The earlier missing-key hazards
are now guarded at the native entry points and the two domain read owners:

- AddAccount, configuration Export and Import inspect existing rows through the
  captured AccountManager before reaching a legacy initializer. Existing rows
  require `load_existing`; empty storage keeps first-use initialization. Missing
  and malformed keys leave ciphertext/key bytes unchanged. Export/Import are
  refused before output/backup or domain mutation. The guard does not silently
  regenerate, replace or repair a key.
- Import checks its current captured source path before the domain re-read. The
  original source/root/version/database fingerprint and verified pre-import
  backup/apply path remain intact. Restoring the original key does not bypass a
  stale preview; a new preview then imports through the original backup owner.
- Check-in, balance and connection/WAF inspection use the existing key before
  constructing a provider service. Recharge checks the selected account/key
  before constructing its client. Benchmark still refuses network work at its
  existing source guard. The controlled loopback fixture path tests the same
  request gate without calling a real provider.
- Metadata-only UpdateAccount and CDK extra-config edits avoid the key check;
  nondecrypting stats remain usable. Cookie replacement requires the existing
  key. Captured TLS lock scope encloses synchronous domain calls only and never
  crosses await/spawn; export retains its separate domain file-write lock scope.
- `AccountManager::get_info` and `get_cookies_json` use `load_existing`; they do
  not create directories/keys through the initializer. The prior update invariant
  remains intact. The normal constructor, domain creation and legacy CLI/Tauri
  import/export APIs were not redesigned.
- Selected credential load returns `CryptoKeyUnavailable` for missing/unreadable
  keys, separately from decryption/cookie failures. `CredentialRead.failed` keeps
  the typed error in the selected editor. Its static notice asks for the original
  key and explicitly allows metadata edits; it neither claims Cookie expiry nor
  suggests replacement cookies as key recovery. Failed replacement leaves the
  editor open; clearing it permits a metadata-only save without making a key.
- Credential DTOs remain without Debug/Serialize, masked fields and decoded data
  stay editor/request-scoped, and retained management results do not receive
  plaintext. Existing request nonce/account identity checks, leave clearing and
  normal Session patching preserve stale-response safety and other Cookie keys.

The owning backend spec already records existing-row/empty-store behavior,
typed recovery, pure reads, import CAS and validation boundaries; no additional
spec change was needed during this check.

## Verification

Independent exact runs used existing compiled artifacts newer than the reviewed
source files (Root artifact 12:07, native artifact 12:12). They retained default
parallel test behavior and `--skip export_bindings`; no Cargo, GUI/CUA or real
provider call was used:

- Root `credential_reads_never_initialize_a_missing_or_malformed_existing_key`:
  **1/1 passed**, covering preserved encrypted/key bytes and successful reads
  after restoring the original owned key.
- Native `missing_or_malformed_existing_keys_block_native_mutation_transfer_and_requests`:
  **1/1 passed** (0.11s). The full matrix asserts typed failure for Add/cookie
  update/Export/Import/check-in/balance/connection/WAF/recharge; unchanged
  ciphertext/key bytes, no output/backup/accepted listener request; permitted
  metadata/CDK-extra/stats; stale import conflict followed by fresh-preview
  success and backup after original-key restoration. Its only listener is owned
  temporary 127.0.0.1, executed with the coordinator's controlled-loopback
  authorization.
- Native `missing_key_editor_requires_recovery_but_keeps_metadata_savable`:
  **1/1 passed** (0.89s), exercising actual headless controls, the typed notice,
  replacement refusal, clearing replacement and successful name-only save.
- Owner receipts: native check-in **17 passed**, domain AccountManager **10
  passed**, Root all-targets/all-features and desktop all-targets strict Clippy
  **passed**, and **3 exact actual 1MiB/poisoned-lock children passed** (new
  recovery editor, existing normal Session/stale editor, empty fixture transfer).
  These are the implementation owner's executions, not repeated executions by
  this reviewer. Compilation/type checks are supported by those receipts; this
  reviewer respected the no-Cargo freeze.
- Coordinator's final integrated lib result: **227 passed / 1 failed / 4 ignored**;
  the **26-child stack wrapper passed**. The failure is the existing
  `features/platforms/tests.rs::oauth_cancel_joins_fixture_cleanup_and_clears_callback_draft`
  timeout, recorded in `integration-20261106-gpui-current-test.log`. The full gate
  stopped at lib and native-shell was not run. OAuth implementation diagnosis is
  separately assigned; this review neither attributes its root cause nor treats
  the failed aggregate as a keyguard/full-gate pass.

## Findings not fixed / acceptance limits

No additional source fix is requested. These guards handle keys already missing
or unreadable at validation; they do not make legacy initializer use atomic with
another process removing/replacing a key afterward. Key rotation, key recovery
workflow and general legacy CLI/Tauri create/import/export redesign remain out
of scope, accurately stated in the implementation receipt/spec. A valid but
different key is not proof of a successful credential decrypt or remote login.

No real user Cookie/key, external provider, live diagnostics/SSH, native window
or performance sample was used. Native GUI acceptance remains user-deferred;
this report does not upgrade it or the performance comparison status.
