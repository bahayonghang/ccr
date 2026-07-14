# Design: UI 工程文档重构

## Target Structure

```text
ccr-ui/docs/
|-- README.md
|-- architecture/
|   `-- overview.md
|-- design-system/
|   `-- page-templates-and-surfaces.md
|-- development/
|   `-- verification.md
`-- archive/
    |-- README.md
    `-- 2026/
        |-- claude-profiles-dashboard-optimization.md
        |-- sync-page-redesign-design.html
        |-- sync-page-redesign-implementation.md
        `-- vibedeck-vs-ccr-ui-analysis.html
```

## Document Contracts

### Root README

- Defines the contributor audience and the boundary with published `../docs/` content.
- Explains durable versus archived material.
- Links every current engineering document and the archive index.

### Architecture overview

- Documents Vue bootstrap/router/views/components, state/API/composables, shared contracts, Tauri commands/state/services, and test/tooling boundaries.
- Uses current code paths rather than exhaustive symbol lists that will immediately drift.
- Includes the current module families and data flow from view to typed API/Tauri command.

### Design-system contract

- Retains valid shell, shared surface, async state, accessibility, and route-template rules.
- Replaces stale snapshot commands with actual web-preview and verification commands.
- Aligns wording with the calm, precise, editorial direction in `ccr-ui/AGENTS.md` and current token contracts.

### Development verification

- Routes changes to the narrowest relevant type, lint, smoke, i18n, build, Tauri, and browser checks.
- Explains web-mode limitations for Tauri invokes.
- Does not claim an automated visual snapshot command that is absent from `package.json`.

### Archive

- `archive/README.md` lists every archived file, date, status, implementation evidence, and whether it remains actionable.
- Markdown plans receive a status block at the top.
- HTML artifacts are moved without rewriting their historical body; the archive index is their lifecycle authority.

## Audit Tooling

Add `ccr-ui/scripts/audit-docs.mjs` and a `docs:audit` package script. The audit will:

- require the durable structure and archive index;
- fail when point-in-time plans/artifacts remain in active top-level directories;
- ensure every archive file is named in the archive index;
- validate local Markdown links;
- reject known stale references such as `test:playwright:snapshots`.

Add `docs:audit` to `check:all` only after the standalone command passes.

## Compatibility And Rollback

- No UI runtime source or behavior changes.
- File moves preserve content and Git history.
- The audit script uses Node standard library only and adds no dependency.
- Reverting this child restores the old doc locations without affecting VitePress.
