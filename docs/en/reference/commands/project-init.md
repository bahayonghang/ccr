# `project init` - Initialize A Project Workflow

`ccr project init` prepares a Git repository, Trellis platforms and a developer name, a Trellis workflow, and an ignored local `ref/` directory in the current working directory. It does not change user-level CCR configuration; [`ccr init`](./init) continues to own that configuration.

## Usage

```bash
# Interactive mode: five platforms checked by default, then a manual username
ccr project init

# Non-interactive: the five platforms plus git config user.name, passed to trellis init --yes
ccr -y project init
```

The command always targets the directory from which CCR was started. It does not accept a project path.

## Prerequisites

Install these commands and make them available on `PATH` before running the command:

- Git (`git`)
- Trellis CLI (`trellis`)

CCR does not install either tool. The platform menu is fixed to the five entries below and does not copy Trellis's full platform registry.

## Stages

### 1. Git

CCR first runs `git rev-parse --show-toplevel`:

- Outside any Git worktree, it runs `git init`.
- At an existing repository root, it skips `git init`.
- Inside a parent repository, it reports that repository root and skips `git init` to avoid creating a nested repository.

When a parent repository is reused, platform selection, Trellis, `ref/`, and `.gitignore` still target the directory where the command was invoked. A Git failure does not ask for platforms or a username.

### 2. Platforms And Username

After Git succeeds, interactive mode shows a checkbox menu. Every item starts checked, and any item can be cleared:

- Claude Code (`--claude`)
- Codex (`--codex`)
- Cursor (`--cursor`)
- Oh My Pi (`--omp`)
- Grok (`--grok`)

Clearing every item shows the menu again and does not start Trellis. After at least one platform remains, the command asks for a developer name. An empty entry asks again until the name is non-empty.

`ccr -y project init` shows neither the menu nor the username prompt. The name comes from `git config user.name` in the current directory. If that command cannot start, initialization fails. A non-zero exit, empty output, or whitespace-only output fails with guidance to set `git config user.name` or rerun without `-y` and type the name. That failure happens before Trellis starts.

### 3. Trellis

Interactive mode inherits the current terminal and appends only the platforms that remain, in menu order:

```bash
trellis init -u <entered name> --claude --codex --cursor --omp --grok
```

Clearing one item omits its flag. The remaining flags stay in the order above. Without `--yes`, Trellis may still ask about the Claude Code statusLine and the spec template.

When a Git username is available, `ccr -y project init` runs:

```bash
trellis init -u <git config user.name> --claude --codex --cursor --omp --grok --yes
```

CCR checks both the Trellis exit status and a minimum workflow postcondition: `.trellis/workflow.md` and `.trellis/scripts/task.py` must exist in the current directory. A zero exit status without those files is treated as a failure.

### 4. `ref/` And `.gitignore`

After the minimum Trellis files are present:

- An existing `ref/` directory keeps its contents.
- A regular file at `ref` fails the command and leaves `.gitignore` unchanged.
- A missing `ref/` directory is created.

CCR then preserves existing content, comments, ordering, and LF/CRLF line endings. It appends only these missing rules:

```text
.agents/
.claude/
.codex/
ref/
```

When all four rules already exist, the file is not rewritten. A missing `ref/` directory is still created when the ignore rules are already complete. Updates use an atomic write, and repeated runs do not duplicate rules.

## Failures And Retry

Stages always run in this order: Git, platforms and username, Trellis, `ref/`, `.gitignore`. A failed stage stops the command without reporting overall success and does not roll back completed external operations:

- If Git is unavailable or `git init` fails, CCR does not ask for platforms or a username, does not run Trellis, does not create `ref/`, and does not update `.gitignore`.
- If platform selection fails, or `-y` cannot read a Git username, Trellis is not invoked.
- If Trellis is unavailable, exits unsuccessfully, or misses the minimum workflow files, `ref/` is not created and `.gitignore` is not updated.
- If `ref` is a regular file, `.gitignore` is not updated.
- If `.gitignore` cannot be read or written, the completed Git, Trellis, and `ref/` results remain in place.

Fix the reported cause and rerun `ccr project init` in the same directory. Git detection, Trellis reinitialization, the `ref/` directory check, and ignore-rule merging are designed for safe retries.

## See Also

- [`init`](./init) - initialize user-level CCR configuration
- [Command Reference](./index) - browse all commands
