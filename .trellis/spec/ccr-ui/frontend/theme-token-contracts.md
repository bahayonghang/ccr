# Theme Token Contracts

> Executable contracts for CCR UI theme, flavor, accent, and material token changes.
> Rewritten by `08-22-design-system` (batch 8) for the Tailwind v4 two-layer token structure on the React side.
> Coordination: the executable guards referenced here are the current smoke suite; `08-22-test-contract-rebuild`
> extends them to the full 19-document contract set (this document is co-owned with that task per its Notes).

## Palette World Notes (value-level history; contracts and thresholds unchanged)

- `09-03-theme-token-world` replaced the palette **values** under the unchanged token names and value domains
  (direction: 行情终端 / warm-black phosphor terminal, impeccable seed `19fe1fa0`):
  - Neutral ramps warmed in both themes: light `#e9e4d8 / #f2eee3 / #faf7ec / #ddd5c2` (warm paper),
    dark `#100f0c / #171410 / #1f1b14 / #2a251b` (warm black); text and border tracks warmed with them.
  - The `clay` **accent key** now resolves to terminal amber — light `#8f650e`, dark `#f0a32b` — at all four
    definition points (`:root`, `[data-theme='dark']`, `[data-accent='clay']`, `[data-theme='dark'][data-accent='clay']`).
    The key was deliberately NOT renamed to `amber`: `amber → clay` already exists in the accent migration map,
    and reintroducing an `amber` value would collide with stored-value migration. The name/color mismatch is
    intentional and documented here.
  - Semantic colors aligned to terminal semantics (dark: success `#5fa05a`, warning `#d9c05a`, danger `#cc5b45`,
    info `#7d94b0`; light warning `#a07c1e` to keep hue distance from the amber accent).
  - Antigravity platform color adopted at its four consumer surfaces (nav swatch, nav icon, platform card,
    usage chart segment + legend): they reference `--color-platform-antigravity` instead of the Gemini blue.
    `--color-platform-antigravity` stays layer-1 only (no `@theme inline` entry), so consumers use the
    arbitrary-value pattern (`text-[color:var(--color-platform-antigravity)]`), same as OpenCode.
  - Boot loader in `ccr-ui/index.html` recolored to the new world (light base `#e9e4d8`, dark base `#100f0c`,
    dark spinner amber `#f0a32b`); the boot IIFE migration maps/whitelists are untouched.
  - Contrast thresholds were NOT lowered to make this palette pass; all values above clear the locked gates.


## Registered Scale Extensions

- **Font scale extension (registered by `07-29-profiles-shared-layer`)**: dense meta information (field labels, tag chips, diff rows, stat hints inside the Profiles shared layer) may use `0.75rem`, one step below the Label floor `0.8125rem`. This is the only sub-Label step; px literal font sizes remain forbidden. The exception carries over verbatim to React (`hardcode-mapping.md`「字号」).

## Three-Layer Theme Model (unchanged semantics)

- `data-theme` (`light` | `dark` | `system` resolution) controls light/dark.
- `data-flavor` (`neutral` | `clay`) controls the palette family.
- `data-accent` (`clay`, plus runtime `custom` overlay) controls emphasis color.
- The three axes stay independent: a visual polish change must not collapse flavor into theme or accent. Flavor blocks must not set `--color-accent-primary` / `--color-border-accent`.

---

## Scenario: Tailwind v4 two-layer token structure (`@theme` / `@theme inline`)

### 1. Scope / Trigger

