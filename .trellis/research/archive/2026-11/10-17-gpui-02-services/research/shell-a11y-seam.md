# Research: 03 same-node accessibility state seam

- Query: Find the smallest exact-version way to add disabled/readOnly/busy/invalid/live state to existing Kit controls, retaining their semantic identity, focus, actions and input/IME implementation.
- Scope: internal; read-only source inspection of locked gpui-kit/base/component 0.7.3, gpui-pre 0.3.9 and accesskit 0.24.1. No product edit, Cargo command, UI operation or 03 activation.
- Date: 2026-10-18

## Findings

### Decision: an application-only Button seam exists; styled Input needs a narrower component seam

The earlier `shell-source-map.md` suggestion to forward `Element::write_a11y_info` needs qualification. It is correct only when the application actually owns the **concrete semantic Element**. Wrapping arbitrary `AnyElement`, `Input::into_element()` or `Button::into_element()` does not expose their inner semantic node.

There is a smaller public mechanism than writing a custom Element for the controls that expose `InteractiveElement`: install `StatefulInteractiveElement::a11y_synthetic_children`, then mutate **`A11ySubtreeBuilder::parent_node()`**, without adding a synthetic child. This changes the existing node after normal production and retains its original role/name/ID/bounds/actions/children. For styled Button, a temporary builder-only newtype can expose this trait and then return the **original Button**. The newtype is never rendered. Styled Input does not expose `InteractiveElement`; this mechanism cannot reach its private generated BaseInput from the application.

