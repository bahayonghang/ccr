#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
ROOT_CARGO_PATH="$ROOT_DIR/Cargo.toml"
TAURI_CARGO_PATH="$ROOT_DIR/ccr-ui/src-tauri/Cargo.toml"
VERBOSE=false

for arg in "$@"; do
  case "$arg" in
    --verbose|-v) VERBOSE=true ;;
  esac
done

die() {
  echo "❌ $1" >&2
  exit 1
}

[[ -f "$ROOT_CARGO_PATH" ]] || die "文件不存在: $ROOT_CARGO_PATH"
[[ -f "$TAURI_CARGO_PATH" ]] || die "文件不存在: $TAURI_CARGO_PATH"

extract_section() {
  local file="$1"
  local section="$2"
  awk -v section="$section" '
    {
      line=$0
      sub(/\015$/, "", line)
    }
    line == "[" section "]" { in_section=1; next }
    /^\[/ && in_section { exit }
    in_section { print line }
  ' "$file"
}

extract_deps() {
  awk '
    /^[[:space:]]*($|#)/ { next }
    {
      line=$0
      sub(/\015$/, "", line)
      sub(/[[:space:]]*#.*/, "", line)
      if (line !~ /^[[:space:]]*[A-Za-z0-9_.-]+[[:space:]]*=/) next
      name=line
      sub(/[[:space:]]*=.*/, "", name)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", name)
      version=""
      if (match(line, /^[[:space:]]*[A-Za-z0-9_.-]+[[:space:]]*=[[:space:]]*"[^"]+"/)) {
        value=line
        sub(/^[^"]*"/, "", value)
        sub(/".*/, "", value)
        version=value
      } else if (match(line, /version[[:space:]]*=[[:space:]]*"[^"]+"/)) {
        value=line
        sub(/.*version[[:space:]]*=[[:space:]]*"/, "", value)
        sub(/".*/, "", value)
        version=value
      }
      if (version != "") print name "\t" version
    }
  '
}

find_version() {
  local deps="$1"
  local needle="$2"
  printf '%s\n' "$deps" | awk -F '\t' -v needle="$needle" '$1 == needle { print $2; found=1; exit } END { if (!found) exit 1 }'
}

has_dependency() {
  local deps="$1"
  local needle="$2"
  printf '%s\n' "$deps" | awk -F '\t' -v needle="$needle" '$1 == needle { found=1; exit } END { exit found ? 0 : 1 }'
}

is_allowed_drift() {
  case "$1" in
    chrono|dirs|reqwest|serde|serde_json|thiserror|tokio|toml|tracing|uuid|walkdir) return 0 ;;
    *) return 1 ;;
  esac
}

allowed_reason() {
  case "$1" in
    chrono) echo "Tauri manifest keeps a broad 0.4 constraint; lockfile resolves to a workspace-compatible 0.4.x release." ;;
    dirs) echo "Tauri manifest keeps a broad 6 constraint; lockfile resolves to the workspace-compatible 6.0.0 release." ;;
    reqwest) echo "Tauri manifest pins a lower 0.13 patch line until desktop HTTP behavior is rechecked; tracked as dependency-governance follow-up." ;;
    serde) echo "Tauri manifest keeps a broad 1.0 constraint; lockfile resolves to a workspace-compatible 1.0.x release." ;;
    serde_json) echo "Tauri manifest keeps a broad 1.0 constraint; lockfile resolves to a workspace-compatible 1.0.x release." ;;
    thiserror) echo "Tauri manifest keeps a broad 2 constraint; lockfile resolves to a compatible version selected by the workspace graph." ;;
    tokio) echo "Tauri manifest pins a lower 1.x floor for desktop runtime compatibility; tracked as dependency-governance follow-up." ;;
    toml) echo "Tauri slash-command parsing still uses the older toml API surface; keep explicit until migration is tested." ;;
    tracing) echo "Tauri manifest keeps a broad 0.1 constraint; lockfile resolves to a workspace-compatible 0.1.x release." ;;
    uuid) echo "Tauri manifest pins a lower 1.x floor without serde feature; keep explicit until feature parity is evaluated." ;;
    walkdir) echo "Tauri manifest keeps a broad 2 constraint; lockfile resolves to the workspace-compatible 2.5.0 release." ;;
  esac
}

WORKSPACE_DEPS="$(extract_section "$ROOT_CARGO_PATH" "workspace.dependencies" | extract_deps | sort)"
TAURI_DEPS="$(extract_section "$TAURI_CARGO_PATH" "dependencies" | extract_deps | sort)"
[[ -n "$WORKSPACE_DEPS" ]] || die "根 Cargo.toml 缺少可解析的 [workspace.dependencies]"
[[ -n "$TAURI_DEPS" ]] || die "ccr-ui/src-tauri/Cargo.toml 缺少可解析的 [dependencies]"

failures=""
drifts=""
checked=0
while IFS=$'\t' read -r name tauri_version; do
  [[ -n "$name" ]] || continue
  if ! workspace_version="$(find_version "$WORKSPACE_DEPS" "$name")"; then
    continue
  fi
  checked=$((checked + 1))
  [[ "$workspace_version" != "$tauri_version" ]] || continue
  entry="$name root=$workspace_version tauri=$tauri_version"
  if is_allowed_drift "$name"; then
    drifts="${drifts}${entry} reason=$(allowed_reason "$name")
"
  else
    failures="${failures}${entry}
"
  fi
done <<EOF
$TAURI_DEPS
EOF

for name in chrono dirs reqwest serde serde_json thiserror tokio toml tracing uuid walkdir; do
  if ! has_dependency "$WORKSPACE_DEPS" "$name" || ! has_dependency "$TAURI_DEPS" "$name"; then
    failures="${failures}allowlist entry '$name' no longer maps to a repeated dependency
"
    continue
  fi
  workspace_version="$(find_version "$WORKSPACE_DEPS" "$name")"
  tauri_version="$(find_version "$TAURI_DEPS" "$name")"
  if [[ "$workspace_version" == "$tauri_version" ]]; then
    failures="${failures}allowlist entry '$name' is stale because versions now match
"
  fi
done

if [[ -n "$failures" ]]; then
  echo "❌ Root/Tauri dependency drift check failed:" >&2
  printf '%s' "$failures" | while IFS= read -r failure; do
    [[ -n "$failure" ]] || continue
    echo "  - $failure" >&2
  done
  echo "Add an explicit reason to the allowlist only after reviewing the drift." >&2
  exit 1
fi

if [[ "$VERBOSE" == true ]]; then
  echo "🔎 Repeated dependencies checked: $checked"
  if [[ -n "$drifts" ]]; then
    echo "📌 Explicitly allowed root/Tauri dependency drifts:"
    printf '%s' "$drifts" | while IFS= read -r drift; do
      [[ -n "$drift" ]] || continue
      echo "  - $drift"
    done
  else
    echo "📌 No root/Tauri dependency drifts found."
  fi
fi

echo "✅ root/Tauri dependency drift 检查通过"
