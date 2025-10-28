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

# 使用 bash 登录 shell 以确保加载完整环境变量（包括 cargo 路径）🧭
# -l: 登录 shell (加载 ~/.profile)
# -c: 执行命令
# -u: 使用未定义变量时报错
set shell := ["bash", "-lcu"]

# ANSI 颜色代码
BOLD := "\\033[1m"
CYAN := "\\033[36m"
GREEN := "\\033[32m"
YELLOW := "\\033[33m"
BLUE := "\\033[34m"
MAGENTA := "\\033[35m"
RED := "\\033[31m"
RESET := "\\033[0m"

# 🎯 默认任务：显示帮助菜单
default: help

# 📋 显示所有可用任务
help:
  @echo -e "{{CYAN}}{{BOLD}}╔════════════════════════════════════════════════════════════════╗{{RESET}}"
  @echo -e "{{CYAN}}{{BOLD}}║   🦀 CCR Justfile - 可用命令列表                              ║{{RESET}}"
  @echo -e "{{CYAN}}{{BOLD}}╠════════════════════════════════════════════════════════════════╣{{RESET}}"
  @echo -e "{{CYAN}}{{BOLD}}║   💡 提示: just <命令> 执行，just --list 查看完整列表           ║{{RESET}}"
  @echo -e "{{CYAN}}{{BOLD}}╚════════════════════════════════════════════════════════════════╝{{RESET}}"
  @echo ""
  @just --list

# ═══════════════════════════════════════════════════════════
# 🏗️  构建命令
# ═══════════════════════════════════════════════════════════

# 🔨 调试构建 (Debug 模式)
build:
  @echo -e "{{BLUE}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{BLUE}}{{BOLD}}🔨 开始调试构建{{RESET}}"
  @echo -e "{{BLUE}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{BLUE}}📌 模式: {{BOLD}}Debug{{RESET}}{{BLUE}} (包含调试符号){{RESET}}"
  @echo ""
  cargo build
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 构建完成{{RESET}} → {{CYAN}}target/debug/{{BIN}}{{RESET}}"

# ⚡ 发布构建 (Release 优化)
release:
  @echo -e "{{MAGENTA}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{MAGENTA}}{{BOLD}}⚡ 开始发布构建{{RESET}}"
  @echo -e "{{MAGENTA}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{MAGENTA}}📌 模式: {{BOLD}}Release{{RESET}}{{MAGENTA}} (LTO优化 + 符号剥离){{RESET}}"
  @echo ""
  cargo build --release
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 构建完成{{RESET}} → {{CYAN}}target/release/{{BIN}}{{RESET}}"
  @printf "{{YELLOW}}📦 二进制大小: {{BOLD}}"; du -h target/release/{{BIN}} | cut -f1; printf "{{RESET}}\n"

# 🔍 快速类型检查 (不生成可执行文件)
check:
  @echo -e "{{CYAN}}{{BOLD}}🔍 运行类型检查...{{RESET}}"
  @echo -e "{{CYAN}}💡 快速验证模式 (不生成二进制文件){{RESET}}"
  @echo ""
  cargo check
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 类型检查通过{{RESET}}"

# ═══════════════════════════════════════════════════════════
# ▶️  运行命令
# ═══════════════════════════════════════════════════════════

# ▶️ 运行程序 (Debug版本) - 示例: just run -- --help
run *args:
  @echo -e "{{BLUE}}{{BOLD}}▶️ 运行 Debug 版本{{RESET}}"
  @echo -e "{{BLUE}}📝 参数: {{BOLD}}{{args}}{{RESET}}"
  @echo ""
  cargo run -- {{args}}

# 🚀 运行程序 (Release版本) - 智能构建
run-release *args:
  @echo -e "{{MAGENTA}}{{BOLD}}🚀 运行 Release 版本{{RESET}}"
  @if [ ! -f target/release/{{BIN}} ]; then \
    echo -e "{{YELLOW}}⚠️  Release 二进制不存在，开始构建...{{RESET}}"; \
    echo ""; \
    just release; \
    echo ""; \
  fi
  @echo -e "{{MAGENTA}}📝 参数: {{BOLD}}{{args}}{{RESET}}"
  @echo ""
  ./target/release/{{BIN}} {{args}}

