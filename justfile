# ccr 项目 Justfile（用于快速执行常用命令）🔧🚀

# 使用指南 📖
# - 在 ccr 目录执行：`just --list` 查看所有任务
# - 构建发布版：`just release`
# - 运行并传参：`just run -- --help` 或 `just run-release -- --version`
# - 安装到本地：`just install`（安装到 ~/.cargo/bin）
# - 前置要求：已安装 Rust 工具链（cargo, rustc）🦀
# - 如果你更改了二进制名，请同步修改下方 BIN 变量 ✍️

# 二进制名称（与 Cargo.toml [[bin]] 保持一致）
BIN := "ccr"

# 使用 bash 作为执行入口以提升跨 shell 的可移植性 🧭
set shell := ["bash", "-cu"]

# 默认任务：显示帮助菜单 ℹ️
default: help

# 显示所有可用任务 📜
help:
  @just --list

# 调试构建（Debug 模式）🏗️
build:
  cargo build

# 发布构建（Release 优化）⚡
release:
  cargo build --release

# 快速类型检查（不生成可执行文件）🩺
check:
  cargo check

# 运行程序（可通过 -- 传递参数）▶️ 示例：`just run -- --help`
run *args:
  cargo run -- {{args}}

# 运行发布版二进制（若未构建会先构建）🚀
run-release *args:
  just release
  ./target/release/{{BIN}} {{args}}

# 运行测试 ✅
test:
  cargo test

# 代码格式化 ✨
fmt:
  cargo fmt

# 静态检查（Clippy），将警告视为错误 ❗
clippy:
  cargo clippy -- -D warnings

# 清理构建产物 🧹
clean:
  cargo clean

# 安装到本地（~/.cargo/bin）📦 建议使用 --locked 保持依赖锁定
install:
  cargo install --path . --locked

# 强制重新安装 ♻️
reinstall:
  cargo install --path . --locked --force

# 卸载已安装的二进制 🗑️
uninstall:
  cargo uninstall {{BIN}}

# 通过程序输出版本号 🏷️
version:
  cargo run -- --version

# 构建文档（不包含依赖）📚
doc:
  cargo doc --no-deps

# 构建并在浏览器中打开文档 🌐
doc-open:
  cargo doc --no-deps --open