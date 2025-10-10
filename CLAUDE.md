# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**CCR (Claude Code Configuration Switcher)** is a Rust-based CLI tool for managing Claude Code API configurations with complete audit trail capabilities. Unlike its shell-based counterpart (CCS), CCR directly writes to `~/.claude/settings.json`, providing atomic operations, file locking, and full history tracking.

**Version**: 0.1.0
**Language**: Rust (Edition 2021)

## Development Commands

### Build and Run

```bash
# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Run with debug build
cargo run -- <command>

# Run release version
./target/release/ccr <command>
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_section_validate

# Run tests in a specific module
cargo test config::tests
```

### Code Quality

```bash
# Check compilation without building
cargo check

# Run clippy linter
cargo clippy

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Quick Test Commands

```bash
# Test help
cargo run -- --help

# Test version display
cargo run -- version

# Test list (requires ~/.ccs_config.toml)
cargo run -- list

# Test validation
cargo run -- validate
```

## Architecture Overview

### Core Design Philosophy

CCR follows a **layered architecture** with strict separation of concerns:

```
CLI Layer (main.rs + commands/)
    ↓
Business Logic Layer (config.rs, settings.rs, history.rs)
    ↓
Infrastructure Layer (lock.rs, logging.rs, error.rs)
```

**Key Architectural Decisions**:

1. **Atomic Operations**: All file writes use file locks + temp files + atomic rename to ensure data integrity
2. **Settings Ownership**: CCR directly manages `~/.claude/settings.json` instead of environment variables, making changes immediately effective
3. **Audit-First**: Every operation is logged to `~/.claude/ccr_history.json` with masked sensitive data
4. **CCS Compatibility**: Shares `~/.ccs_config.toml` format to allow seamless migration

### Data Flow: Configuration Switch

The `switch` command demonstrates the complete data flow:

```
1. Read & Validate Target Config (config.rs)
   └─> ConfigManager::load() → CcsConfig::get_section() → ConfigSection::validate()

2. Backup Current Settings (settings.rs)
   └─> SettingsManager::backup() → timestamped .bak file

3. Update Claude Settings (settings.rs) ⚡ CRITICAL PATH
   ├─> LockManager::lock_settings() [acquire file lock]
   ├─> ClaudeSettings::clear_anthropic_vars() [remove old env vars]
   ├─> ClaudeSettings::update_from_config() [write new env vars]
   └─> SettingsManager::save_atomic() [temp file + atomic rename]

4. Update Config Pointer (config.rs)
   └─> ConfigManager::save() → update current_config field

5. Record History (history.rs)
   ├─> HistoryEntry::new() [UUID, timestamp, actor]
   ├─> add_env_change() [record each var change with masking]
   └─> HistoryManager::add() [append to history file]
```

### Critical Components

#### settings.rs - The Core Differentiator

This module is CCR's primary value proposition. It directly manipulates Claude Code's settings file:

- **Structure**: `ClaudeSettings` has two parts:
  - `env: HashMap<String, String>` - manages ANTHROPIC_* variables
  - `other: HashMap<String, Value>` - preserves all other settings untouched (using `#[serde(flatten)]`)

- **Why This Matters**: Traditional approach (CCS) sets shell environment variables, but Claude Code reads from `settings.json` at startup. CCR's direct write ensures immediate effect without shell reload.

- **Atomic Safety**: Uses `NamedTempFile` + `persist()` to guarantee atomic replacement even under crash scenarios.

#### lock.rs - Concurrency Control

Implements cross-process file locking with timeout protection:

- **LockManager** provides named resource locks (config, settings, history)
- Default timeout: 10 seconds (see `lock_settings()` calls)
- Lock files stored in `~/.claude/.locks/`
- Automatic cleanup on `Drop` prevents orphaned locks

#### history.rs - Audit Trail

Complete operation tracking with structured logging:

- Every `HistoryEntry` includes UUID, timestamp, actor (via `whoami` crate)
- `EnvChange` records track old→new values with automatic masking for TOKEN/KEY/SECRET fields
- Supports filtering by operation type and time range
- Statistics generation for reporting

### Module Relationships

```
main.rs
  ├─> commands/*.rs [CLI handlers]
  │     └─> All commands use Result<()> for unified error handling
  │
  ├─> config.rs [manages ~/.ccs_config.toml]
  │     └─> ConfigManager ←→ CcsConfig ←→ ConfigSection
  │
  ├─> settings.rs [manages ~/.claude/settings.json] ⭐
  │     ├─> Uses LockManager for concurrency
  │     ├─> Uses tempfile for atomic writes
  │     └─> SettingsManager ←→ ClaudeSettings
  │
  ├─> history.rs [manages ~/.claude/ccr_history.json]
  │     └─> HistoryManager ←→ HistoryEntry
  │
  ├─> lock.rs [file locking infrastructure]
  │     └─> LockManager ←→ FileLock (fs4::FileExt)
  │
  ├─> logging.rs [colorized output]
  │     └─> ColorOutput static methods
  │
  └─> error.rs [unified error handling]
        └─> CcrError enum with exit codes
```

