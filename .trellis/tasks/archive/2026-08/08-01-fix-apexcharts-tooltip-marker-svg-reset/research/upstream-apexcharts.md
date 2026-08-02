# Research: ApexCharts stylesheet and tooltip marker lifecycle

- Query: Does ApexCharts 5.16.0 remove, duplicate, or restore its base stylesheet during update, destroy, and remount flows, and is there an upstream defect or supported static CSS path relevant to CCR's enlarged donut tooltip marker failure?
- Scope: mixed (CCR's resolved dependency and integration code, ApexCharts source, package metadata, releases, commits, issues, and pull requests)
- Date: 2026-08-02

## Findings

### Executive answer

CCR resolves ApexCharts 5.16.0. In that version, the base stylesheet is injected only by a chart instance's initial `render()` call. The injection is shared per document or ShadowRoot through the fixed ID `apexcharts-css`. Chart updates rebuild chart-owned DOM through `clear -> create -> mount` without calling `render()`, and `destroy()` does not remove the shared stylesheet.

This yields two distinct conclusions:

1. ApexCharts' own destroy/update lifecycle does not explain loss of `#apexcharts-css`; a normally injected node survives chart destruction and can be reused by a later fresh render.
2. The lifecycle is not self-healing after external corruption. If another actor removes, empties, disables, or shadows the node and only an update follows, 5.16.0 does not re-run the injection guard. A fresh chart instance followed by `render()` is required to recreate a missing node.

No targeted official issue or pull-request search found a report that ties 5.16.0 stylesheet loss across update/destroy/remount to oversized donut tooltip marker SVGs. The upstream record contains related Shadow DOM placement and duplicate-injection defects, but both were fixed before 5.16.0 and neither removes a valid document-level stylesheet.

The installed package exposes `apexcharts/dist/apexcharts.css` through its public wildcard exports. Its source and dist copies are byte-identical. A static import can therefore provide a second, bundler-managed delivery path for the complete base CSS contract while leaving the normal runtime injection enabled. This is stronger than copying only the marker rule because the same stylesheet also controls tooltip visibility, positioning, layout, and interaction state.

### Files found

| File | Description |
| --- | --- |
| `ccr-ui/bun.lock:469` | Resolves `apexcharts@5.16.0`; this is the version analyzed. |
| `ccr-ui/node_modules/apexcharts/package.json:3` | Declares package version 5.16.0. |
| `ccr-ui/node_modules/apexcharts/package.json:15` | Defines public subpath exports, including modular chart entries and `./dist/*` / `./src/*`. |
| `ccr-ui/node_modules/apexcharts/src/apexcharts.js:19` | Imports the complete base CSS as text for runtime injection. |
| `ccr-ui/node_modules/apexcharts/src/apexcharts.js:109` | Implements initial render, per-root stylesheet lookup/injection, chart creation, and mount. |
| `ccr-ui/node_modules/apexcharts/src/apexcharts.js:752` | Implements the full update rebuild without re-entering `render()`. |
| `ccr-ui/node_modules/apexcharts/src/modules/helpers/Destroy.js:17` | Clears chart-owned modules, listeners, and DOM; contains no base stylesheet removal. |
| `ccr-ui/node_modules/apexcharts/src/modules/helpers/UpdateHelpers.js:232` | Selects the series fast path only for eligible axis charts; non-axis charts use the full update path. |
| `ccr-ui/node_modules/apexcharts/src/modules/settings/Options.js:364` | Defaults `chart.injectStyleSheet` to `true`. |
| `ccr-ui/node_modules/apexcharts/types/apexcharts.d.ts:611` | Publicly types `chart.injectStyleSheet?: boolean`. |
| `ccr-ui/node_modules/apexcharts/src/modules/tooltip/Marker.js:18` | Generates 12-unit inline SVG marker shapes with `currentColor`. |
| `ccr-ui/node_modules/apexcharts/src/modules/tooltip/Tooltip.js:213` | Removes an existing tooltip element before drawing its replacement. |
| `ccr-ui/node_modules/apexcharts/src/assets/apexcharts.css:344` | Fixes the marker host at 12x12, sizes its child SVG to 100%, and initially hides series groups. |
| `ccr-ui/node_modules/apexcharts/src/features/legend.js:5` | Registers the optional legend feature on the modular ApexCharts core. |
| `ccr-ui/node_modules/apexcharts/src/modules/legend/Helpers.js:16` | Creates a per-chart legend style element and appends it inside the legend foreignObject. |
| `ccr-ui/node_modules/apexcharts/dist/apexcharts.css` | Public dist copy of the complete base stylesheet. |
| `ccr-ui/node_modules/apexcharts/dist/apexcharts-legend.css` | Separate public dist copy of legend styles. |
| `ccr-ui/src/utils/apexChartsCore.ts:17` | CCR's single modular wrapper imports core, donut, and the legend feature; it does not currently import static base CSS. |
| `ccr-ui/src/styles/base.css:67` | CCR's global responsive reset applies `display:block`, `max-width:100%`, and `height:auto` to every SVG. |
| `.trellis/tasks/08-01-fix-apexcharts-tooltip-marker-svg-reset/research/root-cause.md` | Records the controlled CCR/WebView2 failure mechanism and the still-unknown natural trigger. |

### Code patterns

#### 1. Base CSS injection is a per-root, initial-render operation

`src/apexcharts.js` imports `src/assets/apexcharts.css` into the JavaScript bundle as `apexCSS` (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:19`). During `render()` it:

1. Resolves the chart element's root and distinguishes a `ShadowRoot` from a normal document (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:149`).
2. Looks up `apexcharts-css` in that same root (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:154`).
3. Creates one XHTML `style` element only when lookup returns no element, assigns the fixed ID, and fills it with `apexCSS` (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:158`).
4. Prepends it to a ShadowRoot or appends it to the owning document's head (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:170`).
5. Only then calls `create()` and `mount()` (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:180`).

Consequences:

- Normal initial render cannot create tooltip DOM before its own base CSS injection attempt.
- Multiple charts in the same document or ShadowRoot intentionally share one `#apexcharts-css` node.
- A newly mounted chart instance repairs a missing node because its `render()` repeats the lookup.
- An existing node is trusted solely by ID. ApexCharts does not verify that it is a `style` element, contains current ApexCharts rules, is enabled, or carries the expected nonce.
- In a normal document, `chart.injectStyleSheet === false` suppresses append (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:173`). In a ShadowRoot, the current branch prepends the node without checking that option (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:170`). The public default remains `true` (`ccr-ui/node_modules/apexcharts/src/modules/settings/Options.js:364`).

#### 2. Update and destroy do not own the shared stylesheet

The full update method calls `Destroy.clear({ isUpdating: true })`, then `create()` and `mount()` directly (`ccr-ui/node_modules/apexcharts/src/apexcharts.js:752`). It does not call `render()`, so it does not look up or restore `#apexcharts-css`.

`updateSeries()` has an axis-chart-only fast path. The eligibility check explicitly rejects non-axis charts such as pie and donut (`ccr-ui/node_modules/apexcharts/src/modules/helpers/UpdateHelpers.js:246`). Donut updates therefore fall through to `ctx.update()` (`ccr-ui/node_modules/apexcharts/src/modules/helpers/UpdateHelpers.js:232`) and rebuild chart DOM through the same non-injecting full update method.

Full `destroy()` delegates to `Destroy.clear({ isUpdating: false })`. The helper tears down module references, observers, listeners, chart child nodes, and the chart SVG (`ccr-ui/node_modules/apexcharts/src/modules/helpers/Destroy.js:17`, `ccr-ui/node_modules/apexcharts/src/modules/helpers/Destroy.js:98`). It never queries or removes `#apexcharts-css`. This is consistent with the stylesheet being shared across chart instances rather than owned by any one instance.

The expected lifecycle is therefore:

```text
first instance render
  -> inject shared #apexcharts-css if absent
  -> create and mount chart DOM

updateOptions / donut updateSeries
  -> clear chart-owned DOM
  -> create and mount chart DOM
  -> no stylesheet lookup

destroy last chart
  -> remove chart-owned DOM and listeners
  -> leave shared #apexcharts-css in place

fresh instance render
  -> reuse surviving node, or recreate it if absent
```

#### 3. Tooltip marker geometry depends on the complete base stylesheet

The 5.16.0 marker generator emits inline SVG with `viewBox="0 0 12 12"`, no width or height attributes, and shapes colored with `currentColor` (`ccr-ui/node_modules/apexcharts/src/modules/tooltip/Marker.js:18`). The default circle is centered at 6/6 with radius 5 (`ccr-ui/node_modules/apexcharts/src/modules/tooltip/Marker.js:55`).

The base CSS supplies the missing layout contract:

- `.apexcharts-tooltip-marker` is an inline-flex 12x12 host (`ccr-ui/node_modules/apexcharts/src/assets/apexcharts.css:344`).
- Its child SVG is `width:100%; height:100%; display:block` (`ccr-ui/node_modules/apexcharts/src/assets/apexcharts.css:356`).
- `.apexcharts-tooltip-series-group` starts at `display:none` (`ccr-ui/node_modules/apexcharts/src/assets/apexcharts.css:362`).

Without those rules, the SVG has no intrinsic CSS width/height attributes to resist CCR's global responsive SVG reset (`ccr-ui/src/styles/base.css:67`). The task's controlled WebView2 probe confirms that this combination can enlarge the marker to its container width. This is a downstream interaction with missing base CSS, not evidence that ApexCharts itself deletes the stylesheet.

Tooltip redraw is intentionally destructive at the chart-DOM level: `drawTooltip()` removes any existing `.apexcharts-tooltip` before creating a replacement (`ccr-ui/node_modules/apexcharts/src/modules/tooltip/Tooltip.js:213`). That behavior is independent of the shared base stylesheet, which remains outside the chart root in the document head or ShadowRoot.

#### 4. Base CSS and legend CSS have separate delivery lifecycles

The modular legend entry only registers the `Legend` feature (`ccr-ui/node_modules/apexcharts/src/features/legend.js:5`). When a legend is built, `Legend.Helpers` imports `apexcharts-legend.css`, creates an un-IDed `style` element, and appends it inside that chart's legend foreignObject when stylesheet injection is enabled (`ccr-ui/node_modules/apexcharts/src/modules/legend/Helpers.js:16`, `ccr-ui/node_modules/apexcharts/src/modules/legend/Helpers.js:46`). Because it is chart-owned DOM, it is destroyed and recreated with the chart.

This differs from the singleton base stylesheet:

| Stylesheet | Runtime location | Identity | Lifecycle |
| --- | --- | --- | --- |
| `apexcharts.css` | document head or chart ShadowRoot | shared `#apexcharts-css` | injected by initial `render()`; survives chart destroy |
| `apexcharts-legend.css` | legend foreignObject | no fixed ID; one per rendered legend | created and removed with chart DOM |

Static import of the base stylesheet does not replace the legend feature's per-chart injection. Keeping `injectStyleSheet` at its default preserves both upstream runtime paths.

#### 5. The package supports an explicit static base-CSS import

The 5.16.0 package exports `./dist/*` and `./src/*` (`ccr-ui/node_modules/apexcharts/package.json:165`). It ships all four relevant files:

- `apexcharts/dist/apexcharts.css`
- `apexcharts/dist/apexcharts-legend.css`
- `apexcharts/src/assets/apexcharts.css`
- `apexcharts/src/assets/apexcharts-legend.css`

There is no top-level `style` or `css` metadata field and no explicit `./css` export. Consumers that want a bundler-managed stylesheet must import a public subpath such as `apexcharts/dist/apexcharts.css` explicitly.

Local SHA-256 comparison confirms that each source/dist pair in the resolved package is identical:

| Pair | SHA-256 |
| --- | --- |
| Base CSS | `54B61EC43EBE92812ACCFDBC2E5E32D3F2CE294AD905BC163A59AF91D42D2A6C` |
| Legend CSS | `7095018503C45BE3BBF8F0195078B09B5F8E764983D1B58C52AFE49CD4065A69` |

For CCR, the narrow integration point is the existing modular wrapper at `ccr-ui/src/utils/apexChartsCore.ts:17`. A static base-CSS import there would remain lazy with the chart wrapper and provide an independent Vite-managed copy of the complete tooltip/layout contract. This research does not modify that file.

### Version comparison

| Version | Tag commit | Relevant behavior |
| --- | --- | --- |
| 5.6.0 | [`f20c45afd00473fa02d14c8b70be91f8c86cfce8`](https://github.com/apexcharts/apexcharts.js/commit/f20c45afd00473fa02d14c8b70be91f8c86cfce8) | Uses the same per-root `#apexcharts-css` injection strategy. Tooltip markers are 16px Unicode pseudo-elements, not child SVGs. The package predates public `apexcharts/core` and `apexcharts/donut` exports. |
| 5.13.0 | [release](https://github.com/apexcharts/apexcharts.js/releases/tag/v5.13.0) | First release containing the inline SVG marker implementation and its base CSS rules. Marker markup landed in [`4b2ee4c`](https://github.com/apexcharts/apexcharts.js/commit/4b2ee4ce2cb364ebf48acfb72c5296f1cea3dce8); the paired CSS landed in [`6f3a321`](https://github.com/apexcharts/apexcharts.js/commit/6f3a32119125b7a2080f752cf1ef275fee61f906). |
| 5.16.0 | [`548d828a67c4d4557a744a2fa0a7bb6a7351367d`](https://github.com/apexcharts/apexcharts.js/commit/548d828a67c4d4557a744a2fa0a7bb6a7351367d) | CCR's resolved version. Retains singleton base CSS injection, SVG markers, modular chart entries, and non-axis full rebuilds. |
| 6.7.0 | [`e3912b3be9029d7604a90d8be92e93d16553c326`](https://github.com/apexcharts/apexcharts.js/commit/e3912b3be9029d7604a90d8be92e93d16553c326) | Latest npm/GitHub release observed on 2026-08-02. Retains the same stylesheet injection guard and SVG marker sizing rules. Version 6 adds idempotent repeated `render()` behavior through [`f03fabef`](https://github.com/apexcharts/apexcharts.js/commit/f03fabefa78e911cb45324295a5ab200cf888efe), but updates still do not use `render()` to repair stylesheet loss. |

Per-type modular entries, including `apexcharts/donut`, landed in [`b90a06d`](https://github.com/apexcharts/apexcharts.js/commit/b90a06d951c37a2a47db619905a730cc54e1c2d5). This explains why CCR's current modular wrapper is compatible with 5.16.0 but not with 5.6.0.

### Upstream defect evidence

- [Issue #238, "ShadowDOM support"](https://github.com/apexcharts/apexcharts.js/issues/238) identified that document-head injection did not style charts inside Shadow DOM. [PR #2767](https://github.com/apexcharts/apexcharts.js/pull/2767), implemented by [`698b162`](https://github.com/apexcharts/apexcharts.js/commit/698b162878b6e9a82a6cae1a50a6503a492cd410), added context-aware injection into the chart's ShadowRoot.
- [PR #4333](https://github.com/apexcharts/apexcharts.js/pull/4333) explicitly reports duplicate `#apexcharts-css` nodes when multiple components render in one ShadowRoot. [`e93c14b`](https://github.com/apexcharts/apexcharts.js/commit/e93c14bb39263254b62e97b9e9b36cae5d92bada) changed the existing-node lookup to `ShadowRoot.getElementById()`. This fix predates 5.16.0 and is present in the installed source.
- [Issue #691](https://github.com/apexcharts/apexcharts.js/issues/691) and [issue #1234](https://github.com/apexcharts/apexcharts.js/issues/1234) concern duplicate generated SVG IDs and cross-chart conflicts for multiple charts/donuts. They do not report removal or corruption of the shared stylesheet node.
- No official report found in the targeted issue/PR search describes the exact chain `5.16.0 update/destroy/remount -> base style node disappears -> donut tooltip marker expands`. The absence of such a report is not proof that the field trigger cannot occur; it means upstream issue history does not currently establish that trigger.

### Unresolved trigger hypotheses

The following are derived from the 5.16.0 guard logic and remain unconfirmed in CCR's natural failure path:

1. External removal followed only by `updateOptions()` or donut `updateSeries()` leaves the base CSS absent because neither path re-enters `render()`.
2. Any unrelated element with ID `apexcharts-css` suppresses injection because lookup is ID-only and does not require an HTML style element.
3. An existing style node whose text was emptied, whose sheet was disabled, or whose contents are stale is accepted without validation or refresh.
4. Development HMR or another style-management actor could mutate the node, but the task's normal Web, preview, and Tauri probes did not observe such a mutation.

These hypotheses explain how the upstream recovery gap could be reached; they do not identify which actor, if any, caused the user's original field occurrence.

### External references

- ApexCharts 5.16.0 render and injection source: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/src/apexcharts.js#L109-L177>
- ApexCharts 5.16.0 update source: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/src/apexcharts.js#L752-L779>
- ApexCharts 5.16.0 destroy helper: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/src/modules/helpers/Destroy.js>
- ApexCharts 5.16.0 update helper: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/src/modules/helpers/UpdateHelpers.js>
- ApexCharts 5.16.0 tooltip marker source: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/src/modules/tooltip/Marker.js#L18-L58>
- ApexCharts 5.16.0 tooltip CSS: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/src/assets/apexcharts.css#L344-L377>
- ApexCharts 5.16.0 package exports: <https://github.com/apexcharts/apexcharts.js/blob/548d828a67c4d4557a744a2fa0a7bb6a7351367d/package.json>
- ApexCharts v5.13.0 release: <https://github.com/apexcharts/apexcharts.js/releases/tag/v5.13.0>
- ApexCharts v6.7.0 release: <https://github.com/apexcharts/apexcharts.js/releases/tag/v6.7.0>

### Related specs

- `.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md` defines CCR's reference-stability and KeepAlive contracts for usage charts. Its donut conclusion remains valid: non-axis `updateSeries()` uses the full DOM rebuild path, so stable series/options references still avoid unnecessary rebuilds.
- The same spec's broader statement that every ApexCharts `updateSeries()` fully rebuilds the canvas is now version-sensitive. In 5.16.0, eligible axis charts can use `fastUpdate()` (`ccr-ui/node_modules/apexcharts/src/modules/helpers/UpdateHelpers.js:232`); pie and donut cannot. This is spec-drift evidence for a later `trellis-update-spec` pass, not authorization for this researcher to edit the spec.
- `.trellis/tasks/08-01-fix-apexcharts-tooltip-marker-svg-reset/research/root-cause.md` supplies the CCR-specific WebView2 measurements. This note supplies only the upstream lifecycle, version, and issue-history boundary needed to evaluate that local evidence.

## Caveats / Not Found

- The natural actor that removed or invalidated `#apexcharts-css` in the user's field session remains unidentified. Only the failure mechanism and upstream recovery limitation are established.
- No browser/Tauri run was repeated for this upstream-only pass; current runtime evidence remains in `root-cause.md` and `apexcharts-tooltip-marker-probe.mjs`.
- No product code, dependency version, task metadata, or Trellis spec was changed.
- GitHub issue search is not exhaustive evidence of nonexistence. The precise 5.16.0 source and official issue/PR history reviewed here contain no matching reported defect.
