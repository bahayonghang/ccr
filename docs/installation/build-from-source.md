# 从源码构建

本指南详细介绍如何从源码编译和安装 CCR。

## 🦀 前置要求

### Rust 工具链

**最低版本**: Rust 1.70.0

#### 安装 Rust

**Linux / macOS**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows**:
- 下载 [rustup-init.exe](https://rustup.rs/)
- 运行安装程序

#### 验证安装

```bash
rustc --version
# 输出: rustc 1.75.0 (82e1608df 2023-12-21)

cargo --version
# 输出: cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

#### 更新 Rust

```bash
rustup update
```

### 其他依赖

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install build-essential pkg-config
```

**Linux (Fedora)**:
```bash
sudo dnf install gcc pkg-config
```

**macOS**:
```bash
xcode-select --install
```

**Windows**:
- 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- 或使用 WSL2

## 📦 获取源码

### 克隆仓库

```bash
# HTTPS
git clone https://github.com/bahayonghang/ccs.git
cd ccs/ccr

# SSH
git clone git@github.com:bahayonghang/ccs.git
cd ccs/ccr

# 特定分支
git clone -b dev https://github.com/bahayonghang/ccs.git
```

### 目录结构

```
ccs/
├── ccr/              ← CCR 项目（进入此目录）
│   ├── src/
│   ├── web/
│   ├── Cargo.toml
│   └── ...
├── scripts/          ← CCS Shell 脚本
├── web/              ← CCS Web 界面
└── config/           ← 配置示例
```

## 🔨 构建步骤

### 1. 进入 CCR 目录

```bash
cd ccs/ccr
```

### 2. 检查依赖

```bash
# 查看依赖
cargo tree

# 检查过期依赖
cargo outdated  # 需要: cargo install cargo-outdated
```

### 3. 构建项目

#### Debug 构建（开发用）

```bash
cargo build
```

**特点**:
- 编译快（~45 秒）
- 包含调试信息
- 二进制大（~15MB）
- 性能较低

**产物**:
```
target/debug/ccr
```

#### Release 构建（生产用）

```bash
cargo build --release
```

**特点**:
- 编译慢（~120 秒）
- 完整优化
- 二进制小（~2MB）
- 性能最佳 ⭐

**产物**:
```
target/release/ccr
```

### 4. 验证构建

```bash
# Debug 版本
./target/debug/ccr --version

# Release 版本
./target/release/ccr --version

# 应该输出
ccr 0.2.0
```

## 🚀 安装步骤

### 方式 1: Cargo Install（推荐）

```bash
# 从当前目录安装
cargo install --path . --locked

# 强制重新安装
cargo install --path . --locked --force
```

**安装位置**:
```
~/.cargo/bin/ccr
```

**验证**:
```bash
which ccr
# 输出: /home/user/.cargo/bin/ccr

ccr --version
# 输出: ccr 0.2.0
```

### 方式 2: 手动复制

```bash
# 构建 Release 版本
cargo build --release

# 复制到系统路径
sudo cp target/release/ccr /usr/local/bin/

# 或复制到用户路径
mkdir -p ~/.local/bin
cp target/release/ccr ~/.local/bin/

# 添加到 PATH（如果需要）
export PATH="$HOME/.local/bin:$PATH"
```

### 方式 3: 创建符号链接

```bash
# 构建
cargo build --release

# 创建链接
sudo ln -s $(pwd)/target/release/ccr /usr/local/bin/ccr

# 或用户路径
mkdir -p ~/.local/bin
ln -s $(pwd)/target/release/ccr ~/.local/bin/ccr
```

## 🔧 高级构建选项

### 交叉编译

#### Linux 静态链接二进制

```bash
# 安装目标
rustup target add x86_64-unknown-linux-musl

# 安装 musl 工具
sudo apt install musl-tools  # Ubuntu/Debian

# 构建
cargo build --release --target x86_64-unknown-linux-musl

# 产物
ls -lh target/x86_64-unknown-linux-musl/release/ccr
```

**优势**:
- 无动态库依赖
- 可在任何 Linux 发行版运行

#### macOS 通用二进制

```bash
# 安装目标
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# 分别构建
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# 合并为通用二进制
lipo -create \
  target/x86_64-apple-darwin/release/ccr \
  target/aarch64-apple-darwin/release/ccr \
  -output ccr-universal
```

### 优化二进制大小

#### 1. 启用 strip

```toml
[profile.release]
strip = true  # 剥离符号
```

**效果**: 4MB → 2MB

#### 2. 优化 panic 处理

```toml
[profile.release]
panic = "abort"  # 不展开 panic
```

**效果**: 减少 100-200KB

#### 3. 优化标准库

```toml
[profile.release]
opt-level = "z"  # 优化大小而非速度
```

**权衡**: 性能下降 ~10%，大小减少 ~20%

#### 4. 使用 UPX 压缩

```bash
# 安装 UPX
sudo apt install upx  # Linux
brew install upx      # macOS

# 压缩
upx --best --lzma target/release/ccr

# 效果: 2MB → 800KB
```

**注意**: 压缩后启动时间略有增加（解压缩开销）

## 🧪 验证构建

### 基础测试

```bash
# 1. 运行测试套件
cargo test

# 2. 检查帮助
./target/release/ccr --help

# 3. 测试基本功能
./target/release/ccr list
./target/release/ccr current
./target/release/ccr validate
```

### 性能测试

```bash
# 启动时间
time ./target/release/ccr --version

# 配置切换时间
time ./target/release/ccr switch anthropic

# 内存使用
/usr/bin/time -v ./target/release/ccr list
```

### 完整性测试

```bash
# 检查动态库依赖
ldd target/release/ccr  # Linux
otool -L target/release/ccr  # macOS

# 检查符号
nm target/release/ccr

# 检查二进制信息
file target/release/ccr
```

## 📊 构建基准

### 硬件要求

**最低配置**:
- CPU: 2 核
- 内存: 2GB
- 磁盘: 1GB 可用空间

**推荐配置**:
- CPU: 4 核或更多
- 内存: 4GB 或更多
- 磁盘: 2GB 可用空间
- SSD 存储

### 编译时间参考

| 硬件配置 | Debug | Release |
|---------|-------|---------|
| i5-8250U, 8GB | 60s | 180s |
| i7-10700K, 32GB | 35s | 90s |
| M1 Mac Mini | 25s | 60s |
| Ryzen 9 5900X | 20s | 50s |

## 🔍 故障排除

### 编译错误

#### 错误：找不到 pkg-config

```
error: failed to run custom build command for `openssl-sys`
Could not find pkg-config
```

**解决**:
```bash
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev

# Fedora
sudo dnf install pkgconfig openssl-devel

# macOS
brew install pkg-config openssl
```

#### 错误：链接器失败

```
error: linking with `cc` failed
```

**解决**:
```bash
# 安装 build-essential
sudo apt install build-essential
```

### 内存不足

```
error: could not compile due to previous error
Killed
```

**解决**:
```bash
# 减少并行编译
export CARGO_BUILD_JOBS=1
cargo build --release
```

### 网络问题

```
error: failed to download dependencies
```

**解决**:
```bash
# 使用国内镜像
cat >> ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
EOF

# 重试
cargo build --release
```

## 🔗 相关文档

- [安装指南](/installation/)
- [开发指南](/development/)
- [构建系统](/development/build)
- [故障排除](/installation/troubleshooting)

