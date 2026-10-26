# Research: Windows native accessibility in the locked GPUI stack

- Query: Explain the Windows AccessKit/UIA integration and activation conditions in gpui-kit 0.7.3 / gpui-pre 0.3.9, the actual Input semantics, and the smallest evidence-based diagnosis for `sky.get_window_state(include_text: true).accessibility == null`.
- Scope: mixed; local exact-version source inspection, official AccessKit API documentation, and bundled helper API documentation. No application launch, native-window interaction, OS-setting change, source edit, lock edit, task-state edit, or test execution.
- Date: 2026-10-17

## Findings

### Conclusion supported by source

The locked stack **does implement Windows UI Automation through AccessKit**, with normal application creation enabling integration by default. No missing Cargo accessibility feature or `WindowOptions` switch was found in this code path. Current null helper captures are an unresolved verification result, not evidence that this framework lacks native accessibility.

The existing probe already has an Input node ID, a default TextInput role, and placeholder-derived accessible name. Adding explicit label/automation ID is useful for stable verification but cannot be claimed to fix the helper's null result without a native recheck. The framework constructs the full tree lazily after a UIA client activates the adapter; its first response is intentionally only a Window root, followed by an async refresh. That makes a first-capture timing issue plausible, but repeated null captures have **not** been causally explained.

### Version/source identity

All following abbreviated cache paths are relative to:

`C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`

These are real local Cargo sources, not inferred upstream HEAD behavior.

| Manifest/lock evidence | Exact source/version |
| --- | --- |
| `ccr-desktop/Cargo.toml:18` | `gpui-kit = =0.7.3`, default features disabled, `component`, `assets`; tests additionally use `test-support`. |
| `ccr-desktop/Cargo.lock:2590` | gpui-pre 0.3.9 registry package. |
| `ccr-desktop/Cargo.lock:2186` | gpui-component 0.7.3 registry package. |
| `ccr-desktop/Cargo.lock:3024` | gpui-pre-windows 0.3.9 registry package. |
| `ccr-desktop/Cargo.lock:6`, `:72` | accesskit 0.24.1 and accesskit_windows 0.34.0. |
| `gpui-pre-platform-0.3.9/Cargo.toml:29-33` | Snapshot metadata names upstream zed revision `93d745668805029476e6d7fbb19652116fed51de`. |

`gpui-kit-0.7.3/src/lib.rs:100-103` selects gpui by default; Fast is feature-gated. The installed sources for Fast do not imply that the application selected it. No switch to Fast or another framework is warranted by current evidence.

### Platform chain and activation

| Actual source anchor | Meaning |
| --- | --- |
| `gpui-kit-0.7.3/src/lib.rs:195` | Re-exports platform `application`. |
| `gpui-pre-platform-0.3.9/src/gpui_platform.rs:13-20` | Native `application()` calls `Application::with_platform(current_platform(false))`. |
| `gpui-pre-0.3.9/src/app.rs:179-199` | Normal `with_platform` creates the app. Only explicit `Application::new_inaccessible` forces accessibility off. |
| `gpui-pre-0.3.9/src/app.rs:1011` | `accessibility_force_disabled` defaults false. |
| `gpui-pre-0.3.9/src/window.rs:1666-1686` | Non-Wasm window creation initializes the platform adapter unless force-disabled. Initial tree has a Window role and title, no application children yet. |
| `gpui-pre-0.3.9/src/window.rs:1688-1724` | Activation callback sets atomic flag and returns the initial Window tree; foreground async receiver requests a new frame to provide the full tree. |
| `gpui-pre-windows-0.3.9/src/window.rs:1080-1097` | Windows `a11y_init` creates `accesskit_windows::Adapter::new(HWND, is_focused, action_handler)` and stores its activation handler. |
| `gpui-pre-windows-0.3.9/src/events.rs:164`, `:836-850` | `WM_GETOBJECT` is dispatched to AccessKit; locks/RefCell borrows are released before the nested-UIA-sensitive conversion to LRESULT. |
| `accesskit_windows-0.34.0/src/adapter.rs:535-563` | Handles UIA root/client object requests and lazily activates the tree. Requests for other negative MSAA object IDs intentionally return None. |
| `gpui-pre-0.3.9/src/window.rs:3673-3675`, `:3772-3795` | Full semantic tree is built when active, then sent at frame completion. |
| `gpui-pre-windows-0.3.9/src/window.rs:1100-1113` | Update goes through `adapter.update_if_active`; queued UIA events are raised after releasing the borrow. |
| `gpui-pre-windows-0.3.9/Cargo.toml:55-60` | Windows AccessKit dependencies are target-selected and non-optional. There is no optional `a11y` feature to add for this path. |

Current app creation in `ccr-desktop/src/lib.rs:9` uses `gpui_kit::application()`, not `new_inaccessible`. Its titlebar/app_id/window options do not turn accessibility off. Do not add guessed flags or change the approved dependency stack to address null.

