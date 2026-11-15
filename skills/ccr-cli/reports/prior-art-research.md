# Prior-Art Research — `ccr-cli` (`v1.0.0`)

## 1. Problem Framing & Evaluation Scope

`ccr-cli` packages an Agent Skills interface for end users who have installed the `ccr` CLI (`bahayonghang/ccr`) and want AI coding agents (Claude Code, Codex, Cursor, Grok Build, OpenCode, and generic Agent Skills hosts) to inspect, create, edit, switch, export, import, and diagnose routing profiles across `claude`, `codex`, and `grok`.

Key failure modes without a governed skill package:

1. **Hallucinated or retired CLI verbs**: Agents guess unprefixed top-level commands, unsupported platform profile verbs (`gemini`, `droid`, `qwen`), or non-existent flags instead of introspecting `ccr <platform> profile <action> --help`.
2. **Process hangs on interactive TTY prompts**: Bare `ccr`, `profile open`, `ccr ui`, or unpiped `--auth-token-stdin` (`rpassword`) block non-interactive agent execution indefinitely.
3. **Credential exposure & destructive runtime clobbering**: Agents ask users to paste plaintext API keys in chat, pass secret literals on command lines, stream unredacted exports to stdout, or run `import --replace` / `delete --force` without previewing impact.
4. **Repository vs end-user skill collision**: `bahayonghang/ccr` already tracks contributor-only skills under `.github/skills/` (`ccr-gate-recovery`, `ccr-ui-visual-workflow`); end-user distribution via `npx skills add` requires clean container isolation.

> **Catalog & Live Telemetry Evidence Note**: Live third-party marketplace star/install rankings and cross-vendor hosted multi-turn trigger telemetry were not queried or fabricated during offline repository authoring (`missing evidence`). Prior-art comparison below is grounded in primary upstream specifications and official CLI skill architectures documented in project research Issues `#170` and `#178`.

---

## 2. Studied Anchors & Reference Implementations

| Anchor | Category | Key Structural Pattern Observed | Relevance to `ccr-cli` |
| --- | --- | --- | --- |
| **Agent Skills Specification & `vercel-labs/skills` (`npx skills`)** | Open Format & Installer Discovery | Priority container discovery (`skills/<name>/SKILL.md` at repo root stops fallback full-repo scan into `.github/skills/`); strict `name` = directory name; progressive disclosure (`SKILL.md` -> flat `references/*.md`); `description <= 1024` chars. | Directly governs top-level `skills/ccr-cli/` placement and guarantees `npx skills add bahayonghang/ccr --skill ccr-cli` never leaks internal `.github/skills/` contributor skills. |
| **Google Workspace CLI Skills (`googleworkspace/cli` — `gws-shared`)** | Official Local-CLI Wrapper Skill | Enforces non-interactive JSON output mode, explicit confirmation ladders before mutating user workspace resources, and live CLI `--help` / schema introspection rather than duplicating giant flag tables in `SKILL.md`. | Strongest structural blueprint for `ccr --json` enforcement, `--help` as single source of truth, and L0/L1/L2 confirmation tiers. |
| **Lark / Feishu Official CLI Skills (`larksuite/cli` — `lark-shared`)** | Multi-Surface CLI Skill Family | Separates shared authentication/capability preflight from domain mutations; checks installed binary version first and refuses silent fallback to deprecated endpoints when capability checks fail. | Directly informs `ccr-cli`'s two-step zero-side-effect capability probe (`ccr --version` + `profile export/switch --help`) and zero-fallback policy. |
| **Cloudflare `wrangler` / Official DevOps CLI Skills** | Credential-Sensitive Local CLI Skill | Treats secret injection as stdin/env-reference-only (`wrangler secret put` via pipe/env, never CLI literal args), separates preview/dry-run from live deployment, and isolates OAuth browser login as human-only. | Validates `ccr-cli`'s zero-touch secret rule (`printf '%s' "$VAR" \| ccr ... --auth-token-stdin --json`), `--env-key` preference, and exclusion of `ccr <platform> auth` OAuth flows. |
| **Repository Internal `.github/skills/` (`ccr-gate-recovery`, `ccr-ui-visual-workflow`)** | Same-Repo Contributor Skills | Enforced by stdlib-only Python CI contract script (`scripts/quality/check_harness_contracts.py`) wired into `justfile` and GitHub Actions without external host dependencies. | Provides the exact CI governance pattern for `scripts/quality/check_ccr_cli_skill.py` and `just skill-pack-check`. |

