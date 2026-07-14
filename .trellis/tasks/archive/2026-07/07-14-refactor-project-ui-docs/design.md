# Design: CCR 项目与 UI 文档重构

## Architecture

The work is split into two independently verifiable documentation surfaces:

| Surface | Audience | Source of truth | Output |
|---|---|---|---|
| `docs/` | CCR users and operators | `crates/`, CLI definitions/help, `ccr-ui` routes and stable behavior | Bilingual VitePress site |
| `ccr-ui/docs/` | UI contributors and maintainers | `ccr-ui/code_map.md`, Vue/Tauri source, tests, scripts, Trellis frontend specs | Versioned engineering Markdown plus archived decision records |

The parent task owns terminology, cross-links, verification integration, and final consistency review. It does not own a third documentation implementation surface.

## Dependency Order

1. Finish `refactor-ui-docs` so UI engineering ownership and archive status are stable.
2. Finish `refactor-product-docs`, consuming current code facts and the finalized documentation boundary.
3. Run parent integration checks across both surfaces.

## Shared Contracts

- Product claims must point to current commands, routes, config, or tests.
- Internal plans must carry lifecycle status and must not be linked as current user guidance.
- Existing public VitePress URLs remain stable unless a path is demonstrably internal and unused.
- Chinese and English active product pages remain semantic mirrors.
- The root docs gate uses Bun consistently and runs both audit surfaces.

## Compatibility And Rollback

- Root page paths stay stable, minimizing external-link breakage.
- UI historical files are moved, not deleted; Git history plus the archive index preserves provenance.
- Each child is independently revertible. The root gate integration lands only after both audit commands exist.
