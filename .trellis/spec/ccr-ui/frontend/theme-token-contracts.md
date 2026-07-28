# Theme Token Contracts

> Executable contracts for CCR UI theme, flavor, accent, and material token changes.

---

## Scenario: Flavor/accent value domains and Catppuccin token overrides

### 1. Scope / Trigger

- Trigger: changing `ccr-ui/src/styles/tokens.css`, `ccr-ui/src/styles/home.css`, theme bootstrap behavior, or tests that guard theme/flavor/accent semantics.
- Applies to the three-layer model: `data-theme` controls light/dark/system resolution, `data-flavor` controls the palette family, and `data-accent` controls emphasis color.
- Since `07-28-color-system-rebuild`: default flavor is `neutral` (explicitly migrated away from `clay` by that task; the "do not replace default clay" rule required — and got — an explicit task declaration).

### 2. Signatures

- Theme bootstrap: `ccr-ui/src/utils/themeBootstrap.ts`
- Global theme tokens: `ccr-ui/src/styles/tokens.css`
- Home/dashboard material tokens: `ccr-ui/src/styles/home.css`
- Value domains (narrowed by `07-28-color-system-rebuild`):
  - `FlavorMode = 'neutral' | 'clay' | 'catppuccin'`; resolved flavor = `'neutral' | 'clay' | 'latte' | 'mocha'`
  - `AccentMode = 'clay' | 'sage' | 'sky' | 'mauve'`
  - `DEFAULT_FLAVOR = 'neutral'`, `DEFAULT_ACCENT = 'clay'`
  - `catppuccin` resolves by resolved theme: light → `latte`, dark → `mocha`
- Visual preference storage keys:
  - `ccr-theme`
  - `ccr-flavor`
  - `ccr-accent`
- Guards:
  - `ccr-ui/tests/theme-bootstrap.smoke.test.ts`
  - `ccr-ui/tests/app-settings.smoke.test.ts`
  - `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts`
  - `ccr-ui/tests/theme-contrast-contract.smoke.test.ts` (contrast/opacity geometry guard)

### 3. Contracts

- Do not replace the default flavor unless the task explicitly asks for a default theme migration (precedent: `07-28-color-system-rebuild` declared it in its PRD).
- Flavor blocks must remap existing semantic tokens instead of adding a second component language.
- Keep `data-theme`, `data-flavor`, and `data-accent` independent. A visual polish change must not collapse flavor into theme or accent. Flavor blocks must not set `--color-accent-primary` / `--color-border-accent` (guarded by a `not.toMatch` smoke assertion for mocha).
- Token geometry axioms (locked by `theme-contrast-contract.smoke.test.ts`):
  - Dark: elevation steps lighten monotonically (`bg-base < bg-elevated < bg-surface < bg-overlay`); light: desktop dimmed, card lightest.
  - All surface tokens and all text tokens (incl. every `--color-stage-*`) resolve to 100% opacity.
  - WCAG contrast vs `bg-surface`: text-primary ≥ 12:1, secondary ≥ 7:1, muted ≥ 4.5:1; accent vs accent-contrast ≥ 3.5:1.
- Flavor-specific exceptions belong in scoped selectors such as `[data-resolved-flavor="mocha"]` or `html:root[data-resolved-flavor="mocha"]`.
- If a later scoped override must beat an earlier flavor block in the CSS cascade, use a selector with deliberately higher specificity and lock that block in smoke tests. Text matching alone is not enough.
- The two Catppuccin blocks are `[data-resolved-flavor='latte']` (single block, palette + semantic remap) and `html:root[data-resolved-flavor='mocha']` (high-specificity full remap). There is no shared multi-flavor Catppuccin remap block; frappe/macchiato palettes no longer exist.
- When adding a controlled font exception, narrow the test exception to the exact override block. Do not skip the whole `tokens.css` file.
- Visual automation that preloads theme preferences must write `ccr-theme`, `ccr-flavor`, and `ccr-accent`, then assert the rendered `data-theme`, `data-flavor`, `data-resolved-flavor`, and `data-accent` values before trusting computed styles.

### 4. Validation & Error Matrix

