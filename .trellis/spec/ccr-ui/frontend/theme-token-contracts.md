# Theme Token Contracts

> Executable contracts for CCR UI theme, flavor, accent, and material token changes.

---

## Scenario: Catppuccin flavor token overrides

### 1. Scope / Trigger

- Trigger: changing `ccr-ui/src/styles/tokens.css`, `ccr-ui/src/styles/home.css`, theme bootstrap behavior, or tests that guard theme/flavor/accent semantics.
- Applies to the three-layer model: `data-theme` controls light/dark/system resolution, `data-flavor` controls the palette family, and `data-accent` controls emphasis color.

### 2. Signatures

- Theme bootstrap: `ccr-ui/src/utils/themeBootstrap.ts`
- Global theme tokens: `ccr-ui/src/styles/tokens.css`
- Home/dashboard material tokens: `ccr-ui/src/styles/home.css`
- Visual preference storage keys:
  - `ccr-theme`
  - `ccr-flavor`
  - `ccr-accent`
- Guards:
  - `ccr-ui/tests/theme-bootstrap.smoke.test.ts`
  - `ccr-ui/tests/app-settings.smoke.test.ts`
  - `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts`

### 3. Contracts

- Do not replace the default `clay` flavor unless the task explicitly asks for a default theme migration.
- Catppuccin flavors must remap existing semantic tokens instead of adding a second component language.
- Keep `data-theme`, `data-flavor`, and `data-accent` independent. A visual polish change must not collapse flavor into theme or accent.
- Flavor-specific exceptions belong in scoped selectors such as `[data-resolved-flavor="mocha"]` or `html:root[data-resolved-flavor="mocha"]`.
- If a later scoped override must beat an earlier flavor block in the CSS cascade, use a selector with deliberately higher specificity and lock that block in smoke tests. Text matching alone is not enough.
- When adding a controlled font exception, narrow the test exception to the exact override block. Do not skip the whole `tokens.css` file.
- Visual automation that preloads theme preferences must write `ccr-theme`, `ccr-flavor`, and `ccr-accent`, then assert the rendered `data-theme`, `data-flavor`, `data-resolved-flavor`, and `data-accent` values before trusting computed styles.

### 4. Validation & Error Matrix

- Changed theme token semantics without targeted theme smoke tests -> not accepted.
- Mocha/material override present in source but lower specificity than the shared Catppuccin block -> rendered CSS may stay gray even though text tests pass.
- New mono/display stack outside a documented scoped exception -> `apple-glass-surface-contract.smoke.test.ts` should fail.
- Changed theme/flavor/accent persistence semantics -> `theme-bootstrap.smoke.test.ts` and `app-settings.smoke.test.ts` must cover it.
- Visual verification uses a bundle-shaped preference key instead of `ccr-theme` / `ccr-flavor` / `ccr-accent` -> the page silently falls back to default light/clay and the evidence is invalid.

### 5. Good/Base/Bad Cases

- Good: add `html:root[data-resolved-flavor="mocha"]` after the shared Catppuccin block when Mocha needs a sharper material treatment.
- Good: in Playwright, preload `localStorage.setItem('ccr-theme', 'dark')`, `localStorage.setItem('ccr-flavor', 'mocha')`, and `localStorage.setItem('ccr-accent', 'sky')`, then assert the document dataset before recording screenshots.
- Good: strip only the Mocha override block before scanning for otherwise-forbidden legacy mono stacks.
- Base: keep shared Catppuccin palette variables and semantic remaps unchanged for Latte, Frappe, and Macchiato when the task only targets Mocha.
- Bad: preload a custom JSON blob such as `ccr-ui-shell-preferences-v1` and report dark/mocha visual evidence without checking the rendered dataset.
- Bad: exclude all of `tokens.css` from a font-stack guard because one scoped exception exists.
- Bad: change `data-theme` behavior to force Mocha directly instead of preserving `data-flavor`.

### 6. Tests Required

- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint`
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`
- For visual work, inspect the web preview or a static build in a real browser and record the route, viewport, document dataset values, and key computed tokens.

### 7. Wrong vs Correct

#### Wrong

```css
[data-resolved-flavor="mocha"] {
  --color-bg-base: var(--ctp-crust);
}
```

