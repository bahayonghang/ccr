#!/usr/bin/env bash
set -euo pipefail

# 版本同步脚本（Bash 版本）
# 以 crates/ccr/Cargo.toml 为主，同步到各目标文件
#
# ═══════════════════════════════════════════════════════════
# 📋 同步目标配置（单一数据源）
# 修改同步逻辑时，优先检查此配置表
# ═══════════════════════════════════════════════════════════
# 格式：名称:相对路径:类型(cargo|json|vue)
# 新增目标：在此添加配置行，并检查 extract/update 函数是否支持该类型
# 删除目标：从此表中移除即可
SYNC_TARGETS=(
  "ccr-types:crates/ccr-types/Cargo.toml:cargo"
  "ccr-db:crates/ccr-db/Cargo.toml:cargo"
  "frontend:ccr-ui/package.json:json"
  "tauri-cargo:ccr-ui/src-tauri/Cargo.toml:cargo"
  "tauri-conf:ccr-ui/src-tauri/tauri.conf.json:json"
  "ui-component:ccr-ui/src/components/MainLayout.vue:vue"
  "ui-legacy:ccr-ui/src/layouts/MainLayout.vue:vue"
  "vscode:ccr-vscode/package.json:json"
)
# ═══════════════════════════════════════════════════════════

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"

# 安全：确保脚本退出时清理所有临时文件
TEMP_FILES=()
cleanup_temp() {
  for tmp in "${TEMP_FILES[@]}"; do
    rm -f "$tmp" 2>/dev/null || true
  done
}
trap cleanup_temp EXIT ERR INT TERM

ROOT_CARGO="$ROOT_DIR/Cargo.toml"
CCR_TYPES_CARGO="$ROOT_DIR/crates/ccr-types/Cargo.toml"
CCR_DB_CARGO="$ROOT_DIR/crates/ccr-db/Cargo.toml"
FRONTEND_PKG="$ROOT_DIR/ccr-ui/package.json"
TAURI_CARGO="$ROOT_DIR/ccr-ui/src-tauri/Cargo.toml"
TAURI_CONF="$ROOT_DIR/ccr-ui/src-tauri/tauri.conf.json"
COMPONENT_MAIN_LAYOUT="$ROOT_DIR/ccr-ui/src/components/MainLayout.vue"
LEGACY_MAIN_LAYOUT="$ROOT_DIR/ccr-ui/src/layouts/MainLayout.vue"
VSCODE_PKG="$ROOT_DIR/ccr-vscode/package.json"

die() {
  echo "❌ $1" >&2
  exit 1
}

