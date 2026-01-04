# 🦀 CCR 项目 Justfile - 快速执行常用命令

# 📖 使用指南
# ────────────────────────────────────────────────────────
# 查看命令：just --list 或 just help
# 快速开发：just dev (检查+测试) 或 just watch (自动重编译)
# 代码检查：just lint (格式+Clippy) 或 just ci (完整CI)
# 构建程序：just build (Debug) 或 just release (优化版)
# 运行程序：just run -- <参数> 或 just run-release -- <参数>
# 本地安装：just install (安装到 ~/.cargo/bin)
# 前置要求：Rust工具链 (cargo, rustc)
# 提示事项：修改二进制名需同步更新 BIN 变量

# 二进制名称(与 Cargo.toml [[bin]] 保持一致)
BIN := "ccr"

# 🧭 跨平台 Shell 配置
# Windows 使用 PowerShell with UTF-8 encoding
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command", "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; [Console]::InputEncoding = [System.Text.Encoding]::UTF8; $OutputEncoding = [System.Text.Encoding]::UTF8; chcp 65001 | Out-Null;"]

# Unix-like 系统使用 bash
set shell := ["bash", "-cu"]

# ═══════════════════════════════════════════════════════════
# 🎨 跨平台消息打印函数
# ═══════════════════════════════════════════════════════════

# 分隔线
LINE := "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 打印标题 (跨平台)
[private]
[no-cd]
header title:
    @just _header-{{os()}} "{{title}}"

[private]
_header-windows title:
    @Write-Host "{{LINE}}"
    @Write-Host "{{title}}" -ForegroundColor Cyan
    @Write-Host "{{LINE}}"

[private]
_header-linux title:
    @printf '\033[36m%s\033[0m\n' "{{LINE}}"
    @printf '\033[36m\033[1m%s\033[0m\n' "{{title}}"
    @printf '\033[36m%s\033[0m\n' "{{LINE}}"

[private]
_header-macos title:
    @printf '\033[36m%s\033[0m\n' "{{LINE}}"
    @printf '\033[36m\033[1m%s\033[0m\n' "{{title}}"
    @printf '\033[36m%s\033[0m\n' "{{LINE}}"

# 打印成功消息 (跨平台)
[private]
[no-cd]
success message:
    @just _success-{{os()}} "{{message}}"

[private]
_success-windows message:
    @Write-Host "✅ {{message}}" -ForegroundColor Green

[private]
_success-linux message:
    @printf '\033[32m✅ %s\033[0m\n' "{{message}}"

[private]
_success-macos message:
    @printf '\033[32m✅ %s\033[0m\n' "{{message}}"

# 打印信息 (跨平台)
[private]
[no-cd]
info message:
    @just _info-{{os()}} "{{message}}"

[private]
_info-windows message:
    @Write-Host "{{message}}" -ForegroundColor Cyan

[private]
_info-linux message:
    @printf '\033[36m%s\033[0m\n' "{{message}}"

[private]
_info-macos message:
    @printf '\033[36m%s\033[0m\n' "{{message}}"

# 打印警告 (跨平台)
[private]
[no-cd]
warn message:
    @just _warn-{{os()}} "{{message}}"

[private]
_warn-windows message:
    @Write-Host "⚠️  {{message}}" -ForegroundColor Yellow

[private]
_warn-linux message:
    @printf '\033[33m⚠️  %s\033[0m\n' "{{message}}"

[private]
_warn-macos message:
    @printf '\033[33m⚠️  %s\033[0m\n' "{{message}}"

# 🎯 默认任务：显示帮助菜单
default: help

# 📋 显示所有可用任务
help:
    @just _help-{{os()}}
    @just --list

