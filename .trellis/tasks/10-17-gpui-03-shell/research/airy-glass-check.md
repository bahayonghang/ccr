# GPUI airy-glass independent check

Date: 2026-10-27. Role: dispatched `trellis-check` for03. The user resumed the existing task; main explicitly assigned the visual scope and the two narrow background/output follow-ups. Shared changes were preserved. No native app automation, real account/config access, OS setting change, dependency/lock edit, commit, push, publication or archive was performed by this reviewer.

## Reviewed scope

Read the complete saved native hook output and its non-inlined Tauri DESIGN reference,03 PRD/design/implementation, curated specs/research, before-dev/check instructions, relevant spec indexes and shared thinking guides. Tauri design is a visual reference; it does not replace the native density, single-theme owner, input, lifecycle or evidence rules. Reviewed the actual `theme.rs`, shell, all existing route background owners and Kit/GPUI source rather than assuming browser clipping behavior.

The locked source/API capability conclusion is appropriately bounded: no application-backdrop sampling primitive exists in the inspected public rendering surface; window Blurred/Mica and shadow blur cannot substitute. Opaque fallback is correctly enabled unconditionally; unknown native transparency/high-contrast preference APIs do not accidentally enable glass. No speculative flags, upgrade, renderer/shader, extra theme owner or polling loop were introduced.

## Findings fixed

### Opaque descendants covered the rounded paper corners

`gpui-pre-0.3.9/src/style.rs:638-682` creates rectangular overflow masks; `src/window.rs:2231-2234` defines `ContentMask` with bounds only. Parent `.rounded(20px).overflow_hidden()` therefore does not clip child backgrounds to a rounded path. The sidebar, configuration reading root and newly painted status bar could cover the paper silhouette.

The independent regression uses the public `Window::painted_quads()` output of the actual rendered shell, not source string matching. Before repair it failed101 at `sidebar covers paper corner on nav-configurations`; see `outputs/desktop/airy-glass/2026-10-27/check/corners-before-render.log`. Local repair gives only the touching sidebar top-left and configuration top-right backgrounds an inner19px radius, keeping the1px paper outline and existing geometry. Other routes have transparent content roots over the same solid paper and need no new background/container.

A separate status background cannot match the20px bottom paper corners: GPUI clamps each quad's radius to half its smallest dimension, reducing a19px radius on the28px status to14px. The smallest repair removes that newly added duplicate background and lets the existing opaque paper own bottom paint. Status text,28px height, border, focus and semantic ID stay unchanged. `SHEET_CONTENT_RADIUS` belongs to the same theme owner; no renderer changes or extra decorative layers.

The final regression checks actual sidebar/configuration corner radii, actual20px paper bottom radii, and absence of independent wide status overpaint across all five existing routes in light/dark at one headless1280×800 viewport. The headless backend's existing scale factor is respected; this is not native DPI/size-matrix acceptance. Existing query retention, F6/Tab/editing and plaintext-navigation tests still run.

Initial reviewer test-authoring failures are also retained: `corners-before.log` used a nonexistent `f32::from(ScaledPixels)` conversion; subsequent harness assumptions treated scaled dimensions as logical. The final test uses public `as_f32()` with the actual window scale factor. `corners-after*.log` and `corners-diagnostic-paint.log` record those intermediate failures; they are not reported as additional product bugs. Temporary paint diagnostics were removed.

### Explicit output role had no production consumer

`Palette::output` was defined/tested but unused. The existing converter now places a visible bilingual purple `Converted preview` label directly above its readonly result. Only that explicit result label uses output purple; ordinary code text, input, success/error feedback and selection retain their appropriate roles. The same textarea entity, accessibility ID, content, readonly behavior and180px height remain.

Kit Input installs its editor foreground from the Theme directly (`gpui-component-0.7.3/src/input/input.rs:558`), so setting an outer text color would not truthfully establish converted editor text coloring. The visible label avoids another input/theme override. Main explicitly approved this bounded presentation choice.

## Verification

Final commands run with existing actual just1.59 and paired Rust1.98 selected through child-process PATH. `desktop-fmt` was a separate repair before validation. Raw command times/exits are in `outputs/desktop/airy-glass/2026-10-27/check/gates.json`.

- PASS: final `just desktop-fmt-check`, locked all-target `just desktop-check`, locked strict `just desktop-lint`.
- PASS: full `just desktop-test`: Python12, AccessKit patch3, Rust library17, native_shell4, native_ui2; zero failures. Default Rust parallelism and `--skip export_bindings` retained. This combined tree includes the other checker's ccr-config current-marker repair.
- PASS: independent final painted corner regression, `corners-final.log`; existing theme contrast/opaque material/query retention/input/confirmation/plaintext lifetime cases remain in the complete suite.
- PASS: scoped `git diff --check`; no generated-binding or lock edits.
- PASS: `just desktop-build-debug`; the immutable copied candidate is `outputs/desktop/airy-glass/2026-10-27/ccr-gpui-solid-candidate.exe`, SHA256 `ed00067d7a6a2f88356c69d1b6e24417680ba7f9c3c170e49a3ca893e9d0eaab`. `check/candidate-receipt.json` records531 conservative source/manifest hashes and gate identity. A compiled candidate is not native acceptance. The shared compile hold was released only after this copy/hash; subsequent05 sources do not change this artifact.

## Findings not fixed / acceptance limits

No remaining concrete code defect was identified in this bounded visual increment after these repairs. The deliverable remains an opaque fallback; actual application glass is unsupported on the inspected API surface. Native light/dark pixels, keyboard/IME composition, UIA/screen reader, reduced-transparency/high-contrast changes, macOS/Linux, S1/S2 timing/frame/CPU/memory budgets and full native profile flows are not proved by this review. Main owns actual candidate-window evidence and integration. Window-size/DPI matrices retain user-deferred status. No03/04/parent completion or archival is asserted.
