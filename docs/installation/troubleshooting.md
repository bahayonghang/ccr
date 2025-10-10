# 故障排除

本文档收录了 CCR 使用过程中的常见问题和解决方案。

## 🔧 安装问题

### 问题：Rust 未安装

**症状**:
```bash
$ cargo build
bash: cargo: command not found
```

**解决方案**:
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 刷新 PATH
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

---

### 问题：编译失败

**症状**:
```bash
$ cargo build
error: failed to compile ccr v0.2.0
```

**解决方案**:

**1. 检查 Rust 版本**
```bash
rustc --version
# 需要 1.70.0 或更高

# 更新 Rust
rustup update
```

**2. 清理构建缓存**
```bash
cargo clean
cargo build
```

**3. 检查依赖**
```bash
cargo update
cargo build
```

---

### 问题：安装权限不足

**症状**:
```bash
$ cargo install --path .
error: permission denied
```

**解决方案**:
```bash
# 方式 1: 安装到用户目录（推荐）
cargo install --path .
# 会安装到 ~/.cargo/bin/

# 方式 2: 使用 sudo（不推荐）
sudo cargo install --path . --root /usr/local

# 方式 3: 手动复制
cargo build --release
mkdir -p ~/.local/bin
cp target/release/ccr ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

## 📝 配置问题

### 问题：配置文件不存在

**症状**:
```bash
$ ccr list

✗ 配置文件不存在: /home/user/.ccs_config.toml
  建议: 请运行安装脚本创建配置文件
```

**退出码**: 11

**解决方案**:

**方式 1: 手动创建**
```bash
cat > ~/.ccs_config.toml << 'EOF'
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "YOUR_API_KEY_HERE"
model = "claude-sonnet-4-5-20250929"
EOF
```

**方式 2: 从 CCS 安装**
```bash
cd ccs
./scripts/install/install.sh
```

**方式 3: 复制示例文件**
```bash
cp ccs/config/.ccs_config.toml.example ~/.ccs_config.toml
vim ~/.ccs_config.toml  # 修改 API key
```

---

### 问题：配置节不存在

**症状**:
```bash
$ ccr switch myconfig

✗ 配置节 'myconfig' 不存在
  建议: 运行 'ccr list' 查看可用配置
```

**退出码**: 12

**解决方案**:
```bash
# 1. 列出所有配置
ccr list

# 2. 检查配置文件
cat ~/.ccs_config.toml

# 3. 添加配置节
vim ~/.ccs_config.toml
```

添加：
```toml
[myconfig]
base_url = "https://api.example.com"
auth_token = "your-token"
```

---

### 问题：配置验证失败

**症状**:
```bash
$ ccr switch broken-config

✗ 验证失败: base_url 不能为空
```

**退出码**: 90

**解决方案**:
```bash
# 1. 运行完整验证
ccr validate

# 2. 查看详细错误
# 验证报告会显示哪个配置有问题

# 3. 修复配置
vim ~/.ccs_config.toml
```

**常见验证错误**:

| 错误信息 | 原因 | 修复 |
|---------|------|------|
| `base_url 不能为空` | 缺少 base_url | 添加 `base_url = "..."` |
| `base_url 必须以 http:// 开头` | URL 格式错误 | 添加协议前缀 |
| `auth_token 不能为空` | 缺少 auth_token | 添加 `auth_token = "..."` |

---

### 问题：TOML 格式错误

**症状**:
```bash
$ ccr list

✗ 配置格式无效: TOML 解析失败: unexpected character
```

**退出码**: 14

**解决方案**:

**1. 检查语法**
```bash
# 查看配置文件
cat ~/.ccs_config.toml

# 常见错误：
# - 缺少引号
# - 错误的缩进
# - 重复的键
```

**2. 使用 TOML 验证工具**
```bash
# 在线验证：https://www.toml-lint.com/

# 或使用 Python
python3 -c "import toml; toml.load(open('$HOME/.ccs_config.toml'))"
```

