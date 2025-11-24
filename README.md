# CCR - Claude Code Configuration Switcher

High-performance multi-platform configuration management tool written in Rust, version 3.6.2. Supports Claude Code, Codex, Gemini CLI, Qwen. Offers CLI, TUI, Web API interfaces, plus a full-stack CCR UI application built with Vue 3 + Axum + Tauri.

## Highlights

- **Safe Write Mechanism**: Atomic writes + file locking + audit trails + auto backups for `settings.json` and config files
- **Unified Multi-Platform Mode**: Uses `~/.ccr/` by default (platform-specific profiles/history/backups), compatible with legacy `~/.ccs_config.toml` single-file mode
- **Complete Config Lifecycle**: `init`, `add`, `list/current/switch`, `enable/disable`, `validate`, `history`, `optimize`, with `--yes` to skip prompts
- **Multi-Platform Support**: Unified management of Claude, Codex, Gemini, Qwen, iFlow platforms with independent configs, history, and backups
- **Import/Export/Clean**: Export with/without secrets, import with merge/replace modes and auto backup, clean old backups
- **WebDAV Multi-Folder Sync** (`web` feature): Supports multiple folder management with individual push/pull/status
- **Platform Management**: `ccr platform list/switch/current/info/init` with optional JSON output for scripting
- **Temp Tokens & Updates**: `ccr temp-token set/show/clear`, `ccr update --check` for version checks/updates
- **Observability**: Complete history, JSON output options, cost statistics (`ccr stats ...`, `web` feature)
- **TUI Interface**: Ratatui-powered interactive terminal UI for config management, platform switching, and sync status
- **Skills & Prompts Management**: `ccr skills` and `ccr prompts` for AI skills and prompt management
- **Multiple Interfaces**: CLI everywhere, optional TUI (`--features tui`), legacy lightweight API (`ccr web`), modern CCR UI (`ccr ui`, Vue 3 + Axum backend, Tauri desktop build)

## Installation

Requirements: Rust 1.85+ (Edition 2024), Cargo. For CCR UI development: Node.js 18+ (npm) and optionally `just`.

### One-Line Install
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### From Source
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

### Build Options
```bash
# CLI only (fastest)
cargo build --no-default-features

# CLI + Web API + WebDAV sync + UI entrypoint
cargo build --features web

# CLI + TUI terminal interface
cargo build --features tui

# All features (recommended)
cargo build --all-features

# Workspace build and test
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Workspace Structure

```
ccr/
|-- src/                # CLI + shared library (99 files, ~28K lines)
|   |-- commands/       # 27 CLI command implementations
|   |-- services/       # 7 business service layers
|   |-- managers/       # 11 data managers
|   |-- platforms/      # 5 AI platform support
|   |-- sync/           # WebDAV multi-folder sync
|   |-- web/            # Axum Web API (14 endpoints)
|   |-- tui/            # Ratatui terminal interface
|   |-- models/         # Data model definitions
|   |-- core/           # Infrastructure (errors, locks, logging)
|   `-- utils/          # Validation, masking utilities & more
|
|-- ccr-ui/             # Full-stack web application
|   |-- backend/        # Axum backend server (workspace member)
|   `-- frontend/       # Vue 3 + Vite + Pinia + Tauri
|
|-- tests/              # Integration tests (6 test files)
|-- docs/               # VitePress documentation site
|-- examples/           # Examples and demos
`-- justfile            # Common development tasks
```

## CLI Quick Start

### 1. Initialize Configuration

**Unified Multi-Platform Mode** (default):
```bash
ccr init
```
Creates `~/.ccr/config.toml` and platform directories under `~/.ccr/platforms/`.

**Legacy Single-File Mode**:
```bash
export CCR_LEGACY_MODE=1
ccr init
```
Continue using single-file `~/.ccs_config.toml`.

### 2. View and Switch Platforms

```bash
# List all available platforms
ccr platform list

# Switch to a platform (claude/codex/gemini/qwen/iflow)
ccr platform switch claude

# Show current platform and config
ccr platform current

# Show platform details
ccr platform info claude
```

### 3. Create and Manage Configs

