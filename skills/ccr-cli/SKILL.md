---
name: ccr-cli
description: "Manage local ccr CLI routing profiles across claude, codex, and grok (`list`, `current`, `init`, `create`, `edit`, `switch`, `enable`, `disable`, `delete`, `off`, `export`, `import`) and run cross-platform status or health diagnostics (`ccr current`, `ccr validate`, `ccr doctor`, `ccr platform list`). Trigger when requests inspect, create, update, switch, disable, export, import, or troubleshoot Claude Code, Codex, or Grok routing profiles through the installed `ccr` binary. Exclude `ccr` repository development or CI gates, OAuth account snapshots (`ccr <platform> auth`), interactive TUI or editor commands (bare `ccr`, `ccr ui`, `profile open`, `ccr temp`), `sync`/`update`/`clean`/`skills`/`prompts`/`sessions`/`stats`/`budget`/`pricing`, and `gemini`/`droid`/`qwen` profile writes."
license: MIT
metadata:
  owner: bahayonghang
  version: "1.0.0"
  maturity_tier: governed
---

# CCR CLI (`ccr-cli`)

Operate the locally installed `ccr` CLI (`>= 7.5`) to inspect, author, switch, export, import, and diagnose third-party routing profiles for **`claude`**, **`codex`**, and **`grok`** without exposing credentials, hanging on interactive prompts, or mutating live runtime configs unexpectedly.

## Router Rules

- **Activate on supported profile and diagnostic intents**: Route requests that inspect, initialize, create, edit, switch, enable, disable, delete, turn off (`off`), export, or import routing profiles on `claude`, `codex`, or `grok`, as well as cross-platform runtime health checks (`ccr current`, `ccr validate`, `ccr doctor`, `ccr platform list`).
- **Exclude `ccr` repository engineering**: Reject source edits, Cargo/Bun builds, and CI gate repairs inside the `bahayonghang/ccr` repository; repository development belongs to `.github/skills/`.
- **Exclude OAuth account snapshots (`auth`)**: Never execute or wrap `ccr <platform> auth ...` login, save, switch, delete, or `codex auth` export/import flows; direct the operator to run OAuth account commands manually in an interactive terminal.
- **Exclude blocking interactive surfaces**: Never launch fullscreen TUI or blocking editors/servers including bare `ccr`, bare `ccr claude`, bare `ccr codex`, `ccr grok auth`, `ccr ui`, `ccr <platform> profile open`, or `ccr temp`.
- **Exclude non-profile subsystems**: Do not invoke `sync`, `update`, `clean`, `project init`, `skills`, `prompts`, `sessions`, `stats`, `budget`, `pricing`, or `temp-token`.
- **Exclude unsupported platform writes**: Never create, edit, switch, export, or import profiles for `gemini`, `droid`, or `qwen`; interpret those names read-only only if they appear inside `ccr doctor --json` output.
- **Forbid legacy or hidden CLI entrypoints**: Never invoke unprefixed top-level profile verbs, retired positional shortcuts, hidden single-field mutators, or hidden inline credential flags; every profile action must use `ccr <claude|codex|grok> profile <action>`.

## Compact Workflow

1. **Run the zero-side-effect two-step capability probe once per session**:
   - Run `ccr --version` (`>= 7.5` recommended) plus `ccr claude profile export --help` and `ccr claude profile switch --help` (both must exit `0` and list `--json`).
   - Stop immediately before any mutation if `ccr` is missing, older than `7.x`, or lacks `--json` on canonical subcommands; ask the operator to upgrade `ccr` and never fall back to legacy commands.
2. **Select only from the canonical non-interactive command allowlist**:
   - Introspection & cross-platform diagnostics: `ccr --version`, `ccr <claude|codex|grok> profile <action> --help` (single source of truth for optional flags), `ccr platform list --json`, `ccr current --json`, `ccr validate --json`, and `ccr doctor --json [--platform <claude|codex|grok>]`.
   - Canonical `<claude|codex|grok> profile` actions (12 total): `list`, `current`, `init`, `create`, `edit`, `switch`, `enable`, `disable`, `delete`, `off`, `export`, and `import`; always pass `--json` except when streaming redacted TOML via `export --no-secrets --stdout` (`--stdout` and `--json` are mutually exclusive).
3. **Read current state and CAS revision before mutating**:
   - Inspect `ccr <platform> profile list --json` and `ccr <platform> profile current --json` before mutating, and pass top-level `revision` as `--expected-revision <REV>` on `edit` / `import` (consult [references/platforms-and-fields.md](references/platforms-and-fields.md) for platform `auth_mode`, writable fields, and sidecar rules).
4. **Enforce non-interactive stdin secret piping (`*-stdin`)**:
   - Pipe credentials only from an operator-managed environment variable (`printf '%s' "$MY_PROVIDER_KEY" | ccr claude profile create <name> --base-url <url> --model <model> --auth-token-stdin --json`) or protected file redirect (`< "$SECRET_FILE"`), or prefer `--env-key <ENV_NAME>` on `grok` (and `--set env_key=<ENV_NAME>` on `codex`).
   - Never invoke `--auth-token-stdin` or `--api-key-stdin` without piped stdin (prevents interactive `rpassword` TTY hangs); if no env var or file exists yet, create `--disabled` or guide the operator to use `ccr` TUI (`n`) or an interactive terminal.
