#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
README_PATH="$ROOT_DIR/ccr-ui/README.md"
PACKAGE_PATH="$ROOT_DIR/ccr-ui/package.json"
BUN_LOCK_PATH="$ROOT_DIR/ccr-ui/bun.lock"
NPM_LOCK_PATH="$ROOT_DIR/ccr-ui/package-lock.json"
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

for f in "$README_PATH" "$PACKAGE_PATH" "$BUN_LOCK_PATH" "$TAURI_CARGO_PATH"; do
  [[ -f "$f" ]] || die "文件不存在: $f"
done

[[ ! -f "$NPM_LOCK_PATH" ]] || die "ccr-ui/package-lock.json 存在；ccr-ui 只维护 Bun/bun.lock"

extract_json_field() {
  local field="$1"
  local file="$2"
  local value=""
  if command -v jq >/dev/null 2>&1; then
    value="$(jq -r ".$field // empty" "$file" 2>/dev/null || true)"
  fi
  if [[ -z "$value" || "$value" == "null" ]]; then
    value="$(sed -nE "s/.*\"$field\"[[:space:]]*:[[:space:]]*\"([^\"]+)\".*/\1/p" "$file" | head -n1)"
  fi
  printf '%s' "$value"
}

FRONTEND_VERSION="$(extract_json_field version "$PACKAGE_PATH")"
PACKAGE_MANAGER="$(extract_json_field packageManager "$PACKAGE_PATH")"
[[ -n "$FRONTEND_VERSION" ]] || die "ccr-ui/package.json 缺少 version 字段"
[[ "$PACKAGE_MANAGER" =~ ^bun@[0-9] ]] || die "ccr-ui/package.json#packageManager 必须声明 bun@x.y.z，当前: $PACKAGE_MANAGER"

RUST_VERSION="$(sed -nE 's/^[[:space:]]*rust-version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$TAURI_CARGO_PATH" | head -n1)"
EDITION="$(sed -nE 's/^[[:space:]]*edition[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$TAURI_CARGO_PATH" | head -n1)"
[[ -n "$RUST_VERSION" ]] || die "ccr-ui/src-tauri/Cargo.toml 缺少 rust-version"
[[ -n "$EDITION" ]] || die "ccr-ui/src-tauri/Cargo.toml 缺少 edition"

require_readme() {
  local needle="$1"
  grep -Fq "$needle" "$README_PATH" || die "ccr-ui/README.md 缺少当前事实: $needle"
}

require_readme "version-$FRONTEND_VERSION"
require_readme "Bun is the only maintained frontend package manager"
require_readme "bun.lock is the dependency source of truth"
require_readme "Bun | \`$PACKAGE_MANAGER\`"
require_readme "Rust | \`>= $RUST_VERSION\`"
require_readme "Rust edition | Edition $EDITION"
require_readme "Tauri invoke APIs"
require_readme "Web runtime"
require_readme "bun run lint:fix"

stale_patterns=(
  "version-2.5.0"
  "TypeScript-5.7"
  "Rust >= 1.70"
  "Edition 2021"
  "Tokio 1.48"
  "Axios"
  "HTTP API"
  "13 个命令"
  "Web 模式: 浏览器访问，通过 HTTP API"
  "自动检测环境，透明切换后端"
)
for pattern in "${stale_patterns[@]}"; do
  if grep -Fq "$pattern" "$README_PATH"; then
    die "ccr-ui/README.md 仍包含过期描述: $pattern"
  fi
done

if [[ "$VERBOSE" == true ]]; then
  echo "📄 ccr-ui/README.md version: $FRONTEND_VERSION"
  echo "📦 package manager: $PACKAGE_MANAGER"
  echo "🦀 rust-version: $RUST_VERSION, edition: $EDITION"
  echo "🔒 JS lock strategy: bun.lock only"
fi
echo "✅ 文档/锁文件 drift 检查通过"
