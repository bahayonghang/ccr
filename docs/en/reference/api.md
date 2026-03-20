# Legacy Web API (Removed)

The legacy HTTP routes from `crates/ccr` have been removed. This page remains only as migration guidance.

## Migration Targets

- Graphical workflows: use [`ccr ui`](/en/reference/commands/ui) to launch the standalone `ccr-ui`
- Desktop integration: use Tauri IPC commands from `ccr-ui/src-tauri`
- CLI automation: continue using `ccr` commands directly

## Notes

- The old `/api/*` surface is no longer a supported runtime interface
- Any remaining `src/web/**` files are stale implementation leftovers pending physical removal
- For the current graphical product surface, go to [UI Overview](/en/guide/ui-overview)
