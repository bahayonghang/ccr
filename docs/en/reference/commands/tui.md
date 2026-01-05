# TUI - Interactive Terminal Interface

Launch CCR's interactive terminal user interface (TUI) for visual configuration management.

## Basic Usage

```bash
# Simply run ccr without arguments
ccr
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch between Claude Code / Codex CLI platforms |
| `←` / `→` / `h` / `l` | Navigate pages (when >20 configs) |
| `↑` / `↓` / `j` / `k` | Select configuration |
| `Enter` | Apply selected configuration |
| `q` / `Esc` | Quit |

## Features

### Dual-Tab Interface

The TUI provides a dual-tab interface for managing configurations across two platforms:

- **🤖 Claude Code** - Anthropic's official CLI
- **💻 Codex CLI** - GitHub Copilot CLI

Press `Tab` to switch between platforms. Each platform maintains its own configuration list.

### Pagination

- Maximum **20 configurations per page**
- Use `←` / `→` (or `h` / `l` for vim users) to navigate between pages
- Page indicator shown in title: `Claude Profiles (25)  Page 1/2`

### Visual Indicators

**List Items:**
- `▶` - Currently selected item (cursor)
- `●` - Active configuration
- `○` - Inactive configuration
- `✓` - Currently applied configuration

**Platform Colors:**
- **Orange** (`#f59e0b`) - Claude Code theme
- **Purple** (`#6366f1`) - Codex CLI theme

The border color changes based on the active platform tab.

### Status Messages

Status messages appear at the **bottom** of the screen:
- ✅ Green - Success messages (e.g., "已切换到: my-config")
- ❌ Red - Error messages (e.g., "切换失败: ...")

## Layout

```
╭─────────────────────────────────────────────────╮
│       🚀 CCR - Configuration Switcher           │  ← Header
│  ▸ 🤖 Claude Code  │    💻 Codex CLI            │  ← Tabs
╰─────────────────────────────────────────────────╯
╭─ Claude Profiles (3) ───────────────────────────╮
│  ▶ ● anthropic  ─  Official API  ✓              │  ← Selected + Active
│    ○ openrouter  ─  Multi-model gateway         │
│    ○ custom-api  ─  Self-hosted                 │  ← Profile List
│                                                 │
╰─────────────────────────────────────────────────╯
╭─ Keys ──────────────────────────────────────────╮
│  Tab Switch │ ←→ Page │ ↑↓/jk Select │ Enter   │  ← Shortcuts
╰─────────────────────────────────────────────────╯
  ✅ 已切换到: anthropic                            ← Status Message
```

## Examples

### Quick Configuration Switch

```bash
ccr
# Tab → switch to Codex (if needed)
# ↓↓ → select config
# Enter → apply
# q → quit
```

### Navigate Large Config Lists

```bash
ccr
# → → navigate to page 2
# ↓ → select config on page 2
# Enter → apply
# q → quit
```

## Technical Details

- **Framework**: Ratatui 0.30 + Crossterm 0.29
- **Event Loop**: 250ms tick rate
- **Windows Support**: Filters `KeyEventKind::Press` to avoid double-trigger issues
- **Page Size**: 20 items per page (configurable via `PAGE_SIZE` constant)

## Troubleshooting

### Terminal Display Issues

```bash
# Check terminal support
echo $TERM

# Try with 256 colors
export TERM=xterm-256color
ccr
```

### Keys Not Responding

```bash
# Reset terminal
reset
ccr
```

### Exit Issues

```bash
# If terminal is messed up after exit
reset
# or
clear
```

## See Also

- [`list`](./list.md) - List configurations in table format
- [`switch`](./switch.md) - Switch configuration via CLI
- [`current`](./current.md) - Show current configuration
