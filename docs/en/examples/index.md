# Examples

This directory contains reference CCR configuration and host runtime examples covering Claude, Codex, and Grok.

> [!IMPORTANT]
> In CCR 7.x, the authoritative live profile store for each platform is SQLite at `~/.ccr/platforms/<platform>/profiles.db` (legacy `profiles.toml` files are migrated and backed up automatically). Treat `.toml` files in this directory as field references or `ExchangeV1` import templates — **do not manually copy them onto `~/.ccr/platforms/*/profiles.toml` as primary live storage**.

## File List

| File | Purpose |
|------|---------|
| [`config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/config.toml) | Unified platform registry reference example |
| [`claude-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/claude-profiles.toml) | Claude profile fields and exchange-format example |
| [`codex-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-profiles.toml) | Codex profile example (official + third-party relays/models) |
| [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml) | DeepSeek `~/.codex/config.toml` host output example |
| [`codex-auth.example.json`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-auth.example.json) | Example `~/.codex/auth.json` |
| [`grok-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml) | Grok profile example (official session, `env_key`, and direct `api_key`) |
| [`grok-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml) | Example `~/.grok/config.toml` host output |
| [`gemini-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/gemini-profiles.toml) | Historical schema reference (`gemini` is a retained `legacy_adapter` without CLI `profile` subcommands) |
| [`multi-platform-setup.md`](./multi-platform-setup) | Multi-platform setup and workflow guide |
| [`troubleshooting.md`](./troubleshooting) | Common troubleshooting guide |

## Quick Start (Codex Example)

```bash
# 1) Initialize the Codex profile catalog (~/.ccr/platforms/codex/profiles.db)
ccr codex profile init

# 2) Create a profile safely via stdin or edit in $EDITOR
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create duckcoding \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin
ccr codex profile open

# 3) Validate and switch
ccr validate
ccr codex profile switch duckcoding
```

## Codex Quick Reference

- Platform guide: [`/en/reference/platforms/codex`](../reference/platforms/codex)
- Key fields: `model`, `model_reasoning_effort`, `base_url`, `auth_token`, `env_key`
- After switching, CCR atomically writes `~/.codex/config.toml` and `~/.codex/auth.json`
- For backups or sharing, export via `ccr codex profile export --no-secrets -o codex-profiles.toml` and never commit real tokens

## Claude and Grok Quick Reference

- Claude platform guide: [`/en/reference/platforms/claude`](../reference/platforms/claude) (use `ccr claude profile init|create|open|switch|export|import`, piping secrets via `--auth-token-stdin`)
- Grok command reference: [`/en/reference/commands/grok`](../reference/commands/grok) (pipe direct keys via `--api-key-stdin`; pass only an environment variable name such as `XAI_API_KEY` to `--env-key`)
- All examples use placeholder values and contain no real provider credentials or accounts
