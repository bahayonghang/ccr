# Install opaque handle implementation plan

## Ordered work

- [ ] Define `PlanId`, `InstallPlanView`, private `CanonicalInstallPlan`, closed
  `InstallAction`, and stable consume-error codes.
- [ ] Add the TTL/single-use registry to `InstallService`; inject clock and host
  detection seams for deterministic tests.
- [ ] Make planning reconcile renderer hints with backend capabilities before
  storing the canonical plan.
- [ ] Change execution to accept only `plan_id`, consume before spawn, and have
  `install_exec` render commands exclusively from `InstallAction`.
- [ ] Export install DTO bindings with ts-rs; replace the handwritten
  `AttemptId` and execute payload in `ccr-ui/src/api/domains/install.ts`.
- [ ] Update UI call sites and event handling without changing cancellation or
  ring-buffer semantics.
- [ ] Add forged/modified, expired, reused, host-mismatch, unknown-ID, and
  renderer-string-to-command regression tests.
- [ ] Update the typed-IPC spec for the finalized install contract.

## Focused validation

```powershell
cargo test -p ccr-cli install -- --test-threads=1
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml install -- --test-threads=1
cd ccr-ui
bun run type-check
bun run test
```

Then run from the repository root:

```powershell
just tauri-bindings-check
just frontend-check-quick
just lint-strict
just test
```

## Risk and rollback checks

- Confirm registry cleanup does not leak expired entries.
- Confirm concurrent consumption has exactly one winner.
- Confirm logs and serialized errors contain no environment values.
- Rollback may disable auto-install, but must not restore the legacy full-plan
  execute command.
