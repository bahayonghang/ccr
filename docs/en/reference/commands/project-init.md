# `project init` - Initialize A Project Workflow

`ccr project init` prepares a Git repository, a Trellis workflow, and local Agent-directory ignore rules in the current working directory. It does not change user-level CCR configuration; [`ccr init`](./init) continues to own that configuration.

## Usage

```bash
# Run Trellis initialization interactively
ccr project init

# Forward the global --yes flag to Trellis
ccr -y project init
```

The command always targets the directory from which CCR was started. It does not accept a project path.

## Prerequisites

Install these commands and make them available on `PATH` before running the command:

- Git (`git`)
- Trellis CLI (`trellis`)

CCR does not install either tool, and it does not copy Trellis's username prompt, Agent platform registry, or platform flags.

## Stages

### 1. Git

CCR first runs `git rev-parse --show-toplevel`:

- Outside any Git worktree, it runs `git init`.
- At an existing repository root, it skips `git init`.
- Inside a parent repository, it reports that repository root and skips `git init` to avoid creating a nested repository.

When a parent repository is reused, the Trellis and `.gitignore` stages still target the directory where the command was invoked.

### 2. Trellis

Interactive mode inherits the current terminal and runs:

```bash
trellis init
```

Trellis owns username discovery and Agent platform selection. For example, you can select Claude Code and Codex in Trellis's native prompt.

With the global `-y` / `--yes` flag, CCR runs:

```bash
trellis init --yes
```

CCR checks both the Trellis exit status and a minimum workflow postcondition: `.trellis/workflow.md` and `.trellis/scripts/task.py` must exist in the current directory. A zero exit status without those files is treated as a failure.

### 3. `.gitignore`

CCR preserves existing content, comments, ordering, and LF/CRLF line endings. It appends only these missing rules:

```text
.agents/
.claude/
.codex/
```

When all rules already exist, the file is not rewritten. Updates use an atomic write, and repeated runs do not duplicate rules.

## Failures And Retry

Stages always run in this order: Git, Trellis, `.gitignore`. A failed stage stops the command without reporting overall success and does not roll back completed external operations:

- If Git is unavailable or `git init` fails, Trellis is not invoked.
- If Trellis is unavailable, exits unsuccessfully, or misses the minimum workflow files, `.gitignore` is not updated.
- If `.gitignore` cannot be read or written, the completed Git and Trellis results remain in place.

Fix the reported cause and rerun `ccr project init` in the same directory. Git detection, Trellis reinitialization, and ignore-rule merging are designed for safe retries.

## See Also

- [`init`](./init) - initialize user-level CCR configuration
- [Command Reference](./index) - browse all commands
