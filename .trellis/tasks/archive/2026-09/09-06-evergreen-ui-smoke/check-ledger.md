# Check 2.2 ledger — evergreen-ui-smoke

Date: 2026-09-06. Scope: last 2.2 full review of F1 fixture fix. Product diff: `ccr-ui/tests/shell/route-view-mount.smoke.test.tsx` (+66). Tool: Cursor Grok 4.6 trellis-check. Did not edit the whitelist product file.

## Findings

| id | severity | path_or_scope | issue | fix_required | test_required | status | evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| F1 | P1 | `ccr-ui/tests/shell/route-view-mount.smoke.test.tsx` | Route smoke stubbed Agent Sessions start-refresh as `[]`, so `/agent-sessions` ErrorBoundary read `snapshot.status` on undefined. | Typed start/status/list/detail/provider fixtures matching generated DTOs; keep bilingual route-failure assertions; do not change AgentSessionsView or IPC DTO. | Focused `bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx` must exit 0 with 4 passing tests. | fixed | Original scanner previously exit=1 (`Cannot read properties of undefined (reading 'status')` at AgentSessionsView.tsx:216). After fixture, same command exit=0, Tests 4 passed. `assertNoBoundary` / `页面渲染失败` still present. `git diff --stat` is +66 on the whitelist file only. |

No F1a/F1b opened. `AgentSessionPageDto` fixture `{ items: [] }` is assignable; `next_cursor` is optional and correctly omitted for an empty page.

## Commands (cwd=`D:\Documents\Code\Github\ccr\ccr-ui`)

| command | exit |
| --- | --- |
| `bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx` | 0 (1 file, 4 tests, 27.43s) |
| `bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx tests/agent-sessions/agent-sessions.smoke.test.tsx` | 0 (2 files, 10 tests, 26.00s) |
| `bun run type-check` | 0 |
| `bun run lint:ci` | 0 |
| `bun run test` | 0 (i18n 24/24; smoke 151 files / 722 tests, 85.50s) |

## AC evidence

- AC1: focused 4/4 pass; zh-CN/en-US route scans no longer hit ErrorBoundary; full smoke 722/722 (was 720 pass / 2 fail).
- AC2: generated DTO types; start refresh has `job_id`+`snapshot`; list typed as `AgentSessionPageDto` with `items`; product API unchanged.
- AC3: type-check, lint:ci, agent-sessions smoke, full `bun run test` pass; failure assertions and coverage thresholds untouched.
- AC4: product diff whitelist-only; AgentSessionsView / IPC DTO / coverage / other task dirs not edited by this check.

## UNVERIFIED

- Hosted Frontend CI (repo pin Bun 1.4.0; local Bun 1.4.2).
- Native Tauri / Windows WebView `/agent-sessions` with a real local dataset.
- `just ui-check` / `just ci` aggregates, bindings/inventory, Linux/macOS.
- Visual matrix (light/dark, zh/en, 1440×900 and sub-900px) and five-harness live clients.