[private]
_help-windows:
    @Write-Host ""
    @Write-Host "   🦀 CCR Justfile - 可用命令列表"
    @Write-Host "   ────────────────────────────────────────────────────────"
    @Write-Host "   💡 提示: just <命令> 执行，just --list 查看完整列表"
    @Write-Host ""
    @Write-Host "   🔧 版本相关命令（跨平台）："
    @Write-Host "     • just version-sync   同步版本号（以根 Cargo.toml 为主）"
    @Write-Host "                            → 更新 ccr-ui/backend/Cargo.toml"
    @Write-Host "                              和 ccr-ui/frontend/package.json"
    @Write-Host "                              和 ccr-ui/frontend/src-tauri/*"
    @Write-Host "                            → Windows: 使用 version-sync.ps1"
    @Write-Host "                            → Linux/macOS: 使用 version-sync.sh"
    @Write-Host "     • just version-check  仅检查版本一致性（不修改文件）"
    @Write-Host ""
    @Write-Host "   🌐 前端检查命令："
    @Write-Host "     • just frontend-typecheck  前端 TypeScript 类型检查"
    @Write-Host "     • just frontend-lint       前端 Lint 检查"
    @Write-Host "     • just frontend-build      前端构建"
    @Write-Host "     • just docs-check          文档构建检查 (VitePress)"
    @Write-Host "     • just frontend-check      前端完整检查（类型+Lint+构建+文档）"
    @Write-Host "     • just frontend-check-quick 前端快速检查（类型+Lint）"
    @Write-Host ""
    @Write-Host "   🔒 安全审计命令："
    @Write-Host "     • just audit               运行 cargo audit 安全审计"
    @Write-Host ""
    @Write-Host "   🎯 完整 CI 流程："
    @Write-Host "     • just ci                  完整 CI 流程（对齐 GitHub Actions）"
    @Write-Host "                                版本同步 → 格式检查 → Clippy"
    @Write-Host "                                → 测试 → 构建 → 安全审计"
    @Write-Host "                                → 前端完整检查"
    @Write-Host ""
    @Write-Host ""

[private]
_help-linux:
    @printf '%s\n' ""
    @printf '%s\n' "   🦀 CCR Justfile - 可用命令列表"
    @printf '%s\n' "   ────────────────────────────────────────────────────────"
    @printf '%s\n' "   💡 提示: just <命令> 执行，just --list 查看完整列表"
    @printf '%s\n' ""
    @printf '%s\n' "   🔧 版本相关命令（跨平台）："
    @printf '%s\n' "     • just version-sync   同步版本号（以根 Cargo.toml 为主）"
    @printf '%s\n' "                            → 更新 ccr-ui/backend/Cargo.toml"
    @printf '%s\n' "                              和 ccr-ui/frontend/package.json"
    @printf '%s\n' "                              和 ccr-ui/frontend/src-tauri/*"
    @printf '%s\n' "                            → Windows: 使用 version-sync.ps1"
    @printf '%s\n' "                            → Linux/macOS: 使用 version-sync.sh"
    @printf '%s\n' "     • just version-check  仅检查版本一致性（不修改文件）"
    @printf '%s\n' ""
    @printf '%s\n' "   🌐 前端检查命令："
    @printf '%s\n' "     • just frontend-typecheck  前端 TypeScript 类型检查"
    @printf '%s\n' "     • just frontend-lint       前端 Lint 检查"
    @printf '%s\n' "     • just frontend-build      前端构建"
    @printf '%s\n' "     • just docs-check          文档构建检查 (VitePress)"
    @printf '%s\n' "     • just frontend-check      前端完整检查（类型+Lint+构建+文档）"
    @printf '%s\n' "     • just frontend-check-quick 前端快速检查（类型+Lint）"
    @printf '%s\n' ""
    @printf '%s\n' "   🔒 安全审计命令："
    @printf '%s\n' "     • just audit               运行 cargo audit 安全审计"
    @printf '%s\n' ""
    @printf '%s\n' "   🎯 完整 CI 流程："
    @printf '%s\n' "     • just ci                  完整 CI 流程（对齐 GitHub Actions）"
    @printf '%s\n' "                                版本同步 → 格式检查 → Clippy"
    @printf '%s\n' "                                → 测试 → 构建 → 安全审计"
    @printf '%s\n' "                                → 前端完整检查"
    @printf '%s\n' ""
    @printf '\n'

