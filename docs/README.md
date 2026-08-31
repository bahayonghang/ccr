# CCR Documentation Site

`docs/` is the bilingual VitePress site for CCR users and operators. Chinese pages live at the site root; active English mirrors live under `en/`.

UI contributor architecture, design-system rules, and historical plans belong in `ccr-ui/docs/README.md`, not in the published product guide.

## Commands

Run these commands from `docs/`:

```powershell
bun install --frozen-lockfile
bun run dev
bun run audit
bun run build
bun run preview
```

Equivalent local recipes are available through `just dev`, `just audit`, `just build`, and `just verify`. Use `just security-audit` for the dependency vulnerability check.

`docs/bun.lock` is the only maintained docs dependency lockfile; do not generate or maintain `docs/package-lock.json`. `docs/package.json#packageManager` must mirror the canonical `ccr-ui/package.json#packageManager` Bun pin.

From the repository root, `just docs` performs a frozen Bun install and builds VitePress. `just docs-check` additionally runs the product documentation audit and the `ccr-ui` engineering documentation audit.

## Published Structure

```text
docs/
|-- index.md                 # Chinese home
|-- guide/                   # task-oriented Chinese guides
|-- reference/               # Chinese commands, platforms, and internals
|-- examples/                # Chinese examples and troubleshooting
|-- en/                      # English mirror of active pages
|-- .vitepress/config.mjs    # locale navigation and sidebars
|-- scripts/audit-docs.mjs   # source and documentation consistency checks
|-- public/                  # stable public assets
|-- package.json
`-- bun.lock
```

`AGENTS.md`, `reports/`, and this README are repository-maintenance material rather than localized product pages. Historical changelog pages remain published but are excluded from current-behavior assertions.

## Editing Contract

1. Verify product claims against `crates/`, `ccr-ui/`, current command definitions, or tests.
2. Update each active Chinese/English page pair together.
3. Update `.vitepress/config.mjs` when navigable pages are added, moved, or removed.
4. Keep command examples aligned with actual Clap command paths and package/just scripts.
5. Run `bun run audit && bun run build` before handing off changes.

The audit checks locale parity, navigation and Markdown links, stable command-page coverage, workspace crate coverage, removed routes, placeholder translations, and selected source-derived facts.
