# Check ledger — evergreen-omp-context 2.2

Stable IDs across rounds. Statuses: open | fixed | wontfix | blocked.

| id | severity | path_or_scope | issue | fix_required | test_required | status | evidence |
|---|---|---|---|---|---|---|---|
| F5 | P1 | `.omp/extensions/trellis/index.ts`; `.omp/agents/trellis-{implement,check,research}.md`; `scripts/trellis/omp-context.test.ts` | OMP `buildTaskContext` skipped `design.md`/`implement.md`; agent docs lacked pull-trio fallback | yes | yes | fixed | working-tree adjacent reads + role jsonl isolation + untrusted skip; bun test 5/5 exit 0; real OMP session UNVERIFIED |
| F5a | — | — | none this round | no | no | — | same-scope scan; no new defect |

## Scanner envelope

- scanner_identity: trellis-check 2.2 (this child, last before commit)
- git_baseline: branch `dev`; `.omp/**` never in `git log --all --full-history`; `scripts/trellis/omp-context.test.ts` untracked
- required commands: `bun test scripts/trellis/omp-context.test.ts` exit 0; `git diff --check` exit 0
- real OMP client / paid session: not started (UNVERIFIED)
