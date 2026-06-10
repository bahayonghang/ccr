# brainstorm: ccr-ui Catppuccin Mocha visual polish

## Goal

Diagnose why the current `ccr-ui` Catppuccin Mocha flavor feels gray, low-energy, and less polished than the reference terminal screenshot, then converge on a small, verifiable visual correction plan before implementation.

## What I already know

* The user provided a CCR UI screenshot using Catppuccin Mocha that feels visually gray and weak.
* The user provided a reference terminal screenshot that feels more serious, crisp, and visually pleasant.
* The affected app surface is `ccr-ui`, a Vue 3 + Tauri workbench.
* Project guidance requires reading `code_map.md` before broad searches and using the web preview path for visual work.
* Existing CCR UI visual direction should remain calm, precise, and editorial.
* Prior theme work uses the three-layer model: `data-theme` for light/dark/system, `data-flavor` for palette family, and `data-accent` for emphasis color.
* Catppuccin flavors are implemented through semantic token remapping rather than a separate component language.
* The old Trellis task `06-03-ccr-code-audit-optimization` was explicitly confirmed complete by the user and its task directory has been removed.
* Current active Trellis tasks are `00-bootstrap-guidelines` and `06-07-ccr-ui-mocha-visual-polish`.

## Assumptions

* The issue is likely caused by token choices and surface layering, not by product layout alone.
* The desired fix should keep Catppuccin Mocha recognizable while increasing contrast, depth separation, and accent clarity.
* The first step is analysis and direction lock, not immediate broad redesign.

## Open Questions

* Which correction direction should we choose after code and rendered-surface inspection?

## Requirements

* Compare current Mocha UI against the supplied reference screenshot in concrete visual terms.
* Inspect the actual token and layout files responsible for Mocha rendering.
* Identify whether the gray feeling comes from palette hue/chroma, opacity layering, borders/shadows, typography contrast, or all of them.
* Propose 2 to 3 feasible correction approaches with trade-offs.
* Keep the MVP scoped to theme/material polish unless the diagnosis proves layout or typography must change.

## Diagnosis

The gray feeling is not caused by an incorrect Catppuccin Mocha palette alone. The app maps the low-chroma Mocha palette into a layered glass/material system, then stacks many semi-transparent surfaces from the same purple-gray family. The result loses depth separation and reads as foggy.

Concrete causes:

* Mocha base values are correct Catppuccin values: `base #1e1e2e`, `mantle #181825`, `surface0 #313244`, `surface1 #45475a`, `text #cdd6f4`.
* The Catppuccin semantic remap assigns `--color-bg-base` to `base`, `--color-bg-elevated` to `mantle`, `--color-bg-surface` to `surface0`, and `--color-bg-overlay` to `surface1`.
* Catppuccin glass surfaces are built from `mantle` with low alpha: `--glass-bg-light` 44%, `--glass-bg-medium` 56%, `--glass-bg-strong` 68%.
* Dashboard cards add another material layer through `--home-surface-card`, `--home-surface-card-hover`, and `--home-surface-sunk`, then individual widgets add `surface0` overlays at 46% to 70%.
* Blended over `base`, the current glass layers differ by only about 3 to 6 RGB steps, and `surface0` overlays differ by about 9 to 15 RGB steps. That is enough to tint the UI, but not enough to create crisp panel hierarchy.
* The global font mapping uses `MapleBright` for `--font-sans`, `--font-brand`, and `--font-mono`, which softens both headings and numeric/terminal-like details. The reference terminal reads more serious partly because foreground glyphs and number columns are crisper.

Responsible files:

* `ccr-ui/src/styles/tokens.css`: Catppuccin palette, semantic background mapping, glass opacity, borders, shadows, global font variables.
* `ccr-ui/src/styles/home.css`: dashboard/home card, sunk surface, border, and elevation tokens.
* `ccr-ui/src/components/MainLayout.vue`: shell, sidebar, topbar, stage background, nav hover/active materials.
* `ccr-ui/src/views/DashboardView.vue` and `ccr-ui/src/components/dashboard/*`: visible dashboard card hierarchy and nested surface usage.

