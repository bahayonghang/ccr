# ccr-app Backend Spec Index

Framework-independent application orchestration consumed by desktop adapters.

## Guidelines Index

| Guide | Description |
| --- | --- |
| [Shared desktop services](./shared-desktop-services.md) | Configuration projection, bounded runtime, response identity and lifecycle |
| [Configuration policy](./configuration-services.md) | Generic/dedicated patches, source CAS, managed backup and scoped secrets |

## Pre-Development Checklist

- Read [Shared desktop services](./shared-desktop-services.md) before changing ccr-app or a native consumer.
- Read [Configuration policy](./configuration-services.md) before changing profile editors, shared patch/source/backup policy or their desktop adapters.
- Read the affected domain owner's specs before extracting orchestration; persistence, SQL and security stay with that owner.
- GPUI visual changes follow `ccr-desktop/DESIGN.md`; Tauri wire types remain in their existing adapter.

## Quality Check

- `cargo test -p ccr-app --locked -- --skip export_bindings`
- `cargo clippy -p ccr-app --locked --all-targets --all-features -- -D warnings`
- Run the affected domain and consumer checks, including Tauri bindings when its adapter changes.
- Keep default test parallelism. Native screenshots and OS interaction are separate evidence from headless tests.
