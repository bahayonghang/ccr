# Native resume: metadata edit changed the current profile

Date: 2026-10-27. Dispatched `trellis-check`, active task GPUI04. Main owns
native interaction, task status and final acceptance. This reviewer does not
control a window, modify the live native fixture, touch real accounts/configs,
or edit the concurrently owned theme/shell/DESIGN changes.

## Observed cause

Main's unchanged debug executable displayed `a-first` as current, then changed
the marker to `z-last` after saving only `a-first`'s description. Read-only
SQLite queries against the private, ownership-marked fixture confirmed:

| State | Catalog current | Catalog default | Registry current |
| --- | --- | --- | --- |
| Closed seed / initial fixture | `ghost` | `z-last` | `a-first` |
| After native metadata save | `z-last` | `z-last` | `a-first` |

The initial UI correctly follows Claude's valid-catalog-marker, then
valid-registry-marker resolution. `base::mutate_profiles_with_current` currently
replaces any nonempty invalid catalog marker with the first profile on every
CRUD operation. A same-name edit invokes this helper through the existing
lifecycle/platform owner and silently replaces `ghost` with `z-last`. Subsequent
reads therefore stop using the valid registry fallback. This is a catalog
pointer change, not evidence that runtime credentials were reapplied.

The fix belongs in the existing shared platform mutation owner, rather than
native selection/rendering or another current-state cache. Existing invalid
markers in a nonempty catalog must survive unrelated CRUD so the platform
resolver remains authoritative. First creation and deletion of a previously
valid current target retain their existing fallback behavior. Explicit marker
reconciliation continues to use its existing domain operation.

## Repair and verification

`crates/ccr-config/src/platforms/base.rs::mutate_profiles_with_current` now
remembers whether an existing nonempty catalog already had an invalid current
marker. Its usual fallback-to-first step leaves that preexisting marker alone.
It still selects the initial current target for first creation and selects the
remaining target when a previously valid current profile is removed. No public
interface, resolver precedence, persistence owner or native UI was changed.

- Added `injected_metadata_edit_preserves_registry_fallback_current`, exercising metadata edits of
  both the fallback-active and another profile, with `ghost` catalog current
  and a valid registry fallback different from the first profile. It checks
  resolved current, stored marker, unknown fields/secret retention, unchanged
  registry bytes and absence of implicit runtime activation.
- Added `platform_adapter_preserves_invalid_marker_without_changing_initialization_or_removal`, covering first-create initialization,
  unrelated edit with an invalid marker, and existing valid-current removal.
- Initial root Cargo attempt failed before compilation because the existing
  `target` symbolic link points at an unavailable `D:/mbx-cache/targets/...`
  location (`AccessDenied`, exit101). The link/cache were preserved. Further
  root checks use a process-local `CARGO_TARGET_DIR` under
  `outputs/desktop/profiles/resume-current-check/target`, paired with the
  existing Rust/Cargo1.98 binaries. No global environment changes or installs.
- Raw logs and `final-receipt.json` are under
  `outputs/desktop/profiles/resume-current-check/`. The receipt's successful
  package-test and Clippy exits come from actual retained tool completions,
  including the two long-running sessions resumed after the rate limit. They
  were not inferred from partial logs. No source changes followed those checks.

| Command | Exit | Actual result / log |
| --- | --- | --- |
| `cargo test -p ccr-cli --locked --all-features --lib injected_metadata_edit_preserves_registry_fallback_current -- --skip export_bindings` before fix | 101 | One failing regression: catalog current `old`, expected `ghost`; `regression-before-isolated-target.log` |
| Same command after fix | 0 | One regression passed; `regression-after.log` |
| `cargo test -p ccr-config -p ccr-cli -p ccr-codex --locked --all-features -- --skip export_bindings` | 0 | CLI410 + dispatch12, Codex392, config127, doctest1 passed; `affected-tests.log` |
| `cargo test -p ccr-app --locked --all-features -- --skip export_bindings` | 0 | Library9, config-read5, profile-services9 passed; `app-tests.log` |
| `cargo clippy -p ccr-config -p ccr-cli -p ccr-codex -p ccr-app --locked --all-targets --all-features -- -D warnings` | 0 | Lint and all-target/all-feature type checking passed; `affected-clippy.log` |
| Explicit `rustfmt --edition 2024 --check` on the three owned Rust paths | 0 | `fmt-check.log`; earlier formatting repair was separate |
| Scoped `git diff --check` on the three Rust paths and this report | 0 | `diff-check.log`; LF/CRLF advisory only |

The package-suite counts above are top-level suite summaries, not sums of nested
child-process fixture summaries. Across those test commands: **965 passed,
zero failed, eight ignored, 24 export-binding tests filtered**. Default Rust
parallelism and the existing ignored tests were preserved. The infrastructure
failure in `regression-before.log` remains separate from the real failing
regression in `regression-before-isolated-target.log`.

The requested CAS helper was rebuilt using the existing `ccr-app` ignored test
helper. Main froze its executable as
`outputs/desktop/profiles/resume-current-check/profile-services-helper-frozen.exe`
before further package builds. SHA256:
`89dd134542ae6ecd02a0e78994b6e7d3de9c135b68c50f553b9a67c30474c4ef`.
Building this helper is not native CAS acceptance; main owns its invocation and
GUI observation with the existing explicit private-fixture environment keys.

## Source freeze and remaining integration

| Owned file | SHA256 |
| --- | --- |
| `crates/ccr-config/src/platforms/base.rs` | `a682ed9b140af6ebc73bb73173f677df066980232c31c8be52dd4cf5f311f417` |
| `crates/ccr-config/src/managers/config/repository_tests.rs` | `ff6122c2888f1f5755e918f68c6ba69ccc736faf3038ade1dfe6b04a0d6674c0` |
| `crates/ccr-cli/src/application/profile_lifecycle_tests.rs` | `135ee34d5a1153eeef5dacde1cee165cca4d7646bb07d98b4cf13b6f8e25c418` |

Root lock SHA256 stays
`f4a932f8ca98b21ec80b4d0d178e6c3a67f7510993da49b91b03a55c07fa3b15`;
desktop lock stays
`14900fb870ff12effe9129a663aec7fb92d11dd3abef79fa1dce41ecf3c69a56`.
No manifest, lock, generated binding or desktop native source changed here.
Main owns the profile-repository spec update and native retry; the visual
checker owns the final debug build and desktop gates on the combined tree.
Fresh native current-marker behavior, the final visual tree, Tauri integration,
and any full-repository gate remain separate integration evidence. This
reviewer did not run `just ci`, native pixels, IME/UIA, other OS, performance or
packaging, and did not mark GPUI04 complete or archive/commit anything.

This report does not assert native acceptance, full task completion, or new
visual acceptance. Main's old-hash screenshots remain distinct from the new
theme's evidence.
