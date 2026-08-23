# Four interface notices — promotion decision (batch 7)

`design.md` §5 lists four task-directory interface notices that are **not** in
the 19-document spec set:

| Notice | Owner task |
| --- | --- |
| `shared-interfaces.md` | `08-22-shell-port` |
| `profiles-shared-interfaces.md` | `08-22-views-profiles-config` |
| `mcp-shared-interfaces.md` | `08-22-views-sync-tools` |
| generic interface notice | `08-22-views-secondary-platforms` |

**Decision: do not promote.** Contract count stays **19**.

Reasons:

- Shell shared interfaces are already enforced by `confirm-interaction-contracts.md`
  and `tests/react-shell.smoke.test.tsx` / `tests/router.smoke.test.ts`.
- Profiles shared interfaces are already in `profiles-page-contracts.md` plus
  `tests/profiles-shared-layer.smoke.test.tsx`.
- MCP shared interfaces are already in `platform-surface-contracts.md` (MCP
  panels live under `features/platform/mcp` and re-export from `features/mcp`)
  plus `tests/mcp-panels.smoke.test.tsx`.
- Generic / Base interfaces are already in `platform-surface-contracts.md`.

Promoting them would duplicate those four spec files and change the frozen 19
count. They remain task-directory notices for the owning subtasks.