**3. 重新创建配置**
```bash
# 备份旧配置
mv ~/.ccs_config.toml ~/.ccs_config.toml.broken

# 创建新配置
ccr list  # 会提示不存在
# 然后手动创建
```

## 🔒 权限问题

### 问题：权限拒绝

**症状**:
```bash
$ ccr switch anthropic

✗ 权限拒绝: /home/user/.claude/settings.json
```

**退出码**: 70

**解决方案**:

**1. 检查文件权限**
```bash
ls -la ~/.claude/settings.json
ls -la ~/.ccs_config.toml
```

**2. 修复权限**
```bash
# 设置文件
chmod 600 ~/.claude/settings.json

# 配置文件
chmod 644 ~/.ccs_config.toml

# .claude 目录
chmod 755 ~/.claude
```

**3. 检查所有者**
```bash
ls -la ~/.claude/
# 应该是当前用户

# 如果不是，修复所有者
sudo chown -R $USER:$USER ~/.claude/
```

---

### 问题：无法创建目录

**症状**:
```bash
✗ 设置文件错误: 创建设置目录失败: Permission denied
```

**解决方案**:
```bash
# 检查父目录权限
ls -la ~/

# 手动创建目录
mkdir -p ~/.claude
mkdir -p ~/.claude/backups
mkdir -p ~/.claude/.locks

# 设置权限
chmod 755 ~/.claude
chmod 755 ~/.claude/backups
chmod 755 ~/.claude/.locks
```

## 🔐 文件锁问题

### 问题：文件锁超时

**症状**:
```bash
$ ccr switch anthropic

✗ 获取文件锁超时: claude_settings
  建议: 可能有其他 ccr 进程正在运行
```

**退出码**: 31

**解决方案**:

**1. 检查 CCR 进程**
```bash
ps aux | grep ccr

# 如果发现僵死进程
kill <PID>
```

**2. 清理锁文件**
```bash
# 检查锁文件
ls -la ~/.claude/.locks/

# 如果确认没有其他进程，清理锁
rm -rf ~/.claude/.locks/*
```

**3. 检查文件系统**
```bash
# 检查磁盘空间
df -h ~

# 检查 inode
df -i ~
```

---

### 问题：锁文件无法删除

**症状**:
```bash
$ rm ~/.claude/.locks/*
rm: cannot remove: Permission denied
```

**解决方案**:
```bash
# 检查锁文件权限
ls -la ~/.claude/.locks/

# 强制删除
sudo rm -rf ~/.claude/.locks/*

# 重新创建目录
mkdir -p ~/.claude/.locks
chmod 755 ~/.claude/.locks
```

## 🌐 Web 服务器问题

### 问题：端口被占用

**症状**:
```bash
$ ccr web

✗ 无法启动 HTTP 服务器: Address already in use
```

**解决方案**:

**1. 检查端口占用**
```bash
# Linux
sudo netstat -tlnp | grep 8080
sudo lsof -i :8080

# macOS
lsof -i :8080
```

**2. 使用其他端口**
```bash
ccr web --port 3000
ccr web -p 9090
```

**3. 停止占用进程**
```bash
# 找到 PID
lsof -i :8080

# 停止进程
kill <PID>

# 或强制停止
kill -9 <PID>
```

---

### 问题：浏览器无法自动打开

**症状**:
```
✓ CCR Web 服务器已启动
⚠ 无法自动打开浏览器: No such file or directory
```

**解决方案**:
```bash
# 手动打开浏览器
# Linux
xdg-open http://localhost:8080

# macOS
open http://localhost:8080

# Windows (WSL)
explorer.exe http://localhost:8080

# 或直接在浏览器中输入
# http://localhost:8080
```

## 💾 数据问题

### 问题：历史记录损坏

**症状**:
```bash
$ ccr history

✗ 历史记录错误: 解析历史文件失败
```

**退出码**: 80