# 更新 CCR UI 侧边栏版本标识
update_ui_footer_version() {
  local file="$1"
  local tmp
  tmp="$(mktemp)"
  TEMP_FILES+=("$tmp")
  if grep -Eq 'APP_VERSION_LABEL|APP_VERSION|packageJson\.version' "$file"; then
    rm -f "$tmp"
    return
  fi
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

# 更新 Cargo.toml 版本（如果已使用 version.workspace = true 则跳过）
update_cargo_version() {
  local file="$1"
  local new_ver="$2"
  local tmp
  tmp="$(mktemp)"
  TEMP_FILES+=("$tmp")

  # 检查是否使用 workspace 版本继承
  if awk 'BEGIN{p=0} /^\[(workspace\.)?package\]/{p=1;next} /^\[/{p=0} p && /version\.workspace[[:space:]]*=[[:space:]]*true/{found=1;exit} END{exit !found}' "$file"; then
    if [[ "$VERBOSE" == true ]]; then
      echo "  ⏭️  $(basename "$file") 使用 workspace 版本继承，跳过"
    fi
    rm -f "$tmp"
    return
  fi

  # 执行版本更新
  sed -E "s/^([[:space:]]*version[[:space:]]*=[[:space:]]*)\"[^\"]+\"/\1\"$new_ver\"/" "$file" > "$tmp" || {
    rm -f "$tmp"
    die "更新 $file 版本失败"
  }
  mv "$tmp" "$file"
}

require_file() {
  local f="$1"
  [[ -f "$f" ]] || die "文件不存在: $f"
}

require_file "$ROOT_CARGO"
require_file "$CCR_TYPES_CARGO"
require_file "$CCR_DB_CARGO"
require_file "$FRONTEND_PKG"
require_file "$TAURI_CARGO"
require_file "$TAURI_CONF"
require_file "$COMPONENT_MAIN_LAYOUT"
require_file "$LEGACY_MAIN_LAYOUT"
require_file "$VSCODE_PKG"

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

extract_ccr_db_version() {
  local pkg_block
  pkg_block="$(awk 'BEGIN{p=0} /^\[package\]/{p=1;print;next} /^\[/{if(p){exit};} p{print}' "$CCR_DB_CARGO")"
  [[ -n "$pkg_block" ]] || die "ccr-db Cargo.toml 中缺少 [package] 区块"
  
  if echo "$pkg_block" | grep -q "version.workspace[[:space:]]*=[[:space:]]*true"; then
    printf "%s" "$ROOT_VER"
  else
    local ver
    ver="$(printf "%s" "$pkg_block" | sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
    [[ -n "$ver" ]] || die "ccr-db Cargo.toml 的 [package] 区块中没有 version 字段"
    ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
    printf "%s" "$ver"
  fi
}

CCR_DB_VER="$(extract_ccr_db_version)"
[[ "$VERBOSE" == true ]] && echo "📦 ccr-db 版本: $CCR_DB_VER"

# 获取当前前端版本
extract_frontend_version() {
  local ver
  ver="$(jq -r '.version // empty' "$FRONTEND_PKG" 2>/dev/null || true)"
  if [[ -z "$ver" || "$ver" == "null" ]]; then
    # 兼容没有 jq 的环境：用 sed 粗略解析
    ver="$(sed -nE 's/.*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$FRONTEND_PKG" | head -n1)"
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
    ver="$(sed -nE 's/.*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$TAURI_CONF" | head -n1)"
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
  if [[ -z "$ver" ]]; then
    if grep -Eq 'APP_VERSION_LABEL|APP_VERSION|packageJson\.version' "$target"; then
      printf "%s" "$FRONTEND_VER"
      return
    fi
    die "无法在 $target 中解析 CCR UI 版本号"
  fi
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  printf "%s" "$ver"
}

UI_COMPONENT_VER="$(extract_ui_footer_version "$COMPONENT_MAIN_LAYOUT")"
[[ "$VERBOSE" == true ]] && echo "🖼️  MainLayout.vue (components) 版本: $UI_COMPONENT_VER"

UI_LEGACY_LAYOUT_VER="$(extract_ui_footer_version "$LEGACY_MAIN_LAYOUT")"
[[ "$VERBOSE" == true ]] && echo "📐 MainLayout.vue (layouts) 版本: $UI_LEGACY_LAYOUT_VER"

# 获取 VSCode 扩展版本
extract_vscode_version() {
  local ver
  ver="$(jq -r '.version // empty' "$VSCODE_PKG" 2>/dev/null || true)"
  if [[ -z "$ver" || "$ver" == "null" ]]; then
    # 兼容没有 jq 的环境：用 sed 粗略解析
    ver="$(sed -nE 's/.*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$VSCODE_PKG" | head -n1)"
  fi
  [[ -n "$ver" ]] || die "VSCode package.json 缺少 version 字段或解析失败"
  ver="$(printf "%s" "$ver" | tr -d '\r' | sed -e 's/^\s\+//' -e 's/\s\+$//')"
  printf "%s" "$ver"
}

VSCODE_VER="$(extract_vscode_version)"
[[ "$VERBOSE" == true ]] && echo "🔌 VSCode 扩展版本: $VSCODE_VER"

if [[ "$CHECK_ONLY" == true ]]; then
  if [[ "$ROOT_VER" == "$CCR_TYPES_VER" && "$ROOT_VER" == "$CCR_DB_VER" && "$ROOT_VER" == "$FRONTEND_VER" && "$ROOT_VER" == "$TAURI_CARGO_VER" && "$ROOT_VER" == "$TAURI_CONF_VER" && "$ROOT_VER" == "$UI_COMPONENT_VER" && "$ROOT_VER" == "$UI_LEGACY_LAYOUT_VER" && "$ROOT_VER" == "$VSCODE_VER" ]]; then
    echo "✅ 版本一致性检查通过"
    exit 0
  else
    echo "❌ 版本不一致："
    echo "  root Cargo.toml:                        $ROOT_VER"
    echo "  crates/ccr-types/Cargo.toml:            $CCR_TYPES_VER"
    echo "  crates/ccr-db/Cargo.toml:               $CCR_DB_VER"
    echo "  ccr-ui/package.json:           $FRONTEND_VER"
    echo "  ccr-ui/src-tauri/Cargo.toml:   $TAURI_CARGO_VER"
    echo "  ccr-ui/src-tauri/tauri.conf.json: $TAURI_CONF_VER"
    echo "  ccr-ui/src/components/MainLayout.vue: $UI_COMPONENT_VER"
    echo "  ccr-ui/src/layouts/MainLayout.vue:   $UI_LEGACY_LAYOUT_VER"
    echo "  ccr-vscode/package.json:       $VSCODE_VER"
    exit 1
  fi
