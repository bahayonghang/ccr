# CCR - Claude Code Configuration Switcher

🚀 **Configuration Management Tool for Claude Code (Rust Implementation)**

CCR is the Rust implementation of Claude Code Configuration Switcher (CCS), providing powerful configuration management features including complete audit trails, file locking mechanisms, and automatic backup/restore capabilities.

## ✨ Core Features

### 🎯 Direct Claude Code Settings Manipulation
- Directly operates on `~/.claude/settings.json` file
- No manual environment variable configuration needed
- Configuration takes effect immediately

### 🔐 Concurrency Safety
- File locking mechanism ensures multi-process safety
- Atomic write operations prevent data corruption
- Timeout protection avoids deadlocks

### 📝 Complete Audit Trail
- Records all operation history
- Tracks environment variable changes
- Automatic masking of sensitive information

### 💾 Automatic Backup & Recovery
- Automatic backup before switching
- Support for restoration from backups
- Timestamped backup files

### ✅ Configuration Validation
- Automatic configuration integrity validation
- Checks required fields
- URL format validation

### 🌐 Web Interface
- Browser-based configuration management
- RESTful API support
- Real-time configuration switching

### 🔄 Full CCS Compatibility
- Shares `~/.ccs_config.toml` configuration file
- Consistent command-line interface
- Can coexist with CCS

## 📦 Installation

### Quick Install (Recommended)

Install CCR directly from GitHub using cargo:

```bash
cargo install --git https://github.com/bahayonghang/ccr
```

After installation, the `ccr` command will be available in your PATH.

### Build from Source

```bash
# Clone the repository
cd ccs/ccr

# Build release version
cargo build --release

# Install to system path (optional)
cargo install --path .
```

### Run the Program

```bash
# Run directly
cargo run -- <command>

# Or use compiled binary
./target/release/ccr <command>
```

## 🚀 Quick Start

### 1. Initialize Configuration File

Initialize CCR configuration file with example template:

```bash
ccr init
```

This will create `~/.ccs_config.toml` with example configurations. You can also use an existing CCS configuration if you have one.

Example configuration file:

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

### 2. View Available Configurations

```bash
ccr list
# or
ccr ls
```

### 3. Switch Configuration

```bash
ccr switch anthropic
# or shorthand
ccr anthropic
```

### 4. View Current Status

```bash
ccr current
# or
ccr status
```

### 5. Validate Configuration

```bash
ccr validate
# or
ccr check
```

### 6. View History

```bash
ccr history
# Limit display count
ccr history --limit 10
# Filter by type
ccr history -t switch
```

### 7. Launch Web Interface

```bash
ccr web
# Specify port
ccr web --port 8080
```

### 8. Update to Latest Version

```bash
# Check for updates
ccr update --check

# Update to latest version
ccr update
```

### 9. Export and Import Configurations

```bash
# Export configuration (includes API keys by default)
ccr export

# Export without secrets
ccr export --no-secrets

# Export to specific file
ccr export -o my-config.toml

# Import configuration (merge mode)
ccr import config.toml --merge

# Import configuration (replace mode)
ccr import config.toml
```

### 10. Clean Old Backups

```bash
# Clean backups older than 7 days (default)
ccr clean

# Clean backups older than 30 days
ccr clean --days 30

# Dry run (preview without deleting)
ccr clean --dry-run
```

## 📚 Command Reference

### init
Initialize configuration file from template

```bash
ccr init
```

Features:
- Creates `~/.ccs_config.toml` from embedded template
- **Safe mode**: Refuses to overwrite existing config without --force
- Automatic backup when using --force
- Sets proper file permissions (Unix: 644)
- Provides helpful hints on next steps

Behavior:
- If config exists: Shows warning and exits (safe)
- With `--force`: Backs up and overwrites existing config

Options:
```bash
ccr init --force    # Force overwrite with automatic backup
```

### list / ls
List all available configurations, marking the current configuration and validation status

```bash
ccr list
```

Output example:
```
Available Configurations
════════════════════════════════════════════════════════════════
Configuration File: /home/user/.ccs_config.toml
Default Config: anthropic
Current Config: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic Official API
    Base URL: https://api.anthropic.com
    Token: sk-a...key
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    Status: ✓ Configuration Complete
  anyrouter - AnyRouter Proxy Service

✓ Found 2 configurations
```

