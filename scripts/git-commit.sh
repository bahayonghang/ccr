#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# git-commit helper
#
# Git-only commit assistant that:
# - Validates repository state
# - Analyses staged/unstaged changes
# - Suggests split commits by concern
# - Generates Conventional Commit messages (optional emoji)
# - Executes `git add` / `git commit` when appropriate
#
# Usage:
#   scripts/git-commit.sh
#   scripts/git-commit.sh --emoji
#   scripts/git-commit.sh --all --signoff
#   scripts/git-commit.sh --amend
#   scripts/git-commit.sh --scope ui --type feat --emoji
#
# This script uses only git plus basic POSIX utilities.
###############################################################################

print_usage() {
  cat <<'EOF'
git-commit - Conventional Commit helper (git-only)

Usage:
  git-commit [options]

Options:
  --no-verify        Skip git hooks (passes --no-verify to git commit)
  --all              When nothing is staged, run "git add -A" before committing
  --amend            Amend the previous commit instead of creating a new one
  --signoff          Add Signed-off-by line (passes -s to git commit)
  --emoji            Prefix commit type with emoji (✨, 🐛, 📝, etc.)
  --scope <scope>    Explicit commit scope (e.g., ui, web, backend)
  --type <type>      Explicit commit type (feat, fix, docs, refactor, test...)
  -h, --help         Show this help message and exit

Behavior:
  - Validates you are in a git repository and not in the middle of
    a merge, rebase, or detached HEAD state.
  - If no changes are staged:
      * With --all: stages all changes via "git add -A".
      * Without --all: prints a summary and aborts so you can stage
        changes manually, or re-run with --all.
  - Suggests how to split commits by concern (docs/tests/code/config)
    based on currently staged files, without modifying staging.
  - Auto-detects commit type and scope when not provided:
      * docs: only documentation files changed
      * test: only tests changed
      * ci: only CI/workflow files changed
      * chore: only configs/scripts/metadata changed
      * feat/fix: source code changes (heuristic: diff contains "fix"/"bug"/"error"/"panic" → fix)
  - Writes a draft Conventional Commit message to .git/COMMIT_EDITMSG,
    then runs "git commit" with appropriate flags.

NOTE:
  - This helper never runs build tools or tests; it only uses git.
  - To undo staging for a file, run: git restore --staged <path>
EOF
}

die() {
  echo "git-commit: $*" >&2
  exit 1
}

ensure_git_repo() {
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    die "not inside a git work tree. Run this from a git repository."
  fi
}

detect_repo_state() {
  local git_dir
  git_dir="$(git rev-parse --git-dir 2>/dev/null || echo ".git")"

  # Rebase / merge detection via git dir markers
  if [[ -f "$git_dir/MERGE_HEAD" ]]; then
    die "repository is in a merge state. Resolve merge conflicts and commit manually first."
  fi
  if [[ -d "$git_dir/rebase-apply" || -d "$git_dir/rebase-merge" ]]; then
    die "repository is in a rebase state. Complete or abort the rebase before committing."
  fi

  # Detached HEAD
  local head_ref
  head_ref="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "HEAD")"
  if [[ "$head_ref" == "HEAD" ]]; then
    die "HEAD is detached. Create or switch to a branch before committing."
  fi
}

parse_args() {
  NO_VERIFY=false
  STAGE_ALL=false
  AMEND=false
  SIGNOFF=false
  EMOJI=false
  EXPLICIT_SCOPE=""
  EXPLICIT_TYPE=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --no-verify)
        NO_VERIFY=true
        shift
        ;;
      --all)
        STAGE_ALL=true
        shift
        ;;
      --amend)
        AMEND=true
        shift
        ;;
      --signoff)
        SIGNOFF=true
        shift
        ;;
      --emoji)
        EMOJI=true
        shift
        ;;
      --scope)
        [[ $# -ge 2 ]] || die "--scope requires a value"
        EXPLICIT_SCOPE="$2"
        shift 2
        ;;
      --type)
        [[ $# -ge 2 ]] || die "--type requires a value"
        EXPLICIT_TYPE="$2"
        shift 2
        ;;
      -h|--help)
        print_usage
        exit 0
        ;;
      *)
        die "unknown argument: $1"
        ;;
    esac
  done
}