If an earlier selector with equal or higher specificity also sets `--color-bg-base`, this source text can look correct while the rendered value remains unchanged.

#### Correct

```css
html:root[data-resolved-flavor="mocha"] {
  --color-bg-base: var(--ctp-crust);
}
```

Pair the override with a smoke assertion that extracts this exact block and checks the token values that must render.

---

## Scenario: Three-tier material glass budget (`--material-glass-*`)

### 1. Scope / Trigger

- Trigger: adding/removing a `backdrop-filter` surface, or wiring a component to `.glass-floating` / `.glass-chrome` / `.glass-inline` in `ccr-ui/src/styles/utilities.css`.
- Introduced by `07-07-ui-glass-tokens` as the real (blur 8~~16px + saturate 140~~180%) replacement for the old "everything is 2~6px blur" glass tokens, which read as fog rather than material.

### 2. Signatures

- Tier tokens (defined per flavor in `tokens.css`): `--material-glass-floating-{bg,blur,border,highlight,shadow}`, `-chrome-`, `-inline-`.
- Utility classes: `ccr-ui/src/styles/utilities.css` → `.glass-floating`, `.glass-chrome`, `.glass-inline`.
- Legacy tokens `--glass-blur-*` / `--glass-bg-*` / `--liquid-glass-*` are `deprecated` (comment-tagged in `tokens.css`) and intentionally kept at their old, thin values — do not "improve" them into real glass, that would blow the on-screen budget below.

### 3. Contracts

- Budget: at most 3 `backdrop-filter` elements on screen at once; never nest glass inside glass; never put glass inside a scrolling content area (scroll + blur repaints continuously).
- Tier assignment is fixed by role, not by choice: `floating` = modal/command-palette/floating panel (≤1 on screen), `chrome` = sidebar/topbar (≤2, persistent), `inline` = sticky in-page toolbars (rare).
- Ordinary content cards/workspaces are **not** glass: they map to `--surface-card-*` / `--surface-workspace-*`, which must resolve to `blur: none` and ≥98% opacity. If a component needs "depth", use elevation (border + shadow), not transparency.
- Every place a tier's `background`/`blur` is set must have a matching reset inside the `prefers-reduced-transparency: reduce` block, including inside flavor-scoped overrides (e.g. mocha) — a reduced-transparency block that resets the shared tokens but not the mocha-scoped ones still leaves mocha glass on screen.
- Don't repoint legacy `--glass-*`/`--liquid-glass-*` tokens to the new material recipes just because it's tempting to unify — that pushes old call-sites (31 files, 75+ references at time of writing) over budget. Migrate call-sites to the tier classes/tokens explicitly instead, one component at a time.

### 4. Validation & Error Matrix

- New `backdrop-filter` usage added without checking current on-screen count -> likely breaks the ≤3 budget; check for existing floating/chrome/inline usage on the same route first.
- Glass applied to a scrollable list/table row -> forbidden regardless of tier; use opaque surface tokens.
- `prefers-reduced-transparency` block updates the shared `--material-glass-*-bg` but a flavor override (e.g. `html:root[data-resolved-flavor='mocha']`) sets its own copy at higher specificity -> glass survives the reduced-transparency preference for that flavor; `apple-glass-surface-contract.smoke.test.ts` asserts the mocha-scoped reset explicitly, so extend that assertion pattern for any new flavor-scoped material override.
- A component still reads `--liquid-glass-*` / `--glass-blur-*` directly -> acceptable as-is (deprecated but stable); flag for migration in the relevant child task instead of patching the legacy token's recipe.

### 5. Good/Base/Bad Cases

- Good: `BaseModal` uses `.glass-floating` (one modal on screen at a time); sidebar/topbar chrome uses `.glass-chrome`; a page never stacks more than one floating + two chrome surfaces simultaneously.
- Base: a component still reads legacy `--liquid-glass-bg`/`-border`/`-highlight`/`-shadow` directly (e.g. `ConfigCard.vue`) — fine to leave as-is outside this task's scope; the legacy tokens still resolve to sane, budget-safe values because they stay thin, not because they were repointed.
- Bad: adding `backdrop-filter: var(--material-glass-chrome-blur)` to a table row or an infinite-scroll list item.
- Bad: writing a new flavor override block that sets `--material-glass-floating-bg` without adding the matching flavor-scoped reset inside `@media (prefers-reduced-transparency: reduce)`.

