# Check 2.3 notes (trellis-check)

Date: 2026-09-06. Role: independent trellis-check after F4a/F6a repair round 1. Did not rewrite product files. Did not git commit / push / amend / `git add -f .trellis/`. Did not start five harness clients. Last `implement.md` review checkbox marked because verdict is PASS.

## Verdict

**PASS**

F4 / F6 remain fixed. F4a and F6a are fixed (progress vs prior open signatures). No F4b / F6b.

## Ledger

| id | status | path_or_scope | issue | evidence |
|---|---|---|---|---|
| F4 | fixed | AGENTS.md / CLAUDE.md / code_map / crates / ccr-ui maps | Vue + pinned llmusage crate vs CLI+SQLite; inert Claude `@` imports; ccr-ui Vue/Pinia; crates omit ccr-usage; facade CLAUDE monolith tree | AGENTS React 19 + CLI+SQLite/`ccr-usage` + `PascalCase.tsx`; CLAUDE.md:5 bare `@AGENTS.md`; crates list `ccr-usage`; `crates/ccr/src/` is `main.rs`/`lib.rs`/`cli/mod.rs` |
| F6 | fixed | harness skills / docs | Kimi/Grok “platform cannot”; Vue in visual skill; gate-recovery Codex-only + serialized tests; xhigh as universal | Grok/Kimi + four `platform-map.md` say official agents/hooks exist, this repo still pull; visual skill is React; gate-recovery five-tool + keep parallel; CLAUDE.md labels xhigh as Codex-only |
| F4a | fixed | `ccr-ui/CLAUDE.md` | Stale Tauri command tree (`commands/stats.rs`, `commands/droid.rs`, “141+ / 13 子模块”) and version pins vs `package.json` | Command tree now points at `commands/mod.rs` + `handler_registry.rs` + `docs/reference/tauri-command-inventory.md`. `mod.rs` has ~30 `pub mod`s, no `stats`/`droid`. Stack pins match `package.json`: React 19.2.8, Vite 8.2.2, React Router 8.3.1, Zustand 5.0.15, TanStack Query 5.102.8, Tailwind 4.3.3, `@tauri-apps/api` 2.11.1, TypeScript `^6.0.3`. |
| F6a | fixed | `.codex/skills/ccr-ui-visual-workflow/SKILL.md` | Duplicate `## Tauri Boundary` / `## Evidence To Report`; second copy dropped authorization | Single `## Tauri Boundary` (lines 43–47) and single `## Evidence To Report` (49–55). Authorization bullet present: “whether UI operation was explicitly authorized”. Also in frontmatter/intro. No Vue leftover. |

## Commands / exit

| cwd | command | exit |
|---|---|---|
| repo root | `git diff --check` | 0 |
| `docs/` | `bun run audit` | 0 (`docs audit passed`) |
| `docs/` | `bun run build` | 0 (VitePress 1.6.4, 4.11s) |

`docs/.vitepress/config.mjs` has no `harnesses` nav entry. zh/en harness pages exist.

## Not raised (same as prior check; not F4a/F6a signatures)

- `CLAUDE.md` still says `just ci` is 13 steps; `justfile` `_ci-timed-*` lists 14 including `frontend-coverage`. Same bullet tells readers to trust justfile.
- `ccr-ui/CLAUDE.md` platform tree lists `local.rs`/`wsl.rs` and omits existing `ssh.rs`/`config_path.rs`. `ExecutionEnvironment` sample is not the live async trait. Supported-platform Droid line still claims MCP/Agents while `platformCapabilities.droid` has those flags false. Adjacent, not the F4a command-tree/pin signature.

## UNVERIFIED

- Real five-harness hosted sessions (not started)
- Resolved implement/check models
- Hosted Frontend CI / native Tauri (ui-smoke write-back)
- Hosted GitHub Actions / branch protection (ci-verdict write-back)
- Real OMP session (omp-context write-back)
- Numeric write-back exits from sibling tasks (dirs not re-run here)
- Claude `@AGENTS.md` import actually loading in a real Claude Code session
- User enabled/trusted Codex/Claude project hooks
- ci-history P2