### current / show / status
Display detailed status of current configuration, including environment variables

```bash
ccr current
```

### switch <config>
Switch to specified configuration

```bash
ccr switch anyrouter
```

Execution flow:
1. ✓ Read and validate target configuration
2. ✓ Backup current Claude Code settings
3. ✓ Update `~/.claude/settings.json`
4. ✓ Update configuration file `current_config`
5. ✓ Record operation history

### validate / check
Validate configuration and settings integrity

```bash
ccr validate
```

Checks:
- Configuration file format
- Completeness of all configuration sections
- Claude Code settings file
- Required environment variables

### history
Display operation history

```bash
# Default: show last 20 entries
ccr history

# Custom count
ccr history --limit 50

# Filter by type
ccr history -t switch   # Only show switch operations
ccr history -t backup   # Only show backup operations
```

### web
Launch web configuration interface

```bash
# Default port 8080
ccr web

# Specify port
ccr web --port 3000
```

### update
Update CCR to the latest version from GitHub

```bash
# Check what will be updated
ccr update --check

# Update to latest version
ccr update
```

Features:
- Uses `cargo install --git` to get the latest code
- Always gets the latest commit from GitHub
- Requires Rust toolchain (cargo)
- Automatic confirmation before updating

Requirements:
- Rust and Cargo must be installed
- Network access to GitHub

Equivalent to running:
```bash
cargo install --git https://github.com/bahayonghang/ccr --force
```

### export
Export configuration to a file

```bash
# Export with full API keys (default)
ccr export

# Export without secrets (masked tokens)
ccr export --no-secrets

# Export to specific file
ccr export -o backup.toml
```

Features:
- Automatic timestamped filename
- Includes API keys by default for easy migration
- Optional secret masking with --no-secrets flag
- TOML format for easy editing
- Perfect for backup and migration

### import
Import configuration from a file

```bash
# Merge mode (preserve existing configs, add new ones)
ccr import config.toml --merge

# Replace mode (completely replace current config)
ccr import config.toml

# Import without backup
ccr import config.toml --no-backup
```

Features:
- Merge or replace modes
- Automatic backup before import
- Configuration validation
- Detailed import summary

### clean
Clean old backup files to free up disk space

```bash
# Clean backups older than 7 days (default)
ccr clean

# Clean backups older than 30 days
ccr clean --days 30

# Dry run (preview without deleting)
ccr clean --dry-run
```

Features:
- Automatic cleanup of old backup files
- Configurable retention period (default: 7 days)
- Dry run mode for preview
- Shows freed disk space
- Only removes `.bak` files in `~/.claude/backups/`

Options:
```bash
ccr clean --days 14      # Clean backups older than 14 days
ccr clean --dry-run      # Preview cleanup without deleting
```

### version / ver
Display version information and help

```bash
ccr version
```

## 📁 File Structure

CCR uses the following files and directories:

```
~/.ccs_config.toml          # Configuration file (shared with CCS)
~/.claude/settings.json     # Claude Code settings file
~/.claude/backups/          # Automatic backup directory
~/.claude/ccr_history.json  # Operation history log
~/.claude/.locks/           # File lock directory
```

## 🔧 Advanced Features

### Environment Variable Management

CCR manages the following environment variables:

- `ANTHROPIC_BASE_URL` - API endpoint address
- `ANTHROPIC_AUTH_TOKEN` - Authentication token
- `ANTHROPIC_MODEL` - Default model
- `ANTHROPIC_SMALL_FAST_MODEL` - Fast small model (optional)

When switching configurations, CCR will:
1. Clear all environment variables with `ANTHROPIC_*` prefix
2. Set new environment variables based on target configuration
3. Keep other settings unchanged

### Backup & Recovery

Automatic backup:
- Automatic backup before each configuration switch
- Backup files include timestamp and configuration name
- Stored in `~/.claude/backups/` directory

Manual recovery:
```bash
# List available backups
ls ~/.claude/backups/

# Restore from backup (use settings manager API)
# Command-line restore support coming soon
```

### History Records

History records include:
- Operation ID (UUID)
- Timestamp
- Actor (system username)
- Operation type
- Environment variable changes (masked)
- Operation result
- Notes

