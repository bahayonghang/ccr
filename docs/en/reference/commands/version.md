# `ccr version`

Show version information for the current CCR installation.

## Usage

```bash
# Detailed version summary
ccr version

# Short version string
ccr --version
ccr -V
```

## Which one should you use?

### `ccr version`

Use this for humans.

It prints:

- the current version
- authors
- package description
- key help entrypoints
- core task entrypoints

### `ccr --version` / `ccr -V`

Use this for scripts and CI.

It returns a short one-line version string, for example:

```bash
$ ccr --version
ccr 5.9.4
```

## Common Scenarios

### 1. Verify the current install manually

```bash
ccr version
```

### 2. Read the version in a script

```bash
VERSION=$(ccr --version | awk '{print $2}')
echo "Current CCR version: $VERSION"
```

### 3. Validate after an update

```bash
ccr update
ccr --version
ccr --help
```

### 4. Collect issue / troubleshooting context

```bash
ccr version
ccr --version
```

## See Also

- [update](./update)
- [platform](./platform)
- [codex](./codex)
- [opencode](./opencode)
