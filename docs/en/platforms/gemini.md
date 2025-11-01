# Gemini CLI Platform

> **Note:** This documentation is being translated. Please refer to the [Chinese version](../platforms/gemini) for complete information.

## Overview

Gemini CLI platform support for Google's Gemini AI configuration management.

## Quick Start

```bash
ccr platform init gemini
ccr platform switch gemini
ccr add
```

## Configuration

```toml
[profiles.google-gemini]
base_url = "https://generativelanguage.googleapis.com/v1"
auth_token = "AIza..."
model = "gemini-pro"
provider = "google"
```

## See Also

- [Platform Overview](./index)
- [Claude Code Platform](./claude)
- [Multi-Platform Setup](../examples/multi-platform-setup)

---

**Translation in progress.** See [Chinese version](../platforms/gemini) for details.
