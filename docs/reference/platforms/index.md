# Platform Guides

CCR supports managing configurations for multiple AI CLI platforms. Each platform has its own dedicated documentation with setup guides, examples, and best practices.

## Available Platforms

### ✅ Fully Implemented Platforms

| Platform | Icon | Description | Guide |
|----------|------|-------------|-------|
| **Claude Code** | 🤖 | Anthropic's official CLI with support for multiple relay services and third-party models | [Claude Guide](claude.md) |
| **Codex** | 💻 | Codex CLI (OpenAI-compatible providers, GitHub Copilot compatible mode) | [Codex Guide](codex.md) |
| **Gemini CLI** | ✨ | Google Gemini CLI for Google's AI models | [Gemini Guide](gemini.md) |
| **Factory Droid** | 🏭 | Factory AI Droid CLI with custom model support (Anthropic, OpenAI, generic) | [Droid Guide](droid.md) |

### 🚧 Planned Platforms

| Platform | Icon | Description | Status |
|----------|------|-------------|--------|
| **Qwen CLI** | 🔮 | Alibaba Tongyi Qianwen CLI | Planned |
| **iFlow CLI** | 🌊 | iFlow AI CLI | Planned |

## Quick Platform Commands

### List All Platforms

```bash
ccr platform list
```

Output:
```
┌────────┬──────────┬──────┬──────────────┬──────────────────────────┐
│ 状态   ┆ 平台名称 ┆ 启用 ┆ 当前 Profile ┆ 描述                     │
╞════════╪══════════╪══════╪══════════════╪══════════════════════════╡
│ ▶ 当前 ┆ claude   ┆ ✓    ┆ husan        ┆ Claude Code AI Assistant │
│        ┆ codex    ┆ ✓    ┆ github       ┆ GitHub Copilot CLI       │
│        ┆ gemini   ┆ ✓    ┆ google       ┆ Google Gemini CLI        │
└────────┴──────────┴──────┴──────────────┴──────────────────────────┘
```

### Switch Platform

```bash
# Switch to Claude Code
ccr platform switch claude

# Switch to Codex (GitHub Copilot)
ccr platform switch codex

# Switch to Gemini
ccr platform switch gemini
```

### View Current Platform

```bash
ccr platform current
```

### Initialize New Platform

```bash
# Initialize a platform
ccr platform init <platform-name>

# Example: Initialize Gemini
ccr platform init gemini
```

## Platform Comparison

| Feature | Claude Code | Codex | Gemini | Droid |
|---------|-------------|-------|--------|-------|
| **Settings Path** | `~/.claude/settings.json` | `~/.codex/config.toml` | `~/.gemini/settings.json` | `~/.factory/settings.json` |
| **Profile Count** | 16 (example) | 5 (example) | 6 (example) | 4 (example) |
| **Relay Services** | ✅ Multiple (AnyRouter, HuSan, etc.) | ✅ GitHub official | ✅ Google official | ✅ Multiple providers |
| **Third-Party Models** | ✅ GLM, Kimi, SiliconFlow | ❌ | ❌ | ✅ Any OpenAI-compatible |
| **Token Format** | `sk-ant-api03-...` or `sk-...` | `ghp_...` | `AIzaSy...` | `sk-...` or provider-specific |
| **History Tracking** | ✅ | ✅ | ✅ | ✅ |
| **Auto Backup** | ✅ | ✅ | ✅ | ✅ |

## Configuration Modes

CCR supports two configuration modes:

### Legacy Mode (Single Platform)

Traditional CCR setup with single configuration file:

```
~/.ccs_config.toml        # All configurations
~/.claude/settings.json   # Claude Code settings only
```

**Use when:** You only use Claude Code and want simple setup.

### Unified Mode (Multi-Platform)

Modern CCR setup with per-platform organization:

```
~/.ccr/
  ├── config.toml                      # Platform registry
  └── platforms/
      ├── claude/
      │   ├── profiles.toml            # Claude profiles
      │   ├── history.json             # Claude history
      │   └── backups/                 # Claude backups
      ├── codex/
      │   ├── profiles.toml            # Codex profiles
      │   ├── history.json             # Codex history
      │   └── backups/                 # Codex backups
      └── gemini/
          ├── profiles.toml            # Gemini profiles
          ├── history.json             # Gemini history
          └── backups/                 # Gemini backups
```

**Use when:** You use multiple AI CLI platforms or want better organization.

## Platform-Specific Features

### Claude Code

- **16 example profiles** including official API, relay services, and third-party models
- **Provider classification**: Official relay vs third-party models
- **Account tracking**: Multiple accounts per provider
- **Tag-based organization**: Flexible filtering and grouping
- **Temporary token override**: Test tokens without config changes

See [Claude Platform Guide](claude.md) for details.

### Codex

- **GitHub Personal Access Token** authentication
- **GitHub Copilot** model access
- **Organization support** with appropriate scopes
- **Multiple profile management** for different projects

See [Codex Platform Guide](codex.md) for details.

### Gemini

- **Google API Key** authentication
- **Multiple Gemini models** support (2.0-flash, 1.5-flash, etc.)
- **Thread-based profiles** for separate conversations
- **Fast model switching** between experimental and stable versions

See [Gemini Platform Guide](gemini.md) for details.

### Droid

