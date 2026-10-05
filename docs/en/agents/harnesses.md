# Five harness entry files, load chains, and role permissions

This page records Claude Code, Codex, Grok Build, Kimi Code, and OMP (Oh My Pi) entry files in this repo, the current integration (hooks vs pull), official capability sources, and the difference between a pre-approval read-only reviewer and post-approval implement / self-fix check. Product facts stay in root `AGENTS.md`. Do not copy user-global rules five times into the repo. This page is not a VitePress product-nav item; `AGENTS.md` / `CLAUDE.md` link here.

Chinese: [harnesses](/agents/harnesses.md).

## Roles (all five tools)

| Role | When | Permissions |
|---|---|---|
| Read-only reviewer | Before approval: plans, source, official docs | Read the approved scope. Run authorized checks and write evidence only in the approved task directory. Do not change product code, global rules, accounts, or default models. |
| research | Authorized investigation | Read relevant sources. Write only in the assigned task research directory. |
| implement | After the user approves implementation | Edit `implement.md` whitelist files and run the listed checks. |
| Trellis check | After approval, as an executing role | **May write and self-fix** (lint, typecheck, missing tests) within approved files and acceptance criteria. Native role definitions reside in each tool's local agents directory. |

Grok built-in `plan` / `explore` have no shell/edit and cannot run test gates. Custom `.grok/agents` may have tools; that is not the same as built-in plan/explore.

`xhigh` / `low` / `medium` are **Codex** `model_reasoning_effort` values, not a five-tool universal parameter. Do not treat them as Claude Code, Grok, Kimi, or OMP knobs.

A browser, Playwright, or UI tool being **available** is not authorization to operate the UI.

## Official capability vs this repo

Record official capability, Git delivery, local generated configuration, and native runtime evidence separately. The local integration observations below are dated 2026-09-29. A fresh checkout needs separate initialization. File presence does not prove native loading or trust.

| Tool | Repo entry files | Current project integration | Official sources |
|---|---|---|---|
| Claude Code | `CLAUDE.md` (real `@AGENTS.md` import), `.claude/settings.json`, `.claude/hooks/`, `.claude/agents/`, `.claude/skills/` | SessionStart / PreToolUse / PostToolUse **hooks** inject Trellis context; three Trellis agents. Shared facts live only in `AGENTS.md`. | [memory / import](https://code.claude.com/docs/en/memory), [subagents](https://code.claude.com/docs/en/sub-agents) |
| Codex | `AGENTS.md`, `.codex/hooks.json`, `.codex/agents/`, `.agents/skills/`, `.codex/skills/` | **hooks** (SessionStart / UserPromptSubmit / SubagentStart). Files on disk do not prove the user enabled or trusted them. | [AGENTS.md](https://developers.openai.com/codex/guides/agents-md), [subagents](https://developers.openai.com/codex/subagents) |
| Grok Build | `.grok/agents/`, `.grok/skills/`, `.grok/commands/trellis-*.md` | **This repo uses a pull prelude**; no project hooks installed. Official Grok supports [agents](https://docs.x.ai/build/features/subagents) and [hooks](https://docs.x.ai/build/features/hooks). | [subagents](https://docs.x.ai/build/features/subagents), [hooks](https://docs.x.ai/build/features/hooks), [compatibility](https://docs.x.ai/build/features/skills-plugins-marketplaces) |
| Kimi Code | `.kimi-code/agents/`, `.kimi-code/skills/`, shared `.agents/skills/` | The audit machine has `trellis-implement`, `trellis-check`, and `trellis-research` project agents. They use pull context. Project hooks are absent. A fresh checkout without project agents can use a role with execution tools to load the matching skill and task documents explicitly. | [agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents), [hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html) |
| OMP | `.omp/agents/`, `.omp/skills/`, `.omp/extensions/trellis/` | TypeScript **extension** injects task context (`prd.md` / `design.md` / `implement.md` when present, plus role jsonl). No project `settings.json`; OMP scans `.omp/`. | [task](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/task.md), [context files](https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md) |

Local Trellis files may be customized (see `.agents/skills/trellis-meta/references/local-architecture/generated-files.md`). Do not hand-edit `.trellis/.template-hashes.json` or upstream templates.

## Shared skills and command side effects

The skill sources below are tracked under `.github/skills/` and apply to **all five tools**, not Codex only. Each client installs them into its own local skill directory (`.codex/skills/` and similar are gitignored local paths). Do not install new hooks just to match official capability, and do not copy the full rule set five times.

| Skill | Applies to | Notes |
|---|---|---|
| `.github/skills/ccr-ui-visual-workflow/SKILL.md` | `ccr-ui` visual work on any of the five | React + `DESIGN.md`. Default to the web preview, not the Tauri desktop shell. UI tools available ≠ UI operation authorization. |
| `.github/skills/ccr-gate-recovery/SKILL.md` | local gate recovery on any of the five | Rust tests use default parallelism and `--skip export_bindings`. Fixtures isolate shared environment state. The separate generation gate owns bindings. |
| Trellis start / implement / check / research | each tool’s agents or Kimi/Grok pull skills | “No hook” on Grok/Kimi means **this repo did not install them**, not a platform ceiling. |

Command classes:

- **Read-only checks**: `just version-check`, `just fmt-check`, `bun run type-check`, `cd docs && bun run audit`.
- **May rewrite files**: `just fmt`, `just version-sync`, some `lint` / `lint:fix`. Inspect the diff.
- **Tool prerequisites**: validation must fail clearly when a required tool is missing. Keep global installation separate from aggregate checks. Record unavailable gates as unverified.

## Git delivery and fresh checkouts

Repository delivery covers shared guidance and the text of the two CCR-specific skills, tracked under `.github/skills/`. Local install directories such as `.codex/` and `.omp/` remain generated assets under `.gitignore`, so `trellis update` can rewrite them without dirtying the working tree. Files on the audit machine do not establish fresh-checkout installation.

1. Check the installed Trellis CLI against the project version and read `trellis init --help`. Record version differences. Do not automatically upgrade global tools.
2. In a workspace authorized for initialization, use a matching CLI version and select `--claude`, `--codex`, `--grok`, `--kimi`, or `--omp`. Add `--skip-existing` to preserve repository customizations. Example: `trellis init --kimi --skip-existing`.
3. Inspect generated files and the diff. Verify native loading and trust separately. Preserve user trust settings. Do not copy personal configuration, credentials, or caches.

Generated `platform-map.md` files can reveal local generation drift. Shared contracts remain in this page and `AGENTS.md`. Controlled OMP tests verify document injection, cache refresh, path boundaries, and budgets. Native OMP sessions require separate validation.

Record OMP contract PASS only when a local extension generated by a matching Trellis version exists and all 14 tests execute and pass. Without the extension, the tests print `SKIPPED_UNVERIFIED` and skip all 14 tests. Exit code 0 and skipped tests do not close contract acceptance. Verify extension generation in a fresh checkout, native client loading, and trust separately.

## Model allocation

Strong models own root cause, security, permissions, and final review. Lower-cost models execute fixed YAML edits, bilingual synchronization, fixtures, and compatible version updates. Each assignment includes the task path, file whitelist, fixed rules, and acceptance commands. Return unknown causes or scope changes to the main session. Tool and role names do not determine account prices.