- Trigger: changing `ccr-ui/src/styles/tokens.css`, the `@theme` / `@theme inline` blocks in `ccr-ui/src/styles/core.css`, or any token name.
- Established by `08-22-design-system` batch 1–2. The original freeze was 448 **definition points** in `tokens.css` (426 unique names across `src/styles/**`). Governance task `08-25-design-token-consolidation` registered **+6** unique names (448 + 6 definition-point narrative; unique-name union 426 → 432): `--color-success-tint`, `--color-warning-tint`, `--color-danger-tint`, `--color-info-tint`, `--color-platform-opencode`, `--color-platform-opencode-rgb`. Proof: that task's `research/token-names-before.txt` / `token-names-after.txt`.
- Governance task `08-26-profile-registry-tokens` registered **+20** unique names (unique-name union 432 → 452). The 20 names are layer-1 only (`tokens.css` `:root` and `[data-theme='dark']`; not `@theme` / `@theme inline`, no bridge): six platforms × `--color-platform-{key}-surface` / `-border` / `-text` (18) plus `--color-platform-antigravity` and `--color-platform-antigravity-rgb`. Keys: `claude`, `codex`, `grok`, `gemini`, `opencode`, `antigravity`. This child task is the dedicated token-governance registration for these names. Further additions still require a dedicated token-governance task.

### 2. Signatures

- Layer 1 (switchable semantic variables): plain CSS custom properties under `:root`, `[data-theme='dark']`, `[data-flavor='clay']`, `[data-accent='clay']` blocks in `tokens.css`.
- Layer 2 (Tailwind namespace mapping): `@theme inline` in `core.css`, values are `var(<layer-1 name>)` references (e.g. `--color-bg-surface: rgb(var(--color-bg-surface-rgb))`) so utilities inline a `var()` reference, not a literal.
- Constant tokens (all-theme-same: spacing, radius, font sizes, durations, z-index) live in `@theme` (non-inline).
- Classification record: `token-classification.md` (448 entries, three classes).

### 3. Contracts

- Switchable values must never be written as literals inside `@theme inline` — that inlines a dead value into utilities and breaks runtime theme switching.
- Switchable values must not be written into non-inline `@theme` either; theme switching would then need to override Tailwind's own `:root` output and the cascade becomes unordered.
- Utilities must resolve through layer-1 variables at runtime (`theme-switch.smoke.test.tsx` asserts both the utility-rule shape and per-theme computed values).
- Hardcoded style values are mapped per `hardcode-mapping.md` (px → `--space-*`/`--radius-*`, colors → `rgb(var(--color-*-rgb) / A%)`); anything unmappable is registered in `hardcode-exemptions.md` (count parity is the gate: remaining literals == registered exemptions).
- Single-point effect: changing one layer-1 variable must change every consuming domain at once (`token-single-point.smoke.test.tsx` asserts 3+ domains react to a `--color-bg-surface-rgb` flip).

### 4. Validation & Error Matrix

- Literal written into `@theme inline` for a switchable token -> theme switch silently stops affecting that utility; `theme-switch.smoke.test.tsx` anchor-value assertions fail.
- New variable name added without a governance task -> name-set parity breaks; rerun the batch-1 name-set comparison command against `token-names-before.txt`.
- CSS-side px/hex literal without an exemption entry -> `rg -o '[0-9]+px' -g '*.css' src/styles | wc -l` exceeds the registered exemption count.

### 5. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/theme-switch.smoke.test.tsx tests/theme/token-single-point.smoke.test.tsx tests/theme/theme-domain-extension.smoke.test.tsx`

---

## Scenario: Flavor/accent value domains, extension, and runtime custom accent

### 1. Scope / Trigger

- Trigger: changing `ccr-ui/src/styles/tokens.css`, `ccr-ui/src/styles/components/home.css`, theme bootstrap behavior, or tests that guard theme/flavor/accent semantics.
- Since `08-18-ui-visual-refactor`: flavor is `neutral | clay` and accent is `clay` only. Catppuccin (`latte` / `mocha`) is not a CCR flavor. Since `08-22-design-system` batch 5 the domains are extensible: adding a member = extend the type union + the `FLAVOR_MODES`/`ACCENT_MODES` list + one layer-1 CSS block per theme. No component change is needed (asserted by `theme-domain-extension.smoke.test.tsx`: the only writers of `data-theme`/`data-flavor`/`data-accent` in `src` ts/tsx live in `themeBootstrap.ts`).

