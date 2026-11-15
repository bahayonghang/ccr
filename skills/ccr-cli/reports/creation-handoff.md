# Creation Handoff — `ccr-cli` (`1.0.0`)

## 1. Package Identity & Distribution Posture

- **Skill Name**: `ccr-cli`
- **Skill Version**: `1.0.0` (decoupled from the `ccr` Rust workspace version; `just version-sync` intentionally does not touch `skills/ccr-cli/`)
- **Owner / Repository**: `bahayonghang` (`https://github.com/bahayonghang/ccr`, path `skills/ccr-cli`)
- **Maturity & Budget Tier**: `maturity_tier: "governed"`, `context_budget_tier: "production"` (`SKILL.md <= 14,000` UTF-8 bytes)
- **Install Command**: `npx skills add bahayonghang/ccr --skill ccr-cli`
- **Publication Boundary**: Distributed in-tree through `bahayonghang/ccr` `dev -> main` pull requests. External publication scripts (`publish_skill.py`) are strictly prohibited so repository branches, tags, and remotes are never mutated outside monorepo release workflows.

---

## 2. Studied Anchors & Candidate-Specific Lessons

1. **Agent Skills & `vercel-labs/skills` discovery contract**: Placing a single `SKILL.md` at `skills/ccr-cli/SKILL.md` satisfies priority directory discovery and prevents `npx skills` from falling back to recursive `.github/skills/` scanning, cleanly separating end-user CLI operations from internal `ccr` contributor skills.
2. **`googleworkspace/cli` (`gws-shared`)**: Confirmed that CLI skills stay under production token budgets and remain forward-compatible when `SKILL.md` mandates `--json` + live `--help` introspection instead of pasting exhaustive flag matrices.
3. **`larksuite/cli` (`lark-shared`)**: Demonstrated the value of a zero-side-effect binary capability preflight that fails closed with an upgrade prompt rather than silently degrading to legacy command syntax.
4. **Cloudflare `wrangler` secret workflows**: Reinforced passing secrets strictly through non-interactive stdin pipes from user-controlled environment variables or `0600` files while keeping browser/OAuth login human-operated.

---

## 3. Deliberate Rejections

- **Rejected multi-skill splitting**: Kept one `ccr-cli` package covering `claude`, `codex`, and `grok` because all three platforms share the same 12 `profile` actions, `ExchangeV1` format, and L0/L1/L2 safety ladder.
- **Rejected retired/hidden CLI fallbacks**: Zero tolerance for unprefixed top-level verbs, hidden single-field mutators, or inline credential flags (`--auth-token` / `--api-key` value args).
- **Rejected interactive & non-profile surfaces**: Excluded `ccr <platform> auth`, fullscreen TUI, `ccr ui`, `profile open`, `ccr temp`, `sync`, `update`, `clean`, `skills`/`prompts`/`sessions`/`stats`/`budget`/`pricing`, and `gemini`/`droid`/`qwen` profile mutations.
- **Rejected personal/third-party branding & duplicate license files**: Omitted external author social blocks, `assets/qiaomu-profile/`, and redundant `skills/ccr-cli/LICENSE` (governed by root `LICENSE`).

---

## 4. Original Contributions & Claim Classification

- **[validated advantage] Monorepo-isolated end-user discovery + stdlib CI gate (`just skill-pack-check`)**: Verified by `scripts/quality/check_ccr_cli_skill.py` and `scripts/quality/test_check_ccr_cli_skill.py` (covering single-entry discovery, `<= 14,000` byte budget, IR/README/manifest parity, version consistency, secret scan, and banned legacy CLI command scan) plus `qiaomu-meta` local `validate_skill.py`, `export_skill_ir.py`, `trigger_eval.py`, and `release_check.py --phase local`.
- **[validated advantage] Ground-truth alignment with compiled `ccr 7.5.1` CLI**: Every command, flag (`--json`, `--auth-token-stdin`, `--api-key-stdin`, `--env-key`, `--no-secrets`, `--dry-run`, `--on-conflict`, `--dedup`, `--replace`, `--no-apply`, `--expected-revision`), writable field allowlist (`profile_fields.rs`), and exit code was verified directly against `target/debug/ccr` `--help` and Rust source.
- **[design advantage] Dual exit-code + status-card `health` verification & partial-commit recovery**: Bridges CCR's exit-0 status-card contract (`ccr current --json` / `profile current --json`) and `profile edit --json` non-zero stdout recovery receipts (`updated_apply_failed` / `recovery_required`) so agents neither miss silent drift nor lose rollback context.
- **[design advantage] TTY `rpassword` anti-hang & Codex sidecar zero-touch boundary**: Prevents agent subprocess hangs on unpiped `*-stdin` flags while protecting `~/.ccr/platforms/codex/profile_secrets.json` and `profiles.db` from direct file reads.
- **[hypothesis] Cross-host multi-turn prompt adherence**: Expected to eliminate legacy flag hallucination and unapproved `--force` / `--replace` mutations across Claude Code, Codex, and Cursor sessions once live user telemetry is collected.

---

## 5. Verification Summary & Honest `missing evidence` Declaration

### Verified Local Evidence

- `validate_skill.py skills/ccr-cli`: Structural, frontmatter, `14,000`-byte production budget, README anchor, interface adapter target, and link checks (`ok: true`, `warnings: []`).
- `export_skill_ir.py skills/ccr-cli`: Generated `reports/skill-ir.json` (`1.0.0`) with non-empty `router_rules`, `compact_workflow`, `gate_ladder`, and `output_contract`.
- `trigger_eval.py skills/ccr-cli`: Offline lexical trigger evaluation across 26 prompts (`12` `should_trigger`, `6` `should_not_trigger`, `8` `near_neighbor`) in `reports/trigger-eval.json` (`ok: true`, `26/26` passed).
- `release_check.py skills/ccr-cli --phase local`: Passed with `summary.block == 0`.
- `just skill-pack-check`: Self-contained repository CI gate and negative-fixture unit test suite passed.

### Explicit `missing evidence` Boundaries

- **Online multi-turn LLM trigger telemetry (`missing evidence`)**: `reports/trigger-eval.json` measures the deterministic offline lexical/concept benchmark bundled with `qiaomu-meta`; live multi-turn model routing hit rates across hosted Claude, OpenAI Codex, and Cursor sessions have not been measured and are **not** claimed.
- **Blind human/provider output evaluation (`missing evidence`)**: No static or synthetic `reports/output-evidence.json` has been fabricated; blind A/B task execution scoring with real third-party API providers remains `missing evidence` until post-merge field evaluation.
- **External marketplace star/install metrics (`missing evidence`)**: Live third-party skill registry popularity counters were not sampled offline.