### Web API

CCR provides RESTful API for programmatic access:

```bash
# List configurations
GET /api/configs

# Switch configuration
POST /api/switch
Body: {"config_name": "anthropic"}

# Get history
GET /api/history

# Validate configuration
POST /api/validate

# Clean backups
POST /api/clean
Body: {"days": 7, "dry_run": false}

# Add/Update/Delete configuration
POST /api/config
PUT /api/config/{name}
DELETE /api/config/{name}
```

### Logging & Debugging

Set log level:

```bash
# Set environment variable
export CCR_LOG_LEVEL=debug  # trace, debug, info, warn, error

# Run command
ccr switch anthropic
```

## 🔒 Security Features

### Sensitive Information Protection
- API tokens automatically masked for display
- Sensitive values desensitized in history records
- Only shows first and last characters of tokens

### File Permissions
- Settings file permissions set to 600 (owner read/write only)
- Lock files automatically cleaned up
- Atomic operations avoid race conditions

### Concurrency Control
- Cross-process file locking
- Timeout protection (default 10 seconds)
- Automatic lock resource release

## 🆚 CCR vs CCS

| Feature | CCS (Shell) | CCR (Rust) |
|---------|------------|-----------|
| Configuration Switching | ✅ | ✅ |
| Environment Variable Setting | ✅ | ✅ |
| Direct settings.json Write | ❌ | ✅ |
| File Locking Mechanism | ❌ | ✅ |
| Operation History | ❌ | ✅ |
| Automatic Backup | ❌ | ✅ |
| Configuration Validation | Basic | Complete |
| Concurrency Safety | ❌ | ✅ |
| Web Interface | ❌ | ✅ |
| Performance | Fast | Extremely Fast |

## 🤝 CCS Compatibility

CCR is fully compatible with CCS:

1. **Shared Configuration File** - Uses the same `~/.ccs_config.toml`
2. **Seamless Switching** - Can alternate between CCS and CCR commands
3. **Consistent Commands** - Core commands remain consistent
4. **Coexistence** - Both can be installed simultaneously

## 📝 Development

### Project Structure

```
ccr/
├── src/
│   ├── main.rs          # Main program entry
│   ├── error.rs         # Error handling
│   ├── logging.rs       # Logging & colored output
│   ├── lock.rs          # File locking
│   ├── config.rs        # Configuration management
│   ├── settings.rs      # Settings management
│   ├── history.rs       # History records
│   ├── web.rs           # Web server
│   └── commands/        # CLI commands
│       ├── mod.rs
│       ├── list.rs
│       ├── current.rs
│       ├── switch.rs
│       ├── validate.rs
│       └── history_cmd.rs
├── web/                 # Web interface HTML
├── Cargo.toml           # Project configuration
└── README.md            # This file
```

### Run Tests

```bash
cargo test
```

### Code Checks

```bash
cargo check
cargo clippy
```

### Formatting

```bash
cargo fmt
```

## 🐛 Troubleshooting

### Configuration File Not Found

```bash
# Check configuration file
ls -la ~/.ccs_config.toml

# If not exists, install CCS first or create manually
```

### Claude Code Settings File Not Found

```bash
# Check Claude Code directory
ls -la ~/.claude/

# Will be created automatically on first use
ccr switch <config>
```

### File Lock Timeout

```bash
# Check for zombie processes
ps aux | grep ccr

# Clean lock files (use with caution)
rm -rf ~/.claude/.locks/*
```

### Permission Issues

```bash
# Check file permissions
ls -la ~/.claude/settings.json
ls -la ~/.ccs_config.toml

# Fix permissions
chmod 600 ~/.claude/settings.json
chmod 644 ~/.ccs_config.toml
```

## 🛣️ Roadmap

- [x] Web interface support
- [x] RESTful API
- [x] Online update functionality
- [x] Configuration import/export
- [x] Configuration initialization
- [ ] Configuration template system
- [ ] More statistics and reports
- [ ] Cross-platform installation packages

## 📄 License

MIT License

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📮 Contact

- GitHub: https://github.com/bahayonghang/ccr
- Project Home: https://github.com/bahayonghang/ccs/tree/main/ccr

---

**Note**: CCR is currently in active development. Thorough testing is recommended before use in production environments.