### 2. Signatures

- Theme bootstrap: `ccr-ui/src/utils/themeBootstrap.ts`
- Global theme tokens: `ccr-ui/src/styles/tokens.css`
- Home/dashboard material tokens: `ccr-ui/src/styles/components/home.css`
- Value domains: `FlavorMode = 'neutral' | 'clay'`; `ResolvedFlavor = FlavorMode`; `AccentMode = 'clay'`; `DEFAULT_FLAVOR = 'neutral'`, `DEFAULT_ACCENT = 'clay'`; `data-resolved-flavor` equals `data-flavor` (**vestigial**: `resolveFlavorMode` ignores its theme input, so the resolved attribute can never diverge from `data-flavor`; kept as a registered contract surface — removal is a separate governance decision, not part of palette work).
- Custom accent (batch 5): `applyCustomAccent({ light: '#rrggbb', dark?: '#rrggbb' })` injects a style element covering `CUSTOM_ACCENT_VARIABLE_FAMILY` (8 variables, both theme blocks) and sets `data-accent='custom'`; `clearCustomAccent(fallback)` removes it. UI wiring belongs to `08-22-shell-port` (its R6); persistence of a custom accent is not part of the `ccr-accent` enum storage. **Vestigial note (09-03)**: `applyCustomAccent`/`clearCustomAccent` currently have **no callers** — the wiring task never landed; they remain registered contract surface, and `data-accent='custom'` is erased by the next `applyAccentToDocument` call and by the `index.html` IIFE whitelist.
- Visual preference storage keys: `ccr-theme`, `ccr-flavor`, `ccr-accent`.
- Guards: `theme-bootstrap.smoke.test.ts`, `apple-glass-surface-contract.smoke.test.ts`, `theme-contrast-contract.smoke.test.ts`, `theme-domain-extension.smoke.test.tsx`.

### 3. Contracts

- Do not replace the default flavor unless the task explicitly asks for a default theme migration.
- Flavor blocks must remap existing semantic tokens instead of adding a second component language.
- Token geometry axioms (locked by `theme-contrast-contract.smoke.test.ts`):
  - Dark: elevation steps lighten monotonically (`bg-base < bg-elevated < bg-surface < bg-overlay`); light: desktop dimmed, card lightest.
  - All surface tokens and all text tokens (incl. every `--color-stage-*`) resolve to 100% opacity.
  - WCAG contrast vs `bg-surface`: text-primary ≥ 12:1, secondary ≥ 7:1, muted ≥ 4.5:1; accent vs accent-contrast ≥ 3.5:1. These thresholds hold for the built-in themes; runtime custom accents derive contrast heuristically (luminance ≥ 0.3 → ink) and are not threshold-guarded.
- Contrast combos are 4: light/dark × neutral/clay. Do not add latte/mocha combos. Do not lower the thresholds.
- Flavor-specific exceptions belong to range-limited selectors such as `[data-flavor="clay"]` or `html:root[data-flavor="clay"]`.
- If a later range-limited override must beat an earlier flavor block in the CSS cascade, use a selector with deliberately higher specificity and lock that block in smoke tests.
- Retired Catppuccin selectors and multi-accent blocks (`sage` / `sky` / `mauve`) must not return. OpenCode TUI theme string `catppuccin-mocha` is not a CCR flavor — do not delete or rewrite it as part of flavor cleanup.
- When adding a controlled font exception, narrow the test exception to the exact override block; do not skip the whole `tokens.css` file.
- Visual automation that preloads theme preferences must write `ccr-theme`, `ccr-flavor`, and `ccr-accent`, then assert the rendered `data-theme`, `data-flavor`, `data-resolved-flavor`, and `data-accent` values before trusting computed styles.

### 4. Validation & Error Matrix

