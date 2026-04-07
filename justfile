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
CLI_CRATE_PATH := "crates/ccr"
OUTPUTS_DIR := "outputs"

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
    @Write-Host "                            → 更新 crates/ccr-types/Cargo.toml"
    @Write-Host "                              和 crates/ccr-db/Cargo.toml"
    @Write-Host "                              和 ccr-ui/package.json"
    @Write-Host "                              和 ccr-ui/src-tauri/*"
    @Write-Host "                              和 ccr-vscode/package.json"
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
    @printf '%s\n' "                            → 更新 crates/ccr-types/Cargo.toml"
    @printf '%s\n' "                              和 crates/ccr-db/Cargo.toml"
    @printf '%s\n' "                              和 ccr-ui/package.json"
    @printf '%s\n' "                              和 ccr-ui/src-tauri/*"
    @printf '%s\n' "                              和 ccr-vscode/package.json"
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
    @printf '%s\n' "                            → 更新 crates/ccr-types/Cargo.toml"
    @printf '%s\n' "                              和 crates/ccr-db/Cargo.toml"
    @printf '%s\n' "                              和 ccr-ui/package.json"
    @printf '%s\n' "                              和 ccr-ui/src-tauri/*"
    @printf '%s\n' "                              和 ccr-vscode/package.json"
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

# 🔨 调试构建 (Debug 模式，含计时)
build:
    @just header "🔨 开始调试构建"
    @just info "📌 模式: Debug (包含调试符号)"
    @just _build-timed-{{os()}}

[private]
_build-timed-windows:
    #!pwsh.exe
    $ErrorActionPreference = 'Stop'
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    chcp 65001 | Out-Null
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $env:CARGO_INCREMENTAL = '1'
    cargo build -p {{BIN}} --timings
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $sw.Stop()
    $elapsed = $sw.Elapsed
    $ts = "{0:mm\:ss\.fff}" -f $elapsed
    Write-Host ""
    Write-Host "  ⏱️  cargo build    $ts" -ForegroundColor Cyan
    Write-Host "  📈 cargo timings  target/cargo-timings/cargo-timing.html" -ForegroundColor DarkCyan
    Write-Host ""
    Write-Host "✅ 构建完成 → target/debug/{{BIN}}" -ForegroundColor Green

[private]
_build-timed-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    start=$(date +%s%N)
    CARGO_INCREMENTAL=1 cargo build -p {{BIN}} --timings
    end=$(date +%s%N)
    ms=$(( (end - start) / 1000000 ))
    s=$((ms / 1000)); r=$((ms % 1000))
    m=$((s / 60)); s=$((s % 60))
    printf '\n  ⏱️  cargo build    %02d:%02d.%03d\n\n' "$m" "$s" "$r"
    printf '\033[36m  📈 cargo timings  target/cargo-timings/cargo-timing.html\033[0m\n'
    printf '\033[32m✅ 构建完成 → target/debug/{{BIN}}\033[0m\n'

[private]
_build-timed-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    start=$(date +%s)
    CARGO_INCREMENTAL=1 cargo build -p {{BIN}} --timings
    end=$(date +%s)
    elapsed=$((end - start))
    m=$((elapsed / 60)); s=$((elapsed % 60))
    printf '\n  ⏱️  cargo build    %02d:%02d\n\n' "$m" "$s"
    printf '\033[36m  📈 cargo timings  target/cargo-timings/cargo-timing.html\033[0m\n'
    printf '\033[32m✅ 构建完成 → target/debug/{{BIN}}\033[0m\n'

# ⚡ 发布构建 (Release 优化，含计时)
release:
    @just header "⚡ 开始发布构建"
    @just info "📌 模式: Release (LTO优化 + 符号剥离)"
    @just _release-timed-{{os()}}

[private]
_release-timed-windows:
    #!pwsh.exe
    $ErrorActionPreference = 'Stop'
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    chcp 65001 | Out-Null
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $env:CARGO_INCREMENTAL = '1'
    cargo build -p {{BIN}} --release --timings
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $sw.Stop()
    $elapsed = $sw.Elapsed
    $ts = "{0:mm\:ss\.fff}" -f $elapsed
    Write-Host ""
    Write-Host "  ⏱️  cargo build --release    $ts" -ForegroundColor Cyan
    Write-Host "  📈 cargo timings  target/cargo-timings/cargo-timing.html" -ForegroundColor DarkCyan
    Write-Host ""
    Write-Host "✅ 构建完成 → target/release/{{BIN}}" -ForegroundColor Green

