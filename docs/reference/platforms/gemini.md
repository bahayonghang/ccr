# Gemini CLI Platform Guide

## Overview

Gemini CLI is Google's command-line interface for the Gemini AI model. CCR provides full support for managing Gemini configurations and profiles.

## Platform Information

- **Platform Name**: `gemini`
- **Display Name**: Gemini CLI
- **Icon**: ✨
- **Status**: ✅ Fully Implemented
- **Settings Path**: `~/.gemini/settings.json`
- **Profiles Path**: `~/.ccr/gemini/profiles.toml`

## Prerequisites

- Google Cloud account or Google AI Studio access
- Google API Key for Gemini
- Gemini CLI installed (if using settings synchronization)

## API Key Format

Gemini uses Google API Keys in the following format:

```
AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**Characteristics:**
- Starts with `AIzaSy`
- 39 characters total
- Alphanumeric characters

**How to Generate:**

### Option 1: Google AI Studio (Recommended for Quick Start)

1. Go to [Google AI Studio](https://makersuite.google.com/app/apikey)
2. Click "Get API key" or "Create API key"
3. Select existing project or create new project
4. Copy the generated API key

### Option 2: Google Cloud Console (For Production)

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Navigate to "APIs & Services" → "Credentials"
3. Click "Create Credentials" → "API key"
4. Restrict the key (recommended):
   - API restrictions: Select "Generative Language API"
   - Application restrictions: Set IP or HTTP referrer restrictions
5. Copy the API key

## Quick Start

### Initialize Gemini Platform

```bash
# Initialize Gemini platform (creates directory structure)
ccr platform init gemini

# Switch to Gemini platform
ccr platform switch gemini

# Verify current platform
ccr platform current
```

### Add Your First Profile

```bash
# Interactive mode
ccr add

# Follow prompts to enter:
# - Profile name (e.g., "google-official")
# - Description (optional)
# - Base URL (default: https://generativelanguage.googleapis.com/v1beta)
# - API Key (AIzaSy...)
# - Model (default: gemini-pro)
```

### Configuration Example

Create a profile in `~/.ccr/gemini/profiles.toml`:

```toml
[google-official]
description = "Google AI Studio API"
base_url = "https://generativelanguage.googleapis.com/v1beta"
auth_token = "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
model = "gemini-2.0-flash-exp"
small_fast_model = "gemini-1.5-flash"

[google-cloud]
description = "Google Cloud Vertex AI"
base_url = "https://us-central1-aiplatform.googleapis.com/v1"
auth_token = "AIzaSyYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY"
model = "gemini-2.0-flash-exp"
small_fast_model = "gemini-1.5-flash"

[gemini-pro]
description = "Gemini Pro Model"
base_url = "https://generativelanguage.googleapis.com/v1beta"
auth_token = "AIzaSyZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ"
model = "gemini-pro"
small_fast_model = "gemini-pro"
```

## Profile Management

### List Profiles

```bash
# Switch to Gemini platform first
ccr platform switch gemini

# List all Gemini profiles
ccr list
```

### Switch Between Profiles

```bash
# Switch to specific Gemini profile
ccr switch google-official

# Or use shorthand
ccr google-official
```

### Update Profile

```bash
# Edit profiles.toml manually
vim ~/.ccr/gemini/profiles.toml

# Validate changes
ccr validate
```

### Delete Profile

```bash
# Interactive deletion with confirmation
ccr delete google-cloud

# Force deletion (skip confirmation)
ccr delete google-cloud --force
```

## Environment Variables

When a Gemini profile is active, CCR manages these environment variables in `~/.gemini/settings.json`:

```json
{
  "env": {
    "GOOGLE_API_BASE_URL": "https://generativelanguage.googleapis.com/v1beta",
    "GOOGLE_API_KEY": "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "GEMINI_MODEL": "gemini-2.0-flash-exp",
    "GEMINI_SMALL_FAST_MODEL": "gemini-1.5-flash"
  }
}
```

**Variable Mapping:**

| TOML Field | Environment Variable | Description |
|------------|---------------------|-------------|
| `base_url` | `GOOGLE_API_BASE_URL` | API endpoint |
| `auth_token` | `GOOGLE_API_KEY` | Google API key (masked in logs) |
| `model` | `GEMINI_MODEL` | Default model |
| `small_fast_model` | `GEMINI_SMALL_FAST_MODEL` | Fast model (optional) |

## Available Models

### Gemini 2.0 Models (Latest, Recommended)

```toml
# Gemini 2.0 Flash Experimental (fastest, multimodal)
model = "gemini-2.0-flash-exp"
small_fast_model = "gemini-2.0-flash-exp"

# Gemini 2.0 Flash Thinking Experimental (reasoning)
model = "gemini-2.0-flash-thinking-exp-1219"
```

### Gemini 1.5 Models (Stable)

```toml
# Gemini 1.5 Pro (best quality, 2M token context)
model = "gemini-1.5-pro"
small_fast_model = "gemini-1.5-flash"

