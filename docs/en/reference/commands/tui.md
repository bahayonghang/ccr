# TUI - Terminal Interactive Mode

This page documents the terminal interactive mode in the current default build. It is not a standalone `ccr tui` subcommand page.

## How To Enter It

```bash
# Default path: launch with no subcommand
ccr
```

Additional behavior:

```bash
# In the Codex path, no action also enters TUI mode
ccr codex

# In the Grok Auth path, no nested action opens the Grok Auth tab
ccr grok auth
```

## Keyboard Controls

| Key | Action |
|---|---|
| `Tab` | switch between available tabs |
| `←` / `→` / `h` / `l` | paginate |
| `↑` / `↓` / `j` / `k` | move selection |
| `Enter` / `Space` | apply the selected profile and stay in the TUI (result shown in the Focus panel) |
| `q` / `Esc` | quit |

## Current Role

- best for profile browsing and switching inside a terminal
- best for fast movement between Claude, Codex, and Grok tabs
- not a replacement for the exact command-line surface

## Implementation Facts

- the default build enables the `tui` feature
- entry detection lives in `Cli::is_tui_mode()`
- no-subcommand behavior lives in `CommandDispatcher::handle_no_subcommand()`
- the Grok Auth tab shows official session status and supports `o` to log out the official runtime

## Example

```bash
ccr
# Tab to change platform
# ↑↓ to select a profile
# Enter/Space to apply and stay (press q or Esc to quit)

ccr grok auth
# Press o on the Grok Auth tab to log out the current official runtime
```

## See Also

- [`grok`](./grok.md)
- [`list`](./list.md)
- [`switch`](./switch.md)
- [`current`](./current.md)
- [Entrypoints](/en/guide/entrypoints)
