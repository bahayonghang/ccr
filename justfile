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
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command", "$OutputEncoding = [Console]::OutputEncoding = [System.Text.Encoding]::UTF8;"]

# Unix-like 系统使用 bash
set shell := ["bash", "-cu"]

# 🎯 默认任务：显示帮助菜单
default: help

# 📋 显示所有可用任务
help:
  @echo "╔════════════════════════════════════════════════════════════════╗"
  @echo "║   🦀 CCR Justfile - 可用命令列表                              ║"
  @echo "╠════════════════════════════════════════════════════════════════╣"
  @echo "║   💡 提示: just <命令> 执行，just --list 查看完整列表           ║"
  @echo "║                                                                ║"
  @echo "║   🔧 版本相关命令（跨平台）：                                  ║"
  @echo "║     • just version-sync   同步版本号（以根 Cargo.toml 为主）    ║"
  @echo "║                            → 更新 ccr-ui/backend/Cargo.toml    ║"
  @echo "║                              和 ccr-ui/frontend/package.json    ║"
  @echo "║                              和 ccr-ui/frontend/src-tauri/*     ║"
  @echo "║                            → Windows: 使用 version-sync.ps1    ║"
  @echo "║                            → Linux/macOS: 使用 version-sync.sh ║"
  @echo "║     • just version-check  仅检查版本一致性（不修改文件）        ║"
  @echo "║                                                                ║"
  @echo "║   🌐 前端检查命令：                                            ║"
  @echo "║     • just frontend-typecheck  前端 TypeScript 类型检查        ║"
  @echo "║     • just frontend-lint       前端 Lint 检查                  ║"
  @echo "║     • just frontend-build      前端构建                        ║"
  @echo "║     • just docs-check          文档构建检查 (VitePress)        ║"
  @echo "║     • just frontend-check      前端完整检查（类型+Lint+构建+文档）║"
  @echo "║     • just frontend-check-quick 前端快速检查（类型+Lint）      ║"
  @echo "║                                                                ║"
  @echo "║   🔒 安全审计命令：                                            ║"
  @echo "║     • just audit               运行 cargo audit 安全审计       ║"
  @echo "║                                                                ║"
  @echo "║   🎯 完整 CI 流程：                                            ║"
  @echo "║     • just ci                  完整 CI 流程（对齐 GitHub Actions）║"
  @echo "║                                版本同步 → 格式检查 → Clippy    ║"
  @echo "║                                → 测试 → 构建 → 安全审计        ║"
  @echo "║                                → 前端完整检查                   ║"
  @echo "╚════════════════════════════════════════════════════════════════╝"
  @echo ""
  @just --list

# ═══════════════════════════════════════════════════════════
# 🏗️  构建命令
# ═══════════════════════════════════════════════════════════

# 🔨 调试构建 (Debug 模式)
build:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "🔨 开始调试构建"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📌 模式: Debug (包含调试符号)"
  @echo ""
  cargo build
  @echo ""
  @echo "✅ 构建完成 → target/debug/{{BIN}}"

# ⚡ 发布构建 (Release 优化)
release:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "⚡ 开始发布构建"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📌 模式: Release (LTO优化 + 符号剥离)"
  @echo ""
  cargo build --release
  @echo ""
  @echo "✅ 构建完成 → target/release/{{BIN}}"

# 🔍 快速类型检查 (不生成可执行文件)
check:
  @echo "🔍 运行类型检查..."
  @echo "💡 快速验证模式 (不生成二进制文件)"
  @echo ""
  cargo check
  @echo ""
  @echo "✅ 类型检查通过"

# ═══════════════════════════════════════════════════════════
# ▶️  运行命令
# ═══════════════════════════════════════════════════════════

# ▶️ 运行程序 (Debug版本) - 示例: just run -- --help
run *args:
  @echo "▶️ 运行 Debug 版本"
  @echo "📝 参数: {{args}}"
  @echo ""
  cargo run -- {{args}}

# 🚀 运行程序 (Release版本)
run-release *args:
  @echo "🚀 运行 Release 版本"
  @echo "📝 参数: {{args}}"
  @echo ""
  cargo run --release -- {{args}}

# 🏷️ 查看版本信息
version:
  @echo "🏷️ 获取版本信息"
  @echo ""
  @cargo run -- --version

# ═══════════════════════════════════════════════════════════
# ✅ 测试命令
# ═══════════════════════════════════════════════════════════

