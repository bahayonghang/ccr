# 迁移指南

本指南帮助你从 CCS (Shell 版本) 迁移到 CCR (Rust 版本)，或在两者之间切换使用。

## 🔄 从 CCS 迁移到 CCR

### 为什么迁移？

CCR 相比 CCS 提供了以下优势：

✅ **直接生效**: 配置立即生效，无需 shell 重启  
✅ **并发安全**: 文件锁保护，支持多进程使用  
✅ **完整追踪**: 记录所有操作历史  
✅ **自动备份**: 配置切换前自动备份  
✅ **Web 界面**: 现代化的可视化管理  
✅ **更快速度**: Rust 性能优势  

### 迁移步骤

#### 1. 保持配置文件

好消息！CCR 和 CCS 共享同一个配置文件 `~/.ccs_config.toml`，无需任何转换：

```bash
# 检查现有配置
cat ~/.ccs_config.toml
```

配置格式完全相同：

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
```

#### 2. 安装 CCR

```bash
# 克隆仓库（如果还没有）
cd ccs/ccr

# 构建并安装
cargo build --release
cargo install --path . --locked
```

#### 3. 验证安装

```bash
# 检查版本
ccr --version

# 列出配置（应该看到与 ccs list 相同的输出）
ccr list

# 查看当前状态
ccr current
```

#### 4. 第一次切换

```bash
# 使用 CCR 切换配置
ccr switch anyrouter

# CCR 会：
# 1. 验证目标配置
# 2. 备份当前 settings.json
# 3. 直接写入 ~/.claude/settings.json
# 4. 更新 current_config
# 5. 记录操作历史
```

#### 5. 验证效果

```bash
# 查看 Claude Code 设置
cat ~/.claude/settings.json

# 应该看到环境变量已更新：
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.anyrouter.ai/v1",
    "ANTHROPIC_AUTH_TOKEN": "your-token",
    ...
  }
}
```

### 共存使用

CCR 和 CCS 可以完全共存：

```bash
# 使用 CCS（Shell 版本）
ccs list
ccs switch anthropic

# 使用 CCR（Rust 版本）
ccr list
ccr switch anyrouter

# 两者共享配置文件，可以自由切换
```

## 🔀 配置差异

### 工作原理对比

**CCS (Shell 版本)**:
```bash
# 1. 读取配置
parse_toml "anthropic"

# 2. 设置环境变量
export ANTHROPIC_BASE_URL="..."
export ANTHROPIC_AUTH_TOKEN="..."

# 3. 更新 current_config
# 需要重启 shell 或重新加载才能生效
```

**CCR (Rust 版本)**:
```rust
// 1. 读取配置
let config = config_manager.load()?;

// 2. 直接写入 settings.json
settings_manager.save_atomic(&settings)?;

// 3. 配置立即生效
// Claude Code 直接读取 settings.json，无需重启
```

### 命令对比

大多数命令保持一致：

| 功能 | CCS | CCR |
|------|-----|-----|
| 列出配置 | `ccs list` | `ccr list` |
| 查看当前 | `ccs current` | `ccr current` |
| 切换配置 | `ccs anyrouter` | `ccr anyrouter` |
| 验证配置 | `ccs validate` | `ccr validate` |
| 查看版本 | `ccs version` | `ccr version` |
| Web 界面 | `ccs web` | `ccr web` |
| 查看历史 | ❌ | `ccr history` ✅ |

### 新增功能

CCR 独有的功能：

```bash
# 查看操作历史
ccr history

# 限制显示数量
ccr history --limit 50

# 按类型筛选
ccr history --filter-type switch

# 更详细的验证报告
ccr validate
```

## 🎯 使用场景建议

### 推荐使用 CCR 的场景

1. **需要立即生效**: 不想重启 shell
2. **多进程环境**: 需要并发安全保证
3. **审计追踪**: 需要完整的操作历史
4. **Web 管理**: 喜欢可视化界面
5. **高频切换**: 经常切换不同配置

### 可以继续使用 CCS 的场景

1. **轻量级使用**: 偶尔切换配置
2. **纯 Shell 环境**: 不想安装 Rust
3. **兼容性**: 需要特定的 Shell 集成

### 混合使用

两者可以无缝切换：

```bash
# 在服务器上使用 CCS（轻量级）
ssh server
ccs switch production

# 在本地使用 CCR（功能完整）
ccr list
ccr web        # 使用 Web 界面管理
ccr history    # 查看历史
```

## 📁 文件位置变化

### CCS 文件

```
~/.ccs_config.toml           # 配置文件（共享）
~/.bashrc, ~/.zshrc          # Shell 配置
~/.ccs/                      # 脚本目录
```

### CCR 新增文件

```
~/.ccs_config.toml           # 配置文件（共享）
~/.claude/settings.json      # 直接写入
~/.claude/ccr_history.json   # 历史记录
~/.claude/backups/           # 备份目录
~/.claude/.locks/            # 锁文件
~/.cargo/bin/ccr            # CCR 可执行文件
```

## 🔧 故障排除

### 问题 1: CCR 和 CCS 冲突

**症状**: 同时使用 CCR 和 CCS 时配置不一致

**原因**: CCS 设置的环境变量会覆盖 settings.json

**解决**:
```bash
# 清除 CCS 设置的环境变量
unset ANTHROPIC_BASE_URL
unset ANTHROPIC_AUTH_TOKEN
unset ANTHROPIC_MODEL
unset ANTHROPIC_SMALL_FAST_MODEL

# 或者重启 shell
```

### 问题 2: 配置不生效

**症状**: CCR 切换配置后 Claude Code 仍使用旧配置

**解决**:
```bash
# 1. 验证 settings.json
cat ~/.claude/settings.json

# 2. 重启 Claude Code
# 3. 使用 ccr validate 检查配置
ccr validate
```

### 问题 3: 权限错误

**症状**: `Permission denied` 错误

**解决**:
```bash
# 检查文件权限
ls -la ~/.claude/settings.json
ls -la ~/.ccs_config.toml

# 修复权限
chmod 600 ~/.claude/settings.json
chmod 644 ~/.ccs_config.toml
```

## 📝 回退到 CCS

如果需要回退到纯 CCS 环境：

```bash
# 1. 卸载 CCR
cargo uninstall ccr

# 2. 清理 CCR 文件（可选）
rm ~/.claude/ccr_history.json
rm -rf ~/.claude/backups/
rm -rf ~/.claude/.locks/

# 3. 使用 CCS 重新设置配置
ccs switch anthropic

# 4. 重新加载 shell 配置
source ~/.bashrc  # 或 ~/.zshrc
```

**注意**: `~/.ccs_config.toml` 配置文件会保留，可以继续使用。

## 🎉 迁移完成

恭喜！你已经成功迁移到 CCR。现在你可以享受：

- ⚡ 更快的性能
- 🔒 更好的安全性
- 📝 完整的审计追踪
- 🌐 Web 管理界面
- 💾 自动备份功能

## 🔗 相关资源

- [安装指南](/installation/)
- [命令参考](/commands/)
- [常见问题](/installation/troubleshooting)
- [GitHub Issues](https://github.com/bahayonghang/ccs/issues)

