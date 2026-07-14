# Implementation Plan: UI 工程文档重构

## Ordered Checklist

1. Move the four point-in-time files into `ccr-ui/docs/archive/2026/`.
2. Add archive status metadata to Markdown plans and build `archive/README.md` with implementation evidence.
3. Add `ccr-ui/docs/README.md` and the published-versus-engineering ownership contract.
4. Add `architecture/overview.md` from current router, API facade, Vue/Tauri boundaries, and code map.
5. Rewrite `design-system/page-templates-and-surfaces.md` against current components, tokens, accessibility rules, and available commands.
6. Add `development/verification.md` with focused and full verification ladders.
7. Add `ccr-ui/scripts/audit-docs.mjs`, `docs:audit`, and wire it into `check:all`.
8. Run focused audit and UI checks; fix until green.

## Validation

```powershell
cd ccr-ui
bun run docs:audit
bun run type-check
bun run lint
bun run test:smoke
bun run build
```

Also run from the repository root:

```powershell
git diff --check
```

The final full-scope check may use `just ui-check` if the documentation-only focused ladder is green and no unrelated gate blocker appears.

## Risk And Rollback

- File moves are the primary review risk; verify `git diff --summary` and archive index completeness before editing content.
- Keep HTML bodies byte-stable apart from path moves.
- Do not stage or modify route snapshot artifacts, `dist`, caches, dependencies, `AGENTS.md`, or `ccr-vscode/package-lock.json`.
