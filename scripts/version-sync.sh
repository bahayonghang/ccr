#!/usr/bin/env bash
set -euo pipefail

# 版本同步脚本（以根 Cargo.toml 为主）
# 同步到：
# - ccr-ui/backend/Cargo.toml
# - ccr-ui/frontend/package.json
# - ccr-ui/frontend/src-tauri/Cargo.toml
# - ccr-ui/frontend/src-tauri/tauri.conf.json

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"

ROOT_CARGO="$ROOT_DIR/Cargo.toml"
BACKEND_CARGO="$ROOT_DIR/ccr-ui/backend/Cargo.toml"
FRONTEND_PKG="$ROOT_DIR/ccr-ui/frontend/package.json"
TAURI_CARGO="$ROOT_DIR/ccr-ui/frontend/src-tauri/Cargo.toml"
TAURI_CONF="$ROOT_DIR/ccr-ui/frontend/src-tauri/tauri.conf.json"

die() {
  echo "❌ $1" >&2
  exit 1
}

require_file() {
  local f="$1"
  [[ -f "$f" ]] || die "文件不存在: $f"
}

require_file "$ROOT_CARGO"
require_file "$BACKEND_CARGO"
require_file "$FRONTEND_PKG"
require_file "$TAURI_CARGO"
require_file "$TAURI_CONF"

# 提取根 Cargo.toml 的 [package] 版本号
extract_root_version() {
  local content
  content="$(cat "$ROOT_CARGO")" || die "无法读取 $ROOT_CARGO"
  # 找到 [package] 区块并在其中匹配 version = "..."
  local pkg_block
  pkg_block="$(awk 'BEGIN{p=0} /^\[package\]/{p=1;print;next} /^\[/{if(p){exit};} p{print}' "$ROOT_CARGO")"
  [[ -n "$pkg_block" ]] || die "根 Cargo.toml 中缺少 [package] 区块"
  local ver
  ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
  [[ -n "$ver" ]] || die "根 Cargo.toml 的 [package] 区块中没有 version 字段"
  # 去除可能的 CR/LF 和首尾空白
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  printf "%s" "$ver"
}

CHECK_ONLY=false
VERBOSE=false
for arg in "$@"; do
  case "$arg" in
    --check|-c)
      CHECK_ONLY=true
      ;;
    --verbose|-v)
      VERBOSE=true
      ;;
  esac
done

ROOT_VER="$(extract_root_version)"

[[ "$VERBOSE" == true ]] && echo "🔧 根版本: $ROOT_VER"

# 获取当前后端版本
extract_backend_version() {
  local pkg_block
  pkg_block="$(awk 'BEGIN{p=0} /^\[package\]/{p=1;print;next} /^\[/{if(p){exit};} p{print}' "$BACKEND_CARGO")"
  [[ -n "$pkg_block" ]] || die "后端 Cargo.toml 中缺少 [package] 区块"
  local ver
  ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
  [[ -n "$ver" ]] || die "后端 Cargo.toml 的 [package] 区块中没有 version 字段"
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  printf "%s" "$ver"
}

BACKEND_VER="$(extract_backend_version)"
[[ "$VERBOSE" == true ]] && echo "🦀 后端版本: $BACKEND_VER"

# 获取当前前端版本
extract_frontend_version() {
  local ver
  ver="$(jq -r '.version // empty' "$FRONTEND_PKG" 2>/dev/null || true)"
  if [[ -z "$ver" || "$ver" == "null" ]]; then
    # 兼容没有 jq 的环境：用 sed 粗略解析
    ver="$(sed -nE 's/"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$FRONTEND_PKG" | head -n1)"
  fi
  [[ -n "$ver" ]] || die "前端 package.json 缺少 version 字段或解析失败"
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^\s\+//' -e 's/\s\+$//')"
  printf "%s" "$ver"
}

FRONTEND_VER="$(extract_frontend_version)"
[[ "$VERBOSE" == true ]] && echo "⚛️  前端版本: $FRONTEND_VER"

# 获取 Tauri Cargo.toml 版本
extract_tauri_cargo_version() {
  local pkg_block
  pkg_block="$(awk 'BEGIN{p=0} /^\[package\]/{p=1;print;next} /^\[/{if(p){exit};} p{print}' "$TAURI_CARGO")"
  [[ -n "$pkg_block" ]] || die "Tauri Cargo.toml 中缺少 [package] 区块"
  local ver
  ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
  [[ -n "$ver" ]] || die "Tauri Cargo.toml 的 [package] 区块中没有 version 字段"
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  printf "%s" "$ver"
}

TAURI_CARGO_VER="$(extract_tauri_cargo_version)"
[[ "$VERBOSE" == true ]] && echo "🖥️  Tauri Cargo 版本: $TAURI_CARGO_VER"

# 获取 Tauri tauri.conf.json 版本
extract_tauri_conf_version() {
  local ver
  ver="$(jq -r '.version // empty' "$TAURI_CONF" 2>/dev/null || true)"
  if [[ -z "$ver" || "$ver" == "null" ]]; then
    # 兼容没有 jq 的环境：用 sed 粗略解析
    ver="$(sed -nE 's/"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$TAURI_CONF" | head -n1)"
  fi
  [[ -n "$ver" ]] || die "Tauri tauri.conf.json 缺少 version 字段或解析失败"
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^\s\+//' -e 's/\s\+$//')"
  printf "%s" "$ver"
}