get_staged_files() {
  git diff --cached --name-only
}

get_unstaged_files() {
  git diff --name-only
}

summarize_status() {
  echo "Current git status:"
  git status -sb || git status
  echo
}

stage_all_if_requested() {
  local staged unstaged
  staged="$(get_staged_files || true)"
  unstaged="$(get_unstaged_files || true)"

  if [[ -z "$staged" ]]; then
    if [[ -z "$unstaged" ]]; then
      die "no changes to commit."
    fi

    if [[ "$STAGE_ALL" == true ]]; then
      echo "No staged changes, staging all tracked changes with 'git add -A'..."
      git add -A
      return
    fi

    summarize_status
    cat <<'EOF'
No staged changes detected.

You have two options:
  1) Stage changes manually (e.g. with "git add <paths>") and rerun this command.
  2) Rerun this command with --all to stage all modified/deleted files automatically.

Tip: You can use "git restore --staged <paths>" to unstage files safely.
EOF
    exit 1
  fi
}

is_docs_file() {
  local f="$1"
  case "$f" in
    docs/*|*/docs/*|README*|*.md|*.rst|*.adoc|*.txt) return 0 ;;
  esac
  return 1
}

is_test_file() {
  local f="$1"
  case "$f" in
    tests/*|*/tests/*|*test.rs|*_test.rs|*_tests.rs|*test.js|*test.ts|*spec.js|*spec.ts) return 0 ;;
  esac
  return 1
}

is_ci_file() {
  local f="$1"
  case "$f" in
    .github/workflows/*|.gitlab-ci.yml|*.github/*/workflow*|*.github/workflows/*) return 0 ;;
  esac
  return 1
}

is_config_or_script_file() {
  local f="$1"
  case "$f" in
    *.toml|*.json|*.yaml|*.yml|justfile|Justfile|Makefile|Dockerfile|scripts/*) return 0 ;;
  esac
  return 1
}

is_code_file() {
  local f="$1"
  case "$f" in
    *.rs|*.c|*.h|*.cpp|*.hpp|*.ts|*.tsx|*.js|*.jsx|*.vue|*.py|*.sh|*.bash|*.zsh|*.go|*.java|*.kt|*.swift) return 0 ;;
  esac
  return 1
}

infer_commit_type() {
  local files="$1"

  if [[ -n "$EXPLICIT_TYPE" ]]; then
    echo "$EXPLICIT_TYPE"
    return
  fi

  local has_docs=false
  local has_tests=false
  local has_ci=false
  local has_config_or_script=false
  local has_code=false

  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if is_docs_file "$f"; then
      has_docs=true
    fi
    if is_test_file "$f"; then
      has_tests=true
    fi
    if is_ci_file "$f"; then
      has_ci=true
    fi
    if is_config_or_script_file "$f"; then
      has_config_or_script=true
    fi
    if is_code_file "$f"; then
      has_code=true
    fi
  done <<< "$files"

  if [[ "$has_docs" == true && "$has_code" == false && "$has_tests" == false && "$has_ci" == false ]]; then
    echo "docs"
    return
  fi
  if [[ "$has_tests" == true && "$has_code" == false ]]; then
    echo "test"
    return
  fi
  if [[ "$has_ci" == true && "$has_code" == false ]]; then
    echo "ci"
    return
  fi
  if [[ "$has_code" == false && "$has_config_or_script" == true ]]; then
    echo "chore"
    return
  fi

  # For code changes, try to distinguish fix vs feat heuristically
  if git diff --cached | grep -E -i 'fix|bug|panic|error' >/dev/null 2>&1; then
    echo "fix"
  else
    echo "feat"
  fi
}

infer_scope() {
  local files="$1"

  if [[ -n "$EXPLICIT_SCOPE" ]]; then
    echo "$EXPLICIT_SCOPE"
    return
  fi

  local scope=""
  local mixed=false

  while IFS= read -r f; do
    [[ -z "$f" ]] && continue

    local candidate=""
    case "$f" in
      ccr-ui/frontend/*)
        candidate="ui"
        ;;
      ccr-ui/backend/*)
        candidate="web"
        ;;
      src/*)
        candidate="core"
        ;;
      tests/*)
        candidate="tests"
        ;;
      docs/*)
        candidate="docs"
        ;;
      *)
        # Fallback: top-level directory or "repo"
        if [[ "$f" == */* ]]; then
          candidate="${f%%/*}"
        else
          candidate="repo"
        fi
        ;;
    esac

    if [[ -z "$scope" ]]; then
      scope="$candidate"
    elif [[ "$scope" != "$candidate" ]]; then
      mixed=true
      break
    fi
  done <<< "$files"

  if [[ "$mixed" == true || -z "$scope" ]]; then
    echo ""
  else
    echo "$scope"
  fi
}

