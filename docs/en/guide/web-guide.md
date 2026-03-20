# Legacy Web Migration

The built-in legacy Web API / Web UI from `crates/ccr` has been removed and is no longer a supported entrypoint.

## What to use now

| Entry | Role | Best for |
|------|------|----------|
| `ccr` | primary CLI / TUI entrypoint | scripting, automation, daily command workflows |
| `ccr ui` | recommended graphical entrypoint | day-to-day visual management and module browsing |
| `ccr-ui` | standalone graphical app project | frontend development and Tauri desktop runtime |

## Migration guidance

- If you used `ccr web` for UI access, move to `ccr ui`
- If you relied on the embedded browser pages, move to `ccr-ui`
- If you relied on command automation, keep using `ccr` directly

## Related pages
- [UI Overview](/en/guide/ui-overview)
- [UI Modules](/en/guide/ui-modules)
- [`ccr ui` command](/en/reference/commands/ui)