- Changed theme token semantics without targeted theme smoke tests -> not accepted.
- Mocha/material override present in source but lower specificity than the latte block -> rendered CSS may stay wrong even though text tests pass.
- New mono/display stack outside a documented scoped exception -> `apple-glass-surface-contract.smoke.test.ts` should fail.
- Changed theme/flavor/accent persistence semantics -> `theme-bootstrap.smoke.test.ts` and `app-settings.smoke.test.ts` must cover it.
- Visual verification uses a bundle-shaped preference key instead of `ccr-theme` / `ccr-flavor` / `ccr-accent` -> the page silently falls back to default and the evidence is invalid.
- Lowering a contrast threshold in `theme-contrast-contract.smoke.test.ts` to make a palette pass -> not accepted; adjust token values instead (thresholds are the contract).
- Flavor block reintroducing alpha-bearing text/surface tokens (e.g. stage tokens < 100% opacity) -> contrast-contract smoke fails.

### 5. Good/Base/Bad Cases

- Good: keep the mocha semantic remap inside `html:root[data-resolved-flavor="mocha"]` and assert the exact block in smoke tests.
- Good: in Playwright, preload `localStorage.setItem('ccr-theme', 'dark')`, `localStorage.setItem('ccr-flavor', 'catppuccin')`, and `localStorage.setItem('ccr-accent', 'sky')`, then assert the document dataset before recording screenshots.
- Good: strip only the Mocha override block before scanning for otherwise-forbidden legacy mono stacks.
- Base: keep latte block structure untouched when the task only targets mocha.
- Bad: preload a custom JSON blob such as `ccr-ui-shell-preferences-v1` and report dark/mocha visual evidence without checking the rendered dataset.
- Bad: exclude all of `tokens.css` from a font-stack guard because one scoped exception exists.
- Bad: change `data-theme` behavior to force a flavor directly instead of preserving `data-flavor`.
- Bad: fix a failing contrast case by editing the expected threshold constant instead of the token value.

### 6. Tests Required

- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint`
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts tests/theme-contrast-contract.smoke.test.ts`
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

## Scenario: Flavor/accent value migration and bootstrap sync

### 1. Scope / Trigger

- Trigger: narrowing or renaming `FlavorMode` / `AccentMode` values, changing `DEFAULT_FLAVOR` / `DEFAULT_ACCENT`, or touching the `ccr-flavor` / `ccr-accent` read path.
- Introduced by `07-28-color-system-rebuild` (7→3 flavors, 8→4 accents). The migration machinery is the contract that keeps old `localStorage` values safe.

### 2. Signatures

- `ccr-ui/src/utils/themeBootstrap.ts`: `FLAVOR_MIGRATION` / `ACCENT_MIGRATION` maps, `migrateFlavorValue` / `migrateAccentValue` (map → whitelist → fallback), `migratePersistedFlavor` / `migratePersistedAccent` (read + write-back only when changed; never seed a default into empty storage), `readStoredFlavor` / `readStoredAccent` (go through migration).
- `ccr-ui/src/stores/shellPreferences.ts`: `initializeTheme` calls `migratePersisted*` before reading; `setFlavorPreference` / `setAccentPreference` normalize via `migrateFlavorValue` / `migrateAccentValue` before persisting.
- `ccr-ui/index.html` first-paint IIFE: inline duplicate of the same migration maps + whitelist + catppuccin resolution (no import capability).
- Migration tables (current):
  - flavor: `paper|graphite → neutral`; `latte|frappe|macchiato|mocha → catppuccin`; unknown → `neutral`
  - accent: `sand|amber|rose → clay`; `slate → sky`; unknown → `clay`

### 3. Contracts

- The migration map + whitelist + fallback + catppuccin resolution MUST be byte-equivalent in behavior between `themeBootstrap.ts` and the `index.html` IIFE. Changing one without the other splits first-paint from runtime resolution.
- Migration happens on read, never on render; unknown values fall back to the defaults (`neutral` / `clay`).
- Write-back (`migratePersisted*`) only fires when the stored value differs from the migrated value, and never writes when the key is absent (no default seeding).
- Old UI option values (e.g. a stale `mocha` button) must be safe at runtime: setters normalize before persisting, so `data-resolved-flavor` can only ever be `neutral|clay|latte|mocha`.
- Rollback tolerance: an older app version reading new values (`neutral` / `catppuccin`) falls back to its own defaults — acceptable, no data loss.

### 4. Validation & Error Matrix

- IIFE and runtime migration maps diverge -> first paint renders one palette and hydration swaps to another; `theme-bootstrap.smoke.test.ts` IIFE behavior tests must cover both.
- Write-back that seeds defaults into empty storage -> users who never chose a flavor get a pinned value that blocks future default changes.
- Setter persisting an un-normalized legacy value -> `data-flavor` shows a value outside the current domain and the flavor blocks stop matching.

### 5. Good/Base/Bad Cases

