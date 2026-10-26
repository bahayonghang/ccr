# Research: GPUI 03 shell source map

- Query: Identify exact locked Kit APIs needed by the approved 03 shell, including themes, focus/F6/Tab, virtual lists, resize/DPI, semantic accessibility and existing Favorites/Recent persistence owners.
- Scope: internal; read-only prerequisite research while 02 is being implemented. No 03 activation, product changes, native interaction or tests.
- Date: 2026-10-18

## Findings

### Scope and source identity

Read 03 `prd.md`, `design.md`, `implement.md`, complete `ccr-desktop/DESIGN.md`, 01 `research/native-a11y.md`, current `ccr-desktop/src/{theme,probe,lib}.rs`, and current domain sources below. This file locates implementation seams; it does not replace the approved design or claim 03 completion. 01/02 prerequisite acceptance remains the coordinator's responsibility.

Pinned source base for all `CACHE/` references:

`C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`

Kit/component/base are **0.7.3** and gpui-pre is **0.3.9**. These are installed exact-version sources already selected by 01. No version search or dependency change was performed. Current desktop files are concurrently owned by 02 implementers; reconcile the final 02 API/files before editing them in 03.

### Minimum real navigation available for 03

The coordinator confirms the currently integrated business capability is **read-only configuration listing** from 02. The shell's actionable business navigation should therefore lead to that real configuration list and its real selection/details/back states. It can provide actual shell controls (theme/language/focus) and display the acknowledged **Local** environment identity. WSL/SSH switching belongs to 09; do not create a selector that silently continues reading Local data after a remote choice.

Approved grouping in `ccr-desktop/DESIGN.md:96` describes the completed product, not authorization to present clickable placeholder routes. Render only available capabilities as actions; unintegrated domains may have explicit noninteractive availability explanations when needed for clarity. Do not create fake overview statistics, authenticated status, empty success pages, or buttons that only change a title. Favorites/Recent below identifies the actual persistence seam for the parent's ui_state scope; it does not authorize synthetic command execution or require filling an otherwise empty navigation group before that seam exists.

`ccr-desktop/DESIGN.md:158-162` still says only the design exists and no native components/screenshots/input have been verified. That factual Known Gaps text is stale after 01's actual native input probe; the 03 owner should update it to the actual delivered state while retaining unresolved a11y/DPI/cross-platform/performance gaps. This researcher has not edited DESIGN.md.

### Theme: native appearance and one semantic mapping

| File:line under CACHE | Actual API/behavior |
| --- | --- |
| `gpui-component-0.7.3/src/theme/mod.rs:38-43` | Component initialization installs **Light**, then scrollbar appearance. It does not establish a permanent System preference. |
| `.../theme/mod.rs:807-829` | `ThemeMode` has Light/Dark only. There is no System variant in this enum. |
| `.../theme/mod.rs:354-364` | `Theme::sync_system_appearance(Option<&mut Window>, &mut App)` reads `window.appearance()` when provided, otherwise app appearance; source recommends the window path for Linux. |
| `gpui-pre-0.3.9/src/app/context.rs:457-471` | `cx.observe_window_appearance(window, callback) -> Subscription` observes native changes on the entity. Store the subscription in its owner. |
| `gpui-component-0.7.3/src/theme/mod.rs:389-399` | `Theme::change(mode, window, cx)` reloads registered mode palette and refreshes all windows. The window argument is compatibility-only here. |
| `.../theme/mod.rs:278-288` | Changing `mode` within `Theme::update` loads registered palette **after** the edit closure; color edits in that same closure can be overwritten. Apply CCR colors in a separate update after choosing mode. |
| `.../theme/mod.rs:288-337` | `Theme::update(cx, closure)` reconciles colors/tokens, resolves fonts when needed, synchronizes Base, refreshes windows. |
| `.../theme/mod.rs:459-477` | `Theme::global_mut` edits alone do **not** update Base; `Theme::sync_base(cx)` and window refresh are required if bypassing `update`. |
| `.../theme/mod.rs:505`, `:612-647` | `semantic_tokens()` and `apply_semantic_tokens(&SemanticThemeTokens)` expose/update colors, typography, radii and supported shadow fields. Scale-only spacing/elevation details are not all representable in legacy component fields. |
| `.../theme/mod.rs:166-183`, `:770-773` | Default UI family `.SystemUIFont`; mono defaults to Menlo/Consolas/DejaVu Sans Mono with installed fallback. `font_size` and `mono_font_size` are real Pixels fields. |
| `.../theme/system_font.rs:32-49` | Resolves `.SystemUIFont` once to installed fallback family; explicit application families are left alone. No font download is required. |

