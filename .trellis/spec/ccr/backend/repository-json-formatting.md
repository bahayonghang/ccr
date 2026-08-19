# Repository JSON Formatting

> Canonical formatting contract for human-authored JSON configuration.

## Scenario: explicit JSON inventory

### 1. Scope / Trigger

- Trigger: editing `scripts/quality/check_json_format.py`, its inventory, or the
  `json-format` / `json-format-check` recipes.
- The inventory covers human-authored package and application configuration,
  including Tauri capabilities and `tauri.conf.json`.

### 2. Signatures

- Check: `python scripts/quality/check_json_format.py` or `just json-format-check`.
- Repair: `python scripts/quality/check_json_format.py --write` or `just json-format`.
- Aggregate gate: `just fmt-check` depends on `json-format-check`.

### 3. Contracts

- Parse with the standard JSON parser, serialize with two-space indentation,
  preserve Unicode, and end with one newline.
- Maintain an explicit `JSON_CONFIG_PATHS` allowlist. Lockfiles, generated
  bindings, third-party assets, data catalogs, JSONC/`tsconfig*.json`, and
  whitespace-sensitive fixtures remain excluded.
- Formatting must preserve parsed values. Version or metadata changes already
  present in the worktree are separate semantic changes and must not be
  accidentally absorbed into a formatting commit.

### 4. Validation & Error Matrix

- Missing inventory path, malformed JSON, invalid UTF-8, or noncanonical
  formatting -> check mode fails closed.
- Unlisted JSON -> ignored; add it only after ownership and exclusion review.
- JSONC added to the strict inventory -> parser failure; keep it excluded.

### 5. Good/Base/Bad Cases

- Good: format a listed capability file and prove parsed values are unchanged.
- Base: leave `package-lock.json` and `tsconfig.json` outside the inventory.
- Bad: recursively rewrite every `*.json` or use string replacement to format
  JSON.

### 6. Tests Required

- Unit tests cover noncanonical input without mutation, deterministic write,
  malformed/missing files, and excluded files.
- Run `python -m unittest scripts.quality.test_check_json_format`.
- Run `just json-format-check` and `just fmt-check`.

### 7. Wrong vs Correct

#### Wrong

```python
for path in root.rglob("*.json"):
    path.write_text(path.read_text().replace(",", ",\n"))
```

#### Correct

```python
formatted = json.dumps(json.loads(source), ensure_ascii=False, indent=2) + "\n"
```
