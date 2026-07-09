# ccr-ui Appearance System Redesign

## Goal

Redesign `ccr-ui`'s appearance system toward an OxideTerm-like dark command workbench: darker, flatter, sharper, less cloudy, less rounded, and less pale. The first implementation should prove the new visual language on the Settings appearance surface while preserving existing product IA, settings workflows, theme/flavor/accent semantics, i18n keys, and Vue/Tauri architecture.

## Requirements

- Recalibrate the existing token system instead of repainting individual screens ad hoc.
- Preserve the three-layer theme model:
  - `data-theme` for light/dark/system resolution.
  - `data-flavor` and `data-resolved-flavor` for palette family.
  - `data-accent` for emphasis color.
- Keep the current settings information architecture and behavior intact.
- Reduce pale/foggy surfaces in dark and Catppuccin modes.
- Replace heavy frosted glass, large radii, pill-heavy labels, and soft glows with flatter terminal-grade surfaces.
- Reduce the default visual radius scale:
  - Cards and panels target 8px.
  - Inputs, selects, icon buttons, and swatches target 6px to 8px.
  - Pills stay only for controls where the shape communicates chip/status/toggle behavior.
- Make active states legible through border, inset surface, and restrained accent marks instead of glow.
- Preserve expert density and scanning speed.
- Keep hit targets accessible even when visual radii shrink.
- Keep semantic success/warning/danger/info colors distinct and low-chroma.
- Retain Catppuccin palette compatibility by changing semantic surface mappings rather than redefining canonical palette variables.
- Use `AppSettingsView.vue` as the first proving surface.
- Preserve existing `data-testid` selectors unless a test must be intentionally updated.

## Acceptance Criteria

- [ ] Dark Settings page no longer has large pale, foggy, or cloudy panels.
- [ ] Settings cards and controls visually align closer to the provided OxideTerm reference: darker base, sharper panels, lower radius, fewer all-caps/pill treatments.
- [ ] Theme, flavor, accent, language, shell, and diagnostics settings remain functional.
- [ ] `data-theme`, `data-flavor`, `data-resolved-flavor`, and `data-accent` remain independent and persist correctly.
- [ ] Catppuccin resolved flavors still use existing semantic token remaps rather than a second component language.
- [ ] Text and control contrast remain readable in light, dark, system, and Catppuccin resolved modes.
- [ ] No overlapping text, clipped controls, or unreadable status chips in desktop and narrow viewport checks.
- [ ] Existing settings smoke selectors remain stable or are deliberately updated with matching tests.
- [ ] Focus states remain visible without relying on blurred glow alone.
- [ ] Visual verification records route, viewport, theme/flavor/accent state, and the key before/after deltas.

## Definition of Done

- `cd ccr-ui && bun run type-check` passes.
- `cd ccr-ui && bun run lint` passes.
- `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts` passes.
- `cd ccr-ui && bun run test` passes or any failure is documented as unrelated with evidence.
- Web preview is inspected using `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort` at `http://127.0.0.1:5173/`.
- Visual inspection covers Settings in dark/system/Catppuccin modes and at least one narrow viewport.
- Implementation notes mention whether Shell alignment was included or deferred.

## Technical Approach

Use a token-first targeted overhaul:

1. Recalibrate `ccr-ui/src/styles/tokens.css` and the compatibility bridge in `theme.css`.
2. Keep `themeBootstrap.ts` semantics stable unless adding a new flavor option is explicitly necessary.
3. Update shared primitives (`Card.vue`, `Button.vue`, `Input.vue`) so app surfaces inherit flatter material and sharper radii.
4. Redesign `AppSettingsView.vue` as a denser control surface:
   - compact header band instead of large rounded hero card,
   - flatter section nav,
   - compact theme/flavor selectable tiles or rows,
   - square or softly-rounded accent swatches,
   - fewer uppercase micro-labels and pills.
