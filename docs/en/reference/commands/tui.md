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
```

## Keyboard Controls

| Key | Action |
|---|---|
| `Tab` | switch between the Claude and Codex tabs |
| `←` / `→` / `h` / `l` | paginate |
| `↑` / `↓` / `j` / `k` | move selection |
| `Enter` | apply and exit |
| `Space` | apply and stay in TUI |
| `q` / `Esc` | quit |

## Current Role

- best for profile browsing and switching inside a terminal
- best for fast movement between Claude and Codex configs
- not a replacement for the exact command-line surface

## Implementation Facts

- the default build enables the `tui` feature
- entry detection lives in `Cli::is_tui_mode()`
- no-subcommand behavior lives in `CommandDispatcher::handle_no_subcommand()`

## Example

```bash
ccr
# Tab to change platform
# ↑↓ to select a profile
# Enter to apply and exit
```

## See Also

- [`list`](./list.md)
- [`switch`](./switch.md)
- [`current`](./current.md)
- [Entrypoints](/en/guide/entrypoints)
