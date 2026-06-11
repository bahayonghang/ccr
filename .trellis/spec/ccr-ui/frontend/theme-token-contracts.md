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
