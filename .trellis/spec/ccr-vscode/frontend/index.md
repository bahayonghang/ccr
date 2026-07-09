# ccr-vscode Frontend Development Guidelines

> Extension surface contracts for the CCR VS Code extension.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Extension Surface Contracts](./extension-surface-contracts.md) | Manifest, activation, platform exposure, and tree/status boundary rules | Complete |

## Pre-Development Checklist

- Read [Extension Surface Contracts](./extension-surface-contracts.md) before editing `ccr-vscode/package.json`, `src/extension.ts`, or tree/status presentation helpers.
- Keep `ccr-vscode` lazy-activated on contributed commands and views; do not reintroduce eager startup activation without a version-specific reason.
- Keep read-only platforms exposed as browse surfaces only; writable actions remain limited to Claude and Codex.

## Quality Check

- Run `cd ccr-vscode && npm run lint`
- Run `cd ccr-vscode && npm test`
- For manifest changes, verify the contributed command list and activation behavior against the current VS Code implicit activation rules.
