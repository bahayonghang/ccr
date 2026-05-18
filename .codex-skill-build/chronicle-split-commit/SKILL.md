---
name: chronicle-split-commit
description: Memory-backed local split-commit workflow for this user's recurring requests to commit all current changes with Chinese Conventional Commit subjects, emoji, and repo-specific hook constraints. Use when the user asks to split commit all changes, especially with wording like `$git-commit 请使用中文拆分提交所有的改动 ,使用emoji`, `所有改动`, `全部提交`, or repeated `continue` during local commit sessions across repos such as ccr, llmusage, skills-manage-windows, skills-janitor, llmtop, JTICE, or my-claude-code-settings.
---

# Chronicle Split Commit

Turn the user's recurring Chronicle pattern into a disciplined local commit workflow. This skill augments, rather than replaces, a generic git-commit skill.

## Workflow

1. Treat the requested scope literally.
   - If the user says `所有改动`, inventory unstaged, staged, deleted, and untracked files.
   - Keep the workflow local unless the user explicitly asks for push, PR, release, or remote checks.
   - If the user says `continue`, resume the remaining commit slices instead of stopping after one commit.

2. Read the repository contract.
   - Read `AGENTS.md` and any deeper scoped instructions.
   - Inspect recent commits to match local message style, helper scripts, hooks, and trailers.
   - If the repo uses a commit composer, verify the current flags before assuming old ones. The repeated Windows pattern often uses `compose_commit_message.ps1 --summary`.

3. Inventory before staging.
   - Run `git status --short` or porcelain v2 with untracked files.
   - Run `git diff --stat`, `git diff`, and staged variants as needed.
   - Note unrelated dirty-state drift, but do not revert it. Include it only when it is part of the user's `all changes` scope.

4. Choose split boundaries by reversibility, not aesthetics.
   - Prefer logical slices: feature/fix, version metadata, docs/TODO state, backend vs frontend, schema vs UI, tests vs docs.
   - Keep a coupled chain atomic when splitting would leave non-buildable intermediate commits. Explain the rejected split in the message body or Lore `Rejected:` trailer when the repo expects it.
   - Do not use hunk-level staging unless the boundary is obvious and safe.

5. Prove the slices with the smallest useful checks.
   - Before committing, run the narrowest checks that prove the current slice.
   - Common memory-backed checks include `git diff --check`, formatter checks, targeted unit tests, `pnpm sizecheck`, `cargo fmt --check`, and repo-specific `just ci` or `just ui-check`.
   - If an aggregate gate times out or hides the true failure, run its underlying steps directly and stop at the first real blocker.

6. Commit with the repo's required message protocol.
   - Default to Chinese Conventional Commit subjects with emoji when the user asks in that recurring form.
   - Preserve required local conventions such as `[AI]`, Lore trailers, OmX trailers, or inline `git commit -m` if hooks reject `git commit -F`.
   - Do not claim a commit succeeded until reading the command output.

7. Verify the end state.
   - Confirm each expected commit exists with `git log --oneline`.
   - Confirm no intended files are left behind with `git status --short`.
   - If files remain, treat leftovers as still in scope when the user asked for all changes.

## Repeated Decisions

- Chinese prompt plus `拆分提交所有的改动` means Chinese commit subjects, emoji, all dirty-tree files, and local split commits by default.
- If the tree is too coupled to split safely, one atomic commit is better than a theatrical split that breaks intermediate states.
- Version bumps, TODO/progress docs, and generated metadata are often separate slices when they can stand alone.
- Never report a remembered commit preference as a completed commit without fresh git evidence.

## Output

Report:

- commit hashes and headers
- split rationale, including any rejected split
- verification commands and outcomes
- remaining dirty files or verification gaps
