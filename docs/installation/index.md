# 安装指南

CCR 是一个 Rust 编写的命令行工具，需要从源码构建。本指南将引导你完成安装过程。

## 📋 系统要求

### 必需组件

- **Rust 工具链** 1.70.0 或更高版本
- **Cargo** (随 Rust 一起安装)
- **Git** (用于克隆仓库)

### 操作系统支持

- ✅ **Linux** (Ubuntu, Debian, Fedora, Arch, 等)
- ✅ **macOS** (10.15 Catalina 或更高)
- ✅ **Windows** (10/11, WSL 推荐)

## 🦀 安装 Rust

如果你还没有安装 Rust，可以使用 rustup：

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows

下载并运行 [rustup-init.exe](https://rustup.rs/)

### 验证安装

```bash
rustc --version
cargo --version
```

应该看到类似输出：
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

## 📦 从源码构建 CCR

### 1. 克隆仓库

```bash
# 克隆主仓库
git clone https://github.com/bahayonghang/ccs.git
cd ccs/ccr
```

### 2. 构建项目

#### 开发版本（带调试信息）

```bash
cargo build
```

构建产物位于 `target/debug/ccr`

#### 发布版本（优化性能）

```bash
cargo build --release
```

构建产物位于 `target/release/ccr`

**推荐使用发布版本**，性能更好，二进制文件更小。

### 3. 安装到系统

#### 方式 A: 使用 Cargo Install（推荐）

```bash
cargo install --path . --locked
```

这会将 CCR 安装到 `~/.cargo/bin/ccr`

#### 方式 B: 手动复制

```bash
# Linux / macOS
sudo cp target/release/ccr /usr/local/bin/

# 或复制到用户目录
mkdir -p ~/.local/bin
cp target/release/ccr ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

### 4. 验证安装

```bash
ccr --version
```

应该看到：
```
ccr 0.2.0
```

## 🔧 配置 CCR

### 创建配置文件

CCR 使用 TOML 格式的配置文件，位于 `~/.ccs_config.toml`。

#### 手动创建

```bash
cat > ~/.ccs_config.toml << 'EOF'
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key-here"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter 代理服务"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token-here"
model = "claude-sonnet-4-5-20250929"
EOF
```

#### 从 CCS 迁移

如果你已经安装了 Shell 版本的 CCS，可以直接使用现有配置：

```bash
# CCS 和 CCR 共享同一个配置文件
ls -la ~/.ccs_config.toml
```

### 配置文件格式

```toml
# 全局设置
default_config = "anthropic"  # 默认配置
current_config = "anthropic"  # 当前活跃配置

# 配置节 1
[配置名称]
description = "描述信息（可选）"
base_url = "https://api.example.com"  # 必填
auth_token = "your-api-token"         # 必填
model = "model-name"                  # 可选
small_fast_model = "small-model"      # 可选

# 配置节 2
[另一个配置]
base_url = "https://api.another.com"
auth_token = "another-token"
```

### 字段说明

| 字段 | 必填 | 说明 |
|------|------|------|
| `description` | ❌ | 配置的描述信息 |
| `base_url` | ✅ | API 端点 URL（必须以 http:// 或 https:// 开头）|
| `auth_token` | ✅ | API 认证令牌 |
| `model` | ❌ | 默认使用的模型名称 |
| `small_fast_model` | ❌ | 快速小模型名称（用于轻量级任务）|

## 🚀 快速测试

### 1. 验证配置

```bash
ccr validate
```

### 2. 列出配置

```bash
ccr list
```

输出示例：
```
可用配置列表
════════════════════════════════════════════════════════════════
配置文件: /home/user/.ccs_config.toml
默认配置: anthropic
当前配置: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic 官方 API
    Base URL: https://api.anthropic.com
    Token: sk-a...here
    Model: claude-sonnet-4-5-20250929
    状态: ✓ 配置完整
  anyrouter - AnyRouter 代理服务

✓ 共找到 2 个配置
```

### 3. 查看当前状态

```bash
ccr current
```

### 4. 切换配置

```bash
ccr switch anyrouter
```

## 🛠️ 高级配置

### Shell 集成

为了在新终端中自动加载配置，你可以添加以下内容到 shell 配置文件：

#### Bash / Zsh

```bash
# ~/.bashrc 或 ~/.zshrc
if command -v ccr &> /dev/null; then
    alias ccs='ccr'  # 兼容 CCS 命令
fi
```

#### Fish

```fish
# ~/.config/fish/config.fish
if type -q ccr
    alias ccs='ccr'  # 兼容 CCS 命令
end
```

### 命令别名

```bash
# 快捷命令别名
alias clist='ccr list'
alias cswitch='ccr switch'
alias cstat='ccr current'
alias chist='ccr history'
```

## 🔄 更新 CCR

### 从源码更新

```bash
cd ccs/ccr

# 拉取最新代码
git pull

# 重新构建和安装
cargo install --path . --locked --force
```

### 使用 Just（如果安装了 just）

```bash
# 重新安装
just reinstall
```

## 🗑️ 卸载 CCR

### 卸载程序

```bash
# 如果使用 cargo install 安装的
cargo uninstall ccr

# 如果手动复制的
sudo rm /usr/local/bin/ccr
# 或
rm ~/.local/bin/ccr
```

### 清理配置文件（可选）

```bash
# 备份配置
cp ~/.ccs_config.toml ~/.ccs_config.toml.backup

# 删除配置（谨慎操作）
rm ~/.ccs_config.toml

# 删除 Claude Code 相关文件
rm ~/.claude/settings.json
rm ~/.claude/ccr_history.json
rm -rf ~/.claude/backups/
rm -rf ~/.claude/.locks/
```

## 🔗 相关文档

- [配置文件详解](/installation/configuration)
- [环境变量说明](/installation/environment)
- [故障排除](/installation/troubleshooting)
- [从源码构建](/installation/build-from-source)