# Gemini 1.5 Flash (fast, 1M token context)
model = "gemini-1.5-flash"
small_fast_model = "gemini-1.5-flash"

# Gemini 1.5 Flash-8B (ultra-fast, cost-efficient)
model = "gemini-1.5-flash-8b"
```

### Gemini 1.0 Models (Legacy)

```toml
# Gemini Pro (original)
model = "gemini-pro"

# Gemini Pro Vision (multimodal)
model = "gemini-pro-vision"
```

## Common Use Cases

### Development Workflow

```bash
# Morning: Start with AI Studio API
ccr platform switch gemini
ccr switch google-official

# Afternoon: Test with production Vertex AI
ccr switch google-cloud

# View operation history
ccr history
```

### Model Testing

```bash
# Test with different models
[flash-2]
description = "Gemini 2.0 Flash"
auth_token = "AIzaSy..."
model = "gemini-2.0-flash-exp"

[pro-15]
description = "Gemini 1.5 Pro"
auth_token = "AIzaSy..."
model = "gemini-1.5-pro"

[flash-8b]
description = "Gemini 1.5 Flash-8B"
auth_token = "AIzaSy..."
model = "gemini-1.5-flash-8b"

# Quick switch between models
ccr switch flash-2   # Use 2.0 Flash
ccr switch pro-15    # Use 1.5 Pro
ccr switch flash-8b  # Use 8B for speed
```

### Multi-Project Management

```bash
# Project A API key
[project-a]
description = "Project A - Gemini API"
base_url = "https://generativelanguage.googleapis.com/v1beta"
auth_token = "AIzaSyProjectA_key_here"
model = "gemini-2.0-flash-exp"

# Project B API key
[project-b]
description = "Project B - Gemini API"
base_url = "https://generativelanguage.googleapis.com/v1beta"
auth_token = "AIzaSyProjectB_key_here"
model = "gemini-1.5-pro"

# Switch between projects
ccr switch project-a  # Work on Project A
ccr switch project-b  # Work on Project B
```

### Testing Free Quota

```bash
# Use temporary API key (doesn't modify profiles.toml)
ccr temp-token set AIzaSyTestKeyXXXXXXXXXXXXXXXXXXXXXXXX \
  --base-url https://generativelanguage.googleapis.com/v1beta \
  --model gemini-1.5-flash

# Verify temporary config
ccr temp-token show

# Apply and auto-clear
ccr switch google-official

# Next switch uses permanent config
ccr switch google-official
```

## Platform-Specific Features

### API Key Validation

CCR validates Gemini API keys automatically:

```bash
# Validate all Gemini profiles
ccr validate

# Output includes:
# ✅ Valid Google API key format (AIzaSy...)
# ❌ Invalid API key format
# ⚠️ API key format correct but not verified active
```

### Backup and Restore

```bash
# Automatic backup before profile switch
ccr switch new-profile
# → Creates ~/.ccr/gemini/backups/settings_20250125_120000.json.bak

# Manual backup
ccr backup gemini

# List backups
ccr backups list

# Restore from backup
ccr restore ~/.ccr/gemini/backups/settings_20250125_120000.json.bak
```

### History Tracking

```bash
# View Gemini-specific history
ccr history --platform gemini

# View recent 10 operations
ccr history -l 10

# Filter by operation type
ccr history -t switch
```

## API Endpoints

### Google AI Studio (Free Tier)

```toml
base_url = "https://generativelanguage.googleapis.com/v1beta"
```

**Characteristics:**
- Free tier with rate limits
- Good for development and testing
- Easier setup (no GCP project required)

### Google Cloud Vertex AI (Production)

```toml
base_url = "https://us-central1-aiplatform.googleapis.com/v1"
# Or other regions:
# base_url = "https://europe-west1-aiplatform.googleapis.com/v1"
# base_url = "https://asia-northeast1-aiplatform.googleapis.com/v1"
```

**Characteristics:**
- Production-grade SLA
- Advanced quotas
- Regional deployment
- Requires GCP billing account

## Migration Guide

### From Claude to Gemini

```bash
# Initialize Gemini platform
ccr platform init gemini

# Migrate compatible profiles (requires manual API key replacement)
ccr platform migrate claude gemini

# Update API keys manually
vim ~/.ccr/gemini/profiles.toml
```

### From Manual Configuration

If you were manually managing Gemini settings:

```bash
# Old manual setup
export GOOGLE_API_KEY="AIzaSy..."
export GEMINI_MODEL="gemini-pro"

# Migrate to CCR
ccr platform switch gemini
ccr add  # Enter API key and model