- Changed theme token semantics without targeted theme smoke tests -> not accepted.
- Flavor override present in source but lower specificity than an earlier block -> rendered CSS may stay wrong even though text tests pass.
- New flavor member added without the CSS block / list entry -> `readStoredFlavor` round-trip drops the value; `theme-domain-extension.smoke.test.tsx` extension case is the template to extend.
- Custom accent hex rejected (`applyCustomAccent` returns false) but caller ignores the result -> `data-accent` keeps the previous value; callers must branch on the boolean.
- Lowering a contrast threshold in `theme-contrast-contract.smoke.test.ts` to make a palette pass -> not accepted; adjust token values instead (thresholds are the contract).

### 5. Good/Base/Bad Cases

- Good: keep clay remaps inside `[data-flavor="clay"]` / `[data-theme="dark"][data-flavor="clay"]` and assert the remaining flavor set in smoke tests.
- Good: extend the accent domain by adding the union member, the `ACCENT_MODES` entry, and the layer-1 block; the apply/read/storage paths pick it up without component edits.
- Base: leave the four contrast combinations and their thresholds unchanged when a task only restyles a view.
- Bad: preload `ccr-flavor=catppuccin` and `ccr-accent=sky`, then report visual evidence without checking that the dataset resolved to `neutral` / `clay`.
- Bad: fix a failing contrast case by editing the expected threshold constant instead of the token value.

### 6. Tests Required

- `cd ccr-ui && bun run type-check && bun run lint:ci`
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/apple-glass-surface-contract.smoke.test.ts tests/theme/theme-bootstrap.smoke.test.ts tests/theme/theme-contrast-contract.smoke.test.ts tests/theme/theme-domain-extension.smoke.test.tsx`
- For visual work, inspect the web preview or a static build in a real browser and record the route, viewport, document dataset values, and key computed tokens.

### 7. Wrong vs Correct

#### Wrong

```css
[data-flavor="clay"] {
  --color-bg-base: #17120f;
}
```

If an earlier selector with equal or higher specificity also sets `--color-bg-base`, this source text can look correct while the rendered value remains unchanged.

#### Correct

```css
html:root[data-theme="dark"][data-flavor="clay"] {
  --color-bg-base: #17120f;
}
```

Pair the override with a smoke assertion that extracts this exact block and checks the token values that must render.

---

## Scenario: Flavor/accent value migration and bootstrap sync

### 1. Scope / Trigger

- Trigger: narrowing or renaming `FlavorMode` / `AccentMode` values, changing `DEFAULT_FLAVOR` / `DEFAULT_ACCENT`, or touching the `ccr-flavor` / `ccr-accent` read path.
- The migration machinery is the contract that keeps old `localStorage` values safe across value-domain changes.

### 2. Signatures

- `ccr-ui/src/utils/themeBootstrap.ts`: `FLAVOR_MIGRATION` / `ACCENT_MIGRATION` maps, `migrateFlavorValue` / `migrateAccentValue` (map → whitelist → fallback), `migratePersistedFlavor` / `migratePersistedAccent` (read + write-back only when changed; never seed a default into empty storage), `readStoredFlavor` / `readStoredAccent` (go through migration).
- `ccr-ui/index.html` first-paint IIFE: inline duplicate of the same migration maps + whitelist (no import capability).
- Migration tables (current):
  - flavor: `paper|graphite|catppuccin|latte|frappe|macchiato|mocha → neutral`; `neutral|clay` stay; unknown → `neutral`
  - accent: `mauve|sage|sky|slate|sand|amber|rose → clay`; `clay` stays; unknown → `clay`
  - Do not restore `slate → sky`.

### 3. Contracts

- The migration map + whitelist + fallback MUST be byte-equivalent in behavior between `themeBootstrap.ts` and the `index.html` IIFE. Changing one without the other splits first-paint from runtime resolution.
- Migration happens on read, never on render; unknown values fall back to the defaults (`neutral` / `clay`).
- Write-back only fires when the stored value differs from the migrated value, and never writes when the key is absent (no default seeding).
- Setters must normalize before persisting, so `data-flavor` / `data-resolved-flavor` can only ever be `neutral|clay` and `data-accent` only `clay` (plus the runtime `custom` overlay, which is not persisted through `ccr-accent`).
- Rollback tolerance: an older app version reading new values falls back to its own defaults — acceptable, no data loss.

### 4. Validation & Error Matrix

- IIFE and runtime migration maps diverge -> first paint renders one palette and hydration swaps to another; `theme-bootstrap.smoke.test.ts` IIFE behavior tests must cover both.
- Write-back that seeds defaults into empty storage -> users who never chose a flavor get a pinned value that blocks future default changes.
- Setter persisting an un-normalized legacy value -> `data-flavor` shows a value outside the current domain and the flavor blocks stop matching.

### 5. Good/Base/Bad Cases

- Good: `readStoredFlavor()` returns `migrateFlavorValue(stored)`; `migratePersistedFlavor()` writes back only on difference.
- Good: IIFE test feeds `localStorage.setItem('ccr-flavor', 'macchiato')` + light scheme and asserts `data-flavor === 'neutral'` before any CSS loads.
- Bad: adding a third flavor value to the TS union but forgetting the IIFE whitelist -> first-paint fallback loop.
- Bad: mapping `slate → sky` after `sky` left the accent domain.

### 6. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/theme-bootstrap.smoke.test.ts`
- Migration coverage must include every map entry, the unknown-value fallback, write-back semantics, and the empty-storage no-seeding case.

