# update - Update CCR

Build and update the installed `ccr` binary from the official GitHub repository with branch selection, dry-run command preview, and live `cargo` build progress.

## Usage

```bash
ccr update [OPTIONS] [BRANCH]
```

## Arguments and Options

- `[BRANCH]`: target Git branch (default: `main`, optional: `dev`)
- `-c, --check`: preview the `cargo install` command without compiling or replacing the binary

## Requirements

- Rust 1.90+ and `cargo` available on `PATH`
- `git` installed with network access to `https://github.com/bahayonghang/ccr`

Equivalent manual command:

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr --force
```

> When installing from a local repository checkout, run `cargo install --path crates/ccr`.

## Examples

### 1. Preview the update command first

```bash
ccr update --check
ccr update -c
```

### 2. Update from the default `main` branch

```bash
ccr update
```

### 3. Update from the `dev` branch

```bash
ccr update --check dev
ccr update dev
```

> [!WARNING]
> The `dev` branch carries in-progress changes and may be less stable than `main`. Prefer `main` for daily production workflows.

## Recommended Upgrade Loop

Updating the `ccr` executable leaves `~/.ccr/config.toml` and per-platform `profiles.db` catalogs intact. Before major upgrades, optionally export snapshots and verify health afterward:

```bash
# 1. Export optional profile snapshots (includes inline secrets at 0600 by default; add --no-secrets if sharing)
ccr claude profile export -o ~/backups/claude-$(date +%Y%m%d).toml
ccr codex profile export -o ~/backups/codex-$(date +%Y%m%d).toml

# 2. Run the update
ccr update

# 3. Verify version and configuration health
ccr --version
ccr current
ccr validate
ccr doctor
```

> Note: Without `--no-secrets`, `profile export` includes inline secrets by default (written with owner-only `0600` permissions for private local backups). Always pass `--no-secrets` when sharing or committing exported files.

## Troubleshooting and Rollback

### Common Issues

- **`cargo` not found**: ensure Rust is installed via `rustup` and `$HOME/.cargo/env` is sourced.
- **Compilation failure**: run `rustup update stable` to bring the toolchain up to date, then retry.
- **`ccr --version` still prints an older release**: check `which ccr` for another binary shadowing `~/.cargo/bin/ccr` on `PATH`.

### Roll Back to a Specific Tag or Commit

```bash
# Install a specific release tag
cargo install --git https://github.com/bahayonghang/ccr ccr --tag <tag> --force

# Or install a specific commit SHA
cargo install --git https://github.com/bahayonghang/ccr ccr --rev <commit-sha> --force
```

## Related Commands

- [`version`](./version) - print detailed or short version output
- [`validate`](./validate) - validate registry and platform configurations
- [`doctor`](./doctor) - run unified environment diagnostics
- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) - export or import platform catalogs