## Important Files and Paths

### Runtime Files (User's System)

- `~/.ccs_config.toml` - Configuration source (shared with CCS)
- `~/.claude/settings.json` - Target file CCR modifies
- `~/.claude/ccr_history.json` - Operation audit log
- `~/.claude/backups/*.bak` - Automatic backups with timestamps
- `~/.claude/.locks/*.lock` - Temporary lock files

### Source Code Structure

```
src/
├── main.rs (165 lines)           # CLI entry, clap parser
├── error.rs (200 lines)          # CcrError enum, exit codes
├── logging.rs (250 lines)        # ColorOutput utilities
├── lock.rs (250 lines)           # FileLock, LockManager
├── config.rs (350 lines)         # Config TOML management
├── settings.rs (400 lines)       # Settings JSON management ⭐
├── history.rs (400 lines)        # Audit trail management
└── commands/ (600 lines total)   # CLI command implementations
```

## Key Implementation Details

### Error Handling Strategy

All functions return `Result<T>` (aliased to `std::result::Result<T, CcrError>`):

- **13 error types** with specific exit codes (10-255 range)
- `is_fatal()` method determines if program should immediately exit
- `user_message()` provides localized, actionable error messages
- Errors propagate with `?` operator, caught in `main()` for display

### Sensitive Data Masking

Implemented in multiple layers:

1. **Display Level**: `ColorOutput::mask_sensitive()` - shows first 4 + last 4 chars
2. **History Level**: `HistoryEntry::add_env_change()` - auto-detects TOKEN/KEY/SECRET in var names
3. **CLI Output**: All commands use `mask_sensitive()` before printing tokens

### File Format Compatibility

**TOML (config.rs)**:
- Uses `toml` crate for parsing `~/.ccs_config.toml`
- Must preserve all fields when updating `current_config`
- Supports both `"quoted"` and `'single-quoted'` strings

**JSON (settings.rs)**:
- Uses `serde_json` with pretty printing
- `#[serde(flatten)]` preserves unknown fields
- Only modifies `env` object, leaving other settings intact

## Development Workflows

### Adding a New Command

1. Create `src/commands/newcmd.rs`:
```rust
use crate::error::Result;
use crate::logging::ColorOutput;

pub fn newcmd_command(args: YourArgs) -> Result<()> {
    // Implementation
    Ok(())
}
```

2. Export in `src/commands/mod.rs`:
```rust
pub mod newcmd;
pub use newcmd::newcmd_command;
```

3. Add to CLI in `src/main.rs`:
```rust
enum Commands {
    // ...
    Newcmd { /* args */ },
}

// In match block:
Some(Commands::Newcmd { /* args */ }) => commands::newcmd_command(args),
```

### Modifying Settings Write Logic

**CRITICAL**: Always acquire lock before modifying `settings.json`:

```rust
let settings_manager = SettingsManager::default()?;
let _lock = settings_manager.lock_manager
    .lock_settings(Duration::from_secs(10))?;  // Must hold lock

// Safe to modify now
settings_manager.save_atomic(&new_settings)?;
```

### Testing Configuration Changes

Since CCR modifies real system files, use temporary directories for testing:

```rust
#[test]
fn test_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(lock_dir);
    let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

    // Test operations
}
```

## Troubleshooting Development Issues

### Compilation Warnings About Unused Code

Some methods are intentionally unused as they're part of the public API for future features (e.g., `restore`, `list_backups`). This is expected and can be ignored.

### Lock Timeout During Testing

If tests hang, check for leaked locks:
```bash
# Check lock files
ls -la ~/.claude/.locks/

# Force cleanup
rm -rf ~/.claude/.locks/*
```

### Type Mismatches in Commands

When working with references in loops over `config.list_sections()`:
```rust
// ❌ Wrong: takes ownership
for section_name in sections {
    let is_current = section_name == config.current_config; // Type mismatch
}

// ✅ Correct: borrows
for section_name in &sections {
    let is_current = section_name == &config.current_config;
}
```

## CCS Compatibility Notes

CCR maintains full compatibility with CCS (the shell version):

- **Shared Config**: Both read/write `~/.ccs_config.toml`
- **Migration Path**: Users can switch between `ccs` and `ccr` commands freely
- **Behavioral Difference**: CCR writes to `settings.json` directly, CCS sets environment variables

When modifying config format, ensure backwards compatibility with CCS shell scripts.

## Related Documentation

- `CCR_DESIGN.md` - Detailed design specification
- `README.md` - User-facing documentation
- `../CLAUDE.md` - CCS project guidance (parent directory)
