# docs Agent Notes

These notes apply to everything under `docs/` and supplement the repository-level `AGENTS.md`.

## Structure And Ownership
- This subtree is the VitePress documentation site.
- `index.md`, `guide/`, and `reference/` are the primary Chinese documentation surfaces.
- `en/` mirrors active documentation pages for English readers; keep locale parity unless a file is explicitly historical or ignored by `scripts/audit-docs.mjs`.
- `.vitepress/config.mjs` owns navigation/sidebar routing.
- `scripts/audit-docs.mjs` verifies link targets, locale parity, removed pages, and command/documentation consistency.

## Build And Verification
- From `docs/`, run `bun run build` to build the VitePress site.
- Run `bun run audit` for the documentation audit script.
- Run `bun run build && bun run audit` as the default local docs verification before handing off docs changes.
- `just build`, `just verify`, and `just audit` are available inside `docs/` when the local justfile is more convenient.
- From the repository root, `just frontend-check` includes the docs gate through `docs-check`.

## Editing Rules
- Keep Chinese and English mirrors aligned for active pages.
- Update `.vitepress/config.mjs` when adding, removing, or moving navigable pages.
- Do not edit generated output under `.vitepress/dist/`, `.vitepress/cache/`, or dependency files under `node_modules/`.
- Do not add screenshots or binary generated assets unless the documentation task explicitly requires them; prefer source Markdown and stable public assets under `public/`.
- Avoid documenting commands that are not present in the root justfile, docs justfile, or package scripts.
