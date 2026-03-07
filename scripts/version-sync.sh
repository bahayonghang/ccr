#!/usr/bin/env bash
set -euo pipefail

# 版本同步脚本（以 crates/ccr/Cargo.toml 为主）
# 同步到：
# - crates/ccr-types/Cargo.toml
# - ccr-ui/package.json
# - ccr-ui/src-tauri/Cargo.toml
# - ccr-ui/src-tauri/tauri.conf.json

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"

ROOT_CARGO="$ROOT_DIR/Cargo.toml"
CCR_TYPES_CARGO="$ROOT_DIR/crates/ccr-types/Cargo.toml"
FRONTEND_PKG="$ROOT_DIR/ccr-ui/package.json"
TAURI_CARGO="$ROOT_DIR/ccr-ui/src-tauri/Cargo.toml"
TAURI_CONF="$ROOT_DIR/ccr-ui/src-tauri/tauri.conf.json"
COMPONENT_MAIN_LAYOUT="$ROOT_DIR/ccr-ui/src/components/MainLayout.vue"
LEGACY_MAIN_LAYOUT="$ROOT_DIR/ccr-ui/src/layouts/MainLayout.vue"

die() {
  echo "❌ $1" >&2
  exit 1
}

# 更新 CCR UI 侧边栏版本标识
update_ui_footer_version() {
  local file="$1"
  local tmp
  tmp="$(mktemp)"
  if ! grep -q "CCR UI v" "$file"; then
    rm -f "$tmp"
    die "在 $file 中找不到 CCR UI 版本标记"
  fi
  sed -E "s/(CCR UI v)[0-9A-Za-z._-]+/\1$ROOT_VER/g" "$file" > "$tmp" || {
    rm -f "$tmp"
    die "更新 $file 中的 CCR UI 版本失败"
  }
  mv "$tmp" "$file"
}

require_file() {
  local f="$1"
  [[ -f "$f" ]] || die "文件不存在: $f"
}

require_file "$ROOT_CARGO"
require_file "$CCR_TYPES_CARGO"
require_file "$FRONTEND_PKG"
require_file "$TAURI_CARGO"
require_file "$TAURI_CONF"
require_file "$COMPONENT_MAIN_LAYOUT"
require_file "$LEGACY_MAIN_LAYOUT"

