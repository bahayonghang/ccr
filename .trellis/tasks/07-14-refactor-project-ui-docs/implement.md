# Implementation Plan: CCR 项目与 UI 文档重构

## Ordered Checklist

1. Complete and verify `07-14-refactor-ui-docs`.
2. Complete and verify `07-14-refactor-product-docs`.
3. Review terminology and module ownership across both surfaces.
4. Run `just docs-check`, `git diff --check`, and the relevant focused child gates.
5. Confirm unrelated `AGENTS.md` and `ccr-vscode/package-lock.json` changes remain untouched.

## Integration Review

- Compare published UI module names with `ccr-ui/docs/architecture/overview.md` and the router.
- Confirm public docs never point to archived plans as current behavior.
- Confirm the root gate runs product and UI documentation audits.
- Confirm no generated `.vitepress`, `dist`, cache, or dependency output is included.

## Rollback Points

- Revert the UI docs child without affecting public URLs.
- Revert the product docs child without deleting archived UI history.
- Keep root gate changes in the product child so audit integration rolls back atomically.
