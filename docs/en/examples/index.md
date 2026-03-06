# Examples

This directory contains copy-ready CCR configuration examples, including Codex-focused setup.

## File List

| File | Purpose |
|------|---------|
| [`config.toml`](../../examples/config.toml) | Unified platform registry |
| [`claude-profiles.toml`](../../examples/claude-profiles.toml) | Claude profile examples |
| [`codex-profiles.toml`](../../examples/codex-profiles.toml) | Codex profiles (official + third-party) |
| [`codex-cli-config.toml`](../../examples/codex-cli-config.toml) | Example `~/.codex/config.toml` |
| [`codex-auth.example.json`](../../examples/codex-auth.example.json) | Example `~/.codex/auth.json` |
| [`gemini-profiles.toml`](../../examples/gemini-profiles.toml) | Gemini profile examples |
| [`troubleshooting.md`](./troubleshooting) | Common troubleshooting |

## Codex Quick Reference

- Main guide: [`/en/reference/platforms/codex`](../reference/platforms/codex)
- Key fields: `model`, `model_reasoning_effort`, `base_url`, `auth_token`, `env_key`
- Recommended flow: update `profiles.toml` -> `ccr validate` -> `ccr switch <profile>`

## Quick Start (Codex)

```bash
# 1) Initialize and switch to Codex platform
ccr platform init codex
ccr platform switch codex

# 2) Copy Codex profile examples
cp docs/examples/codex-profiles.toml ~/.ccr/platforms/codex/profiles.toml

# 3) Validate and switch
ccr validate
ccr switch duckcoding
```

## Notes

- `codex-profiles.toml` is CCR input.
- After switching, CCR writes to `~/.codex/config.toml` and `~/.codex/auth.json`.
- For sharing, export with `ccr export --no-secrets` and never commit real tokens.