# 🏷️ 查看版本信息
version:
  @echo -e "{{CYAN}}{{BOLD}}🏷️ 获取版本信息{{RESET}}"
  @echo ""
  @cargo run -- --version

# ═══════════════════════════════════════════════════════════
# ✅ 测试命令
# ═══════════════════════════════════════════════════════════

# ✅ 运行测试 (标准模式)
test:
  @echo -e "{{GREEN}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}✅ 运行测试套件{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{GREEN}}📊 模式: 标准测试{{RESET}}"
  @echo ""
  cargo test
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 所有测试通过{{RESET}}"

# 🧪 运行所有测试 (包括忽略的测试)
test-all:
  @echo -e "{{GREEN}}{{BOLD}}🧪 运行完整测试套件{{RESET}}"
  @echo -e "{{GREEN}}📊 模式: 包含被忽略的测试{{RESET}}"
  @echo ""
  cargo test -- --include-ignored
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 完整测试通过{{RESET}}"

# 📊 运行基准测试
bench:
  @echo -e "{{YELLOW}}{{BOLD}}📊 运行基准测试{{RESET}}"
  @echo ""
  cargo bench
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 基准测试完成{{RESET}}"

# ═══════════════════════════════════════════════════════════
# ✨ 代码质量命令
# ═══════════════════════════════════════════════════════════

# ✨ 代码格式化
fmt:
  @echo -e "{{CYAN}}{{BOLD}}✨ 格式化代码{{RESET}}"
  @echo ""
  cargo fmt
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 代码格式化完成{{RESET}}"

# 🔍 检查代码格式 (不修改文件)
fmt-check:
  @echo -e "{{CYAN}}{{BOLD}}🔍 检查代码格式{{RESET}}"
  @echo -e "{{CYAN}}📌 模式: 仅验证，不修改文件{{RESET}}"
  @echo ""
  cargo fmt -- --check
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 代码格式符合规范{{RESET}}"

# 🚨 静态检查 (Clippy) - 警告视为错误
clippy:
  @echo -e "{{YELLOW}}{{BOLD}}🚨 运行 Clippy 静态检查{{RESET}}"
  @echo -e "{{YELLOW}}⚠️  模式: 所有警告视为错误{{RESET}}"
  @echo ""
  cargo clippy -- -D warnings
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ Clippy 检查通过{{RESET}}"

# 🔧 完整代码检查 (格式化 + Clippy)
lint: fmt clippy
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}✅ 代码质量检查全部通过{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"

# ═══════════════════════════════════════════════════════════
# 🚀 开发工作流命令
# ═══════════════════════════════════════════════════════════

# ⚡ 快速开发循环 (检查 → 测试)
dev: check test
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}✅ 开发验证完成{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"

# 👀 监控文件变化并自动重新编译
watch:
  @echo -e "{{BLUE}}{{BOLD}}👀 启动文件监控模式{{RESET}}"
  @echo -e "{{BLUE}}📌 使用 cargo-watch (需要安装: cargo install cargo-watch){{RESET}}"
  @echo ""
  @if ! command -v cargo-watch &> /dev/null; then \
    echo -e "{{RED}}❌ cargo-watch 未安装{{RESET}}"; \
    echo -e "{{YELLOW}}💡 请运行: {{BOLD}}cargo install cargo-watch{{RESET}}"; \
    exit 1; \
  fi
  cargo watch -x check -x test

# 🎯 完整 CI 流程 (格式检查 + Clippy + 测试 + 构建)
ci: fmt-check clippy test release
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}╔════════════════════════════════════════════════════════════════╗{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}║          🎉 CI 流程全部通过 - 代码质量优秀！               ║{{RESET}}"
  @echo -e "{{GREEN}}{{BOLD}}╚════════════════════════════════════════════════════════════════╝{{RESET}}"

