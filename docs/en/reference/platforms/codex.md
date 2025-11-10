# Codex (GitHub Copilot) Platform

> **Note:** This documentation is being translated. Please refer to the [Chinese version](../platforms/codex) for complete information.

## Overview

Codex platform support for GitHub Copilot CLI configuration management.

## Quick Start

```bash
ccr platform init codex
ccr platform switch codex
ccr add
```

## Configuration

```toml
[profiles.github-copilot]
base_url = "https://api.github.com/copilot"
auth_token = "ghp_..."
model = "gpt-4"
provider = "github"
```

## See Also

- [Platform Overview](./index)
- [Claude Code Platform](./claude)
- [Multi-Platform Setup](../examples/multi-platform-setup)

---

**Translation in progress.** See [Chinese version](../platforms/codex) for details.