**解决方案**:

**1. 备份历史文件**
```bash
cp ~/.claude/ccr_history.json ~/.claude/ccr_history.json.backup
```

**2. 检查 JSON 格式**
```bash
# 使用 jq 验证
cat ~/.claude/ccr_history.json | jq .

# 或使用 Python
python3 -c "import json; json.load(open('$HOME/.claude/ccr_history.json'))"
```

**3. 重置历史**
```bash
# 如果无法修复，删除历史文件
rm ~/.claude/ccr_history.json

# CCR 会自动创建新的空历史文件
ccr switch anthropic
ccr history
```

---

### 问题：备份文件太多

**症状**:
```bash
$ ls ~/.claude/backups/ | wc -l
156
```

**解决方案**:
```bash
# 1. 查看备份文件
ls -lht ~/.claude/backups/

# 2. 删除旧备份（保留最近 10 个）
cd ~/.claude/backups/
ls -t | tail -n +11 | xargs rm

# 3. 或者全部删除（谨慎！）
rm ~/.claude/backups/*
```

**自动清理**（未来功能）:
```bash
ccr cleanup --keep-backups 10
```

## 🚀 性能问题

### 问题：启动慢

**症状**:
```bash
$ time ccr list
real    0m2.543s
```

**诊断**:
```bash
# 检查是否使用 debug 版本
which ccr
file $(which ccr)

# Debug 版本会慢很多
```

**解决方案**:
```bash
# 重新构建 release 版本
cargo build --release
cargo install --path . --locked --force

# 验证
time ccr list
# 应该 < 50ms
```

---

### 问题：配置切换慢

**症状**:
```bash
$ time ccr switch anthropic
real    0m0.500s  # 太慢
```

**诊断**:
```bash
# 检查文件系统性能
time ls ~/.claude/
time cat ~/.ccs_config.toml
```

**可能原因**:
- 网络文件系统（NFS）
- 磁盘性能问题
- 文件系统错误

**解决方案**:
```bash
# 检查磁盘健康
sudo smartctl -a /dev/sda

# 检查文件系统
sudo fsck /dev/sda1

# 考虑使用本地文件系统
```

## 🌐 Web 界面问题

### 问题：Web 界面无响应

**症状**:
- 点击按钮无反应
- 配置列表不显示

**解决方案**:

**1. 检查浏览器控制台**
```javascript
// 按 F12 打开开发者工具
// 查看 Console 和 Network 标签
```

**2. 检查 API 连接**
```bash
# 测试 API 端点
curl http://localhost:8080/api/configs

# 应该返回 JSON
```

**3. 重启 Web 服务器**
```bash
# Ctrl+C 停止
# 重新启动
ccr web
```

---

### 问题：CORS 错误

**症状**（在浏览器控制台）:
```
Access to fetch at 'http://localhost:8080/api/configs' from origin 'null' 
has been blocked by CORS policy
```

**说明**: CCR 的 Web 服务器默认允许所有来源，不应该出现 CORS 错误。

**解决方案**:
```bash
# 直接访问服务器地址，不要使用 file:// 协议
# 使用: http://localhost:8080
# 不要: file:///path/to/index.html
```

## 🔄 配置切换问题

### 问题：配置切换后不生效

**症状**:
```bash
$ ccr switch anyrouter
✓ 配置切换成功

# 但 Claude Code 仍使用旧配置
```

**诊断**:
```bash
# 1. 检查 settings.json
cat ~/.claude/settings.json

# 2. 检查环境变量
ccr current

# 3. 验证配置
ccr validate
```

**解决方案**:

**1. 重启 Claude Code**
```bash
# 完全退出 Claude Code
# 然后重新启动
```

**2. 检查环境变量优先级**
```bash
# 如果同时使用了 CCS，可能有环境变量冲突
unset ANTHROPIC_BASE_URL
unset ANTHROPIC_AUTH_TOKEN
unset ANTHROPIC_MODEL
unset ANTHROPIC_SMALL_FAST_MODEL

# 然后重新切换
ccr switch anyrouter
```

