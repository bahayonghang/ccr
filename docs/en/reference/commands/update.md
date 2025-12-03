# `ccr update`

Update CCR from GitHub with support for different branches.

## Usage

```bash
ccr update [OPTIONS] [BRANCH]
```

## Arguments

- `[BRANCH]`: Branch to update from (default: main, options: dev)

## Options

- `--check, -c`: Check mode - preview update without executing

## Examples

### Update from main branch (default)

```bash
ccr update
```

### Update from dev branch

```bash
# Update from dev branch (latest development features)
ccr update dev

# Check dev branch updates first
ccr update --check dev
```

### Check updates only

```bash
ccr update --check
ccr update --check dev
```

## Branch Options

- `main` (default): Stable releases
- `dev`: Latest development features (may be unstable)

> **Warning:** The dev branch contains cutting-edge features but may not be stable. Use it only in testing environments or when you need the latest features.

## See Also

- [Command Reference](./index)
- [Quick Start](../quick-start)
- [Version Command](./version)