- Good: `readStoredFlavor()` returns `migrateFlavorValue(stored)`; `migratePersistedFlavor()` writes back only on difference.
- Good: IIFE test feeds `localStorage.setItem('ccr-flavor', 'macchiato')` + light scheme and asserts `data-resolved-flavor === 'latte'` before any CSS loads.
- Base: `theme-bootstrap.smoke.test.ts` locks IIFE behavior by executing the extracted script against seeded storage (behavior lock, stronger than verbatim text lock).
- Bad: adding a fifth flavor value to the TS union but forgetting the IIFE whitelist -> first-paint fallback loop.

### 6. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`
- Migration coverage must include every map entry, the unknown-value fallback, write-back semantics, and the empty-storage no-seeding case.

---

## Scenario: Font preference override + fallback stack (`--font-*-base`)

### 1. Scope / Trigger

- Trigger: changing the font tracks in `ccr-ui/src/styles/tokens.css`, `ccr-ui/src/utils/fontPreferences.ts`, the font controls in `AppSettingsView.vue`, the `index.html` boot-script font logic, or the `apple-glass-surface-contract` font guards.
- Introduced by `07-13-ccr-ui-font-settings`: user-selectable interface font (sans + brand) and code font (mono) that prepend to the built-in stack and fall back automatically on a missing font or missing glyph (notably CJK), Codex-style. Font preference is a fourth appearance axis alongside `data-theme`/`data-flavor`/`data-accent`, but it is applied as an inline CSS custom property, not a `data-*` attribute.

### 2. Signatures

- Font util: `ccr-ui/src/utils/fontPreferences.ts` — `sanitizeFontFamily`, `applyFontsToDocument` / `applyUiFontToDocument` / `applyCodeFontToDocument`, `readStoredUiFont` / `readStoredCodeFont` / `persist*`, `applyInitialFonts`, `UI_FONT_PRESETS` / `CODE_FONT_PRESETS`.
- Font tracks in `tokens.css`: `--font-{sans,brand,mono}-base` hold the built-in stacks; `--font-{sans,brand,mono}` default to `var(--font-*-base)`.
- Store: `ccr-ui/src/stores/shellPreferences.ts` → `uiFont` / `codeFont`, `setUiFont` / `setCodeFont`.
- First paint: `ccr-ui/index.html` "主题预初始化" IIFE.
- Storage keys: `ccr-font-ui`, `ccr-font-code`.
- Guards: `ccr-ui/tests/font-preferences.smoke.test.ts`, `ccr-ui/tests/theme-bootstrap.smoke.test.ts`, `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts`.

### 3. Contracts

- The built-in fallback stack lives only in `--font-*-base`; `--font-*` defaults to `var(--font-*-base)`. Never re-inline a literal font stack into `--font-*`.
- Override = inline custom property on `document.documentElement`: `--font-sans` / `--font-brand` = `"<uiFont>", var(--font-*-base)`; `--font-mono` = `"<codeFont>", var(--font-mono-base)`. Interface font drives sans + brand; code font drives mono only. Keep the two channels independent.
- Reset (empty / system default) = `removeProperty` the inline var so it resolves back to `var(--font-*-base)`. Do not write the base literal to clear an override.
- User input must pass `sanitizeFontFamily` before entering any CSS var: strip `" ' \` \ ; { } < > ( ) ,` plus control chars (`\u0000`–`\u001f`, `\u007f`), collapse whitespace, cap at 64 chars; empty result = default (no override). Sanitize runs on both persist and apply, and the boot script repeats a lightweight strip because `localStorage` can be hand-edited.
- Font preferences are localStorage-only, like theme/flavor/accent. Do not route them through the Tauri `DesktopShellPreferences`.
- First paint: the `index.html` boot IIFE must apply the same prepend before any CSS loads or the app flashes the default font (FOUC). `var(--font-*-base)` inside the inline value resolves lazily once `tokens.css` loads, so setting it pre-CSS is safe.
- Preset font-name literals (which include otherwise-forbidden mono names such as Cascadia Code / JetBrains Mono / Consolas) live ONLY inside the `/* ========== 字体预设清单 ========== */ … /* ========== 字体预设清单结束 ========== */` block in `fontPreferences.ts` — a controlled exception (`fontPresetBlockPattern`) in the legacy-mono-stack guard. Do not scatter font-name literals into components, i18n, or styles.
- Preset literals are exact OS-visible family names, not aliases. Keep `Source Han Sans SC`, `Source Han Sans CN`, and `Source Han Serif SC VF` as distinct interface presets: SC/CN identify different regional family registrations, while Serif is a different typeface category. Adding one must not replace another, and the app still falls back silently when a family is unavailable on the current host.
- i18n copy for the font controls must not embed `{`, `}`, or `|` (vue-i18n message-compiler metacharacters). A code preview like `() => { 0O il1 }` is parsed as an invalid named interpolation and fails `test:i18n`. Keep preview samples brace/pipe-free.