---

## Scenario: Font preference override + fallback stack (`--font-*-base`)

### 1. Scope / Trigger

- Trigger: changing the font tracks in `ccr-ui/src/styles/tokens.css`, `ccr-ui/src/utils/fontPreferences.ts`, the font controls in the settings appearance view, the `index.html` boot-script font logic, or the `apple-glass-surface-contract` font guards.
- Font preference is a fourth appearance axis alongside `data-theme`/`data-flavor`/`data-accent`, applied as an inline CSS custom property, not a `data-*` attribute.

### 2. Signatures

- Font util: `ccr-ui/src/utils/fontPreferences.ts` — `sanitizeFontFamily`, `applyFontsToDocument` / `applyUiFontToDocument` / `applyCodeFontToDocument`, `readStoredUiFont` / `readStoredCodeFont` / `persist*`, `applyInitialFonts`, `UI_FONT_PRESETS` / `CODE_FONT_PRESETS`.
- Font tracks in `tokens.css`: `--font-{sans,brand,mono}-base` hold the built-in stacks; `--font-{sans,brand,mono}` default to `var(--font-*-base)`.
- First paint: `ccr-ui/index.html` boot IIFE.
- Storage keys: `ccr-font-ui`, `ccr-font-code`.
- Guards: `ccr-ui/tests/theme/font-preferences.smoke.test.ts`, `ccr-ui/tests/theme/theme-bootstrap.smoke.test.ts`, `ccr-ui/tests/theme/apple-glass-surface-contract.smoke.test.ts`.

### 3. Contracts

- The built-in fallback stack lives only in `--font-*-base`; `--font-*` defaults to `var(--font-*-base)`. Never re-inline a literal font stack into `--font-*`.
- Override = inline custom property on `document.documentElement`: `--font-sans` / `--font-brand` = `"<uiFont>", var(--font-*-base)`; `--font-mono` = `"<codeFont>", var(--font-mono-base)`. Interface font drives sans + brand; code font drives mono only. Keep the two channels independent.
- Reset (empty / system default) = `removeProperty` the inline var so it resolves back to `var(--font-*-base)`.
- User input must pass `sanitizeFontFamily` before entering any CSS var (strip quotes/braces/controls, collapse whitespace, cap 64 chars; empty = default). Sanitize runs on both persist and apply, and the boot script repeats a lightweight strip because `localStorage` can be hand-edited.
- Font preferences are localStorage-only, like theme/flavor/accent. Do not route them through the Tauri `DesktopShellPreferences`.
- First paint: the `index.html` boot IIFE must apply the same prepend before any CSS loads or the app flashes the default font (FOUC).
- Preset font-name literals live ONLY inside the marked `/* ========== 字体预设清单 ========== */` block in `fontPreferences.ts` — a controlled exception in the legacy-mono-stack guard. Do not scatter font-name literals into components, i18n, or styles.
- Preset literals are exact OS-visible family names, not aliases. Keep `Source Han Sans SC`, `Source Han Sans CN`, and `Source Han Serif SC VF` as distinct interface presets. Adding one must not replace another.
- i18n copy for the font controls: until `08-22-i18n-port` swaps the compiler, the legacy vue-i18n message-compiler metacharacter constraint (`{`, `}`, `|` forbidden in font copy) still applies to the shipped locale files; the React-side equivalent constraint (if any) is re-registered by that task.

