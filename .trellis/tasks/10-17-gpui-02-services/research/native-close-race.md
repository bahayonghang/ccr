# Research: Windows UIA requests during native window destruction

- Query: Explain the reported real GPUI Alt+F4 panic at AccessKit Windows `util.rs:380:52`, distinguish framework HWND lifetime from application job cleanup, and identify the narrowest defensible repair.
- Scope: mixed; exact locked registry source plus official upstream and Microsoft documentation. Research only; no native launch, OS interaction, dependency mutation, tests, or product changes.
- Date: 2026-10-18

## Findings

### Immediate conclusion

The supplied panic identifies a real failing `ClientToScreen` call in the Windows accessibility provider. It is not a Tokio runtime error or evidence that task cleanup succeeded. Source inspection supports a window-destruction/provider-query lifetime overlap: GPUI retains the AccessKit adapter during its window destruction path, and an in-flight UIA COM method can retain the adapter Context even after an owner drops it. A normal provider call against an HWND which has just become invalid reaches an unconditional unwrap and aborts across a COM boundary.

The specific UIA method and interleaving still require the main session's preserved backtrace/native reproduction. The report does **not** establish which external client initiated that call, nor that every close will panic. Repeated helper `accessibility: null` is a separate unresolved transport result. It does not imply the adapter was inactive: the observed provider panic proves at least one AccessKit coordinate path was entered during this process lifetime.

There is no supported application-level `dispose_accessibility`, `Adapter::deactivate`, or window option in these exact sources that reliably cancels an already in-flight UIA request. Disabling accessibility, waiting an arbitrary delay, swallowing a Rust panic, returning made-up coordinates, or terminating the process early is not an acceptable repair.

### Input evidence and limits

The main session supplied: actual synthetic fixture window with configuration list/filter; a second run with a visible 30-second fixture delay; responsive filter and Ctrl+A while busy; repeated Sky `include_text: true` captures returning null; normal Alt+F4 followed by thread 35952 panic at `accesskit_windows-0.34.0/src/util.rs:380:52`, HRESULT `0x80070578`, Invalid window handle, then cannot-unwind panic; no surviving application process. The first launch wrapper did not explicitly return `$LASTEXITCODE`, so wrapper exit 0 is not a normal-exit pass. Main owns exact-exit reproduction and raw artifacts. These observations were not repeated by this research agent.

Reuse the earlier activation/semantic investigation at `.trellis/tasks/10-17-gpui-01-foundation/research/native-a11y.md`: the default Windows graph already contains AccessKit/UIA, so null helper output is not evidence of a missing compile feature. This report investigates the new destruction failure only.

### Exact local files and code patterns

