# 构建系统

CCR 使用 Cargo 作为构建系统，并提供 Just 脚本简化常用操作。

## 🦀 Cargo 构建

### 基础构建命令

```bash
# 构建 Debug 版本
cargo build

# 构建 Release 版本
cargo build --release

# 仅检查（不生成二进制）
cargo check

# 清理构建产物
cargo clean
```

### 构建配置

#### Cargo.toml 配置

```toml
[package]
name = "ccr"
version = "0.2.0"
edition = "2021"

[[bin]]
name = "ccr"
path = "src/main.rs"

[profile.release]
opt-level = 3        # 最高优化级别
lto = true           # 链接时优化 (Link Time Optimization)
codegen-units = 1    # 单编译单元（更好的优化）
strip = true         # 剥离调试符号
```

### 优化级别说明

| opt-level | 优化程度 | 编译时间 | 二进制大小 | 性能 |
|-----------|---------|---------|-----------|------|
| 0 | 无优化 | 快 | 大 | 慢 |
| 1 | 基础优化 | 中 | 中 | 中 |
| 2 | 中等优化 | 慢 | 小 | 快 |
| 3 | 最高优化 | 很慢 | 最小 | 最快 ⭐ |

**CCR 使用 opt-level = 3** 以获得最佳性能。

### LTO（链接时优化）

```toml
lto = true  # 或 "fat"（完整 LTO）
```

**效果**:
- 跨 crate 内联
- 消除死代码
- 二进制大小减少 20-30%
- 性能提升 10-20%

**代价**:
- 编译时间增加 2-3 倍

---

## ⚙️ 构建选项

### 功能特性

```toml
[features]
default = []
update = ["reqwest", "tokio"]  # 在线更新功能
web = ["tiny_http", "open"]    # Web 界面
```

**使用**:
```bash
# 仅基础功能
cargo build --release --no-default-features

# 包含更新功能
cargo build --release --features update

# 包含所有功能
cargo build --release --all-features
```

### 目标平台

```bash
# 查看当前目标
rustc --version --verbose | grep host

# 交叉编译（示例）
cargo build --target x86_64-unknown-linux-musl
cargo build --target x86_64-apple-darwin
cargo build --target x86_64-pc-windows-gnu
```

## 📊 构建性能

### 编译时间

| 构建类型 | 首次编译 | 增量编译 | 说明 |
|---------|---------|---------|------|
| Debug | ~45s | ~5s | 无优化 |
| Release | ~120s | ~15s | 完整优化 |

**优化建议**:

**1. 使用 sccache**
```bash
# 安装
cargo install sccache

# 配置
export RUSTC_WRAPPER=sccache

# 编译
cargo build --release
```

**2. 使用 mold 链接器**
```bash
# 安装（Linux）
sudo apt install mold

# 配置
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# 编译
cargo build --release
```

**3. 增加并行度**
```bash
# 设置编译并行数
export CARGO_BUILD_JOBS=8

# 或在 ~/.cargo/config.toml 中
[build]
jobs = 8
```

### 二进制大小

| 构建配置 | 大小 | 说明 |
|---------|------|------|
| Debug | ~15MB | 包含调试信息 |
| Release (默认) | ~4MB | 基础优化 |
| Release (strip) | ~2MB | 剥离符号 ⭐ |
| Release (UPX) | ~800KB | 压缩（可选）|

**进一步压缩**:
```bash
# 使用 strip
strip target/release/ccr

# 或使用 UPX
upx --best --lzma target/release/ccr
```

## 🔨 Just 构建脚本

### 常用任务

```bash
# 查看所有任务
just --list

# 构建
just build         # Debug 版本
just release       # Release 版本

# 运行
just run           # 运行 Debug 版本
just run-release   # 运行 Release 版本

# 测试
just test          # 运行测试

# 代码质量
just check         # 类型检查
just clippy        # Linter
just fmt           # 格式化

# 安装
just install       # 安装到 ~/.cargo/bin
just reinstall     # 强制重新安装
just uninstall     # 卸载

# 清理
just clean         # 清理构建产物
```

### 自定义任务

编辑 `justfile` 添加新任务：

```makefile
# 运行基准测试
bench:
  cargo bench

# 生成文档
doc:
  cargo doc --no-deps --open

# 检查依赖更新
outdated:
  cargo outdated
```

## 🧪 测试构建

### 运行测试

```bash
# 所有测试
cargo test

# 单个测试
cargo test test_config_section_validate

# 带输出
cargo test -- --nocapture

# 单线程
cargo test -- --test-threads=1
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html

# 查看报告
open tarpaulin-report.html
```

## 📝 代码检查

### Clippy

```bash
# 基础检查
cargo clippy

# 严格模式
cargo clippy -- -D warnings

# 所有 lint
cargo clippy -- -W clippy::all
```

**常用 Lint**:
```rust
// 在 src/main.rs 顶部添加
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
```

### Rustfmt

```bash
# 格式化代码
cargo fmt

# 检查格式（CI 用）
cargo fmt -- --check
```

**配置文件** (`rustfmt.toml`):
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
```

## 🚀 发布构建

### 完整发布流程

```bash
# 1. 更新版本号
vim Cargo.toml  # version = "0.3.0"

# 2. 更新 CHANGELOG
vim CHANGELOG.md

# 3. 运行测试
cargo test

# 4. 代码检查
cargo clippy
cargo fmt

# 5. 构建发布版本
cargo build --release

# 6. 测试发布版本
./target/release/ccr --version
./target/release/ccr list
./target/release/ccr validate

# 7. 提交更改
git add .
git commit -m "chore: bump version to 0.3.0"
git tag v0.3.0
git push origin main --tags

# 8. 发布到 crates.io（可选）
cargo publish
```

### 发布检查清单

- [ ] 所有测试通过
- [ ] 无 Clippy 警告
- [ ] 代码格式正确
- [ ] 文档更新
- [ ] CHANGELOG 更新
- [ ] 版本号更新
- [ ] Git 标签创建

## 📦 打包分发

### 创建二进制包

```bash
# Linux
tar -czf ccr-v0.2.0-linux-x86_64.tar.gz -C target/release ccr

# macOS
tar -czf ccr-v0.2.0-macos-x86_64.tar.gz -C target/release ccr

# Windows
zip ccr-v0.2.0-windows-x86_64.zip target/release/ccr.exe
```

### GitHub Release

```bash
# 使用 gh CLI
gh release create v0.2.0 \
  target/release/ccr \
  --title "CCR v0.2.0" \
  --notes "Release notes here"
```

## 🔍 构建问题排查

### 问题：编译失败

```bash
# 清理后重试
cargo clean
cargo build --release

# 更新依赖
cargo update
```

### 问题：链接错误

```bash
# 检查系统库
ldd target/release/ccr

# 安装缺失的库（Ubuntu）
sudo apt install build-essential
```

### 问题：内存不足

```bash
# 减少并行编译数
export CARGO_BUILD_JOBS=2
cargo build --release
```

## 🔗 相关文档

- [开发指南](/development/)
- [项目结构](/development/structure)
- [测试指南](/development/testing)
- [贡献指南](/development/contributing)

