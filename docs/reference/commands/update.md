# update - 更新 CCR

从 GitHub 自动更新 CCR 到最新版本,支持实时进度显示。

## 用法

```bash
ccr update [OPTIONS] [BRANCH]
```

## 参数

- `[BRANCH]`: 指定更新的分支（默认: main，可选: dev）

## 选项

- `--check, -c`: 仅检查更新模式,预览更新命令但不执行实际更新

## 功能特性

- ✨ **实时进度显示** - 显示 cargo 编译和下载的实时进度
- 🚀 **自动更新** - 使用 `cargo install --git` 直接从 GitHub 更新
- 🔒 **更新前确认** - 显示版本信息并等待用户确认
- 📋 **详细信息** - 显示当前版本、仓库地址和执行命令
- 🛡️ **错误诊断** - 提供详细的错误原因和解决方案
- 📝 **后续指引** - 更新完成后提供验证和查看新功能的步骤

## 系统要求

- 已安装 Rust 和 Cargo 工具链
- 可访问 GitHub (https://github.com)
- 网络连接稳定

## 等效命令

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr --force
```

> 如果你在本地 checkout 的重构后工作区中手动安装，请使用 `cargo install --path crates/ccr`。

## 示例

### 1. 检查更新(推荐先使用)

```bash
# 查看更新信息但不执行更新
ccr update --check

# 简写形式
ccr update -c
```

### 2. 执行更新

```bash
# 更新到最新版本
ccr update
```

### 3. 从 dev 分支更新

```bash
# 从 dev 分支更新（获取最新开发功能）
ccr update dev

# 先检查 dev 分支更新
ccr update --check dev
```

> [!WARNING]
> **开发分支注意事项**
> - dev 分支包含最新的开发功能，但可能不够稳定
> - 建议仅在测试环境或需要最新功能时使用
> - 生产环境建议使用默认的 main 分支

## 示例输出

### 检查更新模式

```bash
$ ccr update --check

CCR 自动更新
════════

  当前版本: 3.6.2
  仓库地址: https://github.com/bahayonghang/ccr
  更新分支: main

────────────────────────────────────────────────────────────

ℹ 检查模式 - 不会执行实际更新

▶ 更新命令预览
  cargo install --git https://github.com/bahayonghang/ccr ccr --branch main --force

ℹ 💡 提示: 运行 'ccr update' 执行更新(去掉 --check 参数)
```

### 执行更新

```bash
$ ccr update

CCR 自动更新
════════

  当前版本: 0.3.2
  仓库地址: https://github.com/bahayonghang/ccr

? 确认更新到最新版本? [Y/n]: y

────────────────────────────────────────────────────────────

▶ 开始更新 CCR

ℹ 执行命令:
  cargo install --git https://github.com/bahayonghang/ccr ccr --force

────────────────────────────────────────────────────────────

    Updating git repository `https://github.com/bahayonghang/ccr`
   Compiling proc-macro2 v1.0.86
   Compiling unicode-ident v1.0.13
   Compiling libc v0.2.168
   ... (实时显示编译进度)
   Compiling ccr v0.3.2 (https://github.com/bahayonghang/ccr)
    Finished `release` profile [optimized] target(s) in 1m 23s
  Installing /home/user/.cargo/bin/ccr
   Installed package `ccr v0.3.2` (executable `ccr`)

────────────────────────────────────────────────────────────

🎉 更新成功完成

ℹ 后续步骤:
  1. 运行 'ccr version' 查看新版本信息
  2. 运行 'ccr --help' 查看新功能
```

### 取消更新

```bash
$ ccr update

CCR 自动更新
════════

  当前版本: 0.3.2
  仓库地址: https://github.com/bahayonghang/ccr

? 确认更新到最新版本? [Y/n]: n

ℹ 已取消更新
```

## 更新过程详解

### 1. 显示版本信息

显示当前版本和仓库地址,让用户了解更新来源。

### 2. 用户确认

除非使用 `--check` 模式,否则会询问用户是否继续更新。

### 3. 实时执行更新

执行 cargo install 并实时显示：
- Git 克隆进度
- 依赖下载进度
- 编译进度(每个 crate)
- 安装进度

### 4. 显示结果

更新完成后显示成功消息和后续操作建议。

## 使用场景

### 场景 1: 定期检查新功能

```bash
# 每周检查是否有更新
ccr update --check

# 如果有新功能,执行更新
ccr update
```

### 场景 2: 获取 Bug 修复

```bash
# 直接更新到最新版本
ccr update

# 验证修复效果
ccr validate
```

### 场景 3: 更新前备份

```bash
# 导出当前配置
ccr export -o backup-before-update.toml

# 执行更新
ccr update

# 验证配置
ccr validate
```

### 场景 4: 自动化脚本

```bash
#!/bin/bash
# 自动更新检查脚本

echo "检查 CCR 更新..."
ccr update --check

if [ $? -eq 0 ]; then
    read -p "发现可用更新,是否立即更新? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        ccr export -o ~/backups/ccr-$(date +%Y%m%d).toml
        ccr update
    fi
fi
```

## 故障排除

### 问题 1: Cargo 未安装

**错误信息：**
```
无法启动 cargo 命令: No such file or directory

可能原因：
  • 未安装 Rust 工具链
  • cargo 不在系统 PATH 中
```

**解决方案：**
```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境
source $HOME/.cargo/env

# 验证安装
cargo --version

# 重新运行更新
ccr update
```

### 问题 2: 网络连接失败

**错误信息：**
```
❌ 更新失败

ℹ 可能的原因:
  • 网络连接问题(无法访问 GitHub)
  • Git 未安装或配置不正确
```

**解决方案：**
```bash
# 1. 检查网络连接
ping github.com

# 2. 检查 Git 是否安装
git --version

# 3. 配置代理(如需要)
export https_proxy=http://proxy.example.com:8080
export http_proxy=http://proxy.example.com:8080

# 4. 重试更新
ccr update

# 5. 或手动克隆并安装
git clone https://github.com/bahayonghang/ccr
cd ccr
cargo install --path crates/ccr --locked
```

### 问题 3: 编译失败

**错误信息：**
```
error: failed to compile ccr v0.3.2
```

**解决方案：**
```bash
# 1. 更新 Rust 工具链
rustup update stable

# 2. 检查 Rust 版本(需要较新版本)
rustc --version

# 3. 清理缓存
cargo clean

# 4. 重试更新
ccr update

# 5. 如仍失败,尝试手动编译
git clone https://github.com/bahayonghang/ccr
cd ccr
cargo build --release --locked
cargo install --path crates/ccr
```

### 问题 4: 权限错误

**错误信息：**
```
error: failed to install

ℹ 可能的原因:
  • 权限不足(无法写入 ~/.cargo/bin)
```

**解决方案：**
```bash
# 1. 检查目录权限
ls -la ~/.cargo/bin

# 2. 修复权限
chmod 755 ~/.cargo/bin
chmod 755 ~/.cargo/bin/ccr 2>/dev/null || true

# 3. 重试更新
ccr update
```

### 问题 5: 旧版本残留

**症状：** 更新后运行仍显示旧版本

**解决方案：**
```bash
# 1. 检查 ccr 位置
which ccr

# 2. 检查是否有多个版本
find ~ -name ccr -type f 2>/dev/null

# 3. 重新安装
cargo install --git https://github.com/bahayonghang/ccr ccr --force

# 4. 刷新 shell
hash -r

# 5. 验证版本
ccr version
```

## 版本回退

如果更新后遇到问题,可以回退到之前的版本。

### 方法 1: 安装特定版本标签

```bash
# 列出所有可用版本
git ls-remote --tags https://github.com/bahayonghang/ccr

# 安装特定版本
cargo install --git https://github.com/bahayonghang/ccr ccr --tag v0.3.1 --force
```

### 方法 2: 安装特定 Commit

```bash
# 安装指定 commit
cargo install --git https://github.com/bahayonghang/ccr ccr --rev abc123def --force
```

### 方法 3: 从备份恢复

```bash
# 如果之前备份了二进制文件
cp ~/.cargo/bin/ccr.backup ~/.cargo/bin/ccr
chmod 755 ~/.cargo/bin/ccr

# 验证版本
ccr version
```

## 更新后验证

### 1. 检查版本

```bash
ccr version
```

### 2. 验证核心功能

```bash
# 列出配置
ccr list

# 验证配置
ccr validate

# 查看历史
ccr history -l 5
```

### 3. 查看新功能

```bash
# 查看帮助
ccr --help

# 查看各命令帮助
ccr <command> --help
```

### 4. 测试配置切换

```bash
# 记录当前配置
CURRENT=$(ccr current | grep "当前配置" | awk '{print $2}')

# 切换测试
ccr list
ccr switch <test-config>

# 切换回来
ccr switch $CURRENT
```

## 最佳实践

### 1. 更新前备份

```bash
# 导出配置
ccr export -o ~/backups/ccr-backup-$(date +%Y%m%d_%H%M%S).toml

# 备份二进制(可选)
cp ~/.cargo/bin/ccr ~/.cargo/bin/ccr.$(ccr version | head -1 | awk '{print $2}')

# 执行更新
ccr update
```

### 2. 定期检查更新

```bash
# 添加到 crontab(每周一上午 9 点检查)
0 9 * * 1 /home/user/.cargo/bin/ccr update --check > /tmp/ccr-update-check.log
```

### 3. 测试环境先更新

```bash
# 开发/测试环境
ssh dev-server "ccr update"
ssh dev-server "ccr validate"

# 验证成功后更新生产环境
ccr update
```

### 4. 保持工具链最新

```bash
# 更新 Rust 工具链
rustup update

# 更新 CCR
ccr update
```

### 5. 记录更新历史

```bash
# 创建更新日志
echo "$(date): Updated CCR from $(ccr version) to latest" >> ~/ccr-updates.log

# 更新
ccr update

# 记录新版本
echo "$(date): Now running $(ccr version)" >> ~/ccr-updates.log
```

## 相关命令

- [version](./version) - 查看版本信息和功能特性
- [validate](./validate) - 验证配置完整性
- [export](./export) - 导出配置备份
- [init](./init) - 初始化配置文件

## 技术细节

### 更新机制

CCR 使用 `cargo install --git` 机制：

1. 从 GitHub 克隆最新代码
2. 编译 release 版本(带优化)
3. 安装到 `~/.cargo/bin/ccr`
4. 替换旧版本

### 进度显示实现

使用 `Stdio::inherit()` 实现实时输出：

```rust
Command::new("cargo")
    .stdout(Stdio::inherit())  // 实时显示标准输出
    .stderr(Stdio::inherit())  // 实时显示标准错误
    .spawn()?
    .wait()?
```

### 版本识别

当前版本在编译时嵌入：

```rust
env!("CARGO_PKG_VERSION")  // 从 Cargo.toml 读取
```

## 安全注意事项

1. **仅从官方仓库更新** - 确认 URL 为 `https://github.com/bahayonghang/ccr`
2. **检查网络环境** - 避免在不信任的网络环境更新
3. **备份配置** - 更新前导出配置以防万一
4. **验证安装** - 更新后运行 `ccr validate` 确认功能正常

## 常见问题 (FAQ)

**Q: 更新会丢失配置吗？**
A: 不会。配置存储在 `~/.ccs_config.toml` 和 `~/.claude/settings.json`,更新仅替换可执行文件。

**Q: 需要多长时间？**
A: 通常 1-3 分钟,取决于网络速度和机器性能。

**Q: 可以跳过确认吗？**
A: 当前不支持。为了安全,总是需要用户确认。

**Q: 可以自动更新吗？**
A: 不建议完全自动化。可以自动检查(`--check`),但执行更新建议手动确认。

**Q: 更新失败会影响当前版本吗？**
A: 不会。只有成功编译后才会替换旧版本。

**Q: 能看到详细的编译输出吗？**
A: 可以。更新时会实时显示所有 cargo 的输出信息。