### 6. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts` (locks tier token presence, budget comment, and reduced-transparency fallback incl. flavor-scoped resets).
- Manual/DevTools: toggle "Emulate CSS prefers-reduced-transparency: reduce" (Rendering panel) and confirm all glass surfaces on the route go opaque.

---

## Scenario: Migrating a component onto a glass tier via its semantic alias, not the raw tier token

### 1. Scope / Trigger

- Trigger: moving an existing component's `backdrop-filter` rule from one glass tier to another (e.g. `07-07-ui-shell-home` moving `MainLayout.vue`'s `.topbar-glass` from inline to chrome tier).
- The semantic surface-contract layer in `tokens.css` (`--surface-shell-*` = chrome, `--surface-status-*` = inline, `--surface-card-*`/`--surface-workspace-*` = opaque, `--surface-modal-*` = floating) sits _between_ components and the raw `--material-glass-*-*` tokens. Components should read the semantic alias, not the raw tier token directly, wherever an alias already exists.

### 2. Signatures

- Semantic aliases: `tokens.css` → `--surface-shell-{bg,blur,border,shadow}`, `--surface-status-{bg,blur,border,shadow}`, `--surface-card-{bg,blur,border,shadow}`, `--surface-workspace-*`, `--surface-modal-*`.
- `--surface-status-*` consumers at time of writing (07-07-ui-shell-home): `Button.vue`, `Input.vue`, `Card.vue`, `Titlebar.vue`, `ListSearchHeader.vue`, `MultiSelectFloatingBar.vue`, `McpListPanel.vue`/`McpImportPanel.vue`/`McpCreatePanel.vue`/`McpDetailPanel.vue`, `BulkDeleteDialog.vue`, `ThemeToggle.vue`, `UsageDashboardView.vue`, `AppSettingsView.vue`, `UsageInsightPanel.vue` (15+ files) — this is a **shared** general-purpose "sticky/toolbar surface" alias, not a token owned by any one component.

### 3. Contracts

- Before repointing a component to a different tier, `rg` the semantic alias you're about to touch (`--surface-status-`, `--surface-shell-`, etc.) across `ccr-ui/src`. If more than the one component you're editing shows up, do **not** redefine that alias's value in `tokens.css` — instead change the _consumer's_ CSS rule to reference a different, already-correct-tier alias.
- `--surface-shell-*` was, before this task, only consumed by `.sidebar-glass`. Repointing `.topbar-glass` to the same alias (instead of inventing a third alias or redefining `--surface-status-*`) means sidebar+topbar now correctly share one chrome-tier budget slot, and the 15+ unrelated `--surface-status-*` consumers are untouched.
- If no existing alias matches the target tier for a given role, add a new semantic alias in the "Surface Contract" block of `tokens.css` rather than pointing the component at `--material-glass-<tier>-*` directly — keeps the indirection consistent for the next migration.

### 4. Validation & Error Matrix

- Redefining a shared alias's value (e.g. changing `--surface-status-bg` to chrome-tier opacity/blur) to fix one component -> silently changes Button/Input/Card/Titlebar/etc. everywhere; not caught by type-check or lint, only by visual regression.
- Repointing a consumer to a different alias without checking the alias resolves through the intended tier (`--surface-shell-*` -> `--material-glass-chrome-*`) -> verify via `rg "surface-shell" ccr-ui/src/styles/tokens.css` before and after, or `preview_inspect` the element's computed `backdrop-filter`/`background-color` against the sibling that's known-correct (e.g. confirm topbar and sidebar report identical computed `backdrop-filter`).

### 5. Good/Base/Bad Cases

- Good: `.topbar-glass { background: var(--surface-shell-bg); backdrop-filter: var(--surface-shell-blur); ... }` — reuses the existing chrome alias, zero blast radius on other consumers.
- Bad: editing `--surface-status-bg`/`-blur`/`-border`/`-shadow` inside `tokens.css` to chrome-tier values "because the topbar needs it" — breaks every other `--surface-status-*` consumer.
- Bad: adding `background: var(--material-glass-chrome-bg)` directly on a new component instead of introducing/reusing a semantic alias — works today but skips the indirection future theme/flavor work relies on.

