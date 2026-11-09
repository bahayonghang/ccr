# Documentation Boundary Audit

## Scope

This audit treats the current implementation as the source of truth for two documentation surfaces:

- `docs/`: published bilingual product documentation built with VitePress.
- `ccr-ui/docs/`: repository-local UI engineering knowledge, plans, and review artifacts.

## Confirmed Boundaries

- `docs/AGENTS.md` requires Chinese/English parity for active pages, navigation updates for moved pages, and `bun run build && bun run audit` verification.
- `ccr-ui/AGENTS.md` defines the current UI audience and design direction as power-user, calm, precise, editorial, with light/dark and reduced-motion support.
- The root workspace currently contains 13 crates (`Cargo.toml:2-16`), while the UI is a separate Vue/Tauri application.
- Existing unrelated changes in `AGENTS.md` and `ccr-vscode/package-lock.json` are outside this task and must remain untouched.

## Cross-Surface Problem

The published site already contains user-facing UI pages, while `ccr-ui/docs/` contains a mixture of current engineering rules and point-in-time design work. The refactor needs an explicit ownership contract:

- Product behavior, installation, workflows, stable commands, and user-visible UI modules belong in `docs/`.
- UI architecture, design-system contracts, contributor verification, and historical decision material belong in `ccr-ui/docs/`.
- Historical plans must not be presented as current implementation contracts.

## Integration Checks

- Published UI descriptions must be checked against current routes and Tauri capabilities.
- Engineering docs must link to current components, scripts, and checks.
- Terms and module names must agree across both surfaces.
- Final integration must run the product docs build/audit and a repository diff/link review without modifying product behavior.