fi

if [[ "$ROOT_VER" == "$CCR_TYPES_VER" && "$ROOT_VER" == "$CCR_DB_VER" && "$ROOT_VER" == "$FRONTEND_VER" && "$ROOT_VER" == "$TAURI_CARGO_VER" && "$ROOT_VER" == "$TAURI_CONF_VER" && "$ROOT_VER" == "$UI_COMPONENT_VER" && "$ROOT_VER" == "$UI_LEGACY_LAYOUT_VER" && "$ROOT_VER" == "$VSCODE_VER" ]]; then
  echo "✅ 版本一致，无需同步"
  exit 0
fi

echo "♻️  开始同步版本到 UI 文件..."

if [[ "$CCR_DB_VER" != "$ROOT_VER" ]]; then
  echo "  - ccr-db: $CCR_DB_VER -> $ROOT_VER"
  update_cargo_version "$CCR_DB_CARGO" "$ROOT_VER"
fi

update_frontend_version() {
  local tmp
  tmp="$(mktemp)"
  TEMP_FILES+=("$tmp")
  if command -v jq &>/dev/null; then
    jq --arg ver "$ROOT_VER" '.version = $ver' "$FRONTEND_PKG" > "$tmp" || {
      rm -f "$tmp"
      die "更新前端 package.json 版本失败"
    }
  else
    sed -E "s/(\"version\"[[:space:]]*:[[:space:]]*\")[^\"]+\"/\1$ROOT_VER\"/" "$FRONTEND_PKG" > "$tmp" || {
      rm -f "$tmp"
      die "更新前端 package.json 版本失败"
    }
  fi
  mv "$tmp" "$FRONTEND_PKG"
}

if [[ "$FRONTEND_VER" != "$ROOT_VER" ]]; then
  echo "  - 前端: $FRONTEND_VER -> $ROOT_VER"
  update_frontend_version
fi

if [[ "$TAURI_CARGO_VER" != "$ROOT_VER" ]]; then
  echo "  - Tauri Cargo.toml: $TAURI_CARGO_VER -> $ROOT_VER"
  update_cargo_version "$TAURI_CARGO" "$ROOT_VER"
fi

update_tauri_conf_version() {
  local tmp
  tmp="$(mktemp)"
  TEMP_FILES+=("$tmp")
  if command -v jq &>/dev/null; then
    jq --arg ver "$ROOT_VER" '.version = $ver' "$TAURI_CONF" > "$tmp" || {
      rm -f "$tmp"
      die "更新 Tauri tauri.conf.json 版本失败"
    }
  else
    sed -E "s/(\"version\"[[:space:]]*:[[:space:]]*\")[^\"]+\"/\1$ROOT_VER\"/" "$TAURI_CONF" > "$tmp" || {
      rm -f "$tmp"
      die "更新 Tauri tauri.conf.json 版本失败"
    }
  fi
  mv "$tmp" "$TAURI_CONF"
}

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

# 更新 VSCode 扩展版本
update_vscode_version() {
  local tmp
  tmp="$(mktemp)"
  TEMP_FILES+=("$tmp")
  if command -v jq &>/dev/null; then
    jq --arg ver "$ROOT_VER" '.version = $ver' "$VSCODE_PKG" > "$tmp" || {
      rm -f "$tmp"
      die "更新 VSCode package.json 版本失败"
    }
  else
    sed -E "s/(\"version\"[[:space:]]*:[[:space:]]*\")[^\"]+\"/\1$ROOT_VER\"/" "$VSCODE_PKG" > "$tmp" || {
      rm -f "$tmp"
      die "更新 VSCode package.json 版本失败"
    }
  fi
  mv "$tmp" "$VSCODE_PKG"
}

if [[ "$VSCODE_VER" != "$ROOT_VER" ]]; then
  echo "  - VSCode 扩展: $VSCODE_VER -> $ROOT_VER"
  update_vscode_version
fi

echo "✅ 同步完成"