**3. 手动验证 settings.json**
```bash
cat ~/.claude/settings.json | jq .env
# 应该看到正确的环境变量
```

---

### 问题：切换时提示验证失败

**症状**:
```bash
$ ccr switch broken-config

✗ 验证失败: base_url 不能为空
```

**解决方案**:
```bash
# 1. 查看配置详情
ccr list

# 2. 编辑配置文件
vim ~/.ccs_config.toml

# 3. 确保必填字段存在
[broken-config]
base_url = "https://api.example.com"  # 必填
auth_token = "your-token"              # 必填

# 4. 验证修复
ccr validate
```

## 📜 历史记录问题

### 问题：历史记录太多

**症状**:
```bash
$ ccr history

⚠ 历史记录较多 (523 条)，建议定期清理
```

**解决方案**:

**方式 1: 手动清理**
```bash
# 1. 备份历史文件
cp ~/.claude/ccr_history.json ~/.claude/ccr_history.json.backup

# 2. 只保留最近的记录
cat ~/.claude/ccr_history.json | jq '.[0:50]' > temp.json
mv temp.json ~/.claude/ccr_history.json
```

**方式 2: 删除历史**
```bash
# 完全清空历史
rm ~/.claude/ccr_history.json

# CCR 会自动创建新的空历史
```

**未来功能**:
```bash
# 自动清理（计划中）
ccr cleanup --keep-history 100
ccr cleanup --older-than 30d
```

## 🔍 诊断命令

### 系统诊断

```bash
# 检查所有关键文件
ls -la ~/.ccs_config.toml
ls -la ~/.claude/settings.json
ls -la ~/.claude/ccr_history.json
ls -la ~/.claude/.locks/

# 检查权限
stat ~/.ccs_config.toml
stat ~/.claude/settings.json

# 检查磁盘空间
df -h ~
du -sh ~/.claude/
```

### 配置诊断

```bash
# 验证配置
ccr validate

# 列出配置
ccr list

# 查看当前状态
ccr current

# 查看历史
ccr history --limit 5
```

### 锁文件诊断

```bash
# 检查锁文件
ls -la ~/.claude/.locks/

# 检查锁文件年龄
find ~/.claude/.locks/ -name "*.lock" -mmin +10

# 清理旧锁（超过 10 分钟）
find ~/.claude/.locks/ -name "*.lock" -mmin +10 -delete
```

## 📞 获取帮助

### 问题报告

如果问题仍未解决，请提供以下信息：

```bash
# 收集诊断信息
cat > ccr-debug.txt << 'EOF'
# CCR 版本
ccr --version

# 系统信息
uname -a

# Rust 版本
rustc --version
cargo --version

# 配置文件（移除敏感信息！）
cat ~/.ccs_config.toml | sed 's/auth_token = .*/auth_token = "***"/'

# 设置文件（移除敏感信息！）
cat ~/.claude/settings.json | jq 'del(.env.ANTHROPIC_AUTH_TOKEN)'

# 文件权限
ls -la ~/.ccs_config.toml
ls -la ~/.claude/

# 最近的历史
ccr history --limit 3

# 验证报告
ccr validate
EOF

# 查看诊断信息
cat ccr-debug.txt
```

### 提交 Issue

1. 访问 [GitHub Issues](https://github.com/bahayonghang/ccs/issues)
2. 点击 "New Issue"
3. 选择 "Bug Report" 模板
4. 填写问题描述和诊断信息
5. 提交 Issue

### 社区支持

- 💬 [GitHub Discussions](https://github.com/bahayonghang/ccs/discussions)
- 📧 邮件: 参见 GitHub Profile

## 🔗 相关文档

- [安装指南](/installation/)
- [配置文件](/installation/configuration)
- [命令参考](/commands/)
- [API 参考](/api/)

