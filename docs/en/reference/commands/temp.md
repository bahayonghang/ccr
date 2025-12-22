# temp - Quick Temporary Configuration

Interactively set up temporary configuration without relying on existing TOML config files. Directly input base_url, token, and model, then immediately write to settings.json.

## Usage

```bash
ccr temp
```

After execution, you'll be prompted to enter:
1. **Base URL** (required) - API endpoint address
2. **Auth Token** (required) - Authentication token
3. **Model** (optional) - Model name with smart parsing support

## Difference from temp-token

| Feature | `ccr temp` | `ccr temp-token` |
|---------|------------|------------------|
| Input method | Interactive prompts | Command-line arguments |
| Config dependency | None required | Based on existing TOML config |
| Use case | Quick test new providers | Temporarily override existing config |
| Model parsing | Smart parsing supported | Uses input value directly |

## Smart Model Parsing

`ccr temp` supports smart model name parsing, allowing you to use short aliases instead of full model names:

### Claude Models

| Input | Parsed to |
|-------|-----------|
| `sonnet` / `claude-sonnet` | `claude-sonnet-4-20250514` |
| `opus` / `claude-opus` | `claude-opus-4-20250514` |
| `haiku` / `claude-haiku` | `claude-3-5-haiku-20241022` |

### GPT Models

| Input | Parsed to |
|-------|-----------|
| `gpt4` / `gpt-4` / `gpt4o` | `gpt-4o` |
| `gpt4-mini` / `gpt-4o-mini` | `gpt-4o-mini` |

### Gemini Models

| Input | Parsed to |
|-------|-----------|
| `gemini` / `gemini-pro` | `gemini-2.0-flash` |

> 💡 If you enter a complete model name (e.g., `claude-3-5-sonnet-20241022`), it will be used as-is without conversion.

## Use Cases

### Case 1: Quick Test a New API Provider

When you want to quickly test a new API provider without creating a permanent config:

```bash
$ ccr temp

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Quick Temporary Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📝 Let me help you set up a temporary configuration!
   This will be written directly to settings.json
   Next 'ccr switch' will restore values from TOML config

────────────────────────────────────────

1️⃣ Enter Base URL (API endpoint)
   Example: https://api.anthropic.com
            https://api.example.com/v1

* Base URL: https://api.newprovider.com/v1

2️⃣ Enter Auth Token (authentication token)
   Example: sk-ant-api03-xxxxx
            sk-xxxxx

* Auth Token: sk-test-xxxxx

3️⃣ Enter Model (model name, optional)
   Example: claude-sonnet-4-20250514
            claude-3-5-sonnet-20241022
            gpt-4o
   Tip: Press Enter to skip and use provider's default model

  Model: sonnet
   🧠 Smart parse: sonnet → claude-sonnet-4-20250514

────────────────────────────────────────

Step 1/2: Configuration Preview

╔════════════════╤═══════════════════════════════════════════╗
║ Field          │ Value                                     ║
╠════════════════╪═══════════════════════════════════════════╣
║ Base URL       │ https://api.newprovider.com/v1            ║
║ Auth Token     │ sk-t...xxxxx                              ║
║ Model          │ claude-sonnet-4-20250514                  ║
╚════════════════╧═══════════════════════════════════════════╝

Apply this temporary configuration? (Y/n): y

────────────────────────────────────────

Step 2/2: Applying temporary configuration
✅ Temporary configuration applied to settings.json

💡 Tips:
   • Configuration is now active
   • Run 'ccr switch <config>' to restore TOML config
   • Run 'ccr current' to view current status
```

### Case 2: Using Short Model Aliases

```bash
$ ccr temp
...
  Model: opus
   🧠 Smart parse: opus → claude-opus-4-20250514
```

### Case 3: Skip Model Setting

To use the provider's default model:

```bash
$ ccr temp
...
  Model: [Press Enter]
   ℹ️  Skipped, will use provider's default model
```

## Restoring Configuration

Temporary configuration directly modifies `~/.claude/settings.json`. To restore TOML config values:

```bash
# Switch to any config to restore
ccr switch <config_name>
```

## Notes

1. **No config file created**: This command does not create or modify TOML config files
2. **Immediate effect**: Configuration is written to settings.json immediately
3. **One-time use**: Will be overwritten on next `ccr switch`
4. **Token masking**: Tokens are automatically masked when displayed
5. **URL validation**: Base URL must start with `http://` or `https://`

## Related Commands

- [`ccr temp-token`](./temp-token.md) - Temporary override based on existing config
- [`ccr switch`](./switch.md) - Switch configuration (restores TOML config)
- [`ccr current`](./current.md) - View current configuration status
- [`ccr add`](./add.md) - Interactively add permanent configuration

## File Locations

- Temporary config written to: `~/.claude/settings.json`
- Permanent config file: `~/.ccs_config.toml` or `~/.ccr/platforms/<platform>/profiles.toml`