The app must retain its own minimal System/Light/Dark **preference**, then resolve to Kit Light/Dark. Under System, react to native appearance changes; under an explicit choice, do not let native appearance overwrite it. Apply the existing approved CCR token mapping after every resolved mode change with `Theme::update`; do not maintain a competing Base palette. `Theme::change` alone resets custom colors.

Current `ccr-desktop/src/theme.rs:15-31` changes only five palette fields through `global_mut`; it also inherits whichever mode initialization left behind. This is foundation-probe code, not sufficient for the approved shell. 03 owns expanding that existing single mapping rather than copying hex literals into each view. Set/verify sidebar, popover, primary/secondary, muted, control border, focus, destructive/status colors and typography according to DESIGN.md; direct Base-only mutations will be overwritten by later synchronization. The semantic API's partial mapping means not every DESIGN role automatically exists in Kit's legacy field set; keep remaining app-specific roles in the same CCR token owner.

Motion caveat: `.../theme/motion.rs:7-42` exposes MotionTokens with nonzero defaults; sidebar collapse has its own fixed 200ms duration (`.../sidebar/mod.rs:30`) and mode-dependent animation (`:78-107`). No `reduced_motion`/`prefers_reduced` field was found in the inspected platform/theme APIs. The approved default of no nonessential motion must therefore be explicit in app/component choices. A noncollapsible sidebar (`SidebarCollapsible::None`) avoids that collapse animation. Do not assume setting Theme motion to zero controls every hard-coded animation or scrollbar behavior.

### Root, keyboard actions and focus regions

| File:line under CACHE | Actual behavior/API |
| --- | --- |
| `gpui-kit-0.7.3/src/lib.rs:176-187` | `open_window` wraps returned application content in **Base Root**. Do not wrap a second Root around it. |
| `gpui-base-0.7.3/src/root.rs:11-23` | Root registers Tab/Shift+Tab under `Root` key context, plus platform copy action. |
| `.../root.rs:194-261` | Root moves next/previous focus and respects active focus traps. |
| `.../root.rs:282-297` | Render root attaches key context/actions above application content. |
| `gpui-pre-0.3.9/src/window.rs:583-607` | FocusHandle `tab_index`/`tab_stop` configure traversal. |
| `.../window.rs:612-635` | `focus`, `is_focused`, `contains_focused`, `within_focused`, `contains` support region routing and restoration. |
| `.../elements/div.rs:786-826` | `track_focus`, `tab_stop`, `tab_index`, `tab_group`; tab groups localize index ordering. A non-tab-stop container can be programmatically focused then `window.focus_next(cx)` enters its children. |
| `.../window.rs:2440`, `:2451` | `window.focus_next(cx)` / `focus_prev(cx)`. |
| `.../app/context.rs:420-434` | Window activation observer lets the app restore/check a still-valid focus target after reactivation. |
| `gpui-base-0.7.3/src/button.rs:93-128`, `:209-257` | Base button accepts owned FocusHandle, participates in tab traversal unless disabled, supplies button role and named semantics, blocks disabled pointer activation. |
| `.../button.rs:346-367` | Upstream test proves one native keyboard click per Enter/Space activation; do not add another key listener that double-activates. |
| `gpui-component-0.7.3/src/button/button.rs:482-490` | Styled Button exposes `tab_index` and `tab_stop`. |