## Feasible Approaches

**Approach A: Token-only Mocha sharpening (recommended)**

Keep layout and component structure. For `data-resolved-flavor="mocha"` only, deepen shell/base use, reduce glass fog, make dashboard cards more opaque or more explicitly stepped, slightly strengthen borders, and keep accent color controlled. This is the smallest change with the best chance of making Mocha feel serious without redesigning CCR UI.

**Approach B: Dashboard-only material override**

Only adjust `home.css`/dashboard surfaces for Mocha. This has the lowest blast radius and directly targets the screenshot, but other pages using shell glass may still feel gray.

**Approach C: Material plus typography pass**

Do Approach A and also make operational/mono surfaces use a crisper mono or system stack instead of `MapleBright`. This gives the biggest lift toward the reference terminal, but it is a wider visual change and needs more route verification.

Selected direction: Approach C.

## Acceptance Criteria

* [x] Diagnosis names the specific visual causes of the gray appearance.
* [x] Responsible `ccr-ui` files are identified with evidence.
* [x] The recommended correction path is small enough to verify with the CCR UI web preview.
* [x] Out-of-scope larger redesign work is explicitly separated.

## Definition of Done

* User confirms the visual diagnosis and preferred correction direction.
* Implementation plan is recorded if code changes are requested.
* If implementation proceeds, verify with `bun run dev:web -- --host 127.0.0.1 --strictPort` and browser inspection at `http://127.0.0.1:5173/`.
* Run narrow relevant frontend checks after any code changes.

## Implementation Notes

* Added a Mocha-only material override in `ccr-ui/src/styles/tokens.css`.
* Kept the three-layer theme model intact: `data-theme`, `data-flavor`, and `data-accent` remain independent.
* Deepened Mocha base rendering from Catppuccin `base` to `crust`, then stepped shell/card/glass surfaces upward from there.
* Strengthened Mocha borders, shadows, glass opacity, stage surfaces, and dashboard/home card tokens.
* Added a scoped Mocha typography exception for crisper display and mono rendering while preserving MapleBright as the global default.
* Tightened the smoke test so the mono-stack exception is limited to the exact Mocha override block.
* Captured a spec lesson: source text is not enough for token overrides; selector specificity must make the override win in computed CSS.

## Verification

* `cd ccr-ui && bun run type-check` passed.
* `cd ccr-ui && bun run lint` passed.
* `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts` passed.
* `cd ccr-ui && bun run build:web` passed.
* Static build visual check passed through system Edge at `http://127.0.0.1:18080/`.
* Desktop screenshot: `ccr-ui/output/playwright/mocha-polish-desktop.png`.
* Mobile 375px screenshot: `ccr-ui/output/playwright/mocha-polish-mobile.png`.
* Computed token evidence: `ccr-ui/output/playwright/mocha-polish-report.json` showed `data-theme=dark`, `data-flavor=mocha`, `data-resolved-flavor=mocha`, `data-accent=mauve`, and `--color-bg-base=#11111b`.

## Out of Scope

* Replacing the entire CCR UI design system.
* Changing the default `clay` flavor unless explicitly requested.
* Adding new UI dependencies.
* Tauri-native behavior changes unrelated to visual rendering.

## Technical Notes

* Repo orientation file: `code_map.md`.
* Likely files: `ccr-ui/src/styles/tokens.css`, `ccr-ui/src/utils/themeBootstrap.ts`, `ccr-ui/src/views/AppSettingsView.vue`, `ccr-ui/src/components/MainLayout.vue`, and the route components visible in the screenshot.
* Preferred preview workflow is `ccr-ui` web mode, not the Tauri shell, unless native behavior is required.
* If implementation proceeds, load `trellis-before-dev` before editing and use `ccr-ui` web preview: `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`.
* Targeted theme tests should run through Vitest, for example `bunx vitest run --config vitest.smoke.config.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`.