TAURI_CONF_VER="$(extract_tauri_conf_version)"
[[ "$VERBOSE" == true ]] && echo "🖥️  Tauri Conf 版本: $TAURI_CONF_VER"

if [[ "$CHECK_ONLY" == true ]]; then
  if [[ "$ROOT_VER" == "$BACKEND_VER" && "$ROOT_VER" == "$FRONTEND_VER" && "$ROOT_VER" == "$TAURI_CARGO_VER" && "$ROOT_VER" == "$TAURI_CONF_VER" ]]; then
    echo "✅ 版本一致性检查通过"
    exit 0
  else
    echo "❌ 版本不一致："
    echo "  root Cargo.toml:                        $ROOT_VER"
    echo "  ccr-ui/backend/Cargo.toml:              $BACKEND_VER"
    echo "  ccr-ui/frontend/package.json:           $FRONTEND_VER"
    echo "  ccr-ui/frontend/src-tauri/Cargo.toml:   $TAURI_CARGO_VER"
    echo "  ccr-ui/frontend/src-tauri/tauri.conf.json: $TAURI_CONF_VER"
    exit 1
  fi
fi

if [[ "$ROOT_VER" == "$BACKEND_VER" && "$ROOT_VER" == "$FRONTEND_VER" && "$ROOT_VER" == "$TAURI_CARGO_VER" && "$ROOT_VER" == "$TAURI_CONF_VER" ]]; then
  echo "✅ 版本一致，无需同步"
  exit 0
fi

echo "♻️  开始同步版本到 UI 文件..."

# 更新后端 Cargo.toml 的 [package] 区块 version
update_backend_version() {
  local tmp
  tmp="$(mktemp)"
  # 在 [package] 区块内替换第一条 version = "..."
  awk -v NEWVER="$ROOT_VER" '
    BEGIN{p=0;done=0}
    /^\[package\]/{p=1;print;next}
    /^\[/{if(p){p=0};}
    {
      if(p && !done && $0 ~ /^[[:space:]]*version[[:space:]]*=[[:space:]]*"[^"]*"/) {
        sub(/"[^"]*"/, "\"" NEWVER "\"");
        done=1;
      }
      print;
    }
  ' "$BACKEND_CARGO" > "$tmp" || die "更新后端版本失败"
  mv "$tmp" "$BACKEND_CARGO"
}

# 更新前端 package.json 的 version 字段
update_frontend_version() {
  if command -v jq >/dev/null 2>&1; then
    tmp="$(mktemp)"
    jq --arg v "$ROOT_VER" '.version = $v' "$FRONTEND_PKG" > "$tmp" || die "更新前端版本失败(jq)"
    mv "$tmp" "$FRONTEND_PKG"
  else
    # 无 jq 时用 sed 简单替换
    sed -i.bak -E "s/(\"version\"[[:space:]]*:[[:space:]]*)\"[^\"]*\"/\1\"$ROOT_VER\"/" "$FRONTEND_PKG" || die "更新前端版本失败(sed)"
    rm -f "$FRONTEND_PKG.bak"
  fi
}

# 更新 Tauri Cargo.toml 的 [package] 区块 version
update_tauri_cargo_version() {
  local tmp
  tmp="$(mktemp)"
  # 在 [package] 区块内替换第一条 version = "..."
  awk -v NEWVER="$ROOT_VER" '
    BEGIN{p=0;done=0}
    /^\[package\]/{p=1;print;next}
    /^\[/{if(p){p=0};}
    {
      if(p && !done && $0 ~ /^[[:space:]]*version[[:space:]]*=[[:space:]]*"[^"]*"/) {
        sub(/"[^"]*"/, "\"" NEWVER "\"");
        done=1;
      }
      print;
    }
  ' "$TAURI_CARGO" > "$tmp" || die "更新 Tauri Cargo.toml 版本失败"
  mv "$tmp" "$TAURI_CARGO"
}

# 更新 Tauri tauri.conf.json 的 version 字段
update_tauri_conf_version() {
  if command -v jq >/dev/null 2>&1; then
    tmp="$(mktemp)"
    jq --arg v "$ROOT_VER" '.version = $v' "$TAURI_CONF" > "$tmp" || die "更新 Tauri tauri.conf.json 版本失败(jq)"
    mv "$tmp" "$TAURI_CONF"
  else
    # 无 jq 时用 sed 简单替换
    sed -i.bak -E "s/(\"version\"[[:space:]]*:[[:space:]]*)\"[^\"]*\"/\1\"$ROOT_VER\"/" "$TAURI_CONF" || die "更新 Tauri tauri.conf.json 版本失败(sed)"
    rm -f "$TAURI_CONF.bak"
  fi
}

if [[ "$BACKEND_VER" != "$ROOT_VER" ]]; then
  echo "  - 后端: $BACKEND_VER -> $ROOT_VER"
  update_backend_version
fi

if [[ "$FRONTEND_VER" != "$ROOT_VER" ]]; then
  echo "  - 前端: $FRONTEND_VER -> $ROOT_VER"
  update_frontend_version
fi

if [[ "$TAURI_CARGO_VER" != "$ROOT_VER" ]]; then
  echo "  - Tauri Cargo.toml: $TAURI_CARGO_VER -> $ROOT_VER"
  update_tauri_cargo_version
fi

if [[ "$TAURI_CONF_VER" != "$ROOT_VER" ]]; then
  echo "  - Tauri tauri.conf.json: $TAURI_CONF_VER -> $ROOT_VER"
  update_tauri_conf_version
fi

echo "✅ 同步完成"