[private]
_release-timed-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    start=$(date +%s%N)
    CARGO_INCREMENTAL=1 cargo build -p {{BIN}} --release --timings
    end=$(date +%s%N)
    ms=$(( (end - start) / 1000000 ))
    s=$((ms / 1000)); r=$((ms % 1000))
    m=$((s / 60)); s=$((s % 60))
    printf '\n  ⏱️  cargo build --release    %02d:%02d.%03d\n\n' "$m" "$s" "$r"
    printf '\033[36m  📈 cargo timings  target/cargo-timings/cargo-timing.html\033[0m\n'
    printf '\033[32m✅ 构建完成 → target/release/{{BIN}}\033[0m\n'

[private]
_release-timed-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    start=$(date +%s)
    CARGO_INCREMENTAL=1 cargo build -p {{BIN}} --release --timings
    end=$(date +%s)
    elapsed=$((end - start))
    m=$((elapsed / 60)); s=$((elapsed % 60))
    printf '\n  ⏱️  cargo build --release    %02d:%02d\n\n' "$m" "$s"
    printf '\033[36m  📈 cargo timings  target/cargo-timings/cargo-timing.html\033[0m\n'
    printf '\033[32m✅ 构建完成 → target/release/{{BIN}}\033[0m\n'

# 🔍 快速类型检查 (不生成可执行文件)
check:
    @just info "🔍 运行类型检查..."
    @just info "💡 快速验证模式 (不生成二进制文件)"
    cargo check -p {{BIN}}
    @just success "类型检查通过"

# 🔍 工作区类型检查
check-workspace:
    @just info "🔍 运行工作区类型检查..."
    @just info "💡 覆盖新拆分 crates，避免旁路 crate 漂移"
    cargo check --workspace
    @just success "工作区类型检查通过"

# ═══════════════════════════════════════════════════════════
# ▶️  运行命令
# ═══════════════════════════════════════════════════════════

# ▶️ 运行程序 (Debug版本) - 示例: just run -- --help
run *args:
    @just info "▶️ 运行 Debug 版本"
    @just info "📝 参数: {{args}}"
    cargo run -p {{BIN}} -- {{args}}

# 🚀 运行程序 (Release版本)
run-release *args:
    @just info "🚀 运行 Release 版本"
    @just info "📝 参数: {{args}}"
    cargo run -p {{BIN}} --release -- {{args}}

# 🏷️ 查看版本信息
version:
    @just info "🏷️ 获取版本信息"
    @cargo run -p {{BIN}} -- --version

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
    cargo bench -p {{BIN}}
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

# 🎯 完整 CI 流程 (版本同步 + 自动格式化 + 格式检查 + 严格 Clippy + 测试 + 构建 + 安全审计 + 前端完整检查 + VSCode 扩展检查)
# 每步计时，最后输出汇总表
ci:
    @just _ci-timed-{{os()}}

[private]
_ci-timed-windows:
    #!pwsh.exe
    $ErrorActionPreference = 'Stop'
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    chcp 65001 | Out-Null
    $steps = @(
        @{ Name = "version-sync";    Label = "Version Sync" },
        @{ Name = "fmt";             Label = "Format" },
        @{ Name = "fmt-check";       Label = "Format Check" },
        @{ Name = "lint-strict";     Label = "Strict Clippy" },
        @{ Name = "check-workspace"; Label = "Workspace Check" },
        @{ Name = "test";            Label = "Test" },
        @{ Name = "release";         Label = "Release Build" },
        @{ Name = "audit";           Label = "Security Audit" },
        @{ Name = "frontend-check";  Label = "Frontend Check" },
        @{ Name = "vscode-ci";       Label = "VSCode CI" }
    )
    $PAD = 20
    $results = @()
    $totalSw = [System.Diagnostics.Stopwatch]::StartNew()
    foreach ($step in $steps) {
        Write-Host ""
        Write-Host "━━━ $($step.Label) ━━━" -ForegroundColor Cyan
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        just $step.Name
        $code = $LASTEXITCODE
        $sw.Stop()
        $ts = "{0:mm\:ss\.fff}" -f $sw.Elapsed
        $dots = '.' * ($PAD - $step.Label.Length)
        if ($code -ne 0) {
            Write-Host "  $($step.Label) $dots $ts  FAIL" -ForegroundColor Red
            $results += [PSCustomObject]@{ Label = $step.Label; Time = $ts; Ok = $false }
            $totalSw.Stop()
            $totalTs = "{0:mm\:ss\.fff}" -f $totalSw.Elapsed
            Write-Host ""
            Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Red
            Write-Host "  CI Timing Summary (failed at: $($step.Label))" -ForegroundColor Red
            Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Red
            foreach ($r in $results) {
                $d = '.' * ($PAD - $r.Label.Length)
                $mark = if ($r.Ok) { "OK" } else { "FAIL" }
                $color = if ($r.Ok) { "Green" } else { "Red" }
                Write-Host "  $($r.Label) $d $($r.Time)  $mark" -ForegroundColor $color
            }
            Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            Write-Host "  TOTAL .............. $totalTs" -ForegroundColor Yellow
            exit $code
        }
        Write-Host "  $($step.Label) $dots $ts  OK" -ForegroundColor Green
        $results += [PSCustomObject]@{ Label = $step.Label; Time = $ts; Ok = $true }
    }
    $totalSw.Stop()
    $totalTs = "{0:mm\:ss\.fff}" -f $totalSw.Elapsed
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "  CI Timing Summary" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    foreach ($r in $results) {
        $d = '.' * ($PAD - $r.Label.Length)
        Write-Host "  $($r.Label) $d $($r.Time)  OK" -ForegroundColor Green
    }
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host "  TOTAL .............. $totalTs" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "          CI passed - all steps green!" -ForegroundColor Green

