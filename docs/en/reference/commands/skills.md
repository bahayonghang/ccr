# skills - Skills Management

Manage CCR's skill system across supported AI CLI platforms, including repository registration, scanning, installation, listing, and removal.

**Version**: v3.5.0+

## Usage

```bash
ccr skills <ACTION> [OPTIONS]
```

## Subcommands

### `list` — List Installed Skills

Display installed skills for the selected platform.

```bash
ccr skills list
ccr skills list --platform codex
ccr skills list --platform gemini
```

### `install` — Install a Skill

Install a discovered skill from a registered repository.

```bash
ccr skills install <NAME> [--platform <PLATFORM>]
```

### `uninstall` — Remove a Skill

Uninstall an installed skill.

```bash
ccr skills uninstall <NAME> [--platform <PLATFORM>]
```

### `repo` — Manage Skill Repositories

Register, list, or remove remote skill repositories:

```bash
ccr skills repo add official https://github.com/ccr-skills/official
ccr skills repo list
ccr skills repo remove official
```

### `scan` — Discover Available Skills

Scan a registered repository to discover installable skill names:

```bash
ccr skills scan official
```

## Options

### `--platform <PLATFORM>`

Target platform (`claude`, `codex`, `gemini`, `qwen`), defaulting to `claude`.

## End-to-End Example

```bash
# 1. Register a skill repository and scan its catalog
ccr skills repo add official https://github.com/ccr-skills/official
ccr skills repo list
ccr skills scan official

# 2. Install and verify skills per platform
ccr skills install code-review --platform claude
ccr skills install git-helper --platform codex
ccr skills list --platform claude

# 3. Uninstall when no longer needed
ccr skills uninstall old-skill --platform claude
```

## Skill Storage Locations

Installed skills are written to each platform's user-level directory:

- Claude: `~/.claude/skills/`
- Codex: `~/.agents/skills/`
- Antigravity CLI (`gemini` key): `~/.gemini/antigravity-cli/skills/`
- Legacy/shared Gemini: `~/.gemini/skills/`

> Note: These paths describe user-directory installations managed by `ccr skills`. For repository-tracked project skills shared with GitHub Copilot and other harnesses, see [GitHub Copilot Workspace Support](/en/guide/github-copilot-workspace).

## Troubleshooting

- **Installation fails**: verify the repository is registered (`ccr skills repo list`), confirm the skill exists (`ccr skills scan <repo>`), and check network access.
- **Installed skill is not picked up**: confirm it appears in `ccr skills list`, restart the host CLI session, and check runtime status with `ccr current`.

## Related Commands

- [`prompts`](./prompts) - manage prompt presets
- [`version`](./version) - inspect CCR version details
- [Command Reference](./index) - browse all commands