No literal F6/Shift+F6 binding was found in Base/Component. This shell behavior belongs to application actions via `gpui_kit::actions!`, `cx.bind_keys([KeyBinding::new("f6", action, Some("CcrShell")), ...])`, `.key_context("CcrShell")`, `.on_action(cx.listener(...))`. Retain owned sidebar/main/status focus handles; use actual `contains_focused` state to select the next region and an existing valid child/entry target. The shell owns this small coordination; it should not reimplement Input editing keys, global Tab traversal or modal focus trapping.

Focus should follow actual selected list/detail state and return to the trigger when leaving drill-in/dialogs. Entity/FocusHandle ownership and scoped observers prevent a delayed callback restoring a removed view. A plain styled div with `on_click` is not proof of keyboard reachability.

### Sidebar and virtual lists: convenient APIs have limits

| File:line under CACHE | Actual API / risk |
| --- | --- |
| `gpui-component-0.7.3/src/sidebar/mod.rs:238-296` | `Sidebar::new(id)`, `.collapsible(...)`, `.collapsed(...)`, `.header(...)`, `.footer(...)`, `.child/.children`; width is styled through `Styled`, not a dedicated public width setter in this block. |
| `.../sidebar/menu.rs:66-87` | SidebarMenu derives child IDs by **enumerated position**. Inserting/reordering capability-filtered items shifts identity. |
| `.../sidebar/menu.rs:136-170` | Item exposes `.accessibility_label`, `.active`, `.on_click`. |
| `.../sidebar/menu.rs:287-296`, `:366-390` | Item renders TreeItem/name/selected and pointer handlers; disabled suppresses click. This inspected path has **no track_focus or direction-key handler**, so visual SidebarMenu alone is not a complete keyboard navigation implementation. |
| `gpui-pre-0.3.9/src/elements/uniform_list.rs:17-28` | `uniform_list(id, count, Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>)`; renders visible range when constrained by fixed/max height and overflow-hidden parent. Fits approved uniform 36px rows. |
| `.../uniform_list.rs:150`, `:686` | UniformListScrollHandle `scroll_to_item`; list `.track_scroll(&handle)`. |
| `gpui-base-0.7.3/src/virtual_list.rs:139-151` | `v_virtual_list(entity, id, Rc<Vec<Size<Pixels>>>, render_visible_range)` supports measured/differing row sizes. |
| `gpui-component-0.7.3/src/virtual_list.rs:1-2` | Re-exports Base VirtualList/scroll handle and public v/h list constructors. |
| `.../list/list.rs:27-36` | Styled List binds Escape/Enter/secondary-Enter/up/down under `List` context. |
| `.../list/delegate.rs:10-47`, `:135-162` | `ListDelegate`: Item, items_count, render_item, set_selected_index, confirm/cancel; render_initial/empty/loading hooks also exist. |
| `.../list/list.rs:184-195`, `:239-264` | ListState selection and scrolling APIs; index-based selection. |
| `.../list/list.rs:486-495` | Default rendered row identity is `list-item-{IndexPath}` with ListItem/position/count/selected semantics. Identity is index-based, not a domain record key. |
| `.../list/list.rs:552-576`, `:666-674` | List uses v_virtual_list internally and has focusable List container. |

Use the approved real capability/route identifiers as stable navigation IDs; do not instantiate menus for unimplemented domains merely to fill six groups. SidebarMenu's defaults do not satisfy both stable IDs under filtering and keyboard navigation by themselves. A small application-owned list of actual available actions rendered with existing Kit focusable controls is a smaller boundary than patching the entire Sidebar widget or building an imitation tree control. This is an implementation seam, not a new navigation design.

