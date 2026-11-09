# Install opaque handle technical design

## Boundary

The renderer may request a plan and later present only its `plan_id`. It must
never send an executable, arguments, or environment variables back to the
backend. `InstallService` owns the canonical plan from creation through
single-use consumption.

## Types and ownership

- `InstallAction` is a closed backend enum for the supported llmusage install
  mechanisms. Each variant renders its command, arguments, and allowed
  environment inside `install_exec`; arbitrary executable strings are not part
  of the public DTO.
- `InstallPlanView` is the renderer-facing preview. It contains `plan_id`,
  package-manager/action labels, expected effects, host OS, and expiry, but no
  executable capability.
- `CanonicalInstallPlan` is private to the service. It contains the action,
  normalized host capabilities, creation/expiry instants, and an audit-safe
  identifier.
- `InstallPlanRegistry` is held by `InstallService` under the existing service
  synchronization boundary. Entries are keyed by `PlanId`, expire after 120
  seconds, and are removed atomically before execution.

## Data flow

1. `llmusage_install_plan` validates the renderer's detection/capability hint
   against backend detection, constructs a canonical action, stores it, and
   returns an `InstallPlanView`.
2. `llmusage_install_execute(plan_id)` consumes the entry. Unknown, expired,
   reused, or host-mismatched IDs return a stable coded command error.
3. The consumed `InstallAction` is converted to a process specification inside
   the executor. Only that specification reaches `Command::new`.
4. Install audit fields contain action, plan ID, attempt ID, outcome, and
   duration. Environment values and command output are never added to audit
   metadata.

## IPC and generated types

The install command input/output DTOs derive serde and `ts_rs::TS` in the Tauri
crate's established generated-binding surface. `PlanId` and `AttemptId` are
transparent UUID strings on both sides. The handwritten TypeScript
`AttemptId` object and full-plan execute argument are removed.

## Compatibility and rollback

- Existing plan presentation remains available through `InstallPlanView` so
  the UI can show the selected manager and effects.
- There is no compatibility shim accepting legacy full plans; retaining it
  would preserve the vulnerability.
- If automatic installation must be disabled during rollback, planning returns
  the existing manual-catalog outcome. The old executable-bearing IPC is not
  restored.

## Verification strategy

Unit tests use an injectable clock and host snapshot. Executor tests record the
closed `InstallAction` selected rather than launching package managers. A
compile/search assertion ensures renderer-owned strings cannot reach
`Command::new`.