detect_language() {
  # Simple heuristic: if recent commit subjects contain non-ASCII, assume Chinese.
  local subjects
  subjects="$(git log -n 50 --pretty='%s' 2>/dev/null || echo "")"

  if printf "%s" "$subjects" | LC_ALL=C grep -qP '[\x80-\xFF]'; then
    echo "zh"
  else
    echo "en"
  fi
}

emoji_for_type() {
  local type="$1"
  case "$type" in
    feat) echo "✨" ;;
    fix) echo "🐛" ;;
    docs) echo "📝" ;;
    style) echo "🎨" ;;
    refactor) echo "♻️" ;;
    perf) echo "⚡️" ;;
    test) echo "✅" ;;
    chore) echo "🔧" ;;
    ci) echo "👷" ;;
    revert) echo "⏪️" ;;
    *)
      echo ""
      ;;
  esac
}

build_subject() {
  local type="$1"
  local scope="$2"
  local lang="$3"

  if [[ "$lang" == "zh" ]]; then
    case "$type" in
      feat)
        if [[ -n "$scope" ]]; then
          echo "添加 ${scope} 相关功能"
        else
          echo "添加新功能"
        fi
        ;;
      fix)
        if [[ -n "$scope" ]]; then
          echo "修复 ${scope} 相关问题"
        else
          echo "修复问题"
        fi
        ;;
      docs)
        echo "更新文档"
        ;;
      test)
        echo "更新测试用例"
        ;;
      refactor)
        echo "重构代码"
        ;;
      chore)
        echo "更新配置和脚本"
        ;;
      ci)
        echo "更新 CI 配置"
        ;;
      *)
        echo "更新项目文件"
        ;;
    esac
  else
    # English
    case "$type" in
      feat)
        if [[ -n "$scope" ]]; then
          echo "add ${scope} changes"
        else
          echo "add new feature"
        fi
        ;;
      fix)
        if [[ -n "$scope" ]]; then
          echo "fix ${scope} issues"
        else
          echo "fix issues"
        fi
        ;;
      docs)
        echo "update documentation"
        ;;
      test)
        echo "update tests"
        ;;
      refactor)
        echo "refactor code"
        ;;
      chore)
        echo "update configs and scripts"
        ;;
      ci)
        echo "update CI configuration"
        ;;
      *)
        echo "update project files"
        ;;
    esac
  fi
}

build_body() {
  local type="$1"
  local scope="$2"
  local lang="$3"
  local files="$4"

  local file_list
  file_list="$(printf "%s\n" "$files" | sed '/^$/d' | sed 's/^/  - /')"

  if [[ "$lang" == "zh" ]]; then
    cat <<EOF
- 变更类型: ${type}
- 影响范围: ${scope:-(自动推断)}
- 涉及文件:
${file_list}

注: 提交信息由 git-commit 辅助脚本自动生成, 如有需要可手动编辑。
EOF
  else
    cat <<EOF
- Change type: ${type}
- Impact scope: ${scope:-(auto-inferred)}
- Touched files:
${file_list}

Note: Commit message generated by git-commit helper; feel free to edit if needed.
EOF
  fi
}

