#!/usr/bin/env bats
# version-sync.sh 测试套件
# 需要安装 bats: https://github.com/bats-core/bats-core

setup() {
  # 创建临时测试目录
  TEST_DIR="$(mktemp -d)"
  ROOT_DIR="$TEST_DIR/ccr-test"
  mkdir -p "$ROOT_DIR/crates/ccr"
  mkdir -p "$ROOT_DIR/crates/ccr-types"
  mkdir -p "$ROOT_DIR/crates/ccr-db"
  mkdir -p "$ROOT_DIR/ccr-ui/src-tauri"
  mkdir -p "$ROOT_DIR/ccr-ui/src/components"
  mkdir -p "$ROOT_DIR/ccr-ui/src/layouts"
  mkdir -p "$ROOT_DIR/ccr-vscode"

  # 创建根 Cargo.toml
  cat > "$ROOT_DIR/Cargo.toml" <<EOF
[package]
name = "ccr"
version = "1.2.3"
edition = "2021"
EOF

  # 创建 ccr-types Cargo.toml
  cat > "$ROOT_DIR/crates/ccr-types/Cargo.toml" <<EOF
[package]
name = "ccr-types"
version = "1.2.3"
edition = "2021"
EOF

  # 创建 ccr-db Cargo.toml
  cat > "$ROOT_DIR/crates/ccr-db/Cargo.toml" <<EOF
[package]
name = "ccr-db"
version = "1.2.3"
edition = "2021"
EOF

  # 创建前端 package.json
  cat > "$ROOT_DIR/ccr-ui/package.json" <<EOF
{
  "name": "ccr-ui",
  "version": "1.2.3",
  "private": true
}
EOF

  # 创建 Tauri Cargo.toml
  cat > "$ROOT_DIR/ccr-ui/src-tauri/Cargo.toml" <<EOF
[package]
name = "ccr-ui-tauri"
version = "1.2.3"
edition = "2021"
EOF

  # 创建 Tauri 配置
  cat > "$ROOT_DIR/ccr-ui/src-tauri/tauri.conf.json" <<EOF
{
  "version": "1.2.3",
  "build": {
    "beforeDevCommand": "npm run dev"
  }
}
EOF

  # 创建 Vue 组件
  cat > "$ROOT_DIR/ccr-ui/src/components/MainLayout.vue" <<EOF
<template>
  <div class="footer">CCR UI v1.2.3</div>
</template>
EOF

  cat > "$ROOT_DIR/ccr-ui/src/layouts/MainLayout.vue" <<EOF
<template>
  <div class="footer">CCR UI v1.2.3</div>
</template>
EOF

  # 创建 VSCode package.json
  cat > "$ROOT_DIR/ccr-vscode/package.json" <<EOF
{
  "name": "ccr-vscode",
  "version": "1.2.3",
  "publisher": "ccr"
}
EOF

  # 复制脚本到测试目录
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd ../.. && pwd)"
  cp "$SCRIPT_DIR/scripts/version-sync.sh" "$TEST_DIR/"
  chmod +x "$TEST_DIR/version-sync.sh"
}

teardown() {
  # 清理测试目录
  if [[ -n "$TEST_DIR" && -d "$TEST_DIR" ]]; then
    rm -rf "$TEST_DIR"
  fi
}

# 临时修改脚本使其使用测试目录
patch_script_for_test() {
  # 替换 ROOT_DIR 获取方式为测试目录
  sed -i "s|ROOT_DIR=\"\$(cd \"\$(dirname \"\${BASH_SOURCE[0]}\")\"/.. && pwd)\"|ROOT_DIR=\"$ROOT_DIR\"|" "$TEST_DIR/version-sync.sh"
}

@test "版本一致时 --check 返回 0" {
  patch_script_for_test
  
  run "$TEST_DIR/version-sync.sh" --check
  
  [ "$status" -eq 0 ]
  [[ "$output" == *"版本一致性检查通过"* ]]
}

@test "版本不一致时 --check 返回 1" {
  patch_script_for_test
  
  # 修改前端版本为不一致
  cat > "$ROOT_DIR/ccr-ui/package.json" <<EOF
{
  "name": "ccr-ui",
  "version": "0.9.0",
  "private": true
}
EOF

  run "$TEST_DIR/version-sync.sh" --check
  
  [ "$status" -eq 1 ]
  [[ "$output" == *"版本不一致"* ]]
}

@test "同步模式更新不一致的版本" {
  patch_script_for_test
  
  # 修改前端版本为不一致
  cat > "$ROOT_DIR/ccr-ui/package.json" <<EOF
{
  "name": "ccr-ui",
  "version": "0.9.0",
  "private": true
}
EOF

  run "$TEST_DIR/version-sync.sh"
  
  [ "$status" -eq 0 ]
  [[ "$output" == *"同步完成"* ]]
  
  # 验证前端版本已更新
  local new_ver
  new_ver="$(jq -r '.version' "$ROOT_DIR/ccr-ui/package.json")"
  [ "$new_ver" = "1.2.3" ]
}

@test "文件不存在时正确报错" {
  patch_script_for_test
  
  # 删除必需文件
  rm "$ROOT_DIR/ccr-vscode/package.json"

  run "$TEST_DIR/version-sync.sh"
  
  [ "$status" -eq 1 ]
  [[ "$output" == *"文件不存在"* ]]
}

@test "verbose 模式输出详细信息" {
  patch_script_for_test
  
  run "$TEST_DIR/version-sync.sh" --verbose
  
  [ "$status" -eq 0 ]
  [[ "$output" == *"根版本"* ]]
  [[ "$output" == *"前端版本"* ]]
}

@test "workspace 版本继承不被覆盖" {
  patch_script_for_test
  
  # 设置 ccr-types 使用 workspace 版本
  cat > "$ROOT_DIR/crates/ccr-types/Cargo.toml" <<EOF
[package]
name = "ccr-types"
version.workspace = true
edition = "2021"
EOF

  # 设置根 Cargo.toml 的 [workspace.package]
  cat > "$ROOT_DIR/Cargo.toml" <<EOF
[workspace]
members = ["crates/*"]

[workspace.package]
version = "1.2.3"
EOF

  run "$TEST_DIR/version-sync.sh" --verbose
  
  [ "$status" -eq 0 ]
  # 检查 ccr-types 是否保持 workspace 继承
  [[ "$output" == *"workspace 版本继承"* ]] || [[ "$output" == *"ccr-types"*"1.2.3"* ]]
}

@test "版本已是最新时跳过同步" {
  patch_script_for_test
  
  run "$TEST_DIR/version-sync.sh"
  
  [ "$status" -eq 0 ]
  [[ "$output" == *"版本一致，无需同步"* ]] || [[ "$output" == *"同步完成"* ]]
}