Cache paths below use `CACHE = C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. These are the installed locked sources, not current upstream HEAD. The existing Windows adapter vendor patch is concurrently owned elsewhere and is outside this report.

### Exact public types and call chain

| CACHE source:line | Contract |
| --- | --- |
| `gpui-pre-0.3.9/src/elements/div.rs:768-770` | Public `InteractiveElement: Sized`, only required member `fn interactivity(&mut self) -> &mut Interactivity`. |
| `.../elements/div.rs:1306` | Public `StatefulInteractiveElement: InteractiveElement`; **no Element/RenderOnce/sealed supertrait**. An application-owned temporary newtype may implement it. |
| `.../elements/div.rs:1385-1390` | Builder `fn a11y_synthetic_children(self, f: impl FnOnce(&mut A11ySubtreeBuilder) + 'static) -> Self` stores callback on the same Interactivity. It is a single Option, not an additive callback list. |
| `.../window/a11y.rs:304`, `:363-367` | Public builder and `pub fn parent_node(&mut self) -> &mut accesskit::Node`. Parent here is **the element whose callback is running**, not a separate outer container. |
| `.../elements/div.rs:1913-1924` | Div first writes its normal Interactivity semantics; later takes and executes the stored callback. |
| `.../element.rs:428-442` | Existing node is created only with active a11y, global element ID and Some role; normal `write_a11y_info` runs before insertion. |
| `.../element.rs:465-499` | Normal element/children prepaint completes, callback runs against that still-current node, then node is popped. No extra role node is needed. |
| `gpui-component-0.7.3/src/button/button.rs:602-605` | Styled Button publicly implements InteractiveElement by forwarding to `self.base`. |
| `.../button/button.rs:620`, `:654`, `:788-830` | That same base is used for rendering, preserving role (Button or Link), selected/toggled/name, ID, focus and content. |
| `gpui-base-0.7.3/src/button.rs:198-205`, `:214-256` | Base Button forwards Interactivity and renders its existing base; disabled suppresses normal click/focus setup, but does not set native disabled flag. |
| `gpui-base-0.7.3/src/test_support.rs:382-387` | ObservedElement forwards `a11y_synthetic_children` to the inner element, so the Base observation wrapper does not discard this callback. |

This mechanism is synchronous in the render/prepaint path and must only mutate the supplied node. Do not do I/O, acquire application service locks, allocate unbounded data, start jobs or retain the mutable Node reference.

### Concrete Button application shape, with no rendered wrapper

The following shape uses only the public exact-version APIs above. It is a research example, **not compiled or installed by this agent**. The implementation owner should compile it in its focused headless semantic tests before relying on it.

```rust
use gpui_kit::{accesskit, InteractiveElement, Interactivity, StatefulInteractiveElement};
use gpui_kit::component::{button::Button, Disableable};

// Builder access only. Never derives IntoElement or implements RenderOnce.
struct SemanticBuilder<T>(T);

impl<T: InteractiveElement> InteractiveElement for SemanticBuilder<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.0.interactivity()
    }
}

impl<T: InteractiveElement> StatefulInteractiveElement for SemanticBuilder<T> {}

fn button_states(button: Button, disabled: bool, busy: bool) -> Button {
    // Keep behavioral disabling in the real component as well as native state.
    let button = button.disabled(disabled);
    StatefulInteractiveElement::a11y_synthetic_children(
        SemanticBuilder(button),
        move |tree| {
            let node = tree.parent_node();
            if disabled { node.set_disabled(); } else { node.clear_disabled(); }
            if busy { node.set_busy(); } else { node.clear_busy(); }
        },
    ).0
}

fn describe_validation(node: &mut accesskit::Node, invalid: bool) {
    if invalid { node.set_invalid(accesskit::Invalid::True); }
    else { node.clear_invalid(); }
}
```

No role, ID, label, children, action support, focus handle, event handler or layout state is reconstructed. The result is the original styled Button. This also avoids introducing a `ViewElement<Wrapper>` component identity scope around the button.

Constraints that must stay explicit:

- The helper should be private to the shell, applied only to known controls whose single callback slot it owns. There is no public getter to compose an unknown previously installed callback. A second call replaces the first (`div.rs:1389`). A shared setter accepting a flags struct can install **one** closure once per element; do not layer several helpers each overwriting the slot.
- Locked styled Button and Base Button contain no internal `a11y_synthetic_children` installation in their inspected render paths. A future dependency upgrade can change this, so retain a focused regression test and recheck the source diff on upgrade.
- Match `busy` to actual loading/job state. Existing `.loading(...)` has behavioral/visual semantics (`button/button.rs:445`, `:836-863`); setting busy alone does not disable an action or implement cancellation. Do not infer every busy control is disabled.
- Do not treat `set_disabled()` as event security. Existing `.disabled(...)`, enabled predicates and actual action policy remain authoritative. Native state must agree with those behaviors.
- A node with no role/ID still has no callback; this helper is not a way to manufacture missing semantics. Existing Button::new requires ID (`button/button.rs:242`). Keep explicit accessible name for icon-only buttons.

For a shell-owned `div().id(...).role(...)`, call the same public `StatefulInteractiveElement::a11y_synthetic_children` directly; the temporary newtype is unnecessary. For a status node, `tree.parent_node().set_live(accesskit::Live::Polite)` can set live-region policy on that actual status node. Do not mark the whole shell or configuration list live; announce actual bounded status changes. A semantic flag does not prove a native screen reader announced the update.

### Why forwarding AnyElement/RenderOnce does not safely fix Input

| CACHE source:line | Boundary |
| --- | --- |
| `gpui-pre-0.3.9/src/element.rs:52-121`, `:132-136` | Element exposes id/source/layout/prepaint/paint/role/write-info/synthetic-child hooks. A raw semantic Element can be forwarded safely only by preserving **all** these hooks and its associated layout/prepaint state. |
| `.../element.rs:728-775` | AnyElement's Element impl returns `id=None`, no role/write-info override, and delegates to its boxed Drawable through its own layout/prepaint/paint methods. Its inner semantic node is produced later. Calling `Element::write_a11y_info(&any, node)` invokes the default empty hook. |
| `.../element.rs:663-666` | Public downcast_mut only exposes the directly boxed concrete element. It does not traverse private rendered children or unwrap views. Do not use casts or unsafe layout assumptions. |
| `gpui-pre-macros-0.3.9/src/derive_into_element.rs:13-18` | Derive IntoElement creates `ViewElement<Self>`. It does not eagerly render the component's root Div. |
| `gpui-pre-0.3.9/src/view.rs:258-266`, `:328-378` | ViewElement keeps its view private, has no role/write-info override, and renders an erased child during request_layout. A wrapper around this view cannot transparently decorate the child's node by forwarding the view's default hooks. |
| `gpui-component-0.7.3/src/input/input.rs:110-139`, `:529-536` | Styled Input has private fields, implements Styled and RenderOnce, **does not implement InteractiveElement/StatefulInteractiveElement**, and exposes no public callback or root builder. |
| `.../input/input.rs:741-780` | Its RenderOnce builds BaseInput with original ID/focus/content role/name/value, Focus and editable-only SetValue handlers. Return signature is only `impl IntoElement`; callers cannot require the hidden root to implement another trait. |
| `gpui-base-0.7.3/src/input/base/mod.rs:126-135`, `:219-240` | BaseInput itself already exposes both interactive traits, but its instance is hidden inside styled Input. Its final base supplies the actual node. |

Consequences:

1. `Forward { inner: Input::new(...).into_element(), flags }` has no inner semantic role/ID to forward. Adding a TextInput role and new ID to the wrapper yields a second edit node while the inner real TextInput remains; this is not acceptable.
2. Even `RenderOnce::render(input, window, cx).into_element()` exposes a ViewElement<BaseInput>, not the final Div through a public field. The installed component's semantic test at `input/input.rs:1073-1078` calls `write_a11y_info` on this result; that call is not proof that an application's wrapper can intercept its eventual inner node. Assert the actual rendered tree, not a fresh manually constructed accesskit Node alone.
3. Rebuilding the Input children or substituting another TextInput semantic control would risk text input/selection/IME/security behavior. None is justified by this gap. Keep the existing editor/state entity, `set_disabled`/`set_readonly` (`input/input.rs:608-609`), masked-value guard (`:89-95`), and SetValue gating (`:776-780`).

For a **raw typed Element** the genuine forwarding shape is `struct Flags<E: Element> { inner: E, ... }`, identity `IntoElement<Element=Self>`, associated states exactly `E::RequestLayoutState`/`E::PrepaintState`, id/source/role copied from `inner`, `request_layout`/`prepaint`/`paint` delegated directly with their original IDs/bounds/state, `write_a11y_info` calls inner first then sets flags, and `a11y_synthetic_children` delegates too. Do not call `inner.into_any_element().prepaint` in this adapter: that introduces another Drawable identity/semantic boundary. Exact signatures are at `element.rs:66-105`, `:113-136`; GPUI's own `Stateful<E>` follows this transparent pattern at `elements/div.rs:4096-4168`. This is useful inside the component owner at the final semantic element, but **is not a generic solution to styled Input's opaque root**.

### Smallest upstream patch boundary for styled Input

The narrowest correction does **not require patching GPUI or AccessKit** and need not patch Base at all: change the existing styled Input render in `gpui-component-0.7.3/src/input/input.rs` where it still owns the BaseInput, before `:741-780`'s builder is erased.

- Capture existing `disabled`/`readonly` presentation state, then install **one** `StatefulInteractiveElement::a11y_synthetic_children` callback on that same BaseInput (trait already imported at `:7`, public impl at Base `:230`). Mutate `parent_node().set_disabled/clear_disabled` and `set_read_only/clear_read_only`. Keep original base creation, role/value/name/ID/focus/actions, children and editor entity untouched. No separate semantic node or event system.
- If this particular Input must expose busy/invalid/live, add only a narrow optional semantic-state/callback property to **styled Input**, initialize it in `Input::new` (`:175` onward), and apply it in that same single callback. Prefer explicit flags or an application-owned callback over exposing/moving the entire BaseInput. Treat any override callback as trusted app code; it must not replace masked value or invent action support. No such API exists today.
- This is a small dependency-source patch in one component file plus focused existing component tests. A **Base-owned** alternative for all InputBase consumers would add the readOnly semantic property to `gpui-base/src/input/base/mod.rs` (existing fields at `:127-134`, builder at `:159`, final render `:232-240`), set disabled/readOnly at its existing semantic root, and forward styled Input's existing readonly into it at component `:743`. That spans Base + Component and is broader than the single styled-render correction. Do not pretend `InputBase::readonly(...)` already exists.
- For a framework-level durable mapping, an optional tiny transparent `Element` state decorator inside Base could retain all hooks as described above; wrapping the final typed base would preserve an existing synthetic callback. This is only justified when Base consumers require the same contract. Avoid replacing an existing callback by blindly adding another `.a11y_synthetic_children` to a public Base builder.
- Deliver any accepted dependency correction through a reviewable exact source revision/patch with provenance, lock refresh and `--locked` verification. Editing the machine's Cargo cache is not a repository-delivered fix. Do not copy all of Kit/GPUI, switch to Fast, disable a11y, or conflate this component flag gap with the Windows adapter exit race. Package patch delivery/maintenance is an implementation decision; this researcher changed no manifests or dependencies.

Maintenance cost: one bounded component diff, state-output and editable-action/password regression tests, upstream tracking, and source recheck on upgrades. No application-only safe existing-node Input mutation API was found in this exact version. If 03 uses only an enabled editable search Input, record that actual case; do not invent a disabled/readonly fixture result or silently waive later states required by the accepted contract.

### AccessKit type and feature compatibility: no direct dependency required

`gpui-pre-0.3.9/src/gpui.rs:97` publicly re-exports the **accesskit module**. Kit re-exports GPUI's public surface at `gpui-kit-0.7.3/src/lib.rs:100-103`. Therefore use `gpui_kit::accesskit::{Node, Invalid, Live}`. `gpui_kit::Role` and `AccessibleAction` are also re-exported (`gpui.rs:98-99`). A new direct accesskit dependency is unnecessary.

The exact normal-engine graph is Kit 0.7.3 -> package `gpui-pre =0.3.9` (`gpui-kit/Cargo.toml:404-406`) and Base/Component 0.7.3 (locked resolution of manifest `:408-413`) -> the same GPUI type family -> `accesskit ^0.24.0`, `features=[enumn]` (`gpui-pre/Cargo.toml:267-269`) -> locked 0.24.1 (`ccr-desktop/Cargo.lock:6-7`). Desktop selects component/assets, not gpui-fast; test-support enables the matching GPUI/Base/Component test paths (`gpui-kit/Cargo.toml:72-79`). The flags below are ordinary core Node methods and require no extra feature.

| accesskit 0.24.1 source | Public methods/types |
| --- | --- |
| `src/lib.rs:1087-1102`, `:1805`, `:1814-1816` | `set_busy()`, `set_read_only()`, `set_disabled()` take **no boolean argument**; use matching `clear_*()` for false. |
| `src/lib.rs:465-469`, `:2137` | `Invalid::{True,Grammar,Spelling}`; `Node::set_invalid(Invalid)` or `clear_invalid()`; no `Invalid::False`. |
| `src/lib.rs:584-588`, `:2139` | `Live::{Off,Polite,Assertive}`; `Node::set_live(Live)` or `clear_live()`. |

If a future independent crate truly needed a direct accesskit dependency, the same package/version/source instance is what makes its Node type identical; adding an arbitrary latest accesskit may produce an incompatible Node even if method names match. The re-export removes that risk and the manifest change entirely here.

### Verification handoff and related contracts

No commands below were run by this researcher. The 03 implementation/check owner should:

1. Compile the exact builder-only Button adapter in desktop's existing Kit test-support target. Render the **real styled Button** and assert one node retains role/name/automation ID, existing bounds and original action callbacks while disabled/busy transitions are reflected. Test keyboard/pointer and a11y invocation, not only node flags. Check that active-to-disabled transitions prevent the actual handler; do not assume the flag repairs pre-existing focus behavior.
2. For any accepted Input patch, assert real-tree exactly-one text input, unchanged ID/name/focus, no SetValue when disabled/readOnly, normal SetValue when editable, readOnly still selectable/copyable, and no masked/password value leakage. Existing tests plus new state cases are better than copying/replacing Input/IME handlers.
3. Verify live status changes and invalid correction in the actual app tree. Own one synthetic callback slot and assert status updates are bounded/local. Internal tree assertions are semantic-output evidence; main-session native UIA/screen-reader evidence is still required separately.
4. Run the authorized desktop test/lint/locked gates and native state matrix after the independent Windows provider race patch is integrated. That patch is owned by another agent. This report neither diagnoses nor verifies it.

Related sources/specs read or retained from the prior report: `.trellis/workflow.md`; `.trellis/spec/guides/cross-layer-thinking-guide.md`; approved 03 `prd.md`/`design.md` and `ccr-desktop/DESIGN.md`; `01/research/native-a11y.md`; `02/research/shell-source-map.md`. The accepted keyboard/IME/native-state requirements remain unchanged. No external latest-version documentation was needed: exact local locked source is the authority for these APIs.

## Caveats / Not Found

- Only this research Markdown was written. No product/cache/manifests/lock/task-state edits, builds, tests, UI/process launches, OS setting changes, or git actions occurred. No implement/check manifests were loaded.
- The concrete adapter is source-backed and supplies all required trait members; it has **not** been compilation-tested in this research role. Its focused compile/tree tests remain required before accepting the implementation.
- An application-only existing-node mutation seam exists for styled Button through public Interactivity and temporary trait access. No equivalent public seam was found for styled Input. These are different cases and should not be collapsed into a universal wrapper claim.
- Flags alone neither enforce security/action policy nor prove Windows UIA state delivery or screen-reader announcements. Native validation remains the main session's exclusive responsibility.