### 4. Validation & Error Matrix

- Literal font stack written into `--font-*` -> font-track assertion fails and reset can no longer restore cleanly.
- User font applied without `sanitizeFontFamily` -> CSS-injection / broken quoted string.
- Font-name literal added outside the preset block -> legacy-mono-stack guard fails.
- New font i18n string containing `{` / `}` / `|` -> `test:i18n` critical failure (both locales, legacy compiler).
- Boot script not updated for a new font channel -> first paint flashes the default font.

### 5. Tests Required

- `cd ccr-ui && bun run type-check && bun run lint:ci`
- `cd ccr-ui && bun run test:i18n`
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/font-preferences.smoke.test.ts tests/theme/theme-bootstrap.smoke.test.ts tests/theme/apple-glass-surface-contract.smoke.test.ts`
- For visual work: pick a present font (applies), a missing font (falls back, no tofu), and system default (restores); confirm no first-paint flash.

### 6. Wrong vs Correct

#### Wrong

```js
root.style.setProperty('--font-sans', "'MyFont', 'MapleBright', 'PingFang SC', sans-serif")
```

Re-inlines the fallback stack; a later `--font-sans-base` change silently diverges and reset cannot restore cleanly.

#### Correct

```js
root.style.setProperty('--font-sans', '"MyFont", var(--font-sans-base)')
// reset back to the built-in stack:
root.style.removeProperty('--font-sans')
```

---

## Scenario: Three-tier material glass budget (`--material-glass-*`)

### 1. Scope / Trigger

- Trigger: adding/removing a `backdrop-filter` surface, or wiring a component to `.glass-floating` / `.glass-chrome` / `.glass-inline` in `ccr-ui/src/styles/utilities/utilities.css`.
- Chrome and inline tiers are opaque (solid background + `blur: none`); only the floating tier keeps real blur (`blur(12px)`, bg ≥ 88% opacity, no `saturate()`).

### 2. Signatures

- Tier tokens (defined per flavor in `tokens.css`): `--material-glass-floating-{bg,blur,border,highlight,shadow}`, `-chrome-`, `-inline-`.
- Utility classes: `ccr-ui/src/styles/utilities/utilities.css` → `.glass-floating`, `.glass-chrome`, `.glass-inline`.
- Legacy tokens `--glass-blur-*` / `--glass-bg-*` / `--liquid-glass-*` are `deprecated` (comment-tagged) and intentionally kept at their old, thin values.

### 3. Contracts

- Budget: at most 1 `backdrop-filter` element on screen at once (floating only); never nest glass inside glass; never put glass inside a scrolling content area.
- Tier assignment is fixed by role: `floating` = modal/command-palette/floating panel (≤1 on screen), `chrome` = sidebar/topbar (opaque), `inline` = sticky in-page toolbars (opaque).
- Ordinary content cards/workspaces are **not** glass: they map to `--surface-card-*` / `--surface-workspace-*` (opaque). If a component needs depth, use elevation (border + shadow), not transparency.
- Every place a tier's `background`/`blur` is set must have a matching reset inside the `prefers-reduced-transparency: reduce` block, including inside flavor-range overrides.
- Don't repoint legacy `--glass-*`/`--liquid-glass-*` tokens to the new material recipes; migrate call-sites to the tier classes/tokens explicitly, one component at a time.
- React modal base: `src/ui/dialog.tsx` / `src/ui/base-modal.tsx` consume the semantic `--surface-modal-*` (floating tier) via the `surface-modal` class — one implementation point for modal chrome (batch 4).

### 4. Validation & Error Matrix

- New `backdrop-filter` usage outside the floating tier -> breaks the ≤1 budget.
- Glass applied to a scrollable list/table row -> forbidden regardless of tier.
- `prefers-reduced-transparency` block updates shared tokens but a flavor override sets its own copy at higher specificity -> glass survives the preference for that flavor; extend the flavor-range reset assertions.
- Re-adding `saturate()` to a glass blur recipe -> forbidden.

### 5. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/apple-glass-surface-contract.smoke.test.ts tests/quality/overlay-single-implementation.smoke.test.ts`
- Manual/DevTools: emulate `prefers-reduced-transparency: reduce` and confirm all glass surfaces on the route go opaque.

