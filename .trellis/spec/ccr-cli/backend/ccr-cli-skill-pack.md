# Governed `skills/ccr-cli` Skill Pack

## 1. Scope / Trigger

- Applies to the top-level governed agent skill pack under `skills/ccr-cli/`, its stdlib validator `scripts/quality/check_ccr_cli_skill.py` (`scripts/quality/test_check_ccr_cli_skill.py`), `just skill-pack-check`, `.github/workflows/ci.yml`, and `scripts/ci/ci_surface_policy.py` (`rust` surface).

## 2. Signatures

- Package root: `skills/ccr-cli/{SKILL.md, README.md, manifest.json, agents/interface.yaml, references/*.md, evals/trigger_cases.json, reports/*}`.
- Offline repository gate: `python scripts/quality/check_ccr_cli_skill.py` (`just skill-pack-check`) + `python scripts/quality/test_check_ccr_cli_skill.py`.

## 3. Contracts

- Supported platforms in `skills/ccr-cli` are strictly **`claude`, `codex`, and `grok`**. `gemini`, `droid`, and `qwen` are excluded from active `ccr-cli` workflows and appear only in explicit negative boundaries / negative trigger evals.
- Secret creation/update examples must use stdin flags (`--auth-token-stdin` / `--api-key-stdin`) and never pass secrets in argv (`--auth-token <val>`, `--api-key <val>`) or literal `echo`/`printf` secret pipes.
- Export examples default to `--no-secrets` (`ExchangeV1` `secrets_mode = "redacted"`); `--include-secrets` requires `-o <path>` (`0600`) or an encrypted stream and must never redirect `--stdout` to a regular file without pre-restricted permissions.
- Import workflows preview `6+1` buckets first via `ccr <platform> profile import <file> --dry-run --json` (`--replace` requires explicit user confirmation).
- `skills/ccr-cli` must remain self-contained (zero relative links to `crates/`, `docs/`, `.trellis/`, or `.agents/`), keep `SKILL.md <= 14,000` UTF-8 LF bytes with `<= 18` `## Compact Workflow` items and `>= 3` `## Troubleshooting` items, and keep `SKILL.md` frontmatter `description`, `manifest.json` `description`, `agents/interface.yaml` `default_prompt`, and `reports/skill-ir.json` `description` aligned.
- `skills/ccr-cli/manifest.json` owns an independent skill version (`1.0.0`, `lifecycle_tier: "governed"`) and **must not** be coupled to `scripts/release/version-sync.mjs`. Never run external `publish_skill.py` against this repository.

## 4. Verification

- `just skill-pack-check`
- `python scripts/quality/test_check_ccr_cli_skill.py`
- `just ci-governance-check`
