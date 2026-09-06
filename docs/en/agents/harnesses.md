# Five harness entry files, load chains, and role permissions

This page records Claude Code, Codex, Grok Build, Kimi Code, and OMP (Oh My Pi) entry files in this repo, the current integration (hooks vs pull), official capability sources, and the difference between a pre-approval read-only reviewer and post-approval implement / self-fix check. Product facts stay in root `AGENTS.md`. Do not copy user-global rules five times into the repo. This page is not a VitePress product-nav item; `AGENTS.md` / `CLAUDE.md` link here.

Chinese: [harnesses](/agents/harnesses.md).

## Roles (all five tools)

| Role | When | Permissions |
|---|---|---|
| Read-only reviewer | Before approval: plans, source, official docs | Read the repo. Do not edit code or user-global AGENTS, accounts, or default models. |
| implement | After the user approves implementation | Edit `implement.md` whitelist files and run the listed checks. |
| Trellis check | After approval, as an executing role | **May write and self-fix** (lint, typecheck, missing tests). Not a pre-approval read-only reviewer. See `.codex/agents/trellis-check.toml` and `.claude/agents/trellis-check.md`. |

Grok built-in `plan` / `explore` have no shell/edit and cannot run test gates. Custom `.grok/agents` may have tools; that is not the same as built-in plan/explore.

`xhigh` / `low` / `medium` are **Codex** `model_reasoning_effort` values, not a five-tool universal parameter. Do not treat them as Claude Code, Grok, Kimi, or OMP knobs.

A browser, Playwright, or UI tool being **available** is not authorization to operate the UI.

## Official capability vs this repo

Do not write “the platform cannot do X” when official docs now support it. This repo may still use a manual pull prelude and may not install project hooks/agents. The “current integration” column is this repository only.

| Tool | Repo entry files | Current project integration | Official sources |
|---|---|---|---|
| Claude Code | `CLAUDE.md` (real `@AGENTS.md` import), `.claude/settings.json`, `.claude/hooks/`, `.claude/agents/`, `.claude/skills/` | SessionStart / PreToolUse / PostToolUse **hooks** inject Trellis context; three Trellis agents. Shared facts live only in `AGENTS.md`. | [memory / import](https://code.claude.com/docs/en/memory), [subagents](https://code.claude.com/docs/en/sub-agents) |
| Codex | `AGENTS.md`, `.codex/hooks.json`, `.codex/agents/`, `.agents/skills/`, `.codex/skills/` | **hooks** (SessionStart / UserPromptSubmit / SubagentStart). Files on disk do not prove the user enabled or trusted them. | [AGENTS.md](https://developers.openai.com/codex/guides/agents-md), [subagents](https://developers.openai.com/codex/subagents) |
| Grok Build | `.grok/agents/`, `.grok/skills/`, `.grok/commands/trellis-*.md` | **This repo uses a pull prelude**; no project hooks installed. Official Grok supports [agents](https://docs.x.ai/build/features/subagents) and [hooks](https://docs.x.ai/build/features/hooks). | [subagents](https://docs.x.ai/build/features/subagents), [hooks](https://docs.x.ai/build/features/hooks), [compatibility](https://docs.x.ai/build/features/skills-plugins-marketplaces) |
| Kimi Code | `.kimi-code/skills/trellis-*.md`, shared `.agents/skills/` | **This repo uses pull skills**; the main session dispatches built-in `coder` as the Trellis role. No project agents/hooks installed. Official Kimi supports [agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents) and [hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html). | [agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents), [hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html) |
| OMP | `.omp/agents/`, `.omp/skills/`, `.omp/extensions/trellis/` | TypeScript **extension** injects task context (`prd.md` / `design.md` / `implement.md` when present, plus role jsonl). No project `settings.json`; OMP scans `.omp/`. | [task](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/task.md), [context files](https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md) |

Local Trellis files may be customized (see `.agents/skills/trellis-meta/references/local-architecture/generated-files.md`). Do not hand-edit `.trellis/.template-hashes.json` or upstream templates.

## Shared skills and command side effects

Skills under `.codex/skills/` below apply to **all five tools**, not Codex only. Do not install new hooks just to match official capability, and do not copy the full rule set five times.

| Skill | Applies to | Notes |
|---|---|---|
| `.codex/skills/ccr-ui-visual-workflow/SKILL.md` | `ccr-ui` visual work on any of the five | React + `DESIGN.md`. Default to the web preview, not the Tauri desktop shell. UI tools available ≠ UI operation authorization. |
| `.codex/skills/ccr-gate-recovery/SKILL.md` | local gate recovery on any of the five | Keep existing **parallel** test gates; Rust `--test-threads=1` is the existing flake mitigation, not a new serial recovery engine. Do not invent a second CI. |
| Trellis start / implement / check / research | each tool’s agents or Kimi/Grok pull skills | “No hook” on Grok/Kimi means **this repo did not install them**, not a platform ceiling. |

Command classes:

- **Read-only checks**: `just version-check`, `just fmt-check`, `bun run type-check`, `cd docs && bun run audit`.
- **May rewrite files**: `just fmt`, `just version-sync`, some `lint` / `lint:fix`. Inspect the diff.
- **May install tools**: steps inside aggregate `just ci` such as audit. A green command does not replace a missing required gate.

## Approved child write-back (P2 still open)

Each row includes tool, command, exit, role/model, and UNVERIFIED. This task did not start five hosted harness sessions.

### ui-smoke · F1

- **Whitelist**: `ccr-ui/tests/shell/route-view-mount.smoke.test.tsx`
- **Commands**: original `bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx` was exit **1**, now **0**; `type-check` / `lint:ci` / full `bun run test` exit **0**.
- **Tool / role**: dispatch `trellis-implement` then `trellis-check` **PASS**. Resolved model UNVERIFIED.
- **UNVERIFIED**: hosted Frontend CI, native Tauri desktop.

### ci-verdict · F2 / F3

- **Change**: vscode-ci coverage step `shell: bash`; `.cargo/tauri-ci.toml` → tauri; `.cargo/config.toml` → root+tauri; `.cargo/audit.toml` → root.
- **Commands**: unittest **24 OK**; `check_workflow_governance` exit **0**; vscode-coverage exit **0** (70% threshold unchanged).
- **UNVERIFIED**: hosted GitHub Actions, branch protection.

### omp-context · F5

- **Behavior**: `buildTaskContext` injects `design.md` / `implement.md` when present.
- **Command**: `bun test scripts/trellis/omp-context.test.ts` **5/5** exit **0**.
- **UNVERIFIED**: a real OMP session. The four `.omp` whitelist files are tracked via path-level `git add -f`; `.gitignore` still lists `.omp/`.

### ci-history

Approved but **not yet executed**. Leave as pending P2. This page does not claim the history-evidence review is done.

## Grok / Kimi rows in platform-map

The four `platform-map.md` copies (under `.agents`, `.claude`, `.grok`, `.omp` `trellis-meta/references/platform-files/`) only correct the Grok/Kimi rows: official agents/hooks exist; this repo still uses pull. Other tools’ existing text is unchanged.