---

## Scenario: Migrating a component onto a glass tier via its semantic alias, not the raw tier token

### 1. Scope / Trigger

- Trigger: moving an existing component's `backdrop-filter` rule from one glass tier to another.
- The semantic surface-contract layer in `tokens.css` (`--surface-shell-*` = chrome, `--surface-status-*` = inline, `--surface-card-*`/`--surface-workspace-*` = opaque, `--surface-modal-*` = floating) sits between components and the raw `--material-glass-*` tokens.

### 2. Signatures

- Semantic aliases: `tokens.css` → `--surface-shell-{bg,blur,border,shadow}`, `--surface-status-{bg,blur,border,shadow}`, `--surface-card-*`, `--surface-workspace-*`, `--surface-modal-*`.
- `--surface-status-*` is a shared "sticky/toolbar surface" alias with many consumers across views (buttons, inputs, list headers, floating bars) — not a token owned by any one component. During the view migration (phase 5) consumer counts only grow; treat the alias as frozen.

### 3. Contracts

- Before repointing a component to a different tier, `rg` the semantic alias across `ccr-ui/src`. If more than the one component you're editing shows up, do **not** redefine that alias's value in `tokens.css` — change the consumer's rule to reference a different, already-correct-tier alias.
- If no existing alias matches the target tier for a role, add a new semantic alias in the "Surface Contract" block of `tokens.css` rather than pointing the component at `--material-glass-<tier>-*` directly.
- Outside a dedicated palette-rebuild task, alias values are frozen.

### 4. Validation & Error Matrix

- Redefining a shared alias's value to fix one component -> silently changes every other consumer; only caught by visual regression.
- Repointing a consumer without checking the alias resolves through the intended tier -> verify via `rg "surface-shell" ccr-ui/src/styles/tokens.css`, or inspect computed `backdrop-filter` against a known-correct sibling.

### 5. Tests Required

- `rg "<alias-name>" ccr-ui/src` before touching any semantic alias's definition.
- `cd ccr-ui && bun run type-check && bun run lint:ci` + a computed-style check on the migrated component and one untouched consumer of the same alias.

---

## Scenario: `theme.css` legacy bridge names are non-exhaustive (phantom `var(..., fallback)` tokens)

### 1. Scope / Trigger

- Trigger: a consumer references a short-form variable (no `--color-`/`--stage-` prefix, e.g. `--platform-codex`, `--platform-codex-rgb`) with a literal fallback, e.g. `rgb(var(--platform-codex-rgb, 245 158 11) / 10%)`.
- `theme.css` only bridges a hand-picked subset of short names to their canonical `--color-*` definitions. It is **not** a mechanical 1:1 mirror of every token in `tokens.css`.

### 2. Signatures

