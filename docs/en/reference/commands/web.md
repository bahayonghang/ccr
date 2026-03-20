# web - Removed legacy command

`ccr web` has been removed from the current CLI.

## Replacements

- Graphical workflows: use [`ccr ui`](/en/reference/commands/ui)
- Desktop runtime: use the Tauri mode in `ccr-ui`
- Automation: continue using `ccr` commands directly

## Migration note

The old built-in HTTP service and embedded frontend no longer participate in builds and are no longer supported runtime interfaces.
