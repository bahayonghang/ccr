# temp - Quick Temporary Configuration

Interactively apply a temporary Claude runtime configuration without creating or modifying entries in the persistent profile catalog (`profiles.db`). Enter `base_url`, `token`, and an optional `model` to write directly to `~/.claude/settings.json`.

## Usage

```bash
ccr temp
```

The interactive prompt asks for:
1. **Base URL** (required) — API endpoint URL
2. **Auth Token** (required) — authentication token
3. **Model** (optional) — model identifier, with short-alias resolution

## Difference from `temp-token`

| Feature | `ccr temp` | `ccr temp-token` |
|---------|------------|------------------|
| Input method | Interactive prompts | Command-line flags |
| Config dependency | None required | Overrides currently active settings |
| Use case | Manually testing a new provider | Scripted or flag-driven temporary overrides |
| Model parsing | Supports short aliases | Uses passed value verbatim |

## Smart Model Parsing

`ccr temp` resolves common short model aliases automatically:

### Claude Models

| Input | Resolved to |
|-------|-------------|
| `sonnet` / `claude-sonnet` | `claude-sonnet-4-20250514` |
| `opus` / `claude-opus` | `claude-opus-4-20250514` |
| `haiku` / `claude-haiku` | `claude-3-5-haiku-20241022` |

### GPT Models

| Input | Resolved to |
|-------|-------------|
| `gpt4` / `gpt-4` / `gpt4o` | `gpt-4o` |
| `gpt4-mini` / `gpt-4o-mini` | `gpt-4o-mini` |

### Gemini Models

| Input | Resolved to |
|-------|-------------|
| `gemini` / `gemini-pro` | `gemini-2.0-flash` |

> 💡 Full model identifiers (such as `claude-sonnet-4-5-20250929`) are written verbatim without alias conversion.

## Use Cases

### Case 1: Quickly Test a New API Provider

When you want to verify a provider endpoint without adding a permanent entry to `profiles.db`:

```text
$ ccr temp

1) Base URL: https://api.newprovider.com/v1
2) Auth Token: sk-test-xxxxx
3) Model: sonnet
   Smart parse: sonnet -> claude-sonnet-4-20250514

Apply this temporary configuration? (Y/n): y
✓ Temporary configuration applied to settings.json

Next steps:
  • Run 'ccr claude profile switch <name>' to restore a saved profile
  • Run 'ccr current' to inspect current runtime status
```

### Case 2: Skip Model Selection

Press `Enter` on the model prompt to leave the model unset and use the provider's default model.

## Restoring Saved Profiles

Temporary settings modify `~/.claude/settings.json` directly. When finished, switch back to any saved Claude profile (or exit profile mode):

```bash
# Restore a saved profile from profiles.db
ccr claude profile switch <name>

# Or exit profile mode back to official login
ccr claude profile off
```

## Notes

1. **Catalog untouched**: Does not add or modify records in `~/.ccr/platforms/claude/profiles.db`.
2. **Immediate effect**: Writes directly to `~/.claude/settings.json` upon confirmation.
3. **One-time override**: Overwritten the next time you run `ccr claude profile switch <name>` or `ccr claude profile off`.
4. **Token masking**: Automatically masks tokens in preview output.
5. **URL validation**: Base URL must start with `http://` or `https://`.

## File Locations

- Temporary target: `~/.claude/settings.json`
- Persistent Claude profile catalog: `~/.ccr/platforms/claude/profiles.db` (or `~/.ccs_config.toml` in Legacy Mode)

## Related Commands

- [`temp-token`](./temp-token.md) - command-line temporary override
- [`claude`](./claude.md) - manage persistent Claude profiles (`create` / `switch` / `off`)
- [`current`](./current.md) - inspect current runtime status