Public exact-version diagnostics are available:

- `Window::is_a11y_enabled()` (`gpui-pre-0.3.9/src/window.rs:6992`) reports whether support is force-disabled, independent of client connection.
- `Window::is_a11y_active()` (`:7006`) reports active support for the current frame, after client activation.
- `Window::debug_a11y_tree_json()` (`:7011`) exposes the last captured tree, including frame identity, scale factor, viewport, node count, focus and node provenance in debug builds. Implementation retains the last sent update at `src/window/a11y/debug.rs:69-96`, and returns None before a captured tree exists (`:108-109`). This is diagnostic app-side data, not independent proof of OS delivery or screen-reader behavior.

### Actual Input role, name, value, action and state semantics

`gpui-component-0.7.3/src/input/input.rs` is the relevant styled Input implementation.

| Anchor | Actual behavior |
| --- | --- |
| `:27-44` | Default single-line role TextInput; multiline MultilineTextInput; explicit content types choose phone/email/URL/password/date roles. |
| `:255-272` | `.accessibility_id(...)` sets developer-facing automation identity; `.aria_label(...)` sets explicit accessible name. This is distinct from `.id(...)` used for GPUI element identity. |
| `:608-609` | Component updates editor disabled/read-only behavior in state. |
| `:679-685` | Non-secret text value is materialized only when native a11y is active or test-support is enabled. |
| `:89-95` | Masked and password content does not expose plaintext accessibility value. Do not remove this guard while debugging. |
| `:728-737` | Without explicit aria_label, ordinary non-mask placeholder becomes the accessible name. |
| `:739-771` | Stable ID (provided or state-derived), semantic role, optional automation identity, label, placeholder and value are forwarded to BaseInput. |
| `:772-780` | Focus action exists; SetValue action is registered only when editable. |
| `:1050-1096` | Upstream headless test builds an accesskit Node from the actual rendered Input and verifies emitted value and SetValue support. Test-support eagerly reads the same value path without a native client. |

The current probe at `ccr-desktop/src/probe.rs:55` creates `Input::new(&self.input).id("probe-input")`. Its state placeholder is set at `:22`. Consequently, it already has a global element ID, TextInput role, and expected name `Type here / 在此输入中文` through the fallback. A minimal optional clarification is `.aria_label("Native input probe / 原生输入探针").accessibility_id("probe-input")` on this same Input. It provides deterministic name and UIA AutomationId; it is not a proven null-tree fix.

Input role is mapped to Windows Edit control type in `accesskit_windows-0.34.0/src/node.rs:79-91`. UIA Name/IsEnabled/HasKeyboardFocus/AutomationId properties are exposed at `:1309-1319`; value pattern support and read-only properties are at `:589-615` / `:1346`.

**State caveat with direct source support:** `gpui-base-0.7.3/src/input/base/mod.rs:159-161` only stores the disabled flag; the InputBase render at `:235-243` applies role, children and visual styles. Its disabled styles are resolved at `:175-183`. The styled component removes editing behavior, but no `accesskit::Node::set_disabled` / `set_read_only` call was found in this Input frame path. GPUI Interactivity's `write_a11y_info` at `gpui-pre-0.3.9/src/elements/div.rs:3530-3601` forwards listed ARIA data/actions but does not synthesize these flags. AccessKit UIA `IsEnabled` directly reads `!node.is_disabled()` (`accesskit_windows/src/node.rs:534-536`), and read-only directly reads the consumer node (`:613-615`, consumer `node.rs:452-458`). Therefore behavior-only assertions are insufficient for disabled/read-only state. This is a source-backed semantic-risk item for 03, not an explanation for the current enabled Input's entire tree being null. Capture actual node/UIA flags before proposing a targeted component/adapter fix; do not claim SetValue absence automatically produces correct ReadOnly state.

### Text and structure semantics in this probe

GPUI's guide requires both a stable global ID and non-None role for nodes to enter the accessibility tree (`gpui-pre-0.3.9/src/_accessibility.rs:56-68`). Plain `.child("...")` strings in this probe are not evidence of semantic labels/headings. The guide explains `text!`/`Text` and source-location-derived stable IDs at `:110-180`; repeated list text needs stable per-item IDs.

For 03, mark actual headings/status text/controls with the framework's native semantic APIs and stable IDs. Do not add a single parent label that flattens or hides interactive descendants, or duplicate an entire input with a separate semantic-only control. Root Window title is already supplied by GPUI.

### Helper result: what is known and what remains unknown

Bundled helper API docs at `C:/Users/lyh/.codex/plugins/cache/openai-bundled/computer-use/26.1013.40934/docs/api.md:54-61` define `include_text` and a nullable `accessibility` field. `:122-128` defines the formatted tree and optional focused/selected/document fields. Those docs do **not** specify which provider errors, activation timeouts, empty-tree conditions or app-attribution failures map to null.

