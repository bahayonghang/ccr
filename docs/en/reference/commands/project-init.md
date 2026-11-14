# `project init` - Initialize A Project Workflow

`ccr project init` runs a four-stage terminal wizard in the current working directory to prepare a Git repository, a project workflow engine (`mini-trellis` or `trellis`), an ignored local `ref/` directory, and the seven fixed Agent `.gitignore` rules, finishing with a structured status summary card and next-step commands. It does not modify user-level CCR configuration; [`ccr init`](./init) continues to own user-level configuration.

## Usage

```bash
# Interactive wizard: recommends mini-trellis by default and prefills git config user.name
ccr project init

# Explicitly select a workflow engine (skips engine selection and goes to platform & username prompts)
ccr project init --workflow mini-trellis
ccr project init --workflow trellis

# Non-interactive backward-compatible default: trellis with all 5 platforms + git config user.name
ccr -y project init

# Non-interactive mini-trellis initialization: enables all 4 platforms + git config user.name
ccr -y project init --workflow mini-trellis
```

`--workflow <trellis|mini-trellis>` is case-insensitive. The command does not expose a `--json` flag and always targets the directory from which CCR was started; it does not accept a project path argument.

## Options And Workflow Engines

| Aspect | `mini-trellis` | `trellis` |
|---|---|---|
| Scope | Lightweight specs, research notes, and cross-session memory layer | Full task-driven multi-stage workflow |
| Downstream executable | `mini-trellis` | `trellis` |
| Default mode | Highlighted first as recommended in the interactive wizard | Default when `ccr -y project init` is run without `--workflow` (backward compatible) |
| Supported host platforms | **4 items**: Claude Code (`--claude`), Codex (`--codex`), Cursor (`--cursor`), Oh My Pi (`--omp`) — **excludes Grok** | **5 items**: Claude Code (`--claude`), Codex (`--codex`), Cursor (`--cursor`), Oh My Pi (`--omp`), Grok (`--grok`) |
| Minimum postconditions | `.trellis/.mini-trellis-manifest.json`, `.trellis/scripts/add_session.py`, and `AGENTS.md` containing `Managed by mini-trellis.` | `.trellis/workflow.md` and `.trellis/scripts/task.py` |
| Next-step suggestions | `mini-trellis doctor` and `mini-trellis mem recall "project init"` | `trellis doctor` and `ccr project init --help` |

## Prerequisites

Install these tools and make them available on `PATH` before running the command:

- Git (`git`)
- The CLI matching the selected workflow engine: `mini-trellis` or `trellis`

CCR does not install external CLIs automatically. The platform menu is restricted to the supported sets listed above and never exposes or passes unlisted flags such as `--kimi`.

## Four-Stage Workflow

At startup, CCR prints a header block showing the target directory (`目标目录`), run mode (`运行模式`), and—when `--workflow` is explicitly passed—the workflow preset (`工作流预设`), then executes four numbered stages in order:

### 1. `[1/4]` Check Git Repository

CCR first runs `git rev-parse --show-toplevel` in the current directory:

- Outside any Git worktree, it runs `git init`.
- At an existing repository root, it reuses the repository and skips `git init`.
- Inside a parent repository, it warns with the detected parent repository root and skips nested `git init` to avoid creating a nested repository.

Even when a parent repository is reused, workflow initialization, `ref/`, and `.gitignore` still operate strictly on the directory where the command was invoked. A Git failure stops before any prompt or workflow CLI call.

### 2. `[2/4]` Configure Workflow And Host Platforms

At the start of stage 2, CCR inspects workspace fingerprint markers (see "Cross-Engine Conflict Guard And Retry" below):

- **Interactive mode**:
  - When `--workflow` is omitted, CCR displays a single-select menu with `mini-trellis` highlighted first as the recommended choice; passing `--workflow` skips this menu.
  - Immediately after the target engine is chosen, CCR checks for cross-engine conflicts before prompting further. If compatible, it displays the host platform multi-select menu (`5` items for `trellis`, `4` items for `mini-trellis`, all checked by default). Clearing every item prints a warning and re-opens the menu without starting the downstream CLI.
  - CCR silently probes `git config user.name` in the current directory: a non-empty value prefills the developer name prompt (press Enter to accept), while an unset value requires typing a non-empty name and re-prompts on blank input.
- **Non-interactive mode (`-y` / `--yes`)**:
  - Shows no interactive prompts. Omitting `--workflow` defaults to `trellis`, and CCR enables every supported platform for the resolved engine (`5` for `trellis` or `4` for `mini-trellis`).
  - Reads the developer name strictly from `git config user.name` in the current directory. Spawn failure, non-zero exit status, or blank output fails before invoking the downstream CLI, advising you to set `git config user.name` or rerun without `-y`.

Once resolved, CCR prints confirmation lines for the selected workflow engine, host platforms, and developer name.

### 3. `[3/4]` Run Workflow Initialization

