# Typed IPC and command capability manifest design

## Single declaration

The current handler registry becomes a command manifest. Every command row owns
its Rust handler path plus stable command ID, domain, risk, input/output type
names, timeout policy, concurrency class, confirmation policy, authorization
class, and audit policy. The manifest macro generates:

- the Tauri handler list;
- the runtime/static `CommandDescriptor` inventory;
- machine-readable JSON used by docs and CI;
- the frontend command-name/type mapping for generated clients.

Module-level defaults reduce repetition, but every expanded command must have a
complete descriptor. Tests reject missing/default-placeholder metadata and
duplicate IDs/paths. Platform-specific commands are expanded under the same
schema.

## Risk and policy model

Risk is a closed enum (`ReadOnly`, `LocalMutation`, `SecretMutation`,
`NetworkMutation`, `ProcessExecution`, `Destructive`). Confirmation,
authorization, concurrency, timeout, and audit fields are typed values, not
free-form strings. The runtime uses the descriptor when a domain is migrated;
metadata is not documentation-only.

Audit policies define permitted field names and redaction classes. Raw DTOs,
secrets, environments, and payload bodies cannot be emitted by a generic
debug formatter.

## Runtime completion ownership

All registry commands use a local attribute macro that wraps the real async
command body in the registry runtime policy. Module and singleton permits are
therefore held until the real future completes. Queue admission is bounded by
the descriptor deadline; a hard execution deadline is used only for explicitly
cooperative work. Commands that can detach blocking work use completion-aware
waiting, while process/install commands delegate cancellation and cleanup to
their business-owned gateway or attempt lifecycle.

The handler validates confirmation before dispatch. User-gesture confirmation
uses the repository's action-scoped transport proof; install execution and SSH
fingerprint acceptance use backend-issued opaque handles. A responder timeout
race is explicitly rejected because it cannot observe or cancel the handler
future and would release policy state before side effects finish.

## Typed DTO and client boundary

Input and output DTOs derive serde and ts-rs under domain-specific generated
directories. Generated TypeScript clients expose one function per command and
are the only direct `invoke` owner for migrated domains. `i64/u64` and optional
input fields follow the existing ts-rs wire rules.

Migration order follows stabilized backend contracts:

1. install and process;
2. sync;
3. SSH;
4. auth/provider;
5. configuration writes;
6. remaining read-only commands.

Within a migrated domain, handwritten DTO mirrors and `serde_json::Value`
inputs/outputs are rejected by a source/inventory check. Compatibility shims
may translate old frontend call sites to generated clients during one domain
migration, but may not add a generic untyped escape hatch.

## Generated inventory and drift

The inventory reports total commands, platform expansion, risk coverage,
typed-domain coverage, and unresolved legacy commands. CI regenerates bindings,
clients, and docs, then fails on a diff. Counts in specs/docs are generated from
the inventory rather than frozen by hand.

## Compatibility and rollback

Tauri command names remain stable while clients migrate. A domain can revert
to its prior generated-client version, but manifest metadata and the rule that
new/changed commands are typed remain mandatory. Backend contract changes in
the install/sync/SSH children land before their domain generation to avoid two
incompatible binding churns.