[private]
_help-macos:
    @printf '%s\n' ""
    @printf '%s\n' "   🦀 CCR Justfile - 可用命令列表"
    @printf '%s\n' "   ────────────────────────────────────────────────────────"
    @printf '%s\n' "   💡 提示: just <命令> 执行，just --list 查看完整列表"
    @printf '%s\n' ""
    @printf '%s\n' "   🔧 版本相关命令（跨平台）："
    @printf '%s\n' "     • just version-sync   同步版本号（以根 Cargo.toml 为主）"
    @printf '%s\n' "                            → 更新 ccr-ui/backend/Cargo.toml"
    @printf '%s\n' "                              和 ccr-ui/frontend/package.json"
    @printf '%s\n' "                              和 ccr-ui/frontend/src-tauri/*"
    @printf '%s\n' "                            → Windows: 使用 version-sync.ps1"
    @printf '%s\n' "                            → Linux/macOS: 使用 version-sync.sh"
    @printf '%s\n' "     • just version-check  仅检查版本一致性（不修改文件）"
    @printf '%s\n' ""
    @printf '%s\n' "   🌐 前端检查命令："
    @printf '%s\n' "     • just frontend-typecheck  前端 TypeScript 类型检查"
    @printf '%s\n' "     • just frontend-lint       前端 Lint 检查"
    @printf '%s\n' "     • just frontend-build      前端构建"
    @printf '%s\n' "     • just docs-check          文档构建检查 (VitePress)"
    @printf '%s\n' "     • just frontend-check      前端完整检查（类型+Lint+构建+文档）"
    @printf '%s\n' "     • just frontend-check-quick 前端快速检查（类型+Lint）"
    @printf '%s\n' ""
    @printf '%s\n' "   🔒 安全审计命令："
    @printf '%s\n' "     • just audit               运行 cargo audit 安全审计"
    @printf '%s\n' ""
    @printf '%s\n' "   🎯 完整 CI 流程："
    @printf '%s\n' "     • just ci                  完整 CI 流程（对齐 GitHub Actions）"
    @printf '%s\n' "                                版本同步 → 格式检查 → Clippy"
    @printf '%s\n' "                                → 测试 → 构建 → 安全审计"
    @printf '%s\n' "                                → 前端完整检查"
    @printf '%s\n' ""
    @printf '\n'

# ═══════════════════════════════════════════════════════════
# 🏗️  构建命令
# ═══════════════════════════════════════════════════════════

# 🔨 调试构建 (Debug 模式)
build:
    @just header "🔨 开始调试构建"
    @just info "📌 模式: Debug (包含调试符号)"
    cargo build
    @just success "构建完成 → target/debug/{{BIN}}"

# ⚡ 发布构建 (Release 优化)
release:
    @just header "⚡ 开始发布构建"
    @just info "📌 模式: Release (LTO优化 + 符号剥离)"
    cargo build --release
    @just success "构建完成 → target/release/{{BIN}}"

# 🔍 快速类型检查 (不生成可执行文件)
check:
    @just info "🔍 运行类型检查..."
    @just info "💡 快速验证模式 (不生成二进制文件)"
    cargo check
    @just success "类型检查通过"

# ═══════════════════════════════════════════════════════════
# ▶️  运行命令
# ═══════════════════════════════════════════════════════════

# ▶️ 运行程序 (Debug版本) - 示例: just run -- --help
run *args:
    @just info "▶️ 运行 Debug 版本"
    @just info "📝 参数: {{args}}"
    cargo run -- {{args}}

# 🚀 运行程序 (Release版本)
run-release *args:
    @just info "🚀 运行 Release 版本"
    @just info "📝 参数: {{args}}"
    cargo run --release -- {{args}}

# 🏷️ 查看版本信息
version:
    @just info "🏷️ 获取版本信息"
    @cargo run -- --version

# ═══════════════════════════════════════════════════════════
# ✅ 测试命令
# ═══════════════════════════════════════════════════════════

# ✅ 运行测试 (标准模式)
test:
    @just header "✅ 运行测试套件"
    @just info "📊 模式: 完整工作区测试"
    @just warn "注意: 使用串行模式 (--test-threads=1) 避免并发冲突"
    cargo test --workspace --all-features -- --test-threads=1
    @just success "所有测试通过"

# 🧪 运行所有测试 (包括忽略的测试)
test-all:
    @just info "🧪 运行完整测试套件"
    @just info "📊 模式: 包含被忽略的测试"
    @just warn "注意: 使用串行模式 (--test-threads=1)"
    cargo test --workspace --all-features -- --test-threads=1 --include-ignored
    @just success "完整测试通过"

