---
name: durable-plan-resume
description: Resume and execute repo-local durable planning workflows that use task_plan.md, progress.md, and findings.md. Use when the user says continue, resume, implement the plan in a fresh context, 请开始实施, 请继续实施, 请按照plan开始实施, 请对当前plan进行审计, or asks to update/refine a plan after analysis. Especially useful for this user's recurring planning-with-files work across llmusage, llmtop, skills-manage-windows, academic-writing-skills, and similar repos.
---

# Durable Plan Resume

Use repo planning files as the source of truth. The user repeatedly expects the plan files to carry state across turns and context resets.

## Workflow

1. Restore the current state before deciding anything.
   - Read `task_plan.md`, `progress.md`, and `findings.md` if they exist.
   - Check `git status --short` and recent diffs so file state and plan state can be reconciled.
   - If a repo has a catchup script, use it when cheap, then fold only relevant recovered facts into the planning files.

2. Determine the active mode from the user's wording.
   - `请先...plan`, `不要修改代码`, or `创建实施plan`: stay read-only and produce/update a plan.
   - `Implement the plan in a fresh context`, `请开始实施`, `请按照plan开始实施`, or `continue`: execute the existing plan instead of reopening design.
   - `请对当前plan进行审计`: audit the plan against current files and persist the audit to the planning files.
   - `请根据以上分析完善plan`: edit the plan files; do not stop at chat commentary.

3. Keep durable files synchronized.
   - `task_plan.md`: stages, acceptance criteria, current status, scope cuts.
   - `findings.md`: evidence, root causes, risks, failures, decisions, external-source summaries.
   - `progress.md`: actions taken, commands run, verification output, blockers, next step.
   - Update during implementation, not only at the end.

4. Preserve plan semantics.
   - Do not silently change a plan-only request into implementation.
   - Do not keep stale wording such as `待用户授权动手` when the user has already authorized safe local execution.
   - If the plan says not to change a backend API, not to auto-generate paid/network data, or to preserve a safety boundary, keep that boundary explicit.

5. Verify against the plan, not just tests.
   - Build a checklist from the plan's acceptance criteria.
   - Run targeted checks first, then the repo's expected gate when feasible.
   - If docs, screenshots, generated fixtures, or planning files are part of completion, verify and update them before final reporting.

## Failure Patterns To Avoid

- Replanning from scratch after `continue`.
- Saying "plan complete" while `progress.md` still advertises an open blocker.
- Treating targeted test success as completion when docs, screenshots, or the full gate remain in the plan.
- Copying untrusted web or external text into `task_plan.md`; put source summaries in `findings.md`.

## Output

Report the current phase, files updated, verification evidence, and any remaining plan items. Mention if the request was handled as plan-only or implementation.