For data lists, selection must be retained by the domain stable key (environment/platform/profile identity or database record UUID), then mapped to the current visible index after refresh/filtering. Giving only a child row a domain ID does not eliminate the default styled List's index ancestor identity. If that matters for the accepted identity contract, use `uniform_list` with application-keyed rows and existing focus/actions; or explicitly demonstrate the delegate/reconciliation preserves focus/selection. Do not allocate 10,000 row entities or all-row elements in render; retain records and create only the passed visible range. Measure actual renderer invocation bounds in tests instead of inferring virtualization from using a type named List.

Override List default empty icon/skeleton (`list/delegate.rs:79-126`) with the approved explanatory real state. An empty/missing config source and an error are distinct. Return/back actions must actually change the loaded route/detail state.

### Resize, logical pixels and DPI

- Current desktop `src/lib.rs:16-27` sets centered 1280x800 logical window and `window_min_size` from theme constants (960x640). Keep this native minimum.
- `gpui-pre-0.3.9/src/window.rs:2928` returns **drawable content** viewport size; `:3091` returns scale factor. Use `viewport_size().width` for layout breakpoints, not whole-window screenshot width or device pixels.
- `cx.observe_window_bounds(window, callback)` (`src/app/context.rs:402-416`) updates an entity when resized; `observe_window_display` at `:477` reports display movement; `observe_window_activation` at `:420` handles regain/lose focus. Store subscriptions; no polling timer required.
- `Window::bounds_changed` (`src/window.rs:2791-2802`) updates both scale factor and viewport from the backend, refreshes and notifies observers. Read cached geometry in render; no synchronous system query is needed.
- Windows min size multiplies logical minimum by current scale and adds frame borders (`gpui-pre-windows-0.3.9/src/events.rs:208-220`). WM_DPICHANGED dispatch at `:106` and handler at `:862-916` update scale and handle WM_SIZE, including maximized-window special path. Effective-DPI native testing remains necessary.
- At layout change, preserve selected record ID and choose the already-approved split versus content drill-in. Check real **available main width**, including sidebar, padding, gap and both column minima; do not assume a 3:2 ratio always leaves detail >=320px. For example 1040 total minus 216 sidebar minus 48 padding minus 16 gap leaves 760, whose 2/5 detail is 304. DESIGN.md explicitly allows drill-in through that range; no threshold relaxation is required.
- GPUI headless `TestAppContext::simulate_window_resize(handle, size)` (`src/app/test_context.rs:461`) and `simulate_window_scale_factor_change(handle, factor)` (`:516`) drive actual resize callback paths. TestWindow source `src/platform/test/window.rs:241-249` preserves logical dimensions during scale change. `simulate_appearance_change` at `:260` tests system-mode observation. Headless tests do not prove Windows effective DPI, CJK fallback or screen-reader behavior.

### Minimum native accessibility interfaces for 03

Keep the earlier full investigation in `01/research/native-a11y.md`; this section only identifies shell entry points.