### 4. Validation & Error Matrix

- Literal font stack written into `--font-*` instead of `var(--font-*-base)` -> `apple-glass-surface-contract.smoke.test.ts` font-track assertion fails and reset can no longer restore cleanly.
- User font applied without `sanitizeFontFamily` -> CSS-injection / broken quoted string; `font-preferences.smoke.test.ts` sanitize cases fail.
- Font-name literal added to a component/i18n/style outside the preset block -> `apple-glass-surface-contract.smoke.test.ts` legacy-mono-stack guard fails.
- Source Han regional or serif family used as a replacement for another family -> valid host installations lose their preset or silently fall back; `font-preferences.smoke.test.ts` must assert that all three exact family names remain present and unique.
- New font i18n string containing `{` / `}` / `|` -> `test:i18n` "messages compile with vue-i18n" critical failure (both locales).
- Boot script not updated for a new font channel -> first paint flashes the default font; `theme-bootstrap.smoke.test.ts` first-paint font assertion fails.
- Font preference persisted to a backend/bundle key instead of `ccr-font-ui` / `ccr-font-code` -> not applied at boot; the page silently keeps the built-in stack and the evidence is invalid.

### 5. Good/Base/Bad Cases

- Good: `applyFontsToDocument('Inter', 'JetBrains Mono')` sets `--font-sans`/`--font-brand` to `"Inter", var(--font-*-base)` and `--font-mono` to `"JetBrains Mono", var(--font-mono-base)`; empty strings `removeProperty` all three.
- Good: add a new dropdown option inside the marked preset block; the guard's `fontPresetBlockPattern` strips it automatically, no test edit needed.
- Good: add `Source Han Sans CN` and `Source Han Serif SC VF` beside the existing `Source Han Sans SC`, then lock the three distinct literals in the focused font-preference test.
- Base: leave the `--font-*-base` stacks (MapleBright / SF Pro Display / Cascadia Code) unchanged when a task only adds user overrides.
- Bad: `root.style.setProperty('--font-sans', "'MyFont', 'MapleBright', …")` re-inlining the whole stack instead of `"MyFont", var(--font-sans-base)`.
- Bad: a preview i18n sample `const x = () => { 0O il1 }` (breaks vue-i18n compilation).
- Bad: hardcoding `JetBrains Mono` in the `AppSettingsView.vue` template/script or an i18n placeholder instead of the preset block.
- Bad: replacing `Source Han Sans SC` with `Source Han Serif SC VF` because one Windows host has the latter installed; this conflates regional naming with serif/sans typeface choice.

### 6. Tests Required

