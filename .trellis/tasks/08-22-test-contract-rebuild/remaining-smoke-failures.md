# Remaining smoke failures (not contract-rewrite regressions)

Latest full `bun run test:smoke` after batches 2–7: **16 failed**, all
`TypeError: Cannot read properties of undefined (reading 'invoke')` inside
`@tauri-apps/api/core` (the real module, not the per-file `vi.mock`).

Affected files:

- `api-facade-coverage.smoke.test.ts` (execution / typed-client drive cases)
- `ssh-hardening.smoke.test.ts`
- `sync-encryption-contract.smoke.test.ts`
- `typed-claude-client.smoke.test.ts`
- `typed-codex-system-prompts.smoke.test.ts`
- `checkin-records-api.smoke.test.ts`
- `command-runtime-policy.smoke.test.ts`
- `config-domain-api.smoke.test.ts`
- `install-opaque-handle.smoke.test.ts`

These files mock `@tauri-apps/api/core`. The mock is not replacing
`invokeRuntime.ts`'s `tauriInvoke` import. The same files passed in an earlier
full run in this session; isolation runs of `ssh-hardening` also fail, so this
is mock wiring vs. Vitest 4, not an assertion rewrite.

This task did not change those test files (except `api-facade-coverage` still
imports `helpers/apiInvokeScan.ts`, whose suffix set dropped `.vue` and kept
`.tsx`).

`just frontend-check-quick` typecheck passed. `lint:ci` failed on
`scripts/_probe-i18n.mjs` and `src/features/codex/auth/SaveCodexSessionModal.tsx`
(outside this task's write set).
