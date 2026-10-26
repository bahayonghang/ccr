# CCR: unavailable window during an in-flight UIA coordinate query

Vendored package: **accesskit_windows 0.34.0**, MIT OR Apache-2.0,
edition 2024 / MSRV 1.85. `UPSTREAM.json` records the registry archive checksum,
the crate's pinned upstream VCS identity, and every original file's SHA-256.
`Cargo.toml`, `Cargo.toml.orig`, upstream `Cargo.lock`, examples, changelog and
upstream tests are unchanged. `.cargo-ok` is local Cargo cache bookkeeping and
is intentionally omitted.

Source: [registry archive](https://static.crates.io/crates/accesskit_windows/accesskit_windows-0.34.0.crate),
[upstream commit](https://github.com/AccessKit/accesskit/tree/c88605b96d04431f9c3c792464a0f2f253480e94/platforms/windows).
The archive omits referenced license files; `LICENSE-APACHE`, `LICENSE-MIT` and
`LICENSE.chromium` were retrieved byte-for-byte from the **same commit's root**.
Original copyright and license headers are retained throughout.

## Production change

`util::client_top_left` and `Context::client_top_left` now return the existing
Windows `Result<Point>`. A failed `ClientToScreen` returns the adapter's existing
`UIA_E_ELEMENTNOTAVAILABLE` (0x80040201). Four provider methods propagate it:
BoundingRectangle, ElementProviderFromPoint, RangeFromPoint and
GetBoundingRectangles. BoundingRectangle retains its existing default rectangle
when the node has no bounds; empty text boxes retain their existing null array.

An in-flight provider method can already hold a strong Context when HWND is
destroyed. Dropping the adapter sooner does not revoke that Arc. The original
unwrap panics across a COM ABI and can abort the application on normal close.
No public interface, feature, dependency requirement, activation/event handling,
action handling or normal coordinate calculation changes. Accessibility stays
enabled. Only the independent GPUI workspace applies the patch; root CLI and
Tauri do not inherit it. Do not edit the machine's registry cache.

## Regression gate

From `ccr-desktop`, run `python scripts/test_accesskit_patch.py` (also part of
`just desktop-test`; Python 3.11+ on Windows for standard-library TOML parsing).
Windows runs three tests without creating any HWND:

- The exact null-HWND utility boundary returns 0x80040201 without panicking.
- With a strong Context retained, real COM provider calls for all four methods
  return the same HRESULT. This exercises more than Weak-upgrade failure.
- No node/text bounds preserve their previous successful empty results.

The script copies byte-identical sources into a temporary harness under the
verified desktop target, removes only the upstream window-creating test module
entry and its four unrelated dev-dependency tables, and checks all other source
bytes before/after execution. It reconciles the temporary lock offline, rejects
any selected dependency version/source/checksum absent from the production lock,
then tests with `--locked --offline` and default parallelism. Non-Windows prints
`not_applicable`; this is not a Windows pass. The original tests remain here.

The pre-fix no-panic regression actually failed with 0x80070578 at the original
unwrap. Raw failed/passing logs and limitations are recorded in task 02
`research/accesskit-close-fix.md`. Native repeated UIA/Alt+F4 and ordinary valid
window coordinates must also be verified by the main session. The no-window
regressions do not establish OS accessibility transport or screen-reader output.

## Updating / removing

Review any compatible upstream release against this failure and these tests.
Remove this workspace patch only after the real failure is fixed upstream and
the deterministic/provider and native close checks pass. A newer incompatible
AccessKit schema, disabling accessibility, or ignoring provider errors is not a
substitute. Regenerate and review the independent lock; do not upgrade unrelated
packages as part of removing this exception.