# ═══════════════════════════════════════════════════════════
# 📦 安装与管理命令
# ═══════════════════════════════════════════════════════════

# 📦 安装到本地 (~/.cargo/bin)
install:
  @echo -e "{{MAGENTA}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{MAGENTA}}{{BOLD}}📦 安装到本地{{RESET}}"
  @echo -e "{{MAGENTA}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{MAGENTA}}📍 目标路径: {{BOLD}}~/.cargo/bin/{{BIN}}{{RESET}}"
  @echo -e "{{MAGENTA}}🔒 模式: 锁定依赖版本 (--locked){{RESET}}"
  @echo ""
  cargo install --path . --locked
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 安装完成{{RESET}}"
  @echo ""
  @which {{BIN}} && {{BIN}} --version

# ♻️ 强制重新安装
reinstall:
  @echo -e "{{MAGENTA}}{{BOLD}}♻️ 强制重新安装{{RESET}}"
  @echo -e "{{MAGENTA}}⚠️  模式: 覆盖现有安装{{RESET}}"
  @echo ""
  cargo install --path . --locked --force
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 重新安装完成{{RESET}}"
  @echo ""
  @which {{BIN}} && {{BIN}} --version

# 🗑️ 卸载已安装的二进制
uninstall:
  @echo -e "{{RED}}{{BOLD}}🗑️ 卸载 {{BIN}}{{RESET}}"
  @echo ""
  cargo uninstall {{BIN}}
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 卸载完成{{RESET}}"

# ═══════════════════════════════════════════════════════════
# 📚 文档命令
# ═══════════════════════════════════════════════════════════

# 📚 构建文档 (不包含依赖)
doc:
  @echo -e "{{CYAN}}{{BOLD}}📚 生成文档{{RESET}}"
  @echo -e "{{CYAN}}📌 模式: 仅本项目代码 (--no-deps){{RESET}}"
  @echo ""
  cargo doc --no-deps
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 文档生成完成{{RESET}}"

# 🌐 构建并在浏览器中打开文档
doc-open:
  @echo -e "{{CYAN}}{{BOLD}}🌐 生成并打开文档{{RESET}}"
  @echo -e "{{CYAN}}📖 将在默认浏览器中打开{{RESET}}"
  @echo ""
  cargo doc --no-deps --open

# ═══════════════════════════════════════════════════════════
# 🧹 清理与维护命令
# ═══════════════════════════════════════════════════════════

# 🧹 清理构建产物
clean:
  @echo -e "{{RED}}{{BOLD}}🧹 清理构建产物{{RESET}}"
  @echo -e "{{RED}}📂 清理目标: target/ 目录{{RESET}}"
  @echo ""
  cargo clean
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 清理完成{{RESET}}"

# 📦 检查依赖更新
update-deps:
  @echo -e "{{YELLOW}}{{BOLD}}📦 检查依赖更新{{RESET}}"
  @echo -e "{{YELLOW}}📌 使用 cargo-outdated (需要安装: cargo install cargo-outdated){{RESET}}"
  @echo ""
  @if ! command -v cargo-outdated &> /dev/null; then \
    echo -e "{{RED}}❌ cargo-outdated 未安装{{RESET}}"; \
    echo -e "{{YELLOW}}💡 请运行: {{BOLD}}cargo install cargo-outdated{{RESET}}"; \
    exit 1; \
  fi
  cargo outdated

# 💣 深度清理 (包括 Cargo 缓存和目标文件)
deep-clean: clean
  @echo ""
  @echo -e "{{RED}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{RED}}{{BOLD}}💣 深度清理 - 警告：将清理 Cargo 缓存{{RESET}}"
  @echo -e "{{RED}}{{BOLD}}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{{RESET}}"
  @echo -e "{{RED}}🗑️  清理 Cargo 注册表缓存{{RESET}}"
  @echo ""
  rm -rf ~/.cargo/registry/index/*
  rm -rf ~/.cargo/registry/cache/*
  @echo ""
  @echo -e "{{GREEN}}{{BOLD}}✅ 深度清理完成{{RESET}}"