[private]
_ci-timed-linux:
    #!/usr/bin/env bash
    set -uo pipefail
    steps=("version-sync" "fmt" "fmt-check" "lint-strict" "check-workspace" "test" "release" "audit" "frontend-check" "vscode-ci")
    labels=("Version Sync" "Format" "Format Check" "Strict Clippy" "Workspace Check" "Test" "Release Build" "Security Audit" "Frontend Check" "VSCode CI")
    PAD=20
    times=()
    statuses=()
    pad_dots() { local n=$(( PAD - ${#1} )); printf '%*s' "$n" '' | tr ' ' '.'; }
    total_start=$(date +%s%N)
    for i in "${!steps[@]}"; do
        printf '\n\033[36m━━━ %s ━━━\033[0m\n' "${labels[$i]}"
        step_start=$(date +%s%N)
        just "${steps[$i]}" && code=0 || code=$?
        step_end=$(date +%s%N)
        ms=$(( (step_end - step_start) / 1000000 ))
        s=$((ms / 1000)); r=$((ms % 1000)); m=$((s / 60)); s=$((s % 60))
        ts=$(printf '%02d:%02d.%03d' "$m" "$s" "$r")
        times+=("$ts")
        dots=$(pad_dots "${labels[$i]}")
        if [ "$code" -ne 0 ]; then
            statuses+=("FAIL")
            printf '  \033[31m%s %s %s  FAIL\033[0m\n' "${labels[$i]}" "$dots" "$ts"
            total_end=$(date +%s%N)
            tms=$(( (total_end - total_start) / 1000000 ))
            ts2=$((tms / 1000)); tr2=$((tms % 1000)); tm2=$((ts2 / 60)); ts2=$((ts2 % 60))
            total_ts=$(printf '%02d:%02d.%03d' "$tm2" "$ts2" "$tr2")
            printf '\n\033[31m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
            printf '\033[31m  CI Timing Summary (failed at: %s)\033[0m\n' "${labels[$i]}"
            printf '\033[31m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
            for j in "${!times[@]}"; do
                d=$(pad_dots "${labels[$j]}")
                if [ "${statuses[$j]}" = "FAIL" ]; then
                    printf '  \033[31m%s %s %s  %s\033[0m\n' "${labels[$j]}" "$d" "${times[$j]}" "${statuses[$j]}"
                else
                    printf '  \033[32m%s %s %s  %s\033[0m\n' "${labels[$j]}" "$d" "${times[$j]}" "${statuses[$j]}"
                fi
            done
            printf '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n'
            printf '\033[33m  TOTAL .............. %s\033[0m\n' "$total_ts"
            exit "$code"
        fi
        statuses+=("OK")
        printf '  \033[32m%s %s %s  OK\033[0m\n' "${labels[$i]}" "$dots" "$ts"
    done
    total_end=$(date +%s%N)
    tms=$(( (total_end - total_start) / 1000000 ))
    ts2=$((tms / 1000)); tr2=$((tms % 1000)); tm2=$((ts2 / 60)); ts2=$((ts2 % 60))
    total_ts=$(printf '%02d:%02d.%03d' "$tm2" "$ts2" "$tr2")
    printf '\n\033[36m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
    printf '\033[36m  CI Timing Summary\033[0m\n'
    printf '\033[36m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
    for j in "${!times[@]}"; do
        d=$(pad_dots "${labels[$j]}")
        printf '  \033[32m%s %s %s  OK\033[0m\n' "${labels[$j]}" "$d" "${times[$j]}"
    done
    printf '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n'
    printf '\033[33m  TOTAL .............. %s\033[0m\n' "$total_ts"
    printf '\n          \033[32mCI passed - all steps green!\033[0m\n'

[private]
_ci-timed-macos:
    #!/usr/bin/env bash
    set -uo pipefail
    steps=("version-sync" "fmt" "fmt-check" "lint-strict" "check-workspace" "test" "release" "audit" "frontend-check" "vscode-ci")
    labels=("Version Sync" "Format" "Format Check" "Strict Clippy" "Workspace Check" "Test" "Release Build" "Security Audit" "Frontend Check" "VSCode CI")
    PAD=20
    times=()
    statuses=()
    pad_dots() { local n=$(( PAD - ${#1} )); printf '%*s' "$n" '' | tr ' ' '.'; }
    total_start=$(date +%s)
    for i in "${!steps[@]}"; do
        printf '\n\033[36m━━━ %s ━━━\033[0m\n' "${labels[$i]}"
        step_start=$(date +%s)
        just "${steps[$i]}" && code=0 || code=$?
        step_end=$(date +%s)
        elapsed=$((step_end - step_start))
        m=$((elapsed / 60)); s=$((elapsed % 60))
        ts=$(printf '%02d:%02d' "$m" "$s")
        times+=("$ts")
        dots=$(pad_dots "${labels[$i]}")
        if [ "$code" -ne 0 ]; then
            statuses+=("FAIL")
            printf '  \033[31m%s %s %s  FAIL\033[0m\n' "${labels[$i]}" "$dots" "$ts"
            total_end=$(date +%s)
            total_elapsed=$((total_end - total_start))
            tm=$((total_elapsed / 60)); tsec=$((total_elapsed % 60))
            total_ts=$(printf '%02d:%02d' "$tm" "$tsec")
            printf '\n\033[31m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
            printf '\033[31m  CI Timing Summary (failed at: %s)\033[0m\n' "${labels[$i]}"
            printf '\033[31m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
            for j in "${!times[@]}"; do
                d=$(pad_dots "${labels[$j]}")
                if [ "${statuses[$j]}" = "FAIL" ]; then
                    printf '  \033[31m%s %s %s  %s\033[0m\n' "${labels[$j]}" "$d" "${times[$j]}" "${statuses[$j]}"
                else
                    printf '  \033[32m%s %s %s  %s\033[0m\n' "${labels[$j]}" "$d" "${times[$j]}" "${statuses[$j]}"
                fi
            done
            printf '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n'
            printf '\033[33m  TOTAL .............. %s\033[0m\n' "$total_ts"
            exit "$code"
        fi
        statuses+=("OK")
        printf '  \033[32m%s %s %s  OK\033[0m\n' "${labels[$i]}" "$dots" "$ts"
    done
    total_end=$(date +%s)
    total_elapsed=$((total_end - total_start))
    tm=$((total_elapsed / 60)); tsec=$((total_elapsed % 60))
    total_ts=$(printf '%02d:%02d' "$tm" "$tsec")
    printf '\n\033[36m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
    printf '\033[36m  CI Timing Summary\033[0m\n'
    printf '\033[36m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n'
    for j in "${!times[@]}"; do
        d=$(pad_dots "${labels[$j]}")
        printf '  \033[32m%s %s %s  OK\033[0m\n' "${labels[$j]}" "$d" "${times[$j]}"
    done
    printf '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n'
    printf '\033[33m  TOTAL .............. %s\033[0m\n' "$total_ts"
    printf '\n          \033[32mCI passed - all steps green!\033[0m\n'

# ═══════════════════════════════════════════════════════════
# 🌐 前端检查命令
# ═══════════════════════════════════════════════════════════

# 🔍 前端 TypeScript 类型检查
frontend-typecheck:
    @just header "🔍 前端 TypeScript 类型检查"
    cd ccr-ui && bun install --frozen-lockfile && bun run type-check
    @just success "前端类型检查通过"

# 🎨 前端 Lint 检查
frontend-lint:
    @just header "🎨 前端 Lint 检查"
    cd ccr-ui && bun install --frozen-lockfile && bun run lint
    @just success "前端 Lint 检查通过"

# 🧪 前端 Smoke Tests
frontend-test:
    @just header "🧪 前端 Smoke Tests"
    cd ccr-ui && bun install --frozen-lockfile && bun run test
    @just success "前端 Smoke Tests 通过"

# 🏗️ 前端构建
frontend-build:
    @just header "🏗️ 前端构建"
    cd ccr-ui && bun install --frozen-lockfile && bun run build
    @just success "前端构建完成"

# 📚 文档构建检查 (VitePress) - 可选，有 dead links 时可能失败
docs-check:
    @just header "📚 文档构建检查"
    @just warn "注意: 若有 dead links 会失败，可在 .vitepress/config 中配置 ignoreDeadLinks"
    cd docs && npm install && node ./node_modules/vitepress/bin/vitepress.js build
    @just success "文档构建检查完成"

# 🤖 GitHub Copilot 工作区资产检查
copilot-check:
    @just header "🤖 GitHub Copilot 工作区资产检查"
    node scripts/check-copilot-assets.mjs
    @just success "GitHub Copilot 工作区资产检查通过"

# 🌐 前端完整检查 (类型检查 + Lint + 构建 + 文档构建)
frontend-check: frontend-typecheck frontend-lint frontend-test frontend-build docs-check
    @just success "前端检查全部通过"

# 🌐 前端快速检查 (类型检查 + Lint，不含构建和文档)
frontend-check-quick: frontend-typecheck frontend-lint frontend-test
    @just success "前端快速检查通过"

# ═══════════════════════════════════════════════════════════
# 📦 安装与管理命令
# ═══════════════════════════════════════════════════════════

# 📦 安装到本地 (~/.cargo/bin)
install:
    @just header "📦 安装到本地"
    @just info "📍 目标路径: ~/.cargo/bin/{{BIN}}"
    @just info "🔒 模式: 锁定依赖版本 (--locked)"
    cargo install --path {{CLI_CRATE_PATH}} --locked
    @just success "安装完成"

# ♻️ 强制重新安装
reinstall:
    @just info "♻️ 强制重新安装"
    @just warn "模式: 覆盖现有安装"
    cargo install --path {{CLI_CRATE_PATH}} --locked --force
    @just success "重新安装完成"

# 🗑️ 卸载已安装的二进制
uninstall:
    @just info "🗑️ 卸载 {{BIN}}"
    cargo uninstall {{BIN}}
    @just success "卸载完成"

# ═══════════════════════════════════════════════════════════
# 📚 文档命令
# ═══════════════════════════════════════════════════════════

# 🌐 启动 VitePress 文档站
docs:
    @just header "🌐 启动文档站"
    @just info "📍 项目路径: docs"
    @just info "📝 将转到 docs/ 并执行 npm run dev"
    cd docs && npm install && npm run dev

# 🌐 构建并在浏览器中打开文档
doc-open:
    @just info "🌐 生成并打开文档"
    @just info "📖 将在默认浏览器中打开"
    cargo doc -p {{BIN}} --no-deps --open

outputs-collect: outputs-collect-cli outputs-collect-ui outputs-collect-vscode
    @just success "Outputs collection completed"

outputs-collect-cli: release
    @just _outputs-collect-cli-{{os()}}

[private]
_outputs-collect-cli-linux:
    @mkdir -p {{OUTPUTS_DIR}}/ccr
    cp target/release/{{BIN}} {{OUTPUTS_DIR}}/ccr/
    @just success "CLI artifacts collected to {{OUTPUTS_DIR}}/ccr/"

[private]
_outputs-collect-cli-macos:
    @mkdir -p {{OUTPUTS_DIR}}/ccr
    cp target/release/{{BIN}} {{OUTPUTS_DIR}}/ccr/
    @just success "CLI artifacts collected to {{OUTPUTS_DIR}}/ccr/"

[private]
_outputs-collect-cli-windows:
    @New-Item -ItemType Directory -Force -Path "{{OUTPUTS_DIR}}/ccr" | Out-Null
    @Copy-Item "target/release/{{BIN}}.exe" "{{OUTPUTS_DIR}}/ccr/" -Force
    @just success "CLI artifacts collected to {{OUTPUTS_DIR}}/ccr/"

outputs-collect-ui: ui-build _outputs-collect-ui-sync
    @just success "CCR UI artifacts collection completed"

[private]
_outputs-collect-ui-sync:
    @just _outputs-collect-ui-sync-{{os()}}

[private]
_outputs-collect-ui-sync-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    dest_root="{{OUTPUTS_DIR}}/ccr-ui"
    release_dest="$dest_root/src-tauri/target/release"
    source_bundle="ccr-ui/src-tauri/target/release/bundle"
    mkdir -p "$release_dest"
    rm -rf "$dest_root/dist" "$release_dest/bundle"
    cp -R ccr-ui/dist "$dest_root/"
    cp ccr-ui/src-tauri/target/release/ccr-desktop "$release_dest/"
    if [ -d "$source_bundle" ]; then
        mkdir -p "$release_dest/bundle"
        for format_dir in "$source_bundle"/*; do
            [ -d "$format_dir" ] || continue
            format_name="$(basename "$format_dir")"
            latest_item=""
            latest_mtime=0
            for candidate in "$format_dir"/*; do
                [ -e "$candidate" ] || continue
                mtime="$(stat -c '%Y' "$candidate")"
                if [ -z "$latest_item" ] || [ "$mtime" -gt "$latest_mtime" ]; then
                    latest_item="$candidate"
                    latest_mtime="$mtime"
                fi
            done
            [ -n "$latest_item" ] || continue
            mkdir -p "$release_dest/bundle/$format_name"
            cp -R "$latest_item" "$release_dest/bundle/$format_name/"
        done
    fi
    just success "CCR UI artifacts synchronized to {{OUTPUTS_DIR}}/ccr-ui/"

[private]
_outputs-collect-ui-sync-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    dest_root="{{OUTPUTS_DIR}}/ccr-ui"
    release_dest="$dest_root/src-tauri/target/release"
    source_bundle="ccr-ui/src-tauri/target/release/bundle"
    mkdir -p "$release_dest"
    rm -rf "$dest_root/dist" "$release_dest/bundle"
    cp -R ccr-ui/dist "$dest_root/"
    cp ccr-ui/src-tauri/target/release/ccr-desktop "$release_dest/"
    if [ -d "$source_bundle" ]; then
        mkdir -p "$release_dest/bundle"
        for format_dir in "$source_bundle"/*; do
            [ -d "$format_dir" ] || continue
            format_name="$(basename "$format_dir")"
            latest_item=""
            latest_mtime=0
            for candidate in "$format_dir"/*; do
                [ -e "$candidate" ] || continue
                mtime="$(stat -f '%m' "$candidate")"
                if [ -z "$latest_item" ] || [ "$mtime" -gt "$latest_mtime" ]; then
                    latest_item="$candidate"
                    latest_mtime="$mtime"
                fi
            done
            [ -n "$latest_item" ] || continue
            mkdir -p "$release_dest/bundle/$format_name"
            cp -R "$latest_item" "$release_dest/bundle/$format_name/"
        done
    fi
    just success "CCR UI artifacts synchronized to {{OUTPUTS_DIR}}/ccr-ui/"

[private]
_outputs-collect-ui-sync-windows:
    #!pwsh.exe
    $ErrorActionPreference = 'Stop'
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    chcp 65001 | Out-Null
    $destRoot = Join-Path '{{OUTPUTS_DIR}}' 'ccr-ui'
    $releaseDest = Join-Path $destRoot 'src-tauri/target/release'
    $sourceBundle = 'ccr-ui/src-tauri/target/release/bundle'
    New-Item -ItemType Directory -Force -Path $releaseDest | Out-Null
    if (Test-Path (Join-Path $destRoot 'dist')) {
        Remove-Item (Join-Path $destRoot 'dist') -Recurse -Force
    }
    if (Test-Path (Join-Path $releaseDest 'bundle')) {
        Remove-Item (Join-Path $releaseDest 'bundle') -Recurse -Force
    }
    Copy-Item 'ccr-ui/dist' $destRoot -Recurse -Force
    Copy-Item 'ccr-ui/src-tauri/target/release/ccr-desktop.exe' $releaseDest -Force
    if (Test-Path $sourceBundle) {
        $bundleDest = Join-Path $releaseDest 'bundle'
        New-Item -ItemType Directory -Force -Path $bundleDest | Out-Null
        Get-ChildItem $sourceBundle -Directory | ForEach-Object {
            $latestItem = Get-ChildItem $_.FullName -Force | Sort-Object LastWriteTime -Descending | Select-Object -First 1
            if ($null -ne $latestItem) {
                $formatDest = Join-Path $bundleDest $_.Name
                New-Item -ItemType Directory -Force -Path $formatDest | Out-Null
                Copy-Item $latestItem.FullName $formatDest -Recurse -Force
            }
        }
    }
    just success "CCR UI artifacts synchronized to {{OUTPUTS_DIR}}/ccr-ui/"

# ═══════════════════════════════════════════════════════════
# 🧹 清理与维护命令
# ═══════════════════════════════════════════════════════════

# 🧹 清理构建产物
clean:
    @just info "🧹 清理构建产物"
    @just info "📂 清理目标: target/ 目录"
    cargo clean
    @just success "清理完成"

# 🗂️ 清理归档产物
outputs-clean:
    @just _outputs-clean-{{os()}}

[private]
_outputs-clean-linux:
    rm -rf {{OUTPUTS_DIR}}
    @just success "Collected outputs cleaned"

[private]
_outputs-clean-macos:
    rm -rf {{OUTPUTS_DIR}}
    @just success "Collected outputs cleaned"

[private]
_outputs-clean-windows:
    @if (Test-Path "{{OUTPUTS_DIR}}") { Remove-Item "{{OUTPUTS_DIR}}" -Recurse -Force }
    @just success "Collected outputs cleaned"

# 📦 检查依赖更新
update-deps:
    @just info "📦 检查依赖更新"
    @just info "📌 使用 cargo-outdated (需要安装: cargo install cargo-outdated)"
    cargo outdated

# 💣 深度清理 (包括 Cargo 缓存和目标文件)
deep-clean: clean outputs-clean
    @just header "💣 深度清理"
    @just warn "警告：将清理 Cargo 缓存"
    @just info "🗑️  清理 Cargo 注册表缓存"
    cargo clean
    @just success "深度清理完成"

# ═══════════════════════════════════════════════════════════
# 🔧 版本号同步命令
# ═══════════════════════════════════════════════════════════

# 同步版本号到前端与 Tauri（以根 Cargo.toml 为主）
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

# 🧪 运行脚本测试 (Bats + Pester)
test-scripts:
    @just _test-scripts-{{os()}}

[private]
_test-scripts-windows:
    @just header "🧪 运行脚本测试"
    @just info "📌 检查 Bats/Pester 是否安装"
    @# 检查 Pester
    -pwsh -NoProfile -Command "if (Get-Module -ListAvailable Pester) { Write-Host 'Pester 已安装' -ForegroundColor Green; Invoke-Pester -Path 'tests/scripts/version-sync.Tests.ps1' -PassThru } else { Write-Host 'Pester 未安装，请运行: Install-Module Pester -Force' -ForegroundColor Yellow }"
    @# 检查 Bats (如果安装了 Git Bash 或 WSL)
    -bash -c "if command -v bats &>/dev/null; then echo 'Bats 已安装'; bats tests/scripts/version-sync.bats; else echo 'Bats 未安装，请参考: https://github.com/bats-core/bats-core'; fi"

[private]
_test-scripts-linux:
    @just header "🧪 运行脚本测试"
    @just info "📌 检查 Bats 是否安装"
    @if command -v bats &>/dev/null; then \
        echo "Bats 已安装，运行测试..."; \
        bats tests/scripts/version-sync.bats; \
    else \
        echo "Bats 未安装，请参考: https://github.com/bats-core/bats-core"; \
    fi
    @# 检查 Pester (如果有 pwsh)
    -pwsh -NoProfile -Command "if (Get-Module -ListAvailable Pester) { Write-Host 'Pester 已安装' -ForegroundColor Green; Invoke-Pester -Path 'tests/scripts/version-sync.Tests.ps1' -PassThru } else { Write-Host 'Pester 未安装 (可选)' -ForegroundColor Yellow }"

[private]
_test-scripts-macos:
    @just header "🧪 运行脚本测试"
    @just info "📌 检查 Bats 是否安装"
    @if command -v bats &>/dev/null; then \
        echo "Bats 已安装，运行测试..."; \
        bats tests/scripts/version-sync.bats; \
    else \
        echo "Bats 未安装，请运行: brew install bats-core"; \
    fi

# ===== CCR UI Commands (migrated from ccr-ui/justfile) =====

# CCR UI 子 justfile 路径
CCR_UI_JUSTFILE := "ccr-ui/justfile"

# 内部执行器：在根目录转发到 ccr-ui/justfile
[private]
[no-cd]
_ui-run recipe:
    just --justfile {{CCR_UI_JUSTFILE}} {{recipe}}

# 迁移后的命令：统一使用 ui- 前缀，避免与根命令冲突
ui-default:
    @just _ui-run default

ui-bench-backend:
    @just _ui-run bench-backend

ui-build:
    @just _ui-run build

ui-build-backend:
    @just _ui-run build-backend

ui-build-frontend:
    @just _ui-run build-frontend

ui-check:
    @just _ui-run check

ui-check-backend:
    @just _ui-run check-backend

ui-check-frontend:
    @just _ui-run check-frontend

ui-check-frontend-lint:
    @just _ui-run check-frontend-lint

ui-check-frontend-types:
    @just _ui-run check-frontend-types

ui-check-prereqs:
    @just _ui-run check-prereqs

ui-check-security:
    @just _ui-run check-security

ui-ci:
    @just _ui-run ci

ui-ci-security:
    @just _ui-run ci-security

ui-clean:
    @just _ui-run clean

ui-clean-all:
    @just _ui-run clean-all

ui-clean-backend:
    @just _ui-run clean-backend

ui-clean-frontend:
    @just _ui-run clean-frontend

ui-clean-logs:
    @just _ui-run clean-logs

ui-clippy:
    @just _ui-run clippy

ui-dev:
    @just _ui-run dev

ui-dev-backend:
    @just _ui-run dev-backend

ui-dev-clean:
    @just _ui-run dev-clean

ui-dev-fast:
    @just _ui-run dev-fast

ui-dev-fast-parallel:
    @just _ui-run dev-fast-parallel

ui-dev-frontend:
    @just _ui-run dev-frontend

ui-dev-parallel:
    @just _ui-run dev-parallel

ui-dev-react-frontend:
    @just _ui-run dev-react-frontend

ui-doc-backend:
    @just _ui-run doc-backend

ui-fmt:
    @just _ui-run fmt

ui-fmt-backend:
    @just _ui-run fmt-backend

ui-fmt-frontend:
    @just _ui-run fmt-frontend

ui-help:
    @just _ui-run help

ui-info:
    @just _ui-run info

ui-install:
    @just _ui-run install

ui-install-backend:
    @just _ui-run install-backend

ui-install-frontend:
    @just _ui-run install-frontend

ui-logs-backend:
    @just _ui-run logs-backend

ui-logs-frontend:
    @just _ui-run logs-frontend

ui-prepare-release:
    @just _ui-run prepare-release

ui-quick-start:
    @just _ui-run quick-start

ui-run-prod:
    @just _ui-run run-prod

ui-serve-frontend:
    @just _ui-run serve-frontend

tauri-build:
    @just _ui-run tauri-build
    @just _outputs-collect-ui-sync

tauri-build-debug:
    @just _ui-run tauri-build-debug

tauri-verify-release-window:
    @just _ui-run tauri-verify-release-window

tauri-check:
    @just _ui-run tauri-check

tauri-check-all:
    @just _ui-run tauri-check-all

tauri-check-rust:
    @just _ui-run tauri-check-rust

tauri-clean:
    @just _ui-run tauri-clean

tauri-clippy:
    @just _ui-run tauri-clippy

tauri-dev:
    @just _ui-run tauri-dev

tauri-fmt:
    @just _ui-run tauri-fmt

tauri-test:
    @just _ui-run tauri-test

ui-test:
    @just _ui-run test

ui-test-backend:
    @just _ui-run test-backend

ui-test-frontend:
    @just _ui-run test-frontend

ui-update:
    @just _ui-run update

ui-update-backend:
    @just _ui-run update-backend

ui-update-frontend:
    @just _ui-run update-frontend

ui-watch-backend:
    @just _ui-run watch-backend

ui-dev-web:
    @just _ui-dev-web-{{os()}}

[private]
_ui-dev-web-windows:
    #!pwsh.exe
    $ErrorActionPreference = 'Stop';
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8;
    $env:VITE_PORT = "5173";
    $env:BACKEND_PORT = "38081";
    just --justfile {{CCR_UI_JUSTFILE}} dev-web;

[private]
_ui-dev-web-linux:
    #!/usr/bin/env bash
    VITE_PORT=5173 BACKEND_PORT=38081 just --justfile {{CCR_UI_JUSTFILE}} dev-web

[private]
_ui-dev-web-macos:
    #!/usr/bin/env bash
    VITE_PORT=5173 BACKEND_PORT=38081 just --justfile {{CCR_UI_JUSTFILE}} dev-web

# 迁移后的快捷别名（对应 ccr-ui/justfile 中的别名）
alias ui-s := ui-dev
alias ui-i := ui-install
alias ui-b := ui-build
alias ui-c := ui-check
alias ui-t := ui-test
alias ui-f := ui-fmt

# Tauri 快捷别名（无 ui 前缀）
alias tdev := tauri-dev
alias tbuild := tauri-build
alias tverify := tauri-verify-release-window
alias tcheck := tauri-check
alias tclean := tauri-clean
alias ttest := tauri-test

# ===== End CCR UI Commands =====

# ═══════════════════════════════════════════════════════════
# 🔌 CCR VSCode Extension Commands
# ═══════════════════════════════════════════════════════════

CCR_VSCODE_JUSTFILE := "ccr-vscode/justfile"

[private]
[no-cd]
_vscode-run recipe:
    just --justfile {{CCR_VSCODE_JUSTFILE}} {{recipe}}

# 📦 VSCode 扩展: 安装依赖
vscode-install:
    @just _vscode-run install

# 🔨 VSCode 扩展: 构建 + 打包 .vsix + 复制到 outputs/
vscode-build:
    @just _vscode-run build

# 👀 VSCode 扩展: 监控模式
vscode-watch:
    @just _vscode-run watch

# 🔍 VSCode 扩展: 类型检查
vscode-lint:
    @just _vscode-run lint

# ✅ VSCode 扩展: 运行测试
vscode-test:
    @just _vscode-run test

# 🧹 VSCode 扩展: 清理
vscode-clean:
    @just _vscode-run clean

# 🚀 VSCode 扩展: 完整 CI
vscode-ci:
    @just _vscode-run ci

# 📦 收集 VSCode 扩展构建产物到 outputs/
outputs-collect-vscode: vscode-install vscode-build
    @just success "VSCode extension .vsix collected to {{OUTPUTS_DIR}}/ccr-vscode/"

# VSCode 快捷别名
alias vs-b := vscode-build
alias vs-t := vscode-test
alias vs-l := vscode-lint
alias vs-c := vscode-ci

# ===== End CCR VSCode Extension Commands =====