Cache source prefix below: `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. These are shipped registry versions, not inferred upstream HEAD behavior.

| File and lines | Finding |
| --- | --- |
| `accesskit_windows-0.34.0/src/util.rs:376-381` | `client_top_left(WindowHandle) -> Point` creates a POINT and calls `ClientToScreen(...).unwrap()`. The adjacent comment deliberately treats a vanished window as unexpected. This is the exact reported panic site. |
| `accesskit_windows-0.34.0/src/util.rs:360-362` | Existing `element_not_available()` constructs the UIA missing-element HRESULT. Reuse it rather than inventing a status or panicking. |
| `accesskit_windows-0.34.0/src/context.rs:33-38,72-74` | Context retains raw HWND, tree and action handler. Its coordinate helper simply forwards to the panicking utility. No HWND lifetime lock is present. |
| `accesskit_windows-0.34.0/src/node.rs:804-827,837-843` | COM PlatformNode holds a Weak Context. Each call upgrades it to an Arc and retains that Arc while it executes under the tree read lock. Dropping the adapter cannot revoke an Arc already held by a running provider method. |
| `accesskit_windows-0.34.0/src/adapter.rs:377-385` | Active and Placeholder states own Arc Context; Inactive owns HWND and handlers. |
| `accesskit_windows-0.34.0/src/adapter.rs:535-593,617-625` | WM_GETOBJECT creates/returns provider COM objects. Conversion to LRESULT calls `UiaReturnRawElementProvider`. |
| `gpui-pre-windows-0.3.9/src/events.rs:836-850` | GPUI correctly releases its a11y RefCell borrow before LRESULT conversion because UIA may send nested WM_GETOBJECT. This existing reentrancy guard must remain. |
| `gpui-pre-windows-0.3.9/src/events.rs:110-111,320-349` | WM_CLOSE falls through to DefWindowProc if should-close callback is absent or approves. WM_DESTROY invokes close callback, then posts the platform close-one-window message. It does not explicitly take/drop a11y state. |
| `gpui-pre-0.3.9/src/window.rs:1750-1758` | Platform on-close callback marks the GPUI window removed. This happens as a consequence of WM_DESTROY. |
| `gpui-pre-windows-0.3.9/src/window.rs:611-630` | WindowsWindow Drop hides the HWND, clears visibility callback, clones the whole WindowsWindowInner Rc into an async task, awaits dialog idle, then calls RevokeDragDrop/DestroyWindow. The retained Inner also retains a11y state. No a11y state removal occurs in this path. |
| `gpui-pre-windows-0.3.9/src/window.rs:1442-1485` | WM_NCDESTROY closes dialog owner, then routes message and clears GWLP_USERDATA/drops the Weak box. It does not explicitly drop the accessibility adapter. |
| `accesskit_windows-0.34.0/src/tests/mod.rs:98-113` | Upstream's own test window drops adapter-owning WindowState at WM_DESTROY and handles a null state for nested WM_GETOBJECT during destruction. Comment explicitly identifies COM STA reentrancy. This differs from GPUI's retention path. It is supporting lifetime evidence, not proof that copying this alone resolves in-flight calls. |
| `gpui-pre-0.3.9/src/window.rs:6868-6878` | Public on_window_should_close callback can interrupt normal close. It cannot access private Windows a11y state or cancel another thread's already-upgraded Context. |

There are four coordinate consumers, all already inside Windows `Result`-returning provider methods:

1. `node.rs:1109-1121`: `IRawElementProviderFragment::BoundingRectangle`. Its current `Option::map_or` closure returns a UiaRect, so error propagation needs an explicit if/match or equivalent fallible mapping.
2. `node.rs:1148-1161`: `IRawElementProviderFragmentRoot::ElementProviderFromPoint`.
3. `node.rs:1456-1464`: `ITextProvider::RangeFromPoint`.
4. `text.rs:493-508`: `ITextRangeProvider::GetBoundingRectangles`; empty boxes currently return null and should retain that behavior.

### Narrow repair recommendation for the implementation owner

Patch the **single Windows adapter dependency**, retaining version/API/feature compatibility and accessibility. This is a proposed repair, not code implemented or validated by this agent:

- Change private `util::client_top_left` and `Context::client_top_left` to `windows::core::Result<Point>`.
- Convert the invalid-window failure into the existing `element_not_available()` error and propagate via `?` through the four callers. Preserving other Win32 failures as original errors is a defensible stricter alternative; the directly reproduced invalid-window code must map to UIA missing-element semantics. Do not substitute Point::ZERO or default success.
- Restructure BoundingRectangle's map_or into a fallible branch so `?` is not accidentally placed in a UiaRect-returning closure. Preserve the current no-bounding-box behavior.
- Keep all semantics, COM interfaces, nested WM_GETOBJECT borrow release, masking and existing event behavior unchanged. No generic panic hook or catch_unwind around extern/COM callbacks: the observed aborting boundary cannot be safely fixed that way.
- Reproducibility requires a repository-owned dependency patch under the independent desktop workspace, with exact upstream version/source/checksum, retained license notices and a focused patch diff. Do not mutate the global Cargo registry source. Vendoring this one adapter crate is distinct from copying GPUI/Kit/the full framework. Root and Tauri dependency graphs must remain unaffected; main owns any manifest/lock handoff.

An additional GPUI change that clears a11y state before destroying HWND can reduce stale-provider exposure, but it cannot by itself eliminate the in-flight Arc race. Therefore do not claim an application-level close hook or earlier adapter drop proves the underlying failure impossible.

### Dependency compatibility and upstream inspection

Local exact adapter manifest `accesskit_windows-0.34.0/Cargo.toml:12-33,48-93` declares edition 2024, MSRV 1.85, AccessKit 0.24.1, consumer 0.38.0, windows/windows-core 0.62, and MIT OR Apache-2.0 licensing. GPUI Windows manifest `gpui-pre-windows-0.3.9/Cargo.toml:59-60` selects accesskit_windows `0.34`; blindly selecting 0.35 is outside that compatible line and also moves AccessKit/consumer schema versions.

[Official AccessKit Windows changelog](https://raw.githubusercontent.com/AccessKit/accesskit/main/adapters/windows/CHANGELOG.md) lists 0.35.0 (2026-08-29) and 0.35.1 (2026-09-25); their listed changes do not establish a fix for invalid-HWND coordinate calls. [Official upstream util source](https://raw.githubusercontent.com/AccessKit/accesskit/main/adapters/windows/src/util.rs), retrieved 2026-10-18, still has the same ClientToScreen unwrap at web-source lines 352-357. This moving main reference is evidence of what was fetched, not a pinned released implementation. The latest source observation does not justify recommending a dependency upgrade as a fix.

Searches did not yield a verified matching upstream issue/PR/fix. Attempts to fetch a 0.35.1-tag source and GitHub commit API failed; no tag-level inspection or commit SHA is claimed. A no-result search does not prove no issue exists. Do not invent a patched released version.

### Verification required after an actual repair

1. Build/check the independent desktop workspace with `--locked`, verify the resolved adapter is the repository patch, and retain the final lockfile hash. Verify root/Tauri active dependency trees did not acquire the GPUI patch.
2. Add deterministic adapter-level regression coverage for conversion against an invalid/destroyed HWND, expecting `UIA_E_ELEMENTNOTAVAILABLE` rather than panic/default coordinates. Cover the real provider entry path while a Context remains strongly retained, because testing only Weak-upgrade failure misses this bug. A same-process synthetic HWND/provider fixture can retain Context intentionally after its window is destroyed; assert bounding/hit-test/text paths as applicable. Preserve valid-window coordinate behavior and existing empty-box behavior. Test API design must be compiled against this exact adapter; this report does not provide guessed public constructors.
3. Keep the normal application runtime cleanup checks separate. No terminal completion may be inferred from window disappearance or provider HRESULT.
4. Main repeats exact fixture native scenarios: normal load/filter close, close while delayed read, helper UIA activation/captures followed by Alt+F4, and reopening. Capture app exit code directly using a wrapper which explicitly returns `$LASTEXITCODE`; retain stderr/backtrace/build hash and verify no remaining process. A single non-reproduction cannot establish complete race freedom.
5. Accessibility transport must still be independently validated. A successful close is not a native a11y pass, and a successful headless semantic test is not OS UIA delivery evidence.

### Official API semantics

[Microsoft ClientToScreen](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-clienttoscreen) documents a zero result on failure; code must treat conversion as fallible. [Microsoft UI Automation error codes](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-error-codes) assigns `UIA_E_ELEMENTNOTAVAILABLE` value `0x80040201` to calls on elements that no longer exist, including destroyed elements. This supports the proposed missing-element response instead of terminating the process.

### Related specs

- `.trellis/spec/ccr-app/backend/index.md` and `shared-desktop-services.md`: application runtime outlives native loop, cleanup/terminal are real, native evidence separate from headless tests, no framework leakage into shared layer.
- `ccr-desktop/DESIGN.md` and task 03 native accessibility acceptance remain applicable; this repair cannot disable the platform semantics to pass exit verification.
- Foundation `research/native-a11y.md`: exact activation chain, semantic API and remaining null-provider investigation.

## Caveats / Not Found

- No code was changed, dependency fetched into the project, tests run, application launched, windows controlled, global setting changed, or task state modified. Only this research file was written.
- This agent did not read implement.jsonl/check.jsonl, preserving research role isolation.
- Initial attempts to read package-root index.md paths failed; corrected backend spec index and shared desktop service guide were read. No conclusion relies on the nonexistent root indexes.
- Supplied crash is real evidence from main, but the exact requesting COM interface and destruction ordering still need a preserved stack. The source-backed race explanation is a strong mechanism, not an independently captured trace.
- No confirmed released upstream fix, exact upstream issue, or application-only supported API solution was found. Patch implementation, license packaging, lock regeneration and validation belong to the authorized implementation/check owner.