# 提取根 Cargo.toml 的 [package] 版本号
extract_root_version() {
  local content
  content="$(cat "$ROOT_CARGO")" || die "无法读取 $ROOT_CARGO"
  # 找到 [package] 区块并在其中匹配 version = "..."
  local pkg_block
  pkg_block="$(awk 'BEGIN{p=0} /^\[(workspace\.)?package\]/{p=1;print;next} /^\[/{if(p){exit};} p{print}' "$ROOT_CARGO")"
  [[ -n "$pkg_block" ]] || die "根 Cargo.toml 中缺少 [workspace.package] 或 [package] 区块"
  local ver
  ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
  [[ -n "$ver" ]] || die "根 Cargo.toml 的 [workspace.package] 区块中没有 version 字段"
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

extract_ccr_types_version() {
  local pkg_block
  pkg_block="$(awk 'BEGIN{p=0} /^\[package\]/{p=1;print;next} /^\[/{if(p){exit};} p{print}' "$CCR_TYPES_CARGO")"
  [[ -n "$pkg_block" ]] || die "ccr-types Cargo.toml 中缺少 [package] 区块"
  
  if echo "$pkg_block" | grep -q "version.workspace[[:space:]]*=[[:space:]]*true"; then
    printf "%s" "$ROOT_VER"
  else
    local ver
    ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
    [[ -n "$ver" ]] || die "ccr-types Cargo.toml 的 [package] 区块中没有 version 字段"
    ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
    printf "%s" "$ver"
  fi
}

CCR_TYPES_VER="$(extract_ccr_types_version)"
[[ "$VERBOSE" == true ]] && echo "📦 ccr-types 版本: $CCR_TYPES_VER"

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
  
  if echo "$pkg_block" | grep -q "version.workspace[[:space:]]*=[[:space:]]*true"; then
    printf "%s" "$ROOT_VER"
  else
    local ver
    ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
    [[ -n "$ver" ]] || die "Tauri Cargo.toml 的 [package] 区块中没有 version 字段"
    ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
    printf "%s" "$ver"
  fi
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

# 获取 CCR UI 侧边栏（组件版）版本
extract_ui_footer_version() {
  local target="$1"
  local ver
  ver="$(sed -nE 's/.*CCR UI v([0-9A-Za-z._-]+).*/\1/p' "$target" | head -n1)"
  [[ -n "$ver" ]] || die "无法在 $target 中解析 CCR UI 版本号"
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  printf "%s" "$ver"
}

UI_COMPONENT_VER="$(extract_ui_footer_version "$COMPONENT_MAIN_LAYOUT")"
[[ "$VERBOSE" == true ]] && echo "🖼️  MainLayout.vue (components) 版本: $UI_COMPONENT_VER"

UI_LEGACY_LAYOUT_VER="$(extract_ui_footer_version "$LEGACY_MAIN_LAYOUT")"
[[ "$VERBOSE" == true ]] && echo "📐 MainLayout.vue (layouts) 版本: $UI_LEGACY_LAYOUT_VER"

if [[ "$CHECK_ONLY" == true ]]; then
  if [[ "$ROOT_VER" == "$CCR_TYPES_VER" && "$ROOT_VER" == "$FRONTEND_VER" && "$ROOT_VER" == "$TAURI_CARGO_VER" && "$ROOT_VER" == "$TAURI_CONF_VER" && "$ROOT_VER" == "$UI_COMPONENT_VER" && "$ROOT_VER" == "$UI_LEGACY_LAYOUT_VER" ]]; then
    echo "✅ 版本一致性检查通过"
    exit 0
  else
    echo "❌ 版本不一致："
    echo "  root Cargo.toml:                        $ROOT_VER"
    echo "  crates/ccr-types/Cargo.toml:            $CCR_TYPES_VER"
    echo "  ccr-ui/package.json:           $FRONTEND_VER"
    echo "  ccr-ui/src-tauri/Cargo.toml:   $TAURI_CARGO_VER"
    echo "  ccr-ui/src-tauri/tauri.conf.json: $TAURI_CONF_VER"
    echo "  ccr-ui/src/components/MainLayout.vue: $UI_COMPONENT_VER"
    echo "  ccr-ui/src/layouts/MainLayout.vue:   $UI_LEGACY_LAYOUT_VER"
    exit 1
  fi
fi

if [[ "$ROOT_VER" == "$CCR_TYPES_VER" && "$ROOT_VER" == "$FRONTEND_VER" && "$ROOT_VER" == "$TAURI_CARGO_VER" && "$ROOT_VER" == "$TAURI_CONF_VER" && "$ROOT_VER" == "$UI_COMPONENT_VER" && "$ROOT_VER" == "$UI_LEGACY_LAYOUT_VER" ]]; then
  echo "✅ 版本一致，无需同步"
  exit 0
fi

echo "♻️  开始同步版本到 UI 文件..."

if [[ "$FRONTEND_VER" != "$ROOT_VER" ]]; then
  echo "  - 前端: $FRONTEND_VER -> $ROOT_VER"
  update_frontend_version
fi

if [[ "$TAURI_CARGO_VER" != "$ROOT_VER" ]]; then
  echo "  - Tauri Cargo.toml: $TAURI_CARGO_VER -> $ROOT_VER"
  tmp="$(mktemp)"
  sed -E "s/^([[:space:]]*version[[:space:]]*=[[:space:]]*)\"[^\"]+\"/\1\"$ROOT_VER\"/" "$TAURI_CARGO" > "$tmp" || {
    rm -f "$tmp"
    die "更新 Tauri Cargo.toml 版本失败"
  }
  mv "$tmp" "$TAURI_CARGO"
fi

if [[ "$TAURI_CONF_VER" != "$ROOT_VER" ]]; then
  echo "  - Tauri tauri.conf.json: $TAURI_CONF_VER -> $ROOT_VER"
  update_tauri_conf_version
fi

if [[ "$UI_COMPONENT_VER" != "$ROOT_VER" ]]; then
  echo "  - 前端 MainLayout (components): $UI_COMPONENT_VER -> $ROOT_VER"
  update_ui_footer_version "$COMPONENT_MAIN_LAYOUT"
fi

if [[ "$UI_LEGACY_LAYOUT_VER" != "$ROOT_VER" ]]; then
  echo "  - 前端 MainLayout (layouts): $UI_LEGACY_LAYOUT_VER -> $ROOT_VER"
  update_ui_footer_version "$LEGACY_MAIN_LAYOUT"
fi

echo "✅ 同步完成"