- **Multi-provider support** for Anthropic, OpenAI, and generic APIs
- **Custom models** via `customModels` configuration
- **Max output tokens** configuration per profile
- **Local model support** via Ollama integration

See [Droid Platform Guide](droid.md) for details.

## Migration Guide

If you're using Legacy mode, you can easily migrate to Unified mode:

```bash
# Check if you should migrate
ccr migrate --check

# Migrate all profiles to Unified mode
ccr migrate

# Migrate specific platform
ccr migrate --platform claude
```

See [Migration Guide](migration.md) for detailed migration instructions.

## Multi-Platform Workflow Examples

### Example 1: Daily Development Workflow

```bash
# Morning: Use Claude Code for brainstorming
ccr platform switch claude
ccr switch anthropic

# Afternoon: Use Codex for code completion
ccr platform switch codex
ccr switch github

# Evening: Use Gemini for documentation
ccr platform switch gemini
ccr switch google
```

### Example 2: Project-Specific Setup

```bash
# Frontend project: Use Gemini
ccr platform switch gemini
ccr add  # Add project-specific profile

# Backend project: Use Claude
ccr platform switch claude
ccr add  # Add project-specific profile

# Infrastructure: Use Codex
ccr platform switch codex
ccr add  # Add project-specific profile
```

### Example 3: Cost Optimization

```bash
# Use free relay for development
ccr platform switch claude
ccr switch anyrouter

# Use paid service for production
ccr platform switch claude
ccr switch husan

# Use Google Gemini for specific tasks
ccr platform switch gemini
ccr switch google
```

## Platform Detection

CCR automatically detects which configuration mode to use:

1. **Check `CCR_ROOT` environment variable** → If set, use Unified mode
2. **Check `~/.ccr/config.toml` existence** → If exists, use Unified mode
3. **Fallback to Legacy mode** → Use `~/.ccs_config.toml` (backward compatible)

No manual configuration needed!

## Best Practices

### 1. Platform Isolation

Each platform maintains separate:
- ✅ Profiles (no naming conflicts)
- ✅ History (independent tracking)
- ✅ Backups (isolated recovery)
- ✅ Settings (platform-specific)

### 2. Naming Conventions

**Claude Platform:**
- Official relay: Use provider name (anyrouter, husan, duck)
- Third-party models: Use model provider (glm, moonshot, siliconflow)
- Add descriptive tags: ["free", "paid", "stable", "backup"]

**Codex Platform:**
- GitHub official: `github`
- Test profiles: `profile-1`, `profile-2`, etc.

**Gemini Platform:**
- Google official: `google`
- Thread-based: `thread-1`, `thread-2`, etc.

### 3. Regular Maintenance

```bash
# Clean old backups (per platform)
ccr platform switch claude
ccr clean -d 7

ccr platform switch codex
ccr clean -d 7

ccr platform switch gemini
ccr clean -d 7
```

### 4. Documentation

Document your platform-specific configurations:

```toml
# Claude platform profile
[anyrouter]
description = "AnyRouter Free Service (GitHub #5953) - Development use only"
account = "github_5953"
tags = ["free", "development", "primary"]

# Codex platform profile
[github]
description = "GitHub Copilot Official - Main development account"
provider = "GitHub"
tags = ["official", "primary"]

# Gemini platform profile
[google]
description = "Google Gemini Official - Documentation and research"
provider = "Google"
tags = ["official", "research"]
```

## Troubleshooting

### Issue: Platform Not Found

**Error:**
```
✗ 平台未找到: qwen
```

**Solution:**
```bash
# List available platforms
ccr platform list

# Initialize the platform if it's supported
ccr platform init qwen
```

### Issue: Wrong Platform Active

**Error:**
Profiles not showing after `ccr list`.

**Solution:**
```bash
# Check current platform
ccr platform current

# Switch to correct platform
ccr platform switch claude

# Now list profiles
ccr list
```

### Issue: Migration Needed

**Error:**
```
✗ 配置文件不存在: /home/user/.ccs_config.toml
```

**Solution:**
```bash
# Check if you should migrate
ccr migrate --check

# Migrate to Unified mode
ccr migrate
```

## Additional Resources

- **[Claude Platform Guide](claude.md)** - Comprehensive Claude Code setup
- **[Codex Platform Guide](codex.md)** - GitHub Copilot CLI configuration
- **[Gemini Platform Guide](gemini.md)** - Google Gemini CLI setup
- **[Droid Platform Guide](droid.md)** - Factory Droid CLI configuration
- **[Migration Guide](migration.md)** - Legacy to Unified mode migration
- **[Configuration Guide](../configuration.md)** - Advanced configuration options
- **[Quick Start Guide](../quick-start.md)** - Getting started with CCR

## Statistics (Example Multi-Platform Setup)

**Total Profiles**: 27
- Claude: 16 profiles (59%)
- Codex: 5 profiles (19%)
- Gemini: 6 profiles (22%)

**Configuration Types**:
- Official Relay: 12 (44%)
- Third-Party Models: 4 (15%)
- Test Profiles: 11 (41%)

**Active Platforms**: 4 (Claude, Codex, Gemini, Droid)
**Planned Platforms**: 2 (Qwen, iFlow)

---

*Choose your platform and start managing your AI CLI configurations efficiently!*
