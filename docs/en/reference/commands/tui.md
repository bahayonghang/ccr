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
| `Enter` / `Space` | open apply confirmation. `y` writes and stays in the TUI. `Enter`, `Space`, `n`, or `Esc` cancels |
| `o` | open deactivate confirmation. `y` turns profile mode off. `o`, `n`, or `Esc` cancels |
| `y` | confirm the open dialog |
| `q` / `Esc` | quit. While a confirmation is open, `Esc` only cancels it; the next `Esc` quits |

Profile `Enter` / `Space` and `o` always confirm first, including re-applying the current profile. While that dialog is open, arrows, `Tab`, `q`, and the mouse do not change selection, tab, or quit.

On Claude Auth and Codex Auth, `Enter` opens a confirmation only when the key would actually switch accounts. `o` opens logout confirmation. No switchable account, an already current account, a Codex virtual login, and a read-only registry still show the existing notice and do not open a switch dialog. After `y`, a successful Claude switch still leaves the TUI. A successful Codex switch still leaves the TUI, or opens the existing daemon-restart confirmation.

The Grok Auth tab uses account actions: `s` saves, `Enter` opens switch confirmation, `d` deletes a saved entry, `o` logs out all runtime credentials, and `r` reloads. Confirmations default to cancel; press `y` to confirm. Saving is allowed while Grok runs. End the current Grok session yourself before switching, then use a new session. Navigation and exit wait for a pending operation to finish; language changes and resize remain available. See [`grok`](./grok.md).

## Current Role

- best for profile browsing and switching inside a terminal
- best for fast movement between Claude, Codex, and Grok tabs
- not a replacement for the exact command-line surface

## Implementation Facts

- the default build enables the `tui` feature
- entry detection lives in `Cli::is_tui_mode()`
- no-subcommand behavior lives in `CommandDispatcher::handle_no_subcommand()`
- the Grok Auth tab separates saved accounts, selection, and local session matches; authentication validity is unverified and third-party profiles remain unchanged

## Example

```bash
ccr
# Tab to change platform
# ↑↓ to select a profile
# Enter/Space opens confirmation; y applies and stays (press q or Esc to quit)

ccr grok auth
# s saves the current account copy; Enter opens switch confirmation
# d deletes only a saved entry; o opens confirmation for logout of the entire runtime
```

## See Also

- [`grok`](./grok.md)
- [`list`](./list.md)
- [`switch`](./switch.md)
- [`current`](./current.md)
- [Entrypoints](/en/guide/entrypoints)
