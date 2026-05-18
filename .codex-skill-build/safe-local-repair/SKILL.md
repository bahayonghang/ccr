---
name: safe-local-repair
description: Safety-first workflow for repairing local host config, JSON/TOML/YAML syntax errors, local databases, sync/rebuild risks, and data-retention-sensitive paths. Use when the user gives a parser line/column and asks to fix it, asks to ensure history/data is not lost, mentions lossy rebuild, source files were deleted, sync --rebuild, diagnostics repair, local config corruption, or any host-local repair where secrets or user data may be present.
---

# Safe Local Repair

Use the smallest repair that protects data. This skill captures the user's repeated preference for backups, retention boundaries, and verified non-lossy behavior.

## Workflow

1. Classify the risk before editing.
   - Local syntax/config repair: a parser points to a specific file, line, and column.
   - Data-retention repair: sync/rebuild/reset paths could delete or hide imported history.
   - Secret-bearing config: file contents may contain tokens, keys, paths, or private prompts.

2. Minimize exposure.
   - Inspect only the necessary nearby context around a parser error.
   - Do not copy secrets or unrelated local config values into notes, memory, commits, or final output.
   - Prefer structured parsers over ad hoc text checks when validating JSON/TOML/YAML.

3. Back up before local repair.
   - For host-local config edits, create a timestamped backup beside the original file before changing it.
   - Confirm the backup path before broad recursive operations.
   - If the repair touches repo files, preserve unrelated dirty changes and follow repo instructions.

4. Make the smallest structural fix.
   - Trust parser line/column first when the nearby context confirms the issue.
   - Fix a trailing comma, missing delimiter, malformed quote, or invalid field without reformatting the whole file.
   - Avoid speculative cleanup.

5. For retention-sensitive code, default to safety.
   - Treat deletion/rebuild/reset paths as dangerous until proven lossless.
   - Prefer diagnostics, explicit unsupported states, and guarded refusal over silent data loss.
   - Keep destructive overrides explicit, advanced, and opt-in, such as an `--allow-lossy-rebuild`-style escape hatch.
   - Do not normalize destructive rebuilds into defaults.

6. Verify the repaired artifact.
   - Re-run the parser or command that failed.
   - For data-retention code, add or run tests that prove protected historical data is preserved.
   - Report backup path and validation result without exposing sensitive contents.

## Failure Patterns To Avoid

- Broad cleanup when a parser already identified one structural issue.
- Reporting a repair without a pre-edit backup for host-local config.
- Treating performance rebuilds as harmless when source history may already be gone.
- Zero-filling, hiding, or silently dropping unsupported data instead of surfacing a typed diagnostic state.

## Output

Report the file repaired, backup location, minimal change made, validation command, and any destructive path that remains intentionally blocked or opt-in.