- Public `.role`, `.aria_label`, `.aria_description`, `.aria_keyshortcuts`, `.aria_selected`, `.aria_expanded`, `.aria_toggled`, `.aria_value`, positional/table properties and `.on_a11y_action` are in `gpui-pre-0.3.9/src/elements/div.rs:1306-1512`. Require stable `.id` and role; use real action handlers, not label-only semantic substitutes.
- Input `.aria_label`/`.accessibility_id` are already verified in the prior report. For shell title/section/status text, use `Text`/`text!` with stable unique IDs or actual role/name nodes rather than plain strings assumed to be semantic. List items also need real selection/position state.
- Current div builder surface does **not** expose `aria_disabled`, `aria_readonly`, `aria_busy`, `aria_invalid` or live-region setters in the inspected block. Do not invent those methods. Custom Element hook `a11y_role` / `write_a11y_info` is public (`src/element.rs:113-121`); it receives the actual `accesskit::Node`. Exact AccessKit 0.24.1 has `set_busy`, `set_read_only`, `set_disabled` (`src/lib.rs:1805-1816`) and `set_invalid`, `set_live` (`:2137-2139`). If a semantic flag truly cannot be produced through the chosen component, a narrow forwarding semantic element must preserve layout, identity, child/input handlers and write the existing node, rather than introduce duplicate edit controls. This needs actual code review; no framework patch is prescribed here.
- The Input disabled/read-only and Button/Sidebar disabled paths require semantic node assertions; suppression of events or muted styling is insufficient evidence of native state. The prior report explains the concrete Input risk. Do not label `ElementSnapshot.disabled() == None` as a passed disabled-state check: its docs explicitly mean no native disabled flag (`gpui-base-0.7.3/src/test_support.rs:106-109`).
- `gpui_kit::test::TestWindowExt::{find,press,input,render_frame}` and `ElementSnapshot::{role,label,value,focused,bounds,visible}` provide interaction/semantic evidence; existing `ccr-desktop/tests/native_ui.rs:7-35` shows the real Kit test harness. Public `window.debug_a11y_tree_json()`/enabled/active diagnostics and actual UIA evidence remain separate.
- F6/key reachability and native role/name/state are separate from IME composition. OS keyboard/IME/windows remain main-session-only. No headless input test proves composition timing or screen-reader output.

### Favorites/Recent: true owner, contracts and explicit fixture seams

Parent feature matrix assigns `ui_state` six commands to 03 (`10-17-gpui-desktop/research/feature-matrix.md:54`, `:69`). These are **command favorites and completed-command history**, not general route bookmarks or navigation history. Do not repurpose persisted fields or insert fake executions when navigating.

| Repository file:line | Existing API / meaning |
| --- | --- |
| `ccr-ui/src-tauri/src/commands/ui_state.rs:74`, `:88`, `:113` | Tauri get/add/remove favorites; worker runs via helper at `:61`. |
| `.../commands/ui_state.rs:127`, `:141`, `:159` | Get/add/clear recent command items. TS DTOs and export paths remain adapter-owned (`:10-57`). |
| `crates/ccr-db/src/models/ui_state.rs:8`, `:27`, `:112` | FavoriteCommand, CommandHistory, AddFavoriteRequest. Domain timestamps are DateTime; Tauri maps to RFC3339 strings. Record UUIDs are stable IDs. |
| `crates/ccr-db/src/managers/ui_state.rs:18` | UiStateManager::new checks a global initialized database; manager has no injected pool field today. |
| `.../managers/ui_state.rs:30-37`, `:84-93` | Current reads swallow database errors, log and return Vec::new. Native shell must not claim empty state means a successful query. |
| `.../managers/ui_state.rs:41-67` | Favorite duplicate lookup by command+args, preserve existing record if found; UUID/timestamp generated by manager. |
| `.../managers/ui_state.rs:96-123` | add_history builds `ccr <command> <joined args>`, new UUID, supplied success/duration, persists via repository. No job-idempotency key. |
| `.../managers/ui_state.rs:145-153` | Process-global OnceCell UI-state manager; not a fixture isolation API. |
| `crates/ccr-db/src/database/repositories/ui_state_repo.rs:12-141` | Existing sole SQL implementation: favorite insert/find/list/delete; history insert/list(limit)/clear; counts. Reads return Result with query errors intact. |
| `.../ui_state_repo.rs:57-67`, `:107-121` | Favorites order by created_at descending, history by executed_at descending; history limit is applied in SQL. |
| `crates/ccr-db/src/database/mod.rs:190-218` | Existing `DbAccess::{Global, Pool(DbPool)}` with `with_connection` and `transaction`; exact fixture/embedding seam already exists. |
| `.../database/mod.rs:30`, `:48` | Existing explicit-path `create_pool(path, config)` and isolated `create_memory_pool()`. Pool factory remains ccr-core SQLite seam owner. |
| `.../database/mod.rs:68-74` | Production UI DB resolves actual home `~/.ccr-ui/ccr-ui.db` and creates directory; CCR_ROOT/CCR_DATA_DIR do **not** override it. Never call for fixture setup. |
| `.../database/mod.rs:93-115` | Production initialize_app_pool creates one app pool and registers same instance globally, runs domain migrations. Do not create a second pool to same DB in one app. |
| `crates/ccr-checkin/src/managers/checkin/account_manager.rs:36-51` | Existing manager precedent: default constructor delegates to injected `with_db(..., DbAccess)` constructor. |
| `crates/ccr-db/src/database/migrations.rs:1790-1840` | `run_all_migrations(&Connection, &explicit_home_path)` owns schema/legacy imports. Use an empty temporary fixture home for migration setup; passing real home can import real legacy data. |

