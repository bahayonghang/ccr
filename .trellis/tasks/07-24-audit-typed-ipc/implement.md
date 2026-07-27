# Typed IPC and capability manifest implementation plan

## Ordered work

- [x] Define typed risk, authorization, confirmation, concurrency, timeout, and
  audit policy enums plus the expanded `CommandDescriptor`.
- [x] Extend the registry macro so all base/Windows commands expand to complete
  descriptors and the Tauri handler list from one declaration.
- [x] Generate deterministic JSON/docs/counts and add completeness, uniqueness,
  platform-count, placeholder, and redaction-policy tests for 315/323 commands.
- [x] Add the generated TypeScript client template and a CI guard that bans
  direct `invoke` outside the API facade/generated runtime.
- [x] After each owning child stabilizes its API, migrate install/process, sync,
  SSH, auth/provider, and config-write DTOs and clients in that order.
- [x] Remove handwritten mirrors and command-boundary `Value` from each migrated domain; add a
  zero-count guard per typed domain.
- [x] Migrate remaining read-only commands until measured typed coverage is at
  least 80 percent; record every remaining command and owner in inventory.
- [ ] Wire metadata into runtime timeout/confirmation enforcement rather than
  leaving those descriptor fields passive. Authorization is enforced by
  registry-generated Tauri AppManifest ACLs, and metadata/redaction-class audit
  logging runs before dispatch without reading payloads.
- [x] Update handler-registry, typed-binding, API-facade, and desktop-command
  specs from generated evidence.

## Focused validation

```powershell
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml handler_registry -- --test-threads=1
just tauri-bindings
just tauri-bindings-check
just frontend-check
just lint-strict
just test
```

The child acceptance report must include generated totals for command metadata,
typed commands, and typed-domain `Value`/handwritten mirror counts.

## Rollback checks

- Command names and serde wire fields remain compatible unless the owning PRD
  explicitly changed them.
- A rollback cannot restore direct untyped `invoke` or remove metadata from a
  command.
- Generated audit code must prove secret-bearing DTO fields are excluded or
  redacted.
