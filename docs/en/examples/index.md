# Examples

Practical examples and use cases for CCR.

## Quick Examples

### Basic Usage

**Switch between API providers:**
```bash
# Official Anthropic API
ccr switch anthropic-official

# Free tier provider
ccr switch anthropic-free

# OpenRouter
ccr switch openrouter
```

**View current configuration:**
```bash
ccr current

# Output shows:
# - Current profile name
# - API endpoint
# - Active model
# - Environment variables
```

### Multi-Platform Workflow

**Managing multiple AI platforms:**
```bash
# Claude Code for complex reasoning
ccr platform switch claude
ccr switch anthropic-official

# GitHub Copilot for code completion
ccr platform switch codex
ccr switch github-copilot

# Gemini for quick queries
ccr platform switch gemini
ccr switch google-free
```

### Team Collaboration

**Share configurations across team:**
```bash
# Export profiles (without secrets)
ccr export --no-secrets -o team-config.toml

# Share team-config.toml with team
# Team members import:
ccr import team-config.toml --merge

# Each member adds their own tokens:
ccr add  # Enter personal API token
```

### Automated Workflows

**CI/CD integration:**
```bash
#!/bin/bash
# deploy.sh

# Set API key from environment
ccr temp-token set $ANTHROPIC_API_KEY \
  --base-url https://api.anthropic.com

# Run deployment tasks
ccr switch production
# Your deployment commands here

# Token automatically cleared after switch
```

### Cloud Sync

**Sync configs across multiple machines:**
```bash
# On machine 1: Configure and push
ccr sync config  # Enter WebDAV credentials
ccr sync push    # Upload to cloud

# On machine 2: Pull and use
ccr sync config  # Enter same WebDAV credentials  
ccr sync pull    # Download from cloud
ccr list         # See all synced profiles
```

## Detailed Guides

- [**Multi-Platform Setup**](./multi-platform-setup) - Complete guide for managing multiple AI platforms
- [**Troubleshooting**](./troubleshooting) - Common issues and solutions

## Example Scenarios

### Scenario 1: Developer with Multiple Projects

**Problem:** Need different API keys for personal projects and work projects.

**Solution:**
```bash
# Work projects (official API)
ccr add  # Name: work-anthropic
# Personal projects (third-party provider)
ccr add  # Name: personal-free

# Switch based on context
ccr switch work-anthropic      # For work
ccr switch personal-free       # For personal projects
```

### Scenario 2: Testing New Features

**Problem:** Want to test beta features without affecting production config.

**Solution:**
```bash
# Create test profile
ccr add  # Name: test-beta
# Beta endpoint URL
# Test API key

# Easy switching for testing
ccr switch test-beta           # Test new features
ccr switch production          # Back to stable
```

### Scenario 3: Multiple Team Members

**Problem:** Team needs to share profile configurations but not API keys.

**Solution:**
```bash
# Team lead exports template
ccr export --no-secrets -o team-template.toml

# Team members import and add their keys
ccr import team-template.toml
ccr add  # Add personal API key for each profile
```

### Scenario 4: Development and Production

**Problem:** Need separate configs for dev and prod environments.

**Solution:**
```bash
# Development
ccr add  # Name: dev-api
# Dev endpoint: https://dev.api.example.com

# Production
ccr add  # Name: prod-api
# Prod endpoint: https://api.anthropic.com

# Quick switching
ccr switch dev-api     # Development work
ccr switch prod-api    # Production deployment
```

## Best Practices

### Organization
```bash
# Use descriptive profile names
anthropic-official     # ✅ Good: Clear and specific
my-api                 # ❌ Bad: Too vague

# Group by provider or purpose
anthropic-official
anthropic-free
openrouter-paid
duck-free
```

### Security
```bash
# Never commit config files
echo ".ccs_config.toml" >> .gitignore
echo ".ccr/" >> .gitignore

# Use temp tokens for testing
ccr temp-token set $TEST_TOKEN
# Automatically cleared after use

# Export without secrets for sharing
ccr export --no-secrets -o shareable.toml
```

### Backup
```bash
# Regular exports
ccr export -o backup-$(date +%Y%m%d).toml

# Cloud sync for redundancy
ccr sync push  # Automatic backups to cloud

# Keep important configs in version control (without secrets)
ccr export --no-secrets -o configs/template.toml
git add configs/template.toml
```

### Validation
```bash
# Validate after changes
ccr validate

# Check current status regularly
ccr current

# Review history for audit trail
ccr history -l 20
```

## Command Cheat Sheet

```bash
# Quick Operations
ccr list                          # View all profiles
ccr <profile-name>                # Quick switch
ccr current                       # Show current status

# Management
ccr add                           # Add new profile
ccr delete <name>                 # Remove profile
ccr validate                      # Check all configs

# History & Stats
ccr history -l 10                 # Last 10 operations
ccr stats                         # Usage statistics

# Sync
ccr sync push                     # Upload to cloud
ccr sync pull                     # Download from cloud

# Maintenance
ccr clean                         # Clean old backups
ccr update                        # Update CCR
```

## Advanced Examples

### Custom Scripts

**Automatic profile switching based on project:**
```bash
#!/bin/bash
# project-env.sh

PROJECT_DIR=$(basename "$PWD")

case "$PROJECT_DIR" in
  "work-project")
    ccr switch work-anthropic
    ;;
  "personal-project")
    ccr switch personal-free
    ;;
  *)
    echo "Unknown project, using default"
    ccr switch anthropic-official
    ;;
esac

ccr current
```

### Integration with Other Tools

**Use with direnv:**
```bash
# .envrc
use ccr work-anthropic
```

**Use in Makefile:**
```makefile
.PHONY: dev prod

dev:
	ccr switch dev-api
	@echo "Development environment active"

prod:
	ccr switch prod-api
	@echo "Production environment active"
```

## Next Steps

- 📖 [Command Reference](../commands/) - Learn all commands
- 🔧 [Configuration Guide](../configuration) - Advanced configuration
- 🌐 [Web Guide](../web-guide) - Use the web interface
- 🚀 [Quick Start](../quick-start) - Get started quickly