# ✅ 运行测试 (标准模式)
test:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "✅ 运行测试套件"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📊 模式: 完整工作区测试"
  @echo "⚠️  注意: 使用串行模式 (--test-threads=1) 避免并发冲突"
  @echo ""
  cargo test --workspace --all-features -- --test-threads=1
  @echo ""
  @echo "✅ 所有测试通过"

# 🧪 运行所有测试 (包括忽略的测试)
test-all:
  @echo "🧪 运行完整测试套件"
  @echo "📊 模式: 包含被忽略的测试"
  @echo "⚠️  注意: 使用串行模式 (--test-threads=1)"
  @echo ""
  cargo test --workspace --all-features -- --test-threads=1 --include-ignored
  @echo ""
  @echo "✅ 完整测试通过"

# 📊 运行基准测试
bench:
  @echo "📊 运行基准测试"
  @echo ""
  cargo bench
  @echo ""
  @echo "✅ 基准测试完成"

# ═══════════════════════════════════════════════════════════
# ✨ 代码质量命令
# ═══════════════════════════════════════════════════════════

# ✨ 代码格式化
fmt:
  @echo "✨ 格式化代码"
  @echo ""
  cargo fmt
  @echo ""
  @echo "✅ 代码格式化完成"

# 🔍 检查代码格式 (不修改文件)
fmt-check:
  @echo "🔍 检查代码格式"
  @echo "📌 模式: 仅验证，不修改文件"
  @echo ""
  cargo fmt -- --check
  @echo ""
  @echo "✅ 代码格式符合规范"

# 🚨 静态检查 (Clippy) - 警告视为错误
clippy:
  @echo "🚨 运行 Clippy 静态检查"
  @echo "⚠️  模式: 所有警告视为错误"
  @echo ""
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  @echo ""
  @echo "✅ Clippy 检查通过"

# 🔧 完整代码检查 (格式化 + Clippy)
lint: fmt clippy
  @echo ""
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "✅ 代码质量检查全部通过"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 🔒 安全审计 (cargo audit)
audit:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "🔒 运行安全审计"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📌 使用 cargo-audit (需要安装: cargo install cargo-audit)"
  @echo ""
  cargo audit
  @echo ""
  @echo "✅ 安全审计通过"

# ═══════════════════════════════════════════════════════════
# 🚀 开发工作流命令
# ═══════════════════════════════════════════════════════════

# ⚡ 快速开发循环 (检查 → 测试)
dev: check test
  @echo ""
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "✅ 开发验证完成"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 👀 监控文件变化并自动重新编译
watch:
  @echo "👀 启动文件监控模式"
  @echo "📌 使用 cargo-watch (需要安装: cargo install cargo-watch)"
  @echo ""
  cargo watch -x check -x test

# 🎯 完整 CI 流程 (版本同步 + 格式检查 + Clippy + 测试 + 构建 + 安全审计 + 前端完整检查)
ci: version-sync fmt-check clippy test release audit frontend-check
  @echo ""
  @echo "╔════════════════════════════════════════════════════════════════╗"
  @echo "║          🎉 CI 流程全部通过 - 代码质量优秀！               ║"
  @echo "╚════════════════════════════════════════════════════════════════╝"

# ═══════════════════════════════════════════════════════════
# 🌐 前端检查命令
# ═══════════════════════════════════════════════════════════

# 🔍 前端 TypeScript 类型检查
frontend-typecheck:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "🔍 前端 TypeScript 类型检查"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  cd ccr-ui/frontend && npm install --silent && npm run type-check
  @echo "✅ 前端类型检查通过"

# 🎨 前端 Lint 检查
frontend-lint:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "🎨 前端 Lint 检查"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  cd ccr-ui/frontend && npm install --silent && npm run lint
  @echo "✅ 前端 Lint 检查通过"

# 🏗️ 前端构建
frontend-build:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "🏗️ 前端构建"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  cd ccr-ui/frontend && npm install --silent && npm run build
  @echo "✅ 前端构建完成"

# 📚 文档构建检查 (VitePress) - 可选，有 dead links 时可能失败
docs-check:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📚 文档构建检查"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "⚠️  注意: 若有 dead links 会失败，可在 .vitepress/config 中配置 ignoreDeadLinks"
  cd docs && npm install --silent && npm run build
  @echo "⏭️  跳过 ccr-ui/docs 构建 (VitePress+Mermaid 插件问题)"
  # cd ccr-ui/docs && npm install --silent && npm run build
  @echo "✅ 文档构建检查完成"