# 📊 运行基准测试
bench:
    @just info "📊 运行基准测试"
    cargo bench
    @just success "基准测试完成"

# ═══════════════════════════════════════════════════════════
# ✨ 代码质量命令
# ═══════════════════════════════════════════════════════════

# ✨ 代码格式化
fmt:
    @just info "✨ 格式化代码"
    cargo fmt
    @just success "代码格式化完成"

# 🔍 检查代码格式 (不修改文件)
fmt-check:
    @just info "🔍 检查代码格式"
    @just info "📌 模式: 仅验证，不修改文件"
    cargo fmt -- --check
    @just success "代码格式符合规范"

# 🚨 静态检查 (Clippy) - 警告视为错误
clippy:
    @just info "🚨 运行 Clippy 静态检查"
    @just warn "模式: 所有警告视为错误"
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    @just success "Clippy 检查通过"

# 🔥 严格静态检查 (Clippy + 禁止 unwrap)
lint-strict:
    @just info "🔥 运行严格 Clippy 检查"
    @just warn "模式: 所有警告视为错误 + 禁止 unwrap"
    @just info "📌 注意: 测试代码中的 unwrap 会产生警告"
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used
    @just success "严格 Clippy 检查通过"

# 🔧 完整代码检查 (格式化 + Clippy)
lint: fmt clippy
    @just header "代码质量检查"
    @just success "代码质量检查全部通过"

# 🔒 安全审计 (cargo audit) - 若未安装则跳过
audit:
    @just header "🔒 运行安全审计"
    @just info "📌 使用 cargo-audit (需要安装: cargo install cargo-audit)"
    -cargo audit || just warn "cargo-audit 未安装，跳过安全审计 (安装: cargo install cargo-audit)"
    @just success "安全审计步骤完成"

# ═══════════════════════════════════════════════════════════
# 🚀 开发工作流命令
# ═══════════════════════════════════════════════════════════

# ⚡ 快速开发循环 (检查 → 测试)
dev: check test
    @just header "开发验证"
    @just success "开发验证完成"

# 👀 监控文件变化并自动重新编译
watch:
    @just info "👀 启动文件监控模式"
    @just info "📌 使用 cargo-watch (需要安装: cargo install cargo-watch)"
    cargo watch -x check -x test

# 🎯 完整 CI 流程 (版本同步 + 自动格式化 + 格式检查 + 严格 Clippy + 测试 + 构建 + 安全审计 + 前端完整检查)
ci: version-sync fmt fmt-check lint-strict test release audit frontend-check
    @just _ci-done-{{os()}}

[private]
_ci-done-windows:
    @Write-Host ""
    @Write-Host "          🎉 CI 流程全部通过 - 代码质量优秀！"

[private]
_ci-done-linux:
    @printf '\n'
    @printf '%s\n' "          🎉 CI 流程全部通过 - 代码质量优秀！"

[private]
_ci-done-macos:
    @printf '\n'
    @printf '%s\n' "          🎉 CI 流程全部通过 - 代码质量优秀！"

# ═══════════════════════════════════════════════════════════
# 🌐 前端检查命令
# ═══════════════════════════════════════════════════════════

# 🔍 前端 TypeScript 类型检查
frontend-typecheck:
    @just header "🔍 前端 TypeScript 类型检查"
    cd ccr-ui/frontend && npm install --silent && npm run type-check
    @just success "前端类型检查通过"

# 🎨 前端 Lint 检查
frontend-lint:
    @just header "🎨 前端 Lint 检查"
    cd ccr-ui/frontend && npm install --silent && npm run lint
    @just success "前端 Lint 检查通过"

# 🏗️ 前端构建
frontend-build:
    @just header "🏗️ 前端构建"
    cd ccr-ui/frontend && npm install --silent && npm run build
    @just success "前端构建完成"

# 📚 文档构建检查 (VitePress) - 可选，有 dead links 时可能失败
docs-check:
    @just header "📚 文档构建检查"
    @just warn "注意: 若有 dead links 会失败，可在 .vitepress/config 中配置 ignoreDeadLinks"
    cd docs && npm install --silent && npm run build
    @just info "⏭️  跳过 ccr-ui/docs 构建 (VitePress+Mermaid 插件问题)"
    # cd ccr-ui/docs && npm install --silent && npm run build
    @just success "文档构建检查完成"

