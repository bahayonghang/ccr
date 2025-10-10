# update - 更新 CCR

检查并更新 CCR 到最新版本。

## 用法

```bash
ccr update [OPTIONS]
```

## 选项

- `--check`: 仅检查更新，不执行更新

## 功能特性

- 从 GitHub 获取最新代码
- 使用 `cargo install --git` 更新
- 总是获取最新的 commit
- 需要 Rust 工具链（cargo）
- 更新前自动确认

## 要求

- 已安装 Rust 和 Cargo
- 可访问 GitHub 网络

## 等效命令

```bash
cargo install --git https://github.com/bahayonghang/ccr --force
```

## 示例

```bash
# 检查可用更新
ccr update --check

# 更新到最新版本
ccr update
```

## 示例输出

### 检查更新

```bash
$ ccr update --check
Checking for updates...
────────────────────────────────────────────────────────────────
Current version: 0.3.1
Latest version: 0.4.0

✓ New version available!

Changes in v0.4.0:
  - Added configuration validation improvements
  - Enhanced backup management
  - Performance optimizations
  - Bug fixes

To update, run: ccr update
```

### 执行更新

```bash
$ ccr update
Updating CCR...
────────────────────────────────────────────────────────────────
Current version: 0.3.1
Latest version: 0.4.0

This will update CCR to the latest version from GitHub.
Continue? (y/N): y

Downloading latest version...
Compiling CCR...
   Compiling ccr v0.4.0
    Finished release [optimized] target(s) in 45.23s
  Installing ~/.cargo/bin/ccr

✓ CCR updated successfully!
✓ New version: 0.4.0

Run 'ccr version' to verify the update.
```

### 已是最新版本

```bash
$ ccr update --check
Checking for updates...
────────────────────────────────────────────────────────────────
Current version: 0.4.0

✓ You are already using the latest version!
```

## 更新过程

### 1. 检查当前版本

```bash
ccr version
```

### 2. 检查远程版本

从 GitHub 获取最新的版本信息。

### 3. 确认更新

显示版本变化和更新日志，等待用户确认。

### 4. 下载并编译

```bash
cargo install --git https://github.com/bahayonghang/ccr --force
```

### 5. 验证安装

检查新版本是否正确安装。

## 使用场景

### 1. 定期更新

定期检查并更新到最新版本：

```bash
# 每周检查更新
ccr update --check

# 发现新版本后更新
ccr update
```

### 2. 获取新功能

更新以使用新功能：

```bash
# 查看更新日志
ccr update --check

# 更新
ccr update
```

### 3. 修复 Bug

更新以修复已知问题：

```bash
# 检查是否有修复
ccr update --check

# 应用修复
ccr update
```

### 4. 自动化更新检查

```bash
# 添加到 crontab（每周检查）
0 9 * * 1 ccr update --check | mail -s "CCR Update Available" user@example.com
```

## 更新策略

### 保守策略

仅在必要时更新：

```bash
# 仅检查，不自动更新
ccr update --check

# 阅读更新日志后决定
ccr update
```

### 积极策略

总是使用最新版本：

```bash
# 自动更新脚本
#!/bin/bash
ccr update --check | grep -q "New version available" && ccr update
```

### 测试策略

在测试环境先更新：

```bash
# 测试环境更新
ssh test-server "ccr update"

# 验证功能
ssh test-server "ccr validate && ccr list"

# 生产环境更新
ccr update
```

## 故障排除

### 问题：Cargo 未安装

```bash
$ ccr update
Error: cargo command not found
Please install Rust: https://rustup.rs/
```

**解决：**
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新运行更新
ccr update
```

### 问题：网络连接失败

```bash
$ ccr update
Error: Failed to download from GitHub
Check your network connection and try again
```

**解决：**
```bash
# 检查网络
ping github.com

# 使用代理
export https_proxy=http://proxy:port
ccr update

# 或手动下载并安装
git clone https://github.com/bahayonghang/ccr
cd ccr
cargo install --path .
```

### 问题：编译失败

```bash
$ ccr update
Error: Compilation failed
```

**解决：**
```bash
# 更新 Rust 工具链
rustup update

# 清理并重试
cargo clean
ccr update

# 或手动编译
git clone https://github.com/bahayonghang/ccr
cd ccr
cargo build --release
cargo install --path .
```

### 问题：权限错误

```bash
$ ccr update
Error: Permission denied
```

**解决：**
```bash
# 检查 ~/.cargo/bin 权限
ls -la ~/.cargo/bin/ccr

# 修复权限
chmod 755 ~/.cargo/bin/ccr
```

## 回退到旧版本

如果新版本有问题，可以回退：

### 方法 1：安装特定版本

```bash
# 安装特定 tag/commit
cargo install --git https://github.com/bahayonghang/ccr --tag v0.3.1

# 或指定 commit
cargo install --git https://github.com/bahayonghang/ccr --rev abc123def
```

### 方法 2：从本地构建

```bash
# 克隆并切换到特定版本
git clone https://github.com/bahayonghang/ccr
cd ccr
git checkout v0.3.1

# 构建并安装
cargo install --path .
```

## 更新后操作

### 验证安装

```bash
# 检查版本
ccr version

# 验证功能
ccr list
ccr validate
```

### 查看新功能

```bash
# 查看帮助
ccr --help

# 查看更新日志
cat CHANGELOG.md
```

### 测试核心功能

```bash
# 测试列表
ccr list

# 测试切换
ccr switch test-config && ccr switch back

# 测试验证
ccr validate
```

## 版本管理最佳实践

### 1. 备份配置

更新前备份配置：

```bash
ccr export -o before-update-$(date +%Y%m%d).toml
ccr update
```

### 2. 阅读更新日志

更新前了解变化：

```bash
ccr update --check
# 访问 GitHub 查看详细更新日志
```

### 3. 测试环境优先

先在测试环境更新：

```bash
# 测试环境
ccr update
# 验证功能
ccr validate

# 生产环境
ccr update
```

### 4. 记录版本

记录使用的版本：

```bash
ccr version > ~/ccr-version.txt
date >> ~/ccr-version.txt
```

### 5. 保留旧版本二进制

```bash
# 备份当前版本
cp ~/.cargo/bin/ccr ~/.cargo/bin/ccr.backup

# 更新
ccr update

# 如需回退
cp ~/.cargo/bin/ccr.backup ~/.cargo/bin/ccr
```

## 自动化更新

### CI/CD 集成

```yaml
# .github/workflows/update-ccr.yml
name: Update CCR
on:
  schedule:
    - cron: '0 0 * * 0'  # 每周日
jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Update CCR
        run: |
          ccr update --check
          ccr update
      - name: Verify
        run: ccr version
```

### 脚本自动更新

```bash
#!/bin/bash
# auto-update-ccr.sh

# 检查更新
if ccr update --check | grep -q "New version available"; then
  echo "New CCR version found, updating..."

  # 备份配置
  ccr export -o ~/backups/ccr-pre-update.toml

  # 更新
  ccr update

  # 验证
  if ccr validate; then
    echo "Update successful!"
  else
    echo "Update validation failed!"
    exit 1
  fi
fi
```

## 相关命令

- [version](./version) - 查看当前版本
- [validate](./validate) - 验证更新后的功能
- [export](./export) - 更新前备份配置