The supplied `research/native-probe.md:49` reports repeated include_text captures were null. The same report notes helper process attribution to a parent host app (`:16`). It is reasonable to check exact target window/HWND/PID attribution during the next capture, but this is a hypothesis: no helper implementation was inspected and no causation is established. Do not infer helper supports only MSAA, that it never activates UIA, that security elevation is responsible, or that a third-party tool will necessarily fix it.

### Smallest repair/verification path for the authorized owner

1. Preserve current deps, lock and normal `application()` constructor. No extra Cargo feature, window option or global OS accessibility change is justified by this inspection.
2. If improving probe observability, let the current product writer add only explicit Input aria_label/automation ID and a fixture-only on-demand diagnostic that records `is_a11y_enabled`, `is_a11y_active` and `debug_a11y_tree_json` on the GPUI main thread. Send the captured string to a background writer if saving a file; do not introduce synchronous filesystem work into render. The diagnostic must not create a continuous timer, force fake successful state, or expose business secrets. A bounded keyboard action is sufficient; use the existing fixture input.
3. Main session alone reopens/targets the actual native window. Capture the exact window identity, PID, build SHA, enabled/active state, and a first helper capture. Trigger a normal native accessibility client read, allow GPUI's scheduled frame to complete, then recapture rather than interpreting the initial Window-only tree as final. No evidence here authorizes the research agent to launch clients, modify OS settings, or control windows.
4. If helper remains null, use an available standard Windows UIA inspector/read client under the main session's applicable tool/permission rules to read the **same HWND**. Record Window root, Edit descendant, Name, AutomationId, focus, IsEnabled, Value/ReadOnly patterns. Do not replace the required native check with OCR, screenshots, headless mocks or the app's own JSON tree. The source supports this verification approach; availability and result remain unverified.
5. Interpret observations narrowly: enabled=false indicates explicit app construction problem; enabled=true/active=false after a client query isolates activation/provider/client routing; active=true with a correct app tree but external null narrows to adapter/client retrieval; active=true with missing Input narrows to semantic/render identity; external tree with incorrect flags narrows to component semantic mapping. None alone proves full screen-reader usability.
6. Add focused headless semantic tests for explicit name/role/value/no password leak using `gpui_kit::test::{TestWindowExt, ...}` and `ElementSnapshot` only if changing probe/components. `gpui-base-0.7.3/src/test_support.rs:65-113` exposes role/value/focus/label and notes disabled=None means no native disabled flag; `:301-318` reads actual node info. These tests establish semantic output, not Windows UIA transport. Then re-run desktop native verification and record actual results separately.
7. For disabled/read-only states, first capture the real flags; if the source-backed gap reproduces, scope a fix to existing semantic node production with actual `set_disabled`/`set_read_only` support and preserve action/security guards. Avoid wrapping Input in a second edit-role div or relabeling tests as passed. Framework patch/version decisions need their own demonstrated evidence and compatibility review; none is recommended unconditionally here.

### Official references

- [AccessKit Windows Adapter 0.34.0](https://docs.rs/accesskit_windows/0.34.0/accesskit_windows/struct.Adapter.html): describes lazy tree activation via WM_GETOBJECT, active-only tree updates, and event raising. This agrees with the local exact-version source. Its constructor initializes UIA before WM_GETOBJECT handling; GPUI already calls it during window setup.
- The shipped primary GPUI accessibility guide is `gpui-pre-0.3.9/src/_accessibility.rs`; the exact-version source is preferable to newer online HEAD documentation. An attempted docs.rs rendered-guide fetch failed, so no claim relies on that page being retrieved.
- Attempted Microsoft UIA documentation retrieval failed through the web tool; no external Microsoft page is claimed as verified. Actual UIA properties listed above are verified in accesskit_windows 0.34.0 source.

### Related specs / approved contracts

- `.trellis/tasks/10-17-gpui-01-foundation/prd.md` native feasibility requirement, current `research/native-probe.md` actual evidence and gaps.
- `.trellis/tasks/10-17-gpui-03-shell/prd.md` and parent native input/keyboard/accessibility acceptance remain unchanged; this research does not mark them complete.
- Parent `research/performance.md` and design require real native evidence, no main-thread synchronous I/O and no fabricated success from empty probes.

## Caveats / Not Found

- All file inspections were read-only except this research file. No source/Cargo/task/OS/user-account changes, tests, native tool calls or process launches were performed. No sandbox escalation was needed to read these cache sources.
- No helper internal implementation/error classification was inspected; nullable API docs cannot establish why this session returned null. Repeated null remains unresolved.
- No proof of actual Windows screen-reader output, UIA focus delivery, complete text-selection/reading patterns, disabled/read-only UIA flags, or macOS/Linux behavior was obtained here.
- Explicit label/automation ID improves observability but does not fix an established root cause. The currently enabled probe already has default Input role and placeholder name semantics.
- Research-role manifest isolation was maintained: no implement.jsonl or check.jsonl was loaded. The earlier workflow/context reads remain applicable; task activation was not changed.