```bash
# Interactive config creation
ccr add

# List all configs for current platform
ccr list

# Show current config
ccr current

# Switch configs (two ways)
ccr switch <name>
ccr <name>           # Shortcut

# Enable/disable configs
ccr enable <name>
ccr disable <name> [--force]

# Validate config file integrity
ccr validate

# View history (last 50 entries)
ccr history -l 50

# Optimize config sorting (alphabetical)
ccr optimize
```

### 4. Import/Export/Clean

```bash
# Export config (optional secrets removal)
ccr export -o configs.toml --no-secrets

# Import config (supports merge/replace modes)
ccr import configs.toml --merge --backup

# Clean old backups older than 30 days (dry run)
ccr clean --days 30 --dry-run

# Actually clean
ccr clean --days 30
```

### 5. WebDAV Multi-Folder Sync (requires `web` feature)

#### Configure WebDAV
```bash
# Configure WebDAV connection info
ccr sync config

# System creates default folders:
# - claude: ~/.claude/ → /ccr/claude
# - gemini: ~/.gemini/ → /ccr/gemini
# - conf: ~/.ccs_config.toml → /ccr/config.toml
```

#### Folder Management
```bash
# List all sync folders
ccr sync folder list

# Add custom sync folder
ccr sync folder add scripts ~/my-scripts \
  -r /ccr/scripts \
  -d "My custom scripts"

# Show folder details
ccr sync folder info claude

# Enable/disable sync
ccr sync folder enable claude
ccr sync folder disable gemini
```

#### Sync Operations
```bash
# Sync specific folders
ccr sync claude push      # Upload Claude config
ccr sync gemini pull      # Download Gemini config
ccr sync conf status      # Check config status

# Batch sync all enabled folders
ccr sync all push --force    # Upload all configs
ccr sync all pull --force    # Download all configs
ccr sync all status          # View all status

# Legacy command compatibility
ccr sync push        # Equivalent to: ccr sync all push
ccr sync pull        # Equivalent to: ccr sync all pull
ccr sync status      # Show all status
```

### 6. Platform Management

```bash
# Initialize a platform (if not exists)
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# Platform switching workflow
ccr platform switch claude    # Switch to Claude platform
ccr list                       # Show Claude platform configs
ccr add                        # Add config for Claude
ccr platform switch codex      # Switch to Codex platform
ccr list                       # Show Codex platform configs
```

### 7. Migration

```bash
# Check if migration from Legacy to Unified is needed
ccr migrate --check

# Migrate all platforms
ccr migrate

# Migrate specific platform
ccr migrate --platform claude
```

### 8. Temporary Credential Management

```bash
# Set temporary override credentials (doesn't modify config files)
ccr temp-token set sk-ant-api03-xxxx \
  --base-url https://api.anthropic.com \
  --model claude-sonnet-4-5-20250929

# Show current temporary credentials
ccr temp-token show

# Clear temporary credentials
ccr temp-token clear
```

### 9. Skills & Prompts Management

```bash
# Skills management
ccr skills list                  # List skills
ccr skills scan ~/skills         # Scan skills directory
ccr skills install ~/skills/<skill>  # Install skill

# Prompts management
ccr prompts list                 # List prompts
ccr prompts add                  # Add prompt
ccr prompts apply <name>         # Apply prompt
```

### 10. Statistics (requires `web` feature)

```bash
# Cost statistics
ccr stats cost --today           # Today's cost
ccr stats cost --by-model        # By model
ccr stats cost --this-month      # This month cost
```

### 11. Interfaces & Services

```bash
# Launch full UI (Vue 3 + Axum)
# Auto-detects: workspace → ~/.ccr/ccr-ui → GitHub download
ccr ui -p 3000 --backend-port 8081

# Launch TUI (requires `tui` feature)
ccr tui

# Launch lightweight Web API Server (compatibility mode)
ccr web -p 8080
```

### 12. System Commands

```bash
# Check for conflicts
ccr check conflicts

# Auto update
ccr update --check     # Check updates
ccr update             # Update to latest version

# Show version info
ccr version
```

## TUI Terminal Interface User Guide

### Launch TUI
```bash
ccr tui          # Basic mode
ccr tui --yes    # YOLO mode (auto-confirm)
```

### Keyboard Shortcuts

