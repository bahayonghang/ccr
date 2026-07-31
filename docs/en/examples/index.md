# Examples

This directory contains copy-ready CCR configuration examples, including Codex-focused setup.

## File List

| File | Purpose |
|------|---------|
| [`config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/config.toml) | Unified platform registry |
| [`claude-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/claude-profiles.toml) | Claude profile examples |
| [`codex-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-profiles.toml) | Codex profiles (official + third-party) |
| [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml) | Example `~/.codex/config.toml` |
| [`codex-auth.example.json`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-auth.example.json) | Example `~/.codex/auth.json` |
| [`gemini-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/gemini-profiles.toml) | Gemini profile examples |
| [`grok-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml) | Grok profiles (official + third-party api_key) |
| [`grok-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml) | Example `~/.grok/config.toml` |
| [`troubleshooting.md`](./troubleshooting) | Common troubleshooting |

## Codex Quick Reference

- Main guide: [`/en/reference/platforms/codex`](../reference/platforms/codex)
- Key fields: `model`, `model_reasoning_effort`, `base_url`, `auth_token`, `env_key`
- Recommended flow: update `profiles.toml` -> `ccr validate` -> `ccr switch <profile>`

## Quick Start (Codex)

```bash
# 1) Initialize the Codex profiles template
ccr codex profile init

# 2) Edit, validate, and switch
vim ~/.ccr/platforms/codex/profiles.toml
ccr validate
ccr codex profile switch duckcoding
```

## Notes

- `codex-profiles.toml` is CCR input.
- After switching, CCR writes to `~/.codex/config.toml` and `~/.codex/auth.json`.
- For sharing, export with `ccr export --no-secrets` and never commit real tokens.

## Grok Quick Reference

- Command guide: [`/en/reference/commands/grok`](../reference/commands/grok)
- Use `api_key` for a direct Grok credential; `env_key` accepts only an environment variable name.
- The examples contain no real provider, account, or credential.