# 🌐 前端完整检查 (类型检查 + Lint + 构建 + 文档构建)
frontend-check: frontend-typecheck frontend-lint frontend-build docs-check
    @just success "前端检查全部通过"

# 🌐 前端快速检查 (类型检查 + Lint，不含构建和文档)
frontend-check-quick: frontend-typecheck frontend-lint
    @just success "前端快速检查通过"

# ═══════════════════════════════════════════════════════════
# 📦 安装与管理命令
# ═══════════════════════════════════════════════════════════

# 📦 安装到本地 (~/.cargo/bin)
install:
    @just header "📦 安装到本地"
    @just info "📍 目标路径: ~/.cargo/bin/{{BIN}}"
    @just info "🔒 模式: 锁定依赖版本 (--locked)"
    cargo install --path . --locked
    @just success "安装完成"

# ♻️ 强制重新安装
reinstall:
    @just info "♻️ 强制重新安装"
    @just warn "模式: 覆盖现有安装"
    cargo install --path . --locked --force
    @just success "重新安装完成"

# 🗑️ 卸载已安装的二进制
uninstall:
    @just info "🗑️ 卸载 {{BIN}}"
    cargo uninstall {{BIN}}
    @just success "卸载完成"

# ═══════════════════════════════════════════════════════════
# 📚 文档命令
# ═══════════════════════════════════════════════════════════

# 📚 构建文档 (不包含依赖)
doc:
    @just info "📚 生成文档"
    @just info "📌 模式: 仅本项目代码 (--no-deps)"
    cargo doc --no-deps
    @just success "文档生成完成"

# 🌐 构建并在浏览器中打开文档
doc-open:
    @just info "🌐 生成并打开文档"
    @just info "📖 将在默认浏览器中打开"
    cargo doc --no-deps --open

# ═══════════════════════════════════════════════════════════
# 🧹 清理与维护命令
# ═══════════════════════════════════════════════════════════

# 🧹 清理构建产物
clean:
    @just info "🧹 清理构建产物"
    @just info "📂 清理目标: target/ 目录"
    cargo clean
    @just success "清理完成"

# 📦 检查依赖更新
update-deps:
    @just info "📦 检查依赖更新"
    @just info "📌 使用 cargo-outdated (需要安装: cargo install cargo-outdated)"
    cargo outdated

# 💣 深度清理 (包括 Cargo 缓存和目标文件)
deep-clean: clean
    @just header "💣 深度清理"
    @just warn "警告：将清理 Cargo 缓存"
    @just info "🗑️  清理 Cargo 注册表缓存"
    cargo clean
    @just success "深度清理完成"

# ═══════════════════════════════════════════════════════════
# 🔧 版本号同步命令
# ═══════════════════════════════════════════════════════════

# 同步版本号到 UI 后端与前端（以根 Cargo.toml 为主）
version-sync:
    @just _version-sync-{{os()}}

# 仅检查版本一致性
version-check:
    @just _version-check-{{os()}}

[private]
_version-sync-windows:
    @just info "🔧 同步版本号（以根 Cargo.toml 为主）"
    @.\scripts\version-sync.ps1 -Verbose
    @just success "版本同步完成"

[private]
_version-sync-linux:
    @just info "🔧 同步版本号（以根 Cargo.toml 为主）"
    bash scripts/version-sync.sh
    @just success "版本同步完成"

[private]
_version-sync-macos:
    @just info "🔧 同步版本号（以根 Cargo.toml 为主）"
    bash scripts/version-sync.sh
    @just success "版本同步完成"

[private]
_version-check-windows:
    @just info "🔍 检查版本号一致性"
    @.\scripts\version-sync.ps1 -Check -Verbose

[private]
_version-check-linux:
    @just info "🔍 检查版本号一致性"
    bash scripts/version-sync.sh --check --verbose

[private]
_version-check-macos:
    @just info "🔍 检查版本号一致性"
    bash scripts/version-sync.sh --check --verbose