- Confirmed bridged: `--platform-claude`, `--platform-codex`, `--platform-gemini` (non-`-rgb` only), `--stage-text-*`, `--stage-surface-*`, `--stage-chip-neutral-*`, `--accent-*` (incl. `--accent-primary-contrast`), `--bg-*`, `--text-*`.
- Confirmed **not** bridged: `--platform-claude-rgb`, `--platform-codex-rgb`, `--platform-gemini-rgb`. Any consumer of these three always silently uses its own literal fallback.
- Known unfixed instance: the Codex settings view still carries `var(--platform-codex-rgb, 245 158 11)` in its legacy styles (registered for the `08-22-views-codex` migration; verify the line reference there when the view lands as `.tsx`).

### 3. Contracts

- Before consuming a short-form `var(--foo, <fallback>)`, `rg "^\s*--foo:" ccr-ui/src/styles/theme.css` to confirm the bridge exists. If not, consume the canonical `--color-foo` token directly (preferred), or add the bridge line if the short form must stay public.
- A literal fallback on a `var()` reference is not automatically an intentional default — if the primary never resolves, the fallback **is** the hardcoded value, just spelled to look like a token.
- Don't assume `-rgb` siblings of a bridged name are also bridged; check each suffix independently.

### 4. Validation & Error Matrix

- `var(--platform-codex-rgb, 245 158 11)` used anywhere -> always renders `rgb(245 158 11)` regardless of theme/flavor.
- Treating a passing hex/rgba scan as proof of "no hardcoded colors" -> false negative for this pattern. Grep for `var\(--[a-z-]+-rgb,\s*\d` / `var\(--[a-z-]+,\s*#` too.

### 5. Tests Required

- `rg "var\(--[a-z-]+-rgb,\s*\d|var\(--[a-z-]+,\s*#" <file>` when auditing for hardcoded-color migration.
- Inspect computed `background-color`/`color` against the token's known value from `tokens.css`.

---

## Scenario: Reduced motion single point (`data-reduced-motion`)

### 1. Scope / Trigger

- Trigger: adding any animation/transition degradation for `prefers-reduced-motion`, or touching `ccr-ui/src/utils/reducedMotion.ts`.
- Established by `08-22-design-system` batch 7. The only reader of `matchMedia('(prefers-reduced-motion: reduce)')` in `src` ts/tsx is `reducedMotion.ts` (asserted by `reduced-motion.smoke.test.tsx`); it mirrors the preference onto the root `data-reduced-motion='true'|'false'` attribute and follows system changes. `main.tsx` applies it at startup.

### 2. Signatures

- `ccr-ui/src/utils/reducedMotion.ts` — `applyReducedMotionToDocument()` (attribute + change subscription, returns dispose), `readPrefersReducedMotion()`.
- CSS: degradation rules across `src/styles/**` hang off `[data-reduced-motion='true']` (base.css global wildcard downgrade, utilities.css card/button downgrade, home.css local tokens, profiles-page.css spinner, shell-critical.css spinner).
- Exactly one `@media (prefers-reduced-motion: reduce)` fallback remains: `shell-critical.css` (pre-JS first paint, the critical-layer spinner is the only animation visible before the attribute exists).

### 3. Contracts

- Do not add new `@media (prefers-reduced-motion)` blocks in `src/styles/**`; hang new degradation rules off the attribute selector.
- `motion` consumers use `MotionConfig reducedMotion="user"` or `readPrefersReducedMotion()`; they must not read `matchMedia` themselves (single-track judgment).
- Legacy Vue components still carrying their own `@media (prefers-reduced-motion)` blocks migrate to this convention during phase 5 view tasks.
- CSS animations and `motion` must never drive the same element+property simultaneously (see `animation-disposition.md` for the per-segment adjudication and retained/deleted class sets).

### 4. Tests Required

- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/reduced-motion.smoke.test.tsx`
- Manual/DevTools: emulate `prefers-reduced-motion: reduce` and confirm degradation + attribute flip.