#### Navigation
- `Tab` / `Shift+Tab` - Switch tabs
- `1-4` - Quick jump to tabs 1-4
- `↑↓` / `j/k` - Navigate up/down in lists
- `Enter` - Select/switch config
- `PgUp` / `PgDn` - Page up/down

#### Actions
- `d` - Delete selected config (requires YOLO mode)
- `Y` - Toggle YOLO mode (auto-confirm dangerous ops)
- `Ctrl+C` / `q` - Exit TUI

### Tab Descriptions

#### Tab 1: Config List
- Shows all configs for the current platform
- Green highlight indicates active config
- Supports switch and delete operations

#### Tab 2: Platform Management
- Shows all available platforms (Claude/Codex/Gemini/Qwen/iFlow)
- Highlights current platform
- Supports quick platform switching

#### Tab 3: Sync Status
- Shows sync status for all configs
- Supports status refresh

#### Tab 4: System Information
- Shows CCR version and build info
- Shows system resource usage
- Shows path locations

## Web API Documentation

### Start Web Server
```bash
ccr web                # Default port 8080
ccr web -p 8080        # Specify port
```

### API Endpoints

#### Config Management
```
GET    /api/configs           # Get all configs list
POST   /api/switch            # Switch config
       { "config_name": "anthropic" }

POST   /api/config            # Create new config
       { "name": "anthropic", "config": {...} }

POST   /api/config/{name}     # Update config
       { "description": "...", "base_url": "...", ... }

DELETE /api/config/{name}     # Delete config
```

#### Settings Management
```
GET    /api/settings          # Get current Claude settings.json
GET    /api/settings/backups  # List backup files
POST   /api/settings/restore  # Restore from backup
       { "backup_path": "~/.claude/backups/xxx.json.bak" }
```

#### History
```
GET    /api/history           # Get operation history
       ?limit=50&type=switch
```

#### Operations
```
POST   /api/validate          # Validate config integrity
POST   /api/export            # Export config
       { "output_path": "~/config.toml", "no_secrets": true }

POST   /api/import            # Import config
       { "input_path": "~/config.toml", "mode": "merge" }

POST   /api/clean             # Clean old backups
       { "days": 30, "dry_run": true }
```

#### System Info
```
GET    /api/system            # Get system information
       ?cached=false
```

#### Sync Management (requires `web` feature)
```
POST   /api/sync/folder/{name}/push    # Upload folder
POST   /api/sync/folder/{name}/pull    # Download folder
GET    /api/sync/folder/{name}/status  # Check status
POST   /api/sync/all/push              # Batch upload
POST   /api/sync/all/pull              # Batch download
GET    /api/sync/all/status            # Batch status
```

### Response Format

All APIs return JSON with `success`, `data`, and `message` fields:

```json
{
  "success": true,
  "data": { ... },
  "message": "Operation successful"
}
```

Error response:
```json
{
  "success": false,
  "error": {
    "type": "ConfigNotFound",
    "message": "Config 'xxx' not found"
  }
}
```

### CORS Support

Web API supports CORS by default, allowing requests from any origin. Useful for frontend/backend separation in development.

## CCR UI (Vue 3 + Axum + Tauri)

CCR UI is a full-stack web application providing complete visual management of all CCR features.

### Features

- **Visual Management**: Config management, validation, history, and backups
- **Command Executor**: Visual execution of all CLI commands with real-time output
- **WebDAV Sync Dashboard**: Multi-folder management (add/enable/disable/push/pull/status/batch)
- **System Information**: Platform overview, health checks, resource monitoring
- **Multi-Interface Support**: Web mode (HTTP API) and Desktop mode (Tauri)

### Launch via CLI (Recommended)

```bash
ccr ui                          # Auto-detects:
                                # 1. workspace ccr-ui
                                # 2. ~/.ccr/ccr-ui directory
                                # 3. GitHub auto-download

# Custom ports
ccr ui -p 3000 --backend-port 8081
```

**Default Ports**:
- Frontend: 3000
- Backend API: 8081

### Develop from Repository

First time:
```bash
cd ccr-ui
just quick-start           # Prerequisites check + install + start
```

Development mode:
```bash
cd ccr-ui
just s                     # Start frontend + backend in dev mode
```

Manual (explicit commands):
```bash
# Backend (workspace member, port 8081)
cd ccr-ui/backend
cargo run -- --port 8081

# Frontend (new terminal)
cd ../frontend
npm install
npm run dev                # http://localhost:5173
```