suggest_split_groups() {
  local files="$1"
  if [[ -z "$files" ]]; then
    return
  fi

  echo "Split suggestions (by concern):"

  local docs_files="" tests_files="" ci_files="" config_files="" code_files=""

  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if is_docs_file "$f"; then
      docs_files+=$'\n'"$f"
    elif is_test_file "$f"; then
      tests_files+=$'\n'"$f"
    elif is_ci_file "$f"; then
      ci_files+=$'\n'"$f"
    elif is_config_or_script_file "$f"; then
      config_files+=$'\n'"$f"
    else
      code_files+=$'\n'"$f"
    fi
  done <<< "$files"

  if [[ -n "$docs_files" ]]; then
    echo
    echo "  - docs:"
    printf '%s\n' "$docs_files" | sed '/^$/d' | sed 's/^/      /'
  fi
  if [[ -n "$tests_files" ]]; then
    echo
    echo "  - tests:"
    printf '%s\n' "$tests_files" | sed '/^$/d' | sed 's/^/      /'
  fi
  if [[ -n "$ci_files" ]]; then
    echo
    echo "  - ci:"
    printf '%s\n' "$ci_files" | sed '/^$/d' | sed 's/^/      /'
  fi
  if [[ -n "$config_files" ]]; then
    echo
    echo "  - config/scripts:"
    printf '%s\n' "$config_files" | sed '/^$/d' | sed 's/^/      /'
  fi
  if [[ -n "$code_files" ]]; then
    echo
    echo "  - code:"
    printf '%s\n' "$code_files" | sed '/^$/d' | sed 's/^/      /'
  fi

  cat <<'EOF'

You can create separate commits by staging groups with:
  git restore --staged <paths>           # Unstage mistakenly staged files
  git add <docs paths> && git commit ...
  git add <tests paths> && git commit ...
  git add <code paths> && git commit ...

This script currently performs a single commit based on the current staging area.
EOF
}

write_commit_message_and_commit() {
  local staged_files="$1"

  if [[ -z "$staged_files" ]]; then
    die "no staged files to commit."
  fi

  local type scope lang subject body
  type="$(infer_commit_type "$staged_files")"
  scope="$(infer_scope "$staged_files")"
  lang="$(detect_language)"
  subject="$(build_subject "$type" "$scope" "$lang")"
  body="$(build_body "$type" "$scope" "$lang" "$staged_files")"

  local header type_part scope_part emoji_prefix
  type_part="$type"
  scope_part=""

  if [[ -n "$scope" ]]; then
    scope_part="(${scope})"
  fi

  if [[ "$EMOJI" == true ]]; then
    emoji_prefix="$(emoji_for_type "$type")"
    if [[ -n "$emoji_prefix" ]]; then
      header="${emoji_prefix} ${type_part}${scope_part}: ${subject}"
    else
      header="${type_part}${scope_part}: ${subject}"
    fi
  else
    header="${type_part}${scope_part}: ${subject}"
  fi

  local git_dir
  git_dir="$(git rev-parse --git-dir 2>/dev/null || echo ".git")"
  local editmsg="${git_dir}/COMMIT_EDITMSG"

  {
    printf '%s\n\n' "$header"
    printf '%s\n' "$body"
  } > "$editmsg"

  echo "Draft commit message written to $editmsg:"
  echo "-----------------------------------------------------------------"
  head -n 10 "$editmsg" || cat "$editmsg"
  echo "-----------------------------------------------------------------"

  local commit_cmd=(git commit -F "$editmsg")
  if [[ "$NO_VERIFY" == true ]]; then
    commit_cmd+=(--no-verify)
  fi
  if [[ "$SIGNOFF" == true ]]; then
    commit_cmd+=(-s)
  fi
  if [[ "$AMEND" == true ]]; then
    commit_cmd+=(--amend)
  fi

  echo "Running: ${commit_cmd[*]}"
  "${commit_cmd[@]}"
}

main() {
  parse_args "$@"
  ensure_git_repo
  detect_repo_state

  stage_all_if_requested

  local staged_files
  staged_files="$(get_staged_files || true)"
  if [[ -z "$staged_files" ]]; then
    die "no staged files to commit after processing. Aborting."
  fi

  echo "Staged files:"
  printf '  %s\n' $staged_files
  echo

  suggest_split_groups "$staged_files"
  echo

  write_commit_message_and_commit "$staged_files"
}

main "$@"