CCR invokes the selected CLI with inherited standard streams (`Stdio::inherit()`) wrapped between visual start/end boundary lines (`┌─` / `└─` box rules on normal TTYs, degrading to plain `---` separators on non-TTY pipes and `TERM=dumb`):

```bash
# When trellis is selected (--yes appended only in -y mode)
trellis init -u <developer> --claude --codex --cursor --omp --grok [--yes]

# When mini-trellis is selected (no --grok; --yes appended only in -y mode)
mini-trellis init -u <developer> --claude --codex --cursor --omp [--yes]
```

Unchecking a platform in interactive mode omits its flag while keeping the remaining flags in menu order; without `--yes`, the downstream CLI may still ask its own interactive questions (such as `statusLine` or template prompts in `trellis`). CCR **never passes `-s` / `--skip-existing`**, ensuring same-engine reruns enter downstream reinitialization cleanly.

After the downstream CLI exits with code `0`, CCR verifies the engine's minimum postcondition files. Missing any required artifact fails validation with exit code `90` and halts before stage 4.

### 4. `[4/4]` Prepare `ref/` And `.gitignore` Rules

Once minimum workflow artifacts are verified:

- An existing `ref/` directory keeps its contents.
- A regular file (or non-directory entry) at `ref` fails with exit code `51` and leaves `.gitignore` untouched.
- A missing `ref/` directory is created.

Regardless of which workflow engine or platforms were selected, CCR preserves existing `.gitignore` content, comments, ordering, and LF/CRLF line endings while appending any missing entries from the fixed seven-rule baseline:

```text
.agents/
.claude/
.codex/
.cursor/
.omp/
.grok/
ref/
```

`.cursor/`, `.omp/`, and `.grok/` cover local host directories, and `.agents/` is the shared Codex and cross-agent skills directory; `.trellis/` holds tracked workflow and spec assets and stays out of `.gitignore`. When all seven rules already exist, `.gitignore` is not rewritten (though a missing `ref/` directory is still created). Updates use atomic writes, never duplicate rules, and upgrade older four-rule files with `.cursor/`, `.omp/`, and `.grok/`.

### Summary Card And Terminal Adaptation

When all four stages succeed, CCR prints:

1. **A 6-row summary table** covering Git repository status, workflow engine (initial setup `首次初始化` vs idempotent update `幂等更新`), enabled host platforms, developer name, `ref/` directory, and `.gitignore`.
2. **A final ready status line** and **engine-specific next-step commands**.

When piped, or when `NO_COLOR=1`, `CLICOLOR=0`, or `TERM=dumb` is set, prompts, status prefixes, subprocess boundaries, and summary tables omit ANSI color escapes automatically (and switch to ASCII table borders under `TERM=dumb`) so CI logs and captured output remain clean.

## Cross-Engine Conflict Guard And Retry

### Preflight Workspace Fingerprint Guard (Exit Code `90`)

To prevent cross-engine runs from corrupting an initialized directory (notably, running `trellis init -y` inside an existing `mini-trellis` workspace exits `0` upstream while leaving a broken hybrid state), CCR checks `Trellis` and `mini-trellis` markers before starting any external workflow CLI:

- **Existing `trellis` assets when targeting `mini-trellis`**: blocked with exit code `90` and guidance to preview with `mini-trellis migrate --dry-run` before running `mini-trellis migrate`.
- **Existing `mini-trellis` assets when targeting `trellis` (including bare `ccr -y project init`)**: blocked preflight with exit code `90` and guidance to run `mini-trellis uninstall` first if switching engines.
- **Both `trellis` and `mini-trellis` markers present (mixed workspace)**: blocked immediately with exit code `90` before any prompts or external calls until the mixed assets are cleaned up.

All conflict blocks happen before invoking `trellis` / `mini-trellis`, creating `ref/`, or touching `.gitignore`.

### Failure Boundaries And Idempotent Retry

A failure in any stage stops the command immediately, omits the summary card and next-step block, and leaves previously completed stages in place without rollback:

- Missing `git` or failed `git init` (exit code `75`): skips prompts, workflow CLI, `ref/`, and `.gitignore`.
- Interactive prompt error (exit code `74`), missing `git config user.name` under `-y`, or workspace conflict (exit code `90`): skips workflow CLI, `ref/`, and `.gitignore`.
- Missing or non-zero `trellis` / `mini-trellis` execution (exit code `75`), or missing minimum workflow artifacts (exit code `90`): keeps the Git repository result without creating `ref/` or updating `.gitignore`.
- Regular file conflict at `ref` or `.gitignore` read/write error (exit code `51`): keeps completed Git and workflow results.

Fix the reported cause and rerun `ccr project init` with the same workflow engine in the same directory. Same-engine reruns preserve `ref/` contents, custom specs, research notes, journals, and template hashes, reporting an idempotent update in the summary card.

## See Also

- [`init`](./init) - initialize user-level CCR configuration
- [Command Reference](./index) - browse all commands