### Production Build

```bash
cd ccr-ui
just build                 # Build backend + frontend
just run-prod              # Start compiled backend with built frontend
```

### Desktop (Tauri)

```bash
cd ccr-ui
just tauri-dev             # Desktop dev window
just tauri-build           # Package desktop installer
```

## Advanced Features

### Config Mode Details

CCR supports two modes to meet different needs:

#### 1. Legacy Mode (Single Platform)
For users who only use Claude Code:
```
~/.ccs_config.toml         # All configs in one file
~/.claude/                  # Claude config directory
  -- settings.json
  -- ccr_history.json
  -- backups/
```

#### 2. Unified Mode (Multi-Platform, Recommended)
For users of multiple AI CLI tools:
```
~/.ccr/
├── config.toml            # Platform registry
├── sync_folders.toml      # Sync configuration
└── platforms/
    ├── claude/
    │   ├── profiles.toml
    │   ├── history/
    │   └── backups/
    ├── codex/
    │   ├── profiles.toml
    │   ├── history/
    │   └── backups/
    └── gemini/
        ├── profiles.toml
        ├── history/
        └── backups/
```

**Switch Mode**:
```bash
export CCR_LEGACY_MODE=1    # Legacy mode
unset CCR_LEGACY_MODE       # Unified mode
```

### Environment Variables

#### Runtime Configuration
```bash
export CCR_LOG_LEVEL=debug  # Log level: trace/debug/info/warn/error
export CCR_LEGACY_MODE=1    # Enable legacy mode
export CCR_ROOT=~/.custom    # Custom config root directory
```

#### Claude Code Settings (in settings.json)
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-api03-xxxx",
    "ANTHROPIC_MODEL": "claude-sonnet-4-5-20250929",
    "ANTHROPIC_SMALL_FAST_MODEL": "claude-3-5-haiku-20241022"
  }
}
```

### Concurrency Safety

CCR provides enterprise-grade concurrency protection:

1. **File Locking**: Advisory locks based on `fs4` to prevent simultaneous modifications
2. **Atomic Writes**: All file modifications use temp file + atomic rename
3. **Auto Backup**: Auto backup before every modification, retained for 7 days
4. **Audit Logs**: All operations logged to history with UUID, timestamp, and actor

### Error Handling

CCR provides detailed error messages and exit codes:

| Error Type | Exit Code | Description |
|-----------|-----------|-------------|
| Success | 0 | Operation successful |
| Generic Error | 1 | Uncategorized error |
| Config Not Found | 2 | Config file or entry not found |
| Validation Failed | 3 | Config validation failed |
| File Lock Error | 4 | Cannot acquire file lock (may be in use) |
| IO Error | 5 | File read/write error |
| Settings Error | 6 | settings.json format or permission error |
| Operation Cancelled | 7 | User cancelled operation (Ctrl+C) |

### Sensitive Data Protection

CCR automatically masks sensitive information in all outputs:

**Auto Masking**:
- API Keys: `sk-ant-api03-abcde...6789` → `sk-ant-********`
- Tokens: `ghp_abcdefghijklmnop` → `ghp_********`
- Passwords: Fully masked

**Config Export**:
```bash
ccr export --no-secrets    # Remove all sensitive info
```

## Development Guide

### Quick Checks

```bash
# Code checks
cargo check
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
cargo test --workspace

# Platform tests (must run serially)
cargo test --test platform_tests -- --test-threads=1
```

### just Commands

Project includes `justfile` for streamlined development:

```bash
just dev            # Dev mode compilation
just watch          # Watch files and auto-compile
just ci             # Full CI pipeline (check + test)
just build          # Build release version
just release        # Release build (with optimizations)

cd ccr-ui
just s              # CCR UI dev mode
just build          # Build UI
just tauri-dev      # Tauri desktop dev
```

### Testing Strategy

CCR has 95%+ test coverage:

**Tested Components**:
- **Unit Tests**: Embedded in modules with `#[cfg(test)]`
- **Integration Tests**: 6 test files in `tests/` directory
- **Concurrent Tests**: `concurrent_tests.rs` validates file locking
- **E2E Tests**: `end_to_end_tests.rs` complete workflows