# CCR now manages settings automatically
```

## Troubleshooting

### Issue: API Key Invalid

**Symptoms:**
```
❌ Invalid Google API key format
```

**Solution:**
1. Verify key starts with `AIzaSy` and is 39 characters
2. Regenerate key on Google AI Studio or Cloud Console
3. Update profile with new key

### Issue: Quota Exceeded

**Symptoms:**
```
HTTP 429: Resource exhausted
```

**Solution:**
1. Check quota limits in Google Cloud Console
2. Wait for quota reset (usually per minute)
3. Upgrade to paid tier for higher quotas
4. Switch to different profile/project with available quota

### Issue: Model Not Available

**Symptoms:**
```
HTTP 404: Model not found
```

**Solution:**
1. Verify model name spelling
2. Check model availability in your region
3. Ensure API is enabled in Google Cloud Console
4. Use stable model names (avoid experimental suffixes if needed)

### Issue: Settings Not Updating

**Symptoms:**
Profile switch succeeds but `~/.gemini/settings.json` unchanged

**Solution:**
```bash
# Check file permissions
ls -la ~/.gemini/settings.json

# Fix permissions if needed
chmod 600 ~/.gemini/settings.json

# Verify lock files
ls -la ~/.ccr/gemini/.locks/

# Clean stale locks if present
rm -rf ~/.ccr/gemini/.locks/*
```

## Advanced Configuration

### Custom API Endpoints

```toml
[vertex-custom]
description = "Vertex AI Custom Endpoint"
base_url = "https://custom-endpoint.googleapis.com/v1"
auth_token = "AIzaSy..."
model = "gemini-2.0-flash-exp"
```

### Model Parameters

```toml
[creative]
description = "Creative Writing (high temperature)"
auth_token = "AIzaSy..."
model = "gemini-1.5-pro"
# Note: Temperature and other parameters are typically
# set at API call time, not in CCR config
```

### Multi-Region Setup

```toml
[us-central]
description = "US Central Region"
base_url = "https://us-central1-aiplatform.googleapis.com/v1"
auth_token = "AIzaSy..."
model = "gemini-2.0-flash-exp"

[europe]
description = "Europe West Region"
base_url = "https://europe-west1-aiplatform.googleapis.com/v1"
auth_token = "AIzaSy..."
model = "gemini-2.0-flash-exp"

[asia]
description = "Asia Northeast Region"
base_url = "https://asia-northeast1-aiplatform.googleapis.com/v1"
auth_token = "AIzaSy..."
model = "gemini-2.0-flash-exp"
```

### WebDAV Sync

```bash
# Configure sync for Gemini profiles
ccr sync config

# Push Gemini profiles to cloud
ccr platform switch gemini
ccr sync push

# Pull on another machine
ccr platform switch gemini
ccr sync pull
```

## Security Best Practices

1. **API Key Storage**: API keys are stored in plaintext in `~/.ccr/gemini/profiles.toml`
   ```bash
   # Ensure proper file permissions
   chmod 600 ~/.ccr/gemini/profiles.toml
   ```

2. **API Key Masking**: CCR automatically masks keys in:
   - Console output
   - History logs
   - Error messages

3. **Backup Security**: Backups also contain API keys
   ```bash
   # Secure backup directory
   chmod 700 ~/.ccr/gemini/backups
   ```

4. **Export Without Secrets**:
   ```bash
   # Export profiles without API keys (for sharing)
   ccr export -o gemini-profiles.toml --no-secrets
   ```

5. **API Key Rotation**: Regularly rotate Google API keys
   ```bash
   # Delete old key on Google Cloud Console
   # Create new key
   # Update profile
   vim ~/.ccr/gemini/profiles.toml
   ccr validate  # Verify format
   ```

6. **API Key Restrictions** (Recommended):
   - Restrict by API (only allow Generative Language API)
   - Restrict by IP address (if using from fixed location)
   - Restrict by HTTP referrer (if using from web app)

## Related Commands

```bash
# Platform management
ccr platform list           # List all platforms
ccr platform switch gemini  # Switch to Gemini
ccr platform current        # Show current platform

# Profile management
ccr list                    # List Gemini profiles
ccr switch <name>           # Switch Gemini profile
ccr add                     # Add new profile
ccr delete <name>           # Delete profile

# Validation and diagnostics
ccr validate                # Validate all profiles
ccr history                 # View operation history

# Backup and restore
ccr backups list            # List backups
ccr restore <file>          # Restore from backup
```

## See Also

- [Migration Guide](./migration.md) - Migrating between platforms
- [Codex Platform Guide](./codex.md) - GitHub Copilot CLI configuration
- [Main README](../../README.md) - CCR overview
- [Google AI Studio](https://makersuite.google.com/) - Get API keys
- [Gemini API Docs](https://ai.google.dev/docs) - Official Gemini API documentation
- [Vertex AI Docs](https://cloud.google.com/vertex-ai/docs) - Google Cloud Vertex AI documentation
