# Implementation Plan: 产品文档站重构

## Ordered Checklist

1. Inventory every active Chinese/English page against current crate, CLI, platform, and UI source anchors.
2. Rewrite the core overview, quick start, configuration, workflows, entrypoint, UI overview, and UI module pages in both languages.
3. Rebuild architecture, crate map, and runtime flow pages around all 13 crates and the Vue/Tauri boundary.
4. Audit every command reference against the current command tree; add Chinese/English `claude` pages and remove stale roadmap/generic content.
5. Audit platform, examples, migration, and troubleshooting pages for current behavior and link consistency.
6. Regroup bilingual VitePress navigation without breaking existing URLs.
7. Remove the empty/stale maintenance TODO and refresh `docs/README.md`.
8. Strengthen `docs/scripts/audit-docs.mjs` for publication ownership, command coverage, locale parity, and internal links.
9. Align `docs/justfile` and root `just docs-check` with Bun, product audit, and UI docs audit.
10. Run the full documentation gates and fix until green.

## Validation

```powershell
cd docs
bun install --frozen-lockfile
bun run audit
bun run build
```

From the repository root:

```powershell
just docs-check
git diff --check
```

Run `just frontend-check` only if root tooling or shared frontend gate behavior changes beyond `docs-check`.

## Review Anchors

- Compare generated command requirements to `Commands` and explicit Clap names.
- Confirm all active page paths have Chinese and English mirrors.
- Confirm navigation links resolve and no archived UI plan is presented as current behavior.
- Confirm `docs/.vitepress/dist`, caches, dependencies, and unrelated dirty files are absent from the diff.

## Risk And Rollback

- Large bilingual edits can drift semantically; update and review each Chinese/English pair together.
- Avoid mechanical full-tree rewrites that change untouched formatting.
- Keep command-audit logic small and covered by deterministic fixture-like assertions in the script itself.