The narrow reusable improvement is a **ccr-db-owned** injected UiStateManager path using DbAccess, following the AccountManager precedent, and fallible read variants for native state. Keep existing Tauri list wrappers/return types (including their compatibility behavior) unless explicitly reviewing a behavior change. Avoid copying add_history UUID/string generation or favorite duplicate behavior into desktop, and do not copy SQL into ccr-app. The service/runtime can directly consume this existing domain owner; ccr-app need not gain a ccr-db dependency merely to forward identical CRUD methods.

`DbAccess::Pool` plus existing manager/repository APIs can operate on a private temp file pool; setup through domain migrations with explicit temp home. Repository tests already use `schema::CREATE_TABLES_SQL` and in-memory Connection (`ui_state_repo.rs:201-205`), but schema-only setup is test-level evidence, not a replacement for production migrations. Use two independent temp file pools/reopen tests for persistence; no GLOBAL_POOL mutation or HOME mutation. `initialize_for_test()` at `database/mod.rs:234` registers process-global state and is not the preferred parallel fixture path.

Existing manager methods are not sufficient as-is for both explicit fixtures and visible read errors. New fallible methods/constructor must be implemented by the appropriate domain owner if this surface is included in 03. Extending manager injection is much smaller than duplicating the UI database or inventing a second store. No schema change is needed for equivalent six-command behavior.

No environment_id column exists on these UI-state models/tables (`database/schema.rs:19-43`). They are desktop-local/global persisted command records. Use request environment/generation to guard delivered UI data, but do not claim old records belong to a chosen SSH/WSL environment or rewrite them during environment switch. Executing a stored command remains through the approved command policy/jobs surface when 08 is ready; an arbitrary saved string is not authority to execute it. Until then show real data and capability restrictions, not successful fake jobs.

History submission lifecycle is already explicit in `ccr-ui/src/features/commands/commandJobStore.ts:79-94`: one pending/saved/failed claim per job; ambiguous failure is not replayed because backend has no idempotency key. Preserve that boundary for 08 integration; 03 navigation must not write history. The old in-memory `models::UiState` 100-item truncation (`models/ui_state.rs:94-101`) is not active database retention: current manager/repository does not call that model path. Do not silently add a new pruning policy.

### Minimal handoff boundary (not a new plan)

- 03 primary owner: existing desktop theme mapping, real shell state/views/locales, public app wiring, focused native_shell tests. Coordinate ownership of `src/lib.rs`, new `src/app.rs` and runtime directory with 02 owners before changing them.
- Consume 02's finalized real config list result / RequestKey / subscription lifecycle. At research time 02 was actively creating these files; no stable API signature is asserted here. Do not copy its runtime or add a second Tokio owner.
- Optional required ui_state domain owner: `crates/ccr-db/src/managers/ui_state.rs` plus focused injected-pool tests; database/models/repositories existing APIs should stay reusable. No SQL/schema duplication. Add desktop dependency/lock entry only through assigned manifest owner and `--locked` verification after lock refresh.
- Tauri `commands/ui_state.rs` changes are unnecessary for direct native consumption if default legacy manager methods remain compatible. If extracting/reusing a new fallible shared path there, preserve DTO exports/errors and run binding/command inventory gates.
- No broad parent/child artifact rewrite, no capability placeholders, no favorites-as-bookmarks reinterpretation, no implementation status change from this research.