- `cd ccr-ui && bun run type-check && bun run lint`
- `cd ccr-ui && bun run test:i18n` (font preview/placeholder copy must compile under vue-i18n).
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/font-preferences.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/apple-glass-surface-contract.smoke.test.ts tests/app-settings.smoke.test.ts`
- For visual work: in the web preview, pick a present font (applies), a missing font (falls back, no tofu), and system default (restores), and confirm the first paint has no font flash.

### 7. Wrong vs Correct

#### Wrong

```js
root.style.setProperty('--font-sans', "'MyFont', 'MapleBright', 'PingFang SC', sans-serif")
```

Re-inlines the fallback stack, so a later `--font-sans-base` change silently diverges and reset cannot restore cleanly.

#### Correct

```js
root.style.setProperty('--font-sans', '"MyFont", var(--font-sans-base)')
// reset back to the built-in stack:
root.style.removeProperty('--font-sans')
```

Prepend the user font and keep the single source of truth in `--font-sans-base`.

---

## Scenario: Three-tier material glass budget (`--material-glass-*`)

### 1. Scope / Trigger

- Trigger: adding/removing a `backdrop-filter` surface, or wiring a component to `.glass-floating` / `.glass-chrome` / `.glass-inline` in `ccr-ui/src/styles/utilities.css`.
- Introduced by `07-07-ui-glass-tokens`; recipes redefined by `07-28-color-system-rebuild`: **chrome and inline tiers are now opaque** (solid background + `blur: none`), only the floating tier keeps real blur (`blur(12px)`, bg ≥ 88% opacity, no `saturate()`). The old "everything is a translucent blur surface" look was the 泛白 root cause.

### 2. Signatures

- Tier tokens (defined per flavor in `tokens.css`): `--material-glass-floating-{bg,blur,border,highlight,shadow}`, `-chrome-`, `-inline-`.
- Utility classes: `ccr-ui/src/styles/utilities.css` → `.glass-floating`, `.glass-chrome`, `.glass-inline`.
- Legacy tokens `--glass-blur-*` / `--glass-bg-*` / `--liquid-glass-*` are `deprecated` (comment-tagged in `tokens.css`) and intentionally kept at their old, thin values — do not "improve" them into real glass, that would blow the on-screen budget below.

### 3. Contracts

- Budget: at most 1 `backdrop-filter` element on screen at once (floating only); never nest glass inside glass; never put glass inside a scrolling content area (scroll + blur repaints continuously).
- Tier assignment is fixed by role, not by choice: `floating` = modal/command-palette/floating panel (≤1 on screen), `chrome` = sidebar/topbar (opaque), `inline` = sticky in-page toolbars (opaque).
- Ordinary content cards/workspaces are **not** glass: they map to `--surface-card-*` / `--surface-workspace-*`, which must resolve to `blur: none` and 100% opacity. If a component needs "depth", use elevation (border + shadow), not transparency.
- Every place a tier's `background`/`blur` is set must have a matching reset inside the `prefers-reduced-transparency: reduce` block, including inside flavor-scoped overrides (e.g. mocha) — a reduced-transparency block that resets the shared tokens but not the mocha-scoped ones still leaves mocha glass on screen.
- Don't repoint legacy `--glass-*`/`--liquid-glass-*` tokens to the new material recipes just because it's tempting to unify — that pushes old call-sites (31 files, 75+ references at time of writing) over budget. Migrate call-sites to the tier classes/tokens explicitly instead, one component at a time.

### 4. Validation & Error Matrix

- New `backdrop-filter` usage added outside the floating tier -> breaks the ≤1 budget; check for an existing floating surface on the same route first.
- Glass applied to a scrollable list/table row -> forbidden regardless of tier; use opaque surface tokens.
- `prefers-reduced-transparency` block updates the shared `--material-glass-*-bg` but a flavor override (e.g. `html:root[data-resolved-flavor='mocha']`) sets its own copy at higher specificity -> glass survives the reduced-transparency preference for that flavor; `apple-glass-surface-contract.smoke.test.ts` asserts the mocha-scoped reset explicitly, so extend that assertion pattern for any new flavor-scoped material override.
- A component still reads `--liquid-glass-*` / `--glass-blur-*` directly -> acceptable as-is (deprecated but stable); flag for migration in the relevant child task instead of patching the legacy token's recipe.
- Re-adding `saturate()` to a glass blur recipe -> washed-out amplification of whatever bleeds through; forbidden by the floating-tier recipe contract.

### 5. Good/Base/Bad Cases

- Good: `BaseModal` uses the floating tier via `--surface-modal-*` (one modal on screen at a time); sidebar/topbar chrome reads `--surface-shell-*` which resolves to the opaque chrome recipe.
- Base: a component still reads legacy `--liquid-glass-bg`/`-border`/`-highlight`/`-shadow` directly (e.g. `ConfigCard.vue`) — fine to leave as-is outside a migration task's scope; the legacy tokens still resolve to sane, budget-safe values because they stay thin, not because they were repointed.
- Bad: adding `backdrop-filter: var(--material-glass-chrome-blur)` to a table row or an infinite-scroll list item.
- Bad: writing a new flavor override block that sets `--material-glass-floating-bg` without adding the matching flavor-scoped reset inside `@media (prefers-reduced-transparency: reduce)`.
- Bad: lowering the floating tier's bg opacity below 88% "for aesthetics" — translucency belongs to no content surface.

### 6. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts` (locks tier token presence, opaque chrome/inline recipes, floating-tier ceiling, and reduced-transparency fallback incl. flavor-scoped resets).
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

- Redefining a shared alias's value (e.g. changing `--surface-status-bg` to chrome-tier opacity/blur) to fix one component -> silently changes Button/Input/Card/Titlebar/etc. everywhere; not caught by type-check or lint, only by visual regression. (Token-level redefinitions DO happen in dedicated palette-rebuild tasks such as `07-28-color-system-rebuild` — outside such a task, treat alias values as frozen.)
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

- Confirmed bridged (exist in `theme.css`): `--platform-claude`, `--platform-codex`, `--platform-gemini` (non-`-rgb` only), `--stage-text-*`, `--stage-surface-*`, `--stage-chip-neutral-*`, `--accent-*` (now including `--accent-primary-contrast`), `--bg-*`, `--text-*`.
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