### 6. Tests Required

- `rg "<alias-name>" ccr-ui/src` before touching any semantic alias's _definition_ (not just a consumer's usage).
- `cd ccr-ui && bun run type-check && bun run lint` (CSS-only changes won't be caught by either — pair with a live `preview_inspect` computed-style check on both the migrated component and one representative other consumer of the alias you did _not_ touch).

---

## Scenario: `theme.css` legacy bridge names are non-exhaustive (phantom `var(..., fallback)` tokens)

### 1. Scope / Trigger

- Trigger: a consumer references a short-form variable (no `--color-`/`--stage-` prefix, e.g. `--platform-codex`, `--platform-codex-rgb`) with a literal fallback, e.g. `rgb(var(--platform-codex-rgb, 245 158 11) / 10%)`.
- `theme.css` (`/* Compatibility bridge: keep legacy variable names alive, but source all values from tokens.css. */`) only bridges a hand-picked subset of short names to their canonical `--color-*` definitions. It is **not** a mechanical 1:1 mirror of every token in `tokens.css`.

### 2. Signatures

- Confirmed bridged (exist in `theme.css`): `--platform-claude`, `--platform-codex`, `--platform-gemini` (non-`-rgb` only), `--stage-text-*`, `--stage-surface-*`, `--stage-chip-neutral-*`, `--accent-*`, `--bg-*`, `--text-*`.
- Confirmed **not** bridged (verified via `rg "^\s*--[a-z-]+: var\(--color-" ccr-ui/src/styles/theme.css`): `--platform-claude-rgb`, `--platform-codex-rgb`, `--platform-gemini-rgb`. Any consumer of these three always silently uses its own literal fallback — the CSS custom property never resolves through the cascade.

### 3. Contracts

- Before consuming a short-form `var(--foo, <fallback>)` pattern, `rg "^\s*--foo:" ccr-ui/src/styles/theme.css` to confirm the bridge actually exists. If it doesn't, either bridge-consume the canonical `--color-foo` token directly (preferred — matches how sibling Codex/Gemini components already consume `--color-platform-codex-rgb` / `--color-platform-gemini-rgb` directly, per `rg "color-platform-codex-rgb" ccr-ui/src`), or add the missing bridge line in `theme.css` if the short form must stay the public name.
- A literal fallback on a `var()` reference is not automatically "intentional decorative default" — verify the primary reference actually resolves somewhere in the cascade first. If it never resolves, the fallback **is** the hardcoded value in practice, just spelled to look like a token.
- Don't assume `-rgb` siblings of a bridged name are also bridged; check each suffix independently.

### 4. Validation & Error Matrix

- `var(--platform-codex-rgb, 245 158 11)` used anywhere -> always renders `rgb(245 158 11)` regardless of theme/flavor, because `--platform-codex-rgb` is never set. Found live in `codex-auth-shared.css` (fixed, 07-09-ui-codex-auth-css-tokens) and still present in `CodexSettingsView.vue:899-900` (out of scope for that task, unfixed as of 2026-07-09).
- Treating a passing `rg "#[0-9a-fA-F]{3,8}\b|rgba?\("` scan as proof of "no hardcoded colors" -> false negative for this pattern, since `var(--undefined-name, 245 158 11)` doesn't match a bare hex/rgb literal regex at the call site in the same way a raw `rgb(245 158 11 / 10%)` would, but behaves identically at runtime. Grep for `var\(--[a-z-]+-rgb,\s*\d` / `var\(--[a-z-]+,\s*#` patterns too when auditing a file for this class of bug.

### 5. Good/Base/Bad Cases

- Good: `background: rgb(var(--color-platform-codex-rgb) / 10%);` — consumes the canonical, always-defined token directly, no fallback needed.
- Bad: `background: rgb(var(--platform-codex-rgb, 245 158 11) / 10%);` — looks token-based, is actually a permanently-hardcoded amber regardless of theme/flavor.

### 6. Tests Required

- `rg "var\(--[a-z-]+-rgb,\s*\d|var\(--[a-z-]+,\s*#" <file>` when auditing a file for hardcoded-color migration, in addition to the plain hex/rgba regex.
- `preview_inspect` the computed `background-color`/`color` against the token's known value from `tokens.css` (not just "does it look like a var() call").
