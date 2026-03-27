# Migration Guide

This page maps historical wording to the current repository layout and entrypoints. It does not treat removed commands as part of the current supported surface.

## Entrypoint Mapping

| Historical wording | Current path | Notes |
|---|---|---|
| `ccr web` | `ccr ui` | the graphical entrypoint is now `ccr ui` plus the `ccr-ui` project |
| built-in Web API | no direct replacement | the current UI reuses crates through `ccr-ui/src-tauri` instead of exposing an internal HTTP server |
| `ccr tui` | run `ccr` directly | in the default build, no subcommand enters TUI mode |
| `ccr migrate` | initialize the current layout, then import or recreate profiles | the current command surface no longer documents a dedicated migration subcommand |

## Path Mapping

| Legacy path | Current path | Notes |
|---|---|---|
| `src/` | `crates/ccr/src/` | main CLI crate |
| `tests/` | `crates/ccr/tests/` | CLI integration tests |
| `ccr-db/` | `crates/ccr-db/` | database and desktop-side services |
| `ccr-types/` | `crates/ccr-types/` | shared types |
| `ccr-ui/backend/` | `ccr-ui/src-tauri/` | Tauri desktop shell |
| `ccr-ui/frontend/` | `ccr-ui/src/` | Vue frontend source |

## Recommended Migration Sequence

1. Re-establish the current workspace layout:

```bash
ccr init
ccr platform list
```

2. If you need a graphical entrypoint, use:

```bash
ccr ui
```

3. If you already have an importable profile bundle, use:

```bash
ccr import <file> --merge --backup
```

4. If you only have older config files, treat them as reference input and recreate profiles on the current layout platform by platform.

## What Stays vs What Goes

- keep: historical context about older layouts and entrypoints
- remove: documentation that treats the old Web API, `ccr web`, or `ccr migrate` as current commands

## See Also

- [Architecture](/en/reference/architecture)
- [Crate Map](/en/reference/internals/crate-map)
- [Command Reference](/en/reference/commands/)
- [Entrypoints](/en/guide/entrypoints)
