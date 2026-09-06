# Finding ledger

Stable IDs across rounds. Statuses: open | fixed | wontfix | blocked.

Actionable = `fix_required=yes` and `status=open`.

| id | severity | path_or_scope | issue | fix_required | test_required | status | evidence |
|---|---|---|---|---|---|---|---|
| F1 | P1 | ccr-ui/tests/shell/route-view-mount.smoke.test.tsx | Agent Sessions route smoke stubs session-named APIs as `[]`; start refresh needs `StartSessionIndexJobResponse` | yes | yes | fixed | original replay exit=1; after fixture same command exit=0 (4/4); check PASS |
| F2 | P1 | .github/workflows/vscode-ci.yml | coverage `just vscode-coverage \| tee` without pipefail can swallow failures | yes | yes | fixed | `shell: bash`; pipefail-false exit 1; pipefail-true exit 0; check PASS |
| F2a | P2 | scripts/ci/test_check_workflow_governance.py local WSL bash | concurrent local WSL `true \| tee` flaked once | no | no | wontfix | isolated 24 OK; hosted vscode-ci is ubuntu-24.04 |
| F3 | P1 | scripts/ci/ci_surface_policy.py | `.cargo` config files missing from relevant inputs | yes | yes | fixed | AC2 mapping; governance tests 24 OK; check PASS |
| F4 | P1 | AGENTS.md / CLAUDE.md / code_map.md / docs | project facts conflict; Claude shared import inert | yes | yes | fixed | React 19 + ccr-usage; `@AGENTS.md`; docs audit/build 0; check PASS |
| F4a | P2 | ccr-ui/CLAUDE.md | stale Tauri command inventory and version pins | yes | no | fixed | repair round 1; commands/mod.rs + inventory; pins match package.json |
| F5 | P1 | .omp Trellis extension + agent docs | `buildTaskContext` skips design.md/implement.md | yes | yes | fixed | bun test scripts/trellis/omp-context.test.ts exit 0 (5/5); real OMP UNVERIFIED |
| F6 | P2 | harness skill/docs routing | capability claims, reviewer vs implementer, Vue leftovers | yes | yes | fixed | harnesses.md zh/en; Grok/Kimi pull-not-ceiling; check PASS |
| F6a | P2 | .codex/skills/ccr-ui-visual-workflow/SKILL.md | duplicate Tauri Boundary / Evidence sections | yes | no | fixed | repair round 1; single section with authorization bullet |
| H1 | P2 | crates/ccr-cli doctor persist | historical Root coverage fail; `save_report` swallows IO | no | no | open | not reproduced locally; not marked fixed; coverage-rust UNVERIFIED without llvm-cov; see ci-history verification.md |
| H2 | P2 | Tauri process gateway Windows 5s | historical smoke stdin/flood fail | no | no | open | 3× `just tauri-process-smoke` exit 0; hosted windows-2025 UNVERIFIED |
| H3 | P2 | deleted Vue usage.store.smoke.test.ts | historical Frontend EnvironmentTeardownError | no | no | wontfix | vanished path; do not resurrect Vue tests |

H1/H2 remain documented, not in-scope product work for this approved set (`fix_required=no` here). Open actionable findings = 0.

## Scanner envelope

- scanner_identity: commit `77058135c09063b053d8a77702c6e6b0c3455594`
- config: `.trellis/tasks/09-06-evergreen-harness-audit/research/audit.md` + `research/harnesses.md`; flags none
- git_baseline: branch `dev` HEAD `77058135c09063b053d8a77702c6e6b0c3455594`; dirty-scope six untracked `09-06-evergreen-*` task dirs only
- original focused replay: cwd=`ccr-ui`, command=`bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx`, original exit=1; after F1 same command exit=0