**Run Tests**:
```bash
# All tests
cargo test

# Specific test file (platform tests must be serial)
cargo test --test platform_integration_tests -- --test-threads=1
cargo test --test concurrent_tests

# With output
cargo test -- --nocapture

# Coverage report (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Troubleshooting

### Enable Debug Logging

```bash
export CCR_LOG_LEVEL=debug    # Set debug level
ccr switch anthropic          # Run command with detailed logs

# Log levels:
# - trace: Most detailed, includes all function calls
# - debug: Debug info, includes file operations
# - info: Basic info (default)
# - warn: Warning messages
# - error: Error messages only
```

### Common Issues

#### 1. Lock Timeout Error
**Symptom**: `Lock acquisition timeout: config`

**Cause**: Another CCR process is using the config file

**Fix**:
```bash
# Check processes
ps aux | grep ccr

# Clean lock files (use with caution)
rm -rf ~/.claude/.locks/*
```

#### 2. Permission Error
**Symptom**: `Permission denied` reading/writing config files

**Fix**:
```bash
# Ensure files are owned by current user
sudo chown -R $USER:$USER ~/.claude/
sudo chown -R $USER:$USER ~/.ccr/

# Check file permissions
ls -la ~/.claude/settings.json
ls -la ~/.ccr/config.toml
```

#### 3. Config File Corruption
**Symptom**: `ValidationError: invalid TOML structure`

**Fix**:
```bash
# CCR auto-creates backups, find the most recent one
ls -lt ~/.claude/backups/*.bak | head -5

# Manual restore
cp ~/.claude/backups/config_20250101_120000.toml.bak ~/.ccs_config.toml

# Or use CCR restore
ccr history -t backup  # View backup history
ccr import ~/.claude/backups/xxx.toml --merge
```

#### 4. CCR UI Download Failure
**Symptom**: `ccr ui` fails to download UI

**Cause**: Network issues or GitHub API limits

**Fix**:
```bash
# Manual clone
mkdir -p ~/.ccr
cd ~/.ccr
git clone https://github.com/bahayonghang/ccr.git
cd ccr
git checkout v3.6.2
mv ccr-ui ~/.ccr/

# Launch from workspace
cd /path/to/ccr/ccr-ui
ccr ui
```

#### 5. Sync Conflicts
**Symptom**: `CONFLICT: Remote file newer than local`

**Fix**:
```bash
# Force push (overwrite remote)
ccr sync claude push --force

# Force pull (overwrite local)
ccr sync claude pull --force

# Interactive selection
ccr sync claude push --interactive
```

#### 6. Legacy Config Migration
**Symptom**: Need to migrate all configs to Unified mode

**Fix**:
```bash
# Check what can be migrated
ccr migrate --check

# Auto-migrate all platforms
ccr migrate

# Migrate specific platform
ccr migrate --platform claude
ccr migrate --platform codex

# Verify migration result
ccr platform list
ccr platform switch claude
ccr list
```

### WebDAV Sync Configuration Examples

#### Nutstore (坚果云)
```toml
[webdav]
url = "https://dav.jianguoyun.com/dav/"
username = "your-email@example.com"
password = "your-app-password"  # App password, NOT login password
base_remote_path = "/ccr"
```

#### Nextcloud
```toml
[webdav]
url = "https://cloud.example.com/remote.php/dav/files/username/"
username = "username"
password = "your-password"
base_remote_path = "/ccr"
```

#### ownCloud
```toml
[webdav]
url = "https://owncloud.example.com/remote.php/dav/files/username/"
username = "username"
password = "your-password"
base_remote_path = "/ccr"
```

## Performance Optimization

### Compilation Optimizations

CCR uses multiple compilation optimization strategies:

**Dev Mode**:
```toml
[profile.dev]
opt-level = 1          # Basic optimization, balances compile/run speed
incremental = true     # Enable incremental compilation
debug = 1              # Reduce debug info
```

**Dependency Optimization**:
```toml
[profile.dev.package."*"]
opt-level = 2          # Use higher optimization for dependencies
```

**Release Mode**:
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Enable link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
```

### Runtime Performance

**Parallel Processing**:
- Uses `rayon` for parallel file operations
- WebDAV batch sync uses parallel upload/download

**Memory Optimization**:
- Small vector optimization (`smallvec`) reduces heap allocations
- Lazy initialization (`once_cell`) reduces startup time

**Async I/O**:
- Web server based on Tokio async runtime
- WebDAV operations use async HTTP client

## Security Tips

### Protect Sensitive Information

1. **Don't Commit to Version Control**:
   ```bash
   # Add to .gitignore:
   *.toml
   *.json
   *.bak
   ```

2. **Use Environment Variables**:
   ```bash
   export ANTHROPIC_AUTH_TOKEN="sk-ant-xxx"
   ```

3. **Rotate Tokens Regularly**:
   ```bash
   ccr temp-token set sk-ant-api03-new-token
   ccr export -o backup.toml --no-secrets
   ```

4. **Limit Token Permissions**:
   - Use principle of least privilege
   - Create different tokens for different environments

### Sync Security

1. **Use HTTPS**: Always use HTTPS protocol for WebDAV services
2. **App Passwords**: Use app-specific passwords, not main passwords
3. **Access Control**: Restrict WebDAV directory access permissions
4. **Regular Audits**: Check sync logs for abnormal access

## License

MIT License

## Contributing

Contributions welcome! Please submit Issues and Pull Requests.

### Development Environment

```bash
# 1. Fork repository
git clone https://github.com/yourname/ccr.git
cd ccr

# 2. Install dependencies
cargo build --all-features

# 3. Install just (optional)
cargo install just

# 4. Run tests
just ci
```

### PR Checklist

- [ ] Code follows Rust 2024 style
- [ ] Passes `cargo clippy` without warnings
- [ ] Passes `cargo fmt` formatting
- [ ] Add unit or integration tests
- [ ] Update docs (README, CHANGELOG)
- [ ] Test major features locally

### Reporting Issues

Include when reporting issues:

1. **Environment Info**:
   ```bash
   ccr version
   rustc --version
   uname -a
   ```

2. **Logs**:
   ```bash
   export CCR_LOG_LEVEL=debug
   ccr <command> 2>&1 | tee debug.log
   ```

3. **Config Sample** (sanitize sensitive info):
   ```bash
   ccr export --no-secrets
   ```

4. **Reproduction Steps**: Detailed steps

## Changelog

### v3.6.2 (2025-11-24)
- Add skills and prompts management commands (skills, prompts)
- Optimize TUI interaction experience
- Improve error messages
- Fix Codex config compatibility issues

### v3.6.1 (2025-11-20)
- Refactor skills management system to async architecture
- Optimize UI component performance
- Remove Codex redundant features

### v3.6.0 (2025-11-18)
- Complete multi-platform support (Claude/Codex/Gemini)
- Unified config management (~/.ccr/)
- Platform and config migration tool
- Enhanced cost statistics features

### v3.5.0 (2025-11-06)
- WebDAV multi-folder sync v2.5+
- Batch operations (push/pull/status)
- Folder enable/disable controls
- Exclude pattern support (like .gitignore)

### v3.4.1 (2025-10-25)
- Complete multi-platform config documentation
- Improved TUI interface
- Fix server startup issues

### v3.4.0 (2025-10-22)
- Vue 3 migration complete
- Complete frontend refactoring
- Add Tauri desktop support

See [CHANGELOG.md](CHANGELOG.md) for full history.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=bahayonghang/ccr&type=Date)](https://star-history.com/#bahayonghang/ccr&Date)

## Related Resources

- **GitHub Repo**: https://github.com/bahayonghang/ccr
- **Issues**: https://github.com/bahayonghang/ccr/issues
- **PRs**: https://github.com/bahayonghang/ccr/pulls
- **Wiki**: https://github.com/bahayonghang/ccr/wiki
- **Discussions**: https://github.com/bahayonghang/ccr/discussions

## Acknowledgments

Thanks to these projects and communities:

- [Claude](https://claude.ai) - Excellent AI assistant
- [Rust](https://rust-lang.org) - High-performance systems language
- [Ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI framework
- [Vue.js](https://vuejs.org) - Progressive JavaScript framework
- [Axum](https://github.com/tokio-rs/axum) - Rust web framework
- [Tauri](https://tauri.app) - Build desktop applications

---

**CCR** - Claude Code Configuration Switcher

MIT © 2025 Yonghang Li
