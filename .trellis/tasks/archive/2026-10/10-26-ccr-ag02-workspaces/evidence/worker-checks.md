# AG02 worker verification receipts (summary, not raw logs)

## Product change boundary

Only presentation CSS/class strings and their now-unused presentation maps/helpers were changed. Form register keys, callbacks, request builders, API imports (except an orphaned presentation helper import), draft/mount keys, CAS, confirmation handlers, source content and backend code remain unchanged.

- Solid config rows/filter controls replace legacy glass; profile/editor modals consume shared solid semantic roles.
- Controls use dedicated accent/danger contrast ink, solid focus indicators and strong input borders. Disabled opacity/behavior is retained.
- MCP protocol/instance/kv rows and scroll action become opaque. SSH fields gain explicit solid fills.
- Scoped config/provider/hooks/platform settings/raw-source fields use existing strong-border tokens; no new token system.
- Populated ConfigCard/ConfigBadges and summary counts replace hardcoded pale category text/gradients with semantic ink/status. Filtering, grouping/count computation and actions unchanged. `providerKind` was a presentation-only helper whose only consumer was removed; `rg` confirmed no remaining consumers/tests before removing it.

## Checks

Commands ran from `ccr-ui` with process-only PATH prepended `C:/Users/lyh/scoop/apps/bun/1.4.4`; no installation.

| Command | Exit | Result |
|---|---|---|
| `bun run test:smoke -- tests/ui tests/profiles tests/configs tests/mcp tests/sync` plus selected settings/auth guards | 0 | 50 files / 287 tests; first isolated change set |
| `bun run type-check` | 0 | TypeScript pass |
| `bun run lint` | 1 | ESLint 0 errors/2 pre-existing warnings; stylelint failed AG01 token comment spacing191/356, reported to owner and corrected there |
| `bun run test:smoke -- tests/ui tests/profiles tests/configs tests/mcp tests/sync tests/platforms` | 0 | 68 files / 336 tests, default parallelism; after shared/direct field updates |
| `bun run test:smoke -- tests/configs/configs-actions.smoke.test.tsx tests/configs/configs-view.smoke.test.tsx tests/configs/configs-form-drafts.smoke.test.ts tests/configs/edit-config-draft.smoke.test.tsx` | 0 | 4 files / 15 tests; after populated appearance fixes |
| `git diff --check` scoped to late config changes | 0 | Clean |

Full final lint/build/coverage belongs to AG04 review. No thresholds or parallelism changed.

## Browser evidence

Initial successful matrix: `web/2026-10-26T12-06-41.039Z/` contains44PNG, `results.json` and `contrast-summary.json`.

-24 real page screenshots: configs/converter/mcp-manager/ssh/skills/Claude Profiles ×light/dark×neutral/clay. Browser-unavailable/empty states are honest, no native IPC fixture.
-4 isolated real-component control sets (Button, Checkbox, Select, profile fields);8PNG for focus and select-open states.
-12 actual global ConfirmModal samples (danger/warning/info ×four combinations),12PNG. All dialogs/selects are opaque and have backdrop-filter:none. Including app-sheet pseudo-element, visible blur<=1.
-Initial matrix validates modal focus entry, Tab inside, and Escape cancelling false; it did not validate focus restoration.
-Computed filled-button ink min5.199:1; sampled normal/invalid profile input boundary min3.162:1; select border min3.645:1. Disabled contrast is not presented as WCAG text compliance.

A later supplemental programmatic focus-return assertion failed. That attempt is retained at `web/2026-10-26T12-08-19.055Z/`; it must not be reported as a passed focus-restore check. Actual production Switch→Cancel reproduction and comparison to unchanged BaseModal are separate from that fixture observation. No modal behavior fix was made by AG02.

The first harness attempt at `web/2026-10-26T12-06-21.327Z/` stopped because Vite optimized CommonJS modules expose ReactDOM through default; corrected in the task harness only. Its evidence remains preserved.

## Remaining acceptance

All-route matrix, viewport/DPI/fallbacks, production/native validation and full final gate remain under AG04. Browser fixtures do not prove native behavior, real writes, credentials or real IPC. No install, commit, push or native configuration mutation was performed.

## Final-source supplement

- Final real control/route/dialog matrix: `web/2026-10-26T16-52-10.105Z/`,44PNG and results.json plus contrast-summary.json. Uses final reviewer-calibrated text/overlay palette and solidfocusrings. Hover/pressed ink sampled separately; keyboardTab6steps remains inside everydialog and Escape cancels. Programmatic focus-return observation is false and explicitly not reported as PASS.
- Final scrolled cards: `cards/2026-10-26T16-52-06.009Z/`,4PNG and computed JSON. Existing ConfigFilters label squeeze was demonstrated before fix at `cards/2026-10-26T16-49-05.364Z/`:69.94px wide×114.81px high, normalwrap. The approved flex-wrap/nowrap display fix yields147.88px×40.56px and two readable rows in every combination. Narrow4files/15tests and targeted ESLint/stylelint passed after this change.
- Actual production Switch→Escape reproduction: `populated/2026-10-26T16-46-23.032Z/`,4combinations/8PNG. Actual ConfigCard onSwitch opens global confirmation from memory-only QueryClient display rows; after Escape and2seconds, activeElement is BODY in all4combos. BaseModal diff againstHEAD is empty. Parent/reviewer classify the focus-return gap as pre-existing and retain it without speculative behavior changes in a visual task. This is a remaining acceptance limitation, not a passed check.
- The real Switch dialog exposes the existing missing title key `[configs.switchConfig]`; no locale/key changes were made in this visual scope.
- Final card screenshot inspection also flagged the adjacent refresh action partly clipped at1200px; reported to parent rather than declaring all narrow-layout actions accepted.

- Final reviewer solidfocusring proof: `focus/2026-10-26T16-55-14.532Z/`,12PNG/12computed samples (Checkbox/Select/Combobox ×four combinations). Harness waits until boxShadow contains the exact opaque accent RGB and `:focus-visible` is true, rather than sampling the transparent transition start. The earlier un-settled capture at `focus/2026-10-26T16-54-39.127Z/` is retained as a harness-race receipt and is not used for ring acceptance.
