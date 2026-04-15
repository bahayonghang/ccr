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

# In the OpenCode path, no action opens the OpenCode Auth tab
ccr opencode
```

## Keyboard Controls

| Key | Action |
|---|---|
| `Tab` | switch between available tabs |
| `←` / `→` / `h` / `l` | paginate |
| `↑` / `↓` / `j` / `k` | move selection |
| `Enter` | apply and exit |
| `Space` | apply and stay in TUI |
| `q` / `Esc` | quit |

## Current Role

- best for profile browsing and switching inside a terminal
- best for fast movement between Claude, Codex, and OpenCode-related tabs
- not a replacement for the exact command-line surface

## Implementation Facts

- the default build enables the `tui` feature
- entry detection lives in `Cli::is_tui_mode()`
- no-subcommand behavior lives in `CommandDispatcher::handle_no_subcommand()`
- the OpenCode Auth tab supports `i` to preview and confirm importing compatible saved Codex accounts

## Example

```bash
ccr
# Tab to change platform
# ↑↓ to select a profile
# Enter to apply and exit

ccr opencode
# Press i on the OpenCode Auth tab to preview and confirm importing compatible saved Codex accounts
```

## See Also

- [`opencode`](./opencode.md)
- [`list`](./list.md)
- [`switch`](./switch.md)
- [`current`](./current.md)
- [Entrypoints](/en/guide/entrypoints)