# 🌐 前端完整检查 (类型检查 + Lint + 构建 + 文档构建)
frontend-check: frontend-typecheck frontend-lint frontend-build docs-check
  @echo ""
  @echo "✅ 前端检查全部通过"

# 🌐 前端快速检查 (类型检查 + Lint，不含构建和文档)
frontend-check-quick: frontend-typecheck frontend-lint
  @echo ""
  @echo "✅ 前端快速检查通过"

# ═══════════════════════════════════════════════════════════
# 📦 安装与管理命令
# ═══════════════════════════════════════════════════════════

# 📦 安装到本地 (~/.cargo/bin)
install:
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📦 安装到本地"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "📍 目标路径: ~/.cargo/bin/{{BIN}}"
  @echo "🔒 模式: 锁定依赖版本 (--locked)"
  @echo ""
  cargo install --path . --locked
  @echo ""
  @echo "✅ 安装完成"

# ♻️ 强制重新安装
reinstall:
  @echo "♻️ 强制重新安装"
  @echo "⚠️  模式: 覆盖现有安装"
  @echo ""
  cargo install --path . --locked --force
  @echo ""
  @echo "✅ 重新安装完成"

# 🗑️ 卸载已安装的二进制
uninstall:
  @echo "🗑️ 卸载 {{BIN}}"
  @echo ""
  cargo uninstall {{BIN}}
  @echo ""
  @echo "✅ 卸载完成"

# ═══════════════════════════════════════════════════════════
# 📚 文档命令
# ═══════════════════════════════════════════════════════════

# 📚 构建文档 (不包含依赖)
doc:
  @echo "📚 生成文档"
  @echo "📌 模式: 仅本项目代码 (--no-deps)"
  @echo ""
  cargo doc --no-deps
  @echo ""
  @echo "✅ 文档生成完成"

# 🌐 构建并在浏览器中打开文档
doc-open:
  @echo "🌐 生成并打开文档"
  @echo "📖 将在默认浏览器中打开"
  @echo ""
  cargo doc --no-deps --open

# ═══════════════════════════════════════════════════════════
# 🧹 清理与维护命令
# ═══════════════════════════════════════════════════════════

# 🧹 清理构建产物
clean:
  @echo "🧹 清理构建产物"
  @echo "📂 清理目标: target/ 目录"
  @echo ""
  cargo clean
  @echo ""
  @echo "✅ 清理完成"

# 📦 检查依赖更新
update-deps:
  @echo "📦 检查依赖更新"
  @echo "📌 使用 cargo-outdated (需要安装: cargo install cargo-outdated)"
  @echo ""
  cargo outdated

# 💣 深度清理 (包括 Cargo 缓存和目标文件)
deep-clean: clean
  @echo ""
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "💣 深度清理 - 警告：将清理 Cargo 缓存"
  @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  @echo "🗑️  清理 Cargo 注册表缓存"
  @echo ""
  cargo clean
  @echo ""
  @echo "✅ 深度清理完成"

# ═══════════════════════════════════════════════════════════
# 🔧 版本号同步命令
# ═══════════════════════════════════════════════════════════

# 同步版本号到 UI 后端与前端（以根 Cargo.toml 为主）
version-sync:
  @just _version-sync-{{os()}}

# 仅检查版本一致性
version-check:
  @just _version-check-{{os()}}

# 同步版本 - Windows
_version-sync-windows:
  @Write-Output '🔧 同步版本号（以根 Cargo.toml 为主）'
  @.\scripts\version-sync.ps1 -Verbose
  @Write-Output '✅ 版本同步完成'

# 同步版本 - Linux
_version-sync-linux:
  @echo "🔧 同步版本号（以根 Cargo.toml 为主）"
  bash scripts/version-sync.sh
  @echo "✅ 版本同步完成"

# 同步版本 - macOS
_version-sync-macos:
  @echo "🔧 同步版本号（以根 Cargo.toml 为主）"
  bash scripts/version-sync.sh
  @echo "✅ 版本同步完成"

# 检查版本 - Windows
_version-check-windows:
  @Write-Output '🔍 检查版本号一致性'
  @.\scripts\version-sync.ps1 -Check -Verbose

# 检查版本 - Linux
_version-check-linux:
  @echo "🔍 检查版本号一致性"
  bash scripts/version-sync.sh --check --verbose

# 检查版本 - macOS
_version-check-macos:
  @echo "🔍 检查版本号一致性"
  bash scripts/version-sync.sh --check --verbose