5. Run targeted theme smoke tests and browser visual verification.
6. Decide after the first visual pass whether to include Shell alignment in the same task or split it.

## Implementation Plan

### Phase 1: Token Recalibration

Likely files:
- `ccr-ui/src/styles/tokens.css`
- `ccr-ui/src/styles/theme.css`
- `ccr-ui/src/utils/themeBootstrap.ts`
- `ccr-ui/tests/theme-bootstrap.smoke.test.ts`

Work:
- Add/promote an Oxide-like dark command palette inside the existing flavor model, or retune `graphite` if avoiding a new user-facing flavor is safer.
- Lower global radius tokens.
- Reduce blur, opacity, pale inner highlights, glow, and shadow intensity for normal surfaces.
- Make `--surface-card-*`, `--surface-workspace-*`, and `--surface-status-*` resolve to mostly opaque stepped panels.
- Preserve `clay` default unless the implementation explicitly documents a default migration decision.

### Phase 2: Shared Primitive Cleanup

Likely files:
- `ccr-ui/src/components/ui/Card.vue`
- `ccr-ui/src/components/ui/Button.vue`
- `ccr-ui/src/components/ui/Input.vue`

Work:
- Make card variants flatter with lower border radius and reduced blur/shadow.
- Make button shapes less pill-heavy by default.
- Replace blurred input focus glow with crisp border/ring feedback.
- Remove or replace decorative hand-rolled SVG affordances where they conflict with the existing icon system.

### Phase 3: Settings Page Redesign

Likely files:
- `ccr-ui/src/views/AppSettingsView.vue`
- `ccr-ui/src/i18n/locales/en-US.ts`
- `ccr-ui/src/i18n/locales/zh-CN.ts`
- `ccr-ui/tests/app-settings.smoke.test.ts`

Work:
- Compact the page header.
- Flatten section nav and option blocks.
- Reduce all-caps badge noise where it does not encode state.
- Keep settings behaviors and `data-testid` contracts stable.

### Phase 4: Optional Shell Alignment

Likely files:
- `ccr-ui/src/components/MainLayout.vue`
- `ccr-ui/src/config/mainLayoutShell.ts`
- `ccr-ui/src/composables/useMainLayoutShell.ts`

Work:
- Only proceed if Settings proves the language and the shell mismatch is visually disruptive.
- Flatten sidebar/topbar material and active nav treatment while preserving layout state and mobile behavior.

## Decision (ADR-lite)

**Context**: The current Settings appearance surface has a warm editorial token system, Catppuccin support, and shared primitives, but the reference screenshot asks for less white bloom, less roundness, and a more terminal-native dark control surface.

**Decision**: Use a token-first targeted redesign. Prove the direction on Settings before broad shell alignment.

**Consequences**:
- Safer than a full UI rewrite because existing theme contracts and smoke tests remain useful.
- Token changes can affect many routes, so visual verification must inspect more than just Settings before final acceptance.
- Catppuccin compatibility must be preserved through semantic mappings, not source palette rewrites.

## Out of Scope

- Full product IA redesign.
- Route, navigation label, or settings workflow changes.
- Backend/Tauri command changes.
- New design-system dependencies.
- Removing light/system theme support.
- Rewriting unrelated product pages in the first implementation pass.
- Replacing Vue, Tailwind, Iconify, Pinia, or the existing theme bootstrap architecture.

## Technical Notes

- Source planning artifact: `.plannings/2026-06-11-ccr-ui-appearance-system-redesign.md`.
- Project visual guidance: `ccr-ui/AGENTS.md`, `ccr-ui/DESIGN.md`, and `ccr-ui/PRODUCT.md`.
- Relevant Trellis spec: `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`.
- Relevant tests:
  - `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts`
  - `ccr-ui/tests/theme-bootstrap.smoke.test.ts`
  - `ccr-ui/tests/app-settings.smoke.test.ts`
- Visual preview path:
  - `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`
  - `http://127.0.0.1:5173/`