---

## 3. Synthesis Matrix: `Keep / Adapt / Reject / Invent`

### Keep

- **Single top-level `skills/ccr-cli/SKILL.md` package**: Matches `npx skills` priority discovery so end users install one cohesive router skill while `.github/skills/` stays isolated for `ccr` contributors.
- **Flat one-level `references/` split**: Keeps `SKILL.md` well under the `14,000`-byte production context budget while putting platform field allowlists, `ExchangeV1` 6+1 bucket math, and recovery playbooks one hop away.
- **Machine-readable `--json` default + `--help` self-introspection**: Keeps prompts compact and resilient to additive optional flags.

### Adapt

- **Capability preflight adapted from `lark-shared` / `gws-shared`**: Adapted into a zero-side-effect two-step probe (`ccr --version` + `ccr claude profile export --help` & `switch --help` checking `--json`) tailored to `ccr >= 7.5`.
- **Dry-run gate adapted for `ExchangeV1`**: Specifically bound to `profile import <FILE> --dry-run --json` and its 6+1 conservation buckets rather than inventing synthetic dry-run flags on single-profile CRUD commands.
- **Stdlib CI contract validator adapted from `check_harness_contracts.py`**: Built `scripts/quality/check_ccr_cli_skill.py` so GitHub Actions ubuntu/windows/macos runners validate `skills/ccr-cli/` without requiring `~/.gemini` or `qiaomu-meta` installed on CI hosts.

### Reject

- **Splitting into multiple sibling skills (`ccr-profile`, `ccr-exchange`, `ccr-doctor`, or per-platform skills)**: Rejected because `claude`, `codex`, and `grok` share identical 12-verb lifecycle semantics, and multi-step user prompts ("export my codex profiles, import to grok/claude template, switch and validate") would suffer near-neighbor trigger contention across 27+ governed files.
- **Running `publish_skill.py` or standalone skill-repo release tags**: Explicitly rejected; `skills/ccr-cli/` ships inside `bahayonghang/ccr` via standard `dev -> main` PR governance without mutating git remotes or creating `v1.0.0` tags on the monorepo.
- **Coupling `skills/ccr-cli/manifest.json` (`1.0.0`) to `just version-sync` (`7.5.1`)**: Rejected because workspace bumps would desynchronize `manifest.json` from `reports/skill-ir.json` and `reports/creation-handoff.md`.
- **Wrapping `ccr <platform> auth`, TUI, `sync`, `update`, `clean`, or `gemini`/`droid`/`qwen` writes**: Rejected due to interactive browser/TTY blocking and distinct risk surfaces.

### Invent

- **Dual Exit-Code + Status-Card Health Gate**: Explicitly pairs non-zero exit code inspection (including `profile edit --json` partial-commit stdout receipts) with exit-0 JSON status-card `"health"` checks (`"error"` / `"invalid"` -> `ccr validate --json`).
- **TTY `rpassword` Anti-Hang + Sidecar Zero-Touch Contract**: Codifies non-interactive stdin piping from user env vars/files into `--auth-token-stdin` / `--api-key-stdin`, Grok `--env-key` preference, and strict read-prohibition on `profiles.db`, `profile_secrets.json`, and unredacted export bundles.
- **CAS `--expected-revision` Loop**: Connects `profile list --json` top-level `revision` directly into `profile edit` and `profile import` to prevent silent lost updates alongside TUI or `ccr-ui` desktop usage.