5. **Execute preview-first ExchangeV1 import and redacted export**:
   - Default exports to `--no-secrets` (omit only when the operator explicitly requests a local `0600` migration file via `-o <FILE>`, never with `--stdout`), and always run `ccr <platform> profile import <FILE> --dry-run --json` first to inspect `counts.*` 6+1 buckets per [references/export-import-and-safety.md](references/export-import-and-safety.md).
6. **Verify dual exit-code and JSON health criteria**:
   - Treat non-zero exit codes as failures (checking both `stderr` and `profile edit --json` stdout recovery receipts), and when exit code is `0` still inspect JSON for `"ok": false` or status-card `"health"` in `["error", "invalid"]`, following up with `ccr validate --json` per [references/diagnostics-and-recovery.md](references/diagnostics-and-recovery.md).

## Gate Ladder

- **L0 — Read-Only & Dry-Run Preview (Auto-Allowed)**:
  - Execute `--version`, `--help`, `platform list --json`, `current --json`, `validate --json`, `doctor --json`, `<platform> profile list --json`, `<platform> profile current --json`, `<platform> profile export --no-secrets`, and `<platform> profile import <FILE> --dry-run --json` without extra confirmation.
- **L1 — Profile Import Gate (Mandatory Dry-Run First)**:
  - Always run `ccr <platform> profile import <FILE> --dry-run --json` first and present `platform`, `source_format`, `counts.total_input`, the 6+1 `counts.*` breakdown (`added`, `updated`, `unchanged`, `renamed`, `skipped_conflict`, `skipped_duplicate`, `removed`, `disabled_missing_secret`), `missing_secrets`, and `warnings`.
  - Pure additive merges (`--on-conflict skip` or `rename`, no `--replace`) requested in the current turn may proceed after a clean dry-run.
  - Any `--on-conflict overwrite` or `--replace` import requires listing `updated_names` / `removed_names` from dry-run output and obtaining explicit operator approval before adding global `-y` (`--yes`).
- **L2 — Live Runtime & Destructive Profile Gate (Explicit Intent Required)**:
  - Require explicit current-turn operator intent naming the target profile and action before running `switch`, `enable`, `off`, `disable`, or `delete` (which rewrite live `~/.claude/settings.json`, `~/.codex/config.toml` + `auth.json`, or `$GROK_HOME/config.toml`, or permanently delete catalog entries).
  - Never self-attach `--force` on `profile disable` or `profile delete` for an active profile unless the operator specifically authorizes forcing the currently active profile.
  - Never self-attach global `-y` / `--yes` to bypass overwrite or replacement guards (`export -o <existing-file>` or `import --replace`) without explicit confirmation.
- **Zero-Touch Secret Redlines (Hard Block)**:
  - Never ask the operator to paste plaintext API keys or tokens into chat.
  - Never place secret literals in CLI arguments or shell strings; pipe exclusively from an operator-managed environment variable (`printf '%s' "$MY_VAR" | ccr ...`) or protected local file (`< "$SECRET_FILE"`).
  - Never run `export` without `--no-secrets` alongside `--stdout`, and never read secret-bearing export files, `~/.ccr/platforms/*/profiles.toml`, `~/.ccr/platforms/*/profiles.db`, `*.bak` backups, `~/.ccr/platforms/codex/profile_secrets.json`, or `~/.codex/auth.json` via file-reading tools or shell commands.

## Output Contract

- **Capability & target header**: State the target platform (`claude`, `codex`, `grok`, or cross-platform) and report probe failure immediately if `ccr` capability checks do not pass.
- **Dry-run & mutation receipt**: For `import`, report `counts.*` 6+1 bucket totals (`added`, `updated`, `unchanged`, `renamed`, `skipped_conflict`, `skipped_duplicate`, `removed`, `disabled_missing_secret`), matching detail arrays (`added_names`, `updated_names`, `renamed_items`, `removed_names`, `missing_secrets`), `backup_path` (when written), and `revision`.
- **Runtime & CAS status**: Report `profile`, `previous_profile` (on `switch`), `revision`, `remaining_suppressors` (on `claude profile off --json`), and whether live runtime config was rewritten or skipped (`--no-apply`).
- **Dual-check health verdict**: Confirm both exit code `0` and JSON `"ok"` / `"health"` (`"ready"` vs `"error"` / `"invalid"`); whenever `"ok": false` or `"health"` is `"error"` or `"invalid"`, include `ccr validate --json` findings and concrete non-destructive recovery options without exposing any secret values.

## References

- Platform auth modes, writable field allowlists, env-var references vs secret values, and Codex sidecar storage: [references/platforms-and-fields.md](references/platforms-and-fields.md)
- `ExchangeV1` schema, 3-format import compatibility, 6+1 bucket math, redaction, and `0600` backup/rollback contract: [references/export-import-and-safety.md](references/export-import-and-safety.md)
- `current` / `validate` / `doctor` dual-check interpretation, `--expected-revision` CAS loop, and `codex fix` recovery: [references/diagnostics-and-recovery.md](references/diagnostics-and-recovery.md)
