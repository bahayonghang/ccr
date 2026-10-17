# `ccr import`

## Overview

Command: `ccr import`

## Usage

```bash
ccr import [OPTIONS]
```

## Options

- `--platform <name>`: write that platform catalog. The default is `claude`. `ccr platform switch` does not change the target.
- `--merge`: keep existing profiles and add or update imported ones.
- `--no-backup`: skip the pre-import backup.

Run `ccr import --help` for the full list.

## Examples

```bash
# Basic usage
ccr import
```

## See Also

- [Command Reference](./index)
- [Quick Start](/en/guide/quick-start)
- [Configuration Guide](/en/guide/configuration)