### Verification interfaces for the implementation owner (not run)

1. Theme tests: switch explicit light/dark and simulated native appearance under System; assert CCR palette survives every change, Base and Component colors agree, explicit preference ignores native changes. Check both computed contrast and actual focus/disabled rendering. Headless appearance callback: TestWindow::simulate_appearance_change at CACHE `gpui-pre-0.3.9/src/platform/test/window.rs:260`.
2. Layout tests: four approved viewport sizes, simulated scale factors 1/1.5/2, real available column bounds and drill-in/back state preservation; no horizontal whole-window overflow. Observe/render via TestWindowExt; native Windows/macOS/Linux matrix remains separate.
3. Keyboard tests: F6/Shift+F6 cycles actual region targets; Tab/Shift+Tab traverse real available controls; list up/down/Enter/Escape preserve Input composition/editing scope; focus returns after back/recovery. Use Kit's real rendered controls, not tests that call state setters alone.
4. Virtualization tests: synthetic 10k records with stable domain keys; visible-render counter is bounded; reordering/filtering retains selected key; selection scrolls into view. No all-row entity creation.
5. ui_state tests: private file pool with empty temp-home migrations; reopen and read/write favorites/history through domain manager; assert duplicate favorite semantics, args/order/timestamps, history limit, delete/clear, error-vs-empty and concurrent fixture isolation. Do not execute stored commands to test persistence.
6. Gates: `just desktop-test-ui`, `just desktop-test`, `just desktop-lint` plus format-check and affected ccr-db tests/clippy if extending manager. Use `cargo test -p ccr-db --locked -- --skip export_bindings` with default parallelism; existing spec examples with serial flags do not override this user's explicit requirement. Tauri bindings and inventory only if its surface changes.
7. Evidence: real native screenshots and actual keyboard/IME/accessibility at specified window/DPI/theme/language combinations, recording failed/unrun items. Existing 01 null a11y captures do not prove 03 passed. Screenshot or headless tree output alone does not establish screen-reader behavior.

### Related specs / external references

- `.trellis/spec/ccr-db/backend/{index,backend-guidelines,test-fixtures}.md`: database owner/pool boundaries, DbAccess injection, migrations, tests.
- `.trellis/spec/guides/{code-reuse-thinking-guide,cross-layer-thinking-guide}.md`: reuse existing owners and preserve boundary semantics.
- `.trellis/spec/ccr-ui/frontend/command-job-lifecycle.md` and `01/research/native-a11y.md`: already read in this research session; history terminal semantics and native semantic risks remain relevant.
- Tauri `theme-token-contracts.md` is React/CSS-specific. Its old palette is not the GPUI design authority; do not transplant it over approved `ccr-desktop/DESIGN.md`.
- No external/new API claim or latest-version comparison was needed. Primary references are exact local crate source files listed above. The earlier a11y report retains its official exact-version AccessKit link; this report does not repeat that investigation.

## Caveats / Not Found

- No implementation, tests, app launch, native window/IME input, OS setting change or tool installation occurred. Only this research file was written. 03 was not activated and no implement/check manifests were read.
- Active task is 02 by current dispatch. Product source is changing concurrently; final file ownership, line numbers in desktop app source, and 02 signatures must be reconciled at handoff.
- No built-in Kit System theme preference, F6 region routing, or complete keyboard traversal on SidebarMenuItem was found in inspected exact-version sources. These are small explicit shell responsibilities, not reasons to change frameworks or design.
- Existing ui_state manager is global-only and loses query errors. `DbAccess` seam is real, but a UiStateManager::with_db/fallible reads are **not yet present** and must not be cited as implemented.
- `get_db_path()` creates/uses the real home directory; CCR_ROOT does not isolate UI state. Fixtures must avoid it.
- The earlier missing RTK.md and unresolved native helper null result remain unchanged; no new assumption about either was made.
