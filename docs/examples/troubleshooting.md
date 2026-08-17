# 🔧 CCR 多平台配置故障排除指南

## 常见问题和解决方案

### 1. 平台切换失败

#### 问题: `平台 'xxx' 未找到`
```bash
$ ccr platform switch unknown
Error: 平台 'unknown' 未找到
```

**原因**: 指定的平台名称不在支持列表中

**解决方案**:
```bash
# 查看所有支持的平台
ccr platform list

# 支持的平台: claude, codex, gemini, qwen, droid
ccr platform switch claude
```

---

#### 问题: 切换平台后配置未生效
```bash
$ ccr platform switch codex
✓ 已切换到平台 'codex'

$ ccr current
# 但仍显示 Claude 配置
```

**原因**: 平台虽然切换了，但该平台尚未配置 profile

**解决方案**:
```bash
# 1. 确认当前平台
ccr platform current

# 2. 查看该平台是否有 profiles
ccr platform info codex

# 3. 如果没有 profiles，需要添加
# (需要等待平台特定命令实现，或手动创建 profiles.toml)
```

---

### 2. 目录和文件权限问题

#### 问题: `创建目录失败: Permission denied`
```bash
$ ccr claude profile init
Error: 创建目录失败 "/home/user/.ccr/platforms/claude": Permission denied
```

**原因**: 没有权限在 `~/.ccr/` 目录创建子目录

**解决方案**:
```bash
# 检查并修复权限
chmod 755 ~/.ccr
chmod 755 ~/.ccr/platforms

# 或删除后重新创建
rm -rf ~/.ccr
ccr claude profile init
```

---

#### 问题: `读取配置文件失败`
```bash
$ ccr platform list
Error: 读取配置文件失败: Permission denied
```

**原因**: config.toml 文件权限不正确

**解决方案**:
```bash
# 修复配置文件权限
chmod 644 ~/.ccr/config.toml

# 如果文件损坏，可以删除重建
rm ~/.ccr/config.toml
ccr platform list  # 会自动创建默认配置
```

---

### 3. 配置文件格式错误

#### 问题: `TOML 解析失败`
```bash
$ ccr platform list
Error: 配置格式错误: TOML 解析失败: expected = , but found...
```

**原因**: config.toml 或 profiles.toml 文件语法错误

**解决方案**:
```bash
# 1. 备份当前文件
cp ~/.ccr/config.toml ~/.ccr/config.toml.backup

# 2. 使用示例文件作为模板
cat docs/examples/config.toml > ~/.ccr/config.toml

# 3. 或使用 TOML 验证工具检查
# 在线工具: https://www.toml-lint.com/

# 4. 检查常见错误:
#    - 缺少引号: key = value  ❌ 应为 key = "value" ✓
#    - 重复的键: [claude] 出现两次
#    - 注释语法: # 开头而不是 //
```

---

### 4. Legacy 和 Unified 模式冲突

#### 问题: `配置模式不一致`
```bash
$ ccr list
# 显示 Legacy 模式配置

$ ccr platform list
# 显示 Unified 模式配置

# 两者内容不一致
```

**原因**: 同时存在 `~/.ccs_config.toml` (Legacy) 和 `~/.ccr/config.toml` (Unified)

**解决方案**:
```bash
# 方案 1: 以当前布局为准
ccr init
ccr platform list

# 方案 2: 继续使用 Legacy 模式
mv ~/.ccr ~/.ccr.disabled  # 暂时移走 Unified 配置
# 继续使用 ccr list / ccr switch 等命令

# 方案 3: 统一迁移后再逐个平台重建或导入 profile
ccr import <file> --merge --backup
```

---

### 5. Profile 配置问题

#### 问题: `Profile 'xxx' 未找到`
```bash
$ ccr switch my-profile
Error: 配置 'my-profile' 未找到
```

**原因**: 指定的 profile 名称不存在

**解决方案**:
```bash
# 1. 查看所有可用 profiles
ccr list

# 2. 确认 profile 是否在当前平台
ccr platform current

# 3. 如果在不同平台，需要先切换
ccr platform switch codex
ccr list  # 查看 Codex 平台的 profiles
```

---

#### 问题: `Profile 验证失败: 缺少必需字段`
```bash
$ ccr add test
Error: Profile 验证失败: 缺少必需字段 'base_url'
```

**原因**: Profile 配置缺少必需字段

**解决方案**:
```toml
# 确保 profile 包含所有必需字段:
[my-profile]
description = "My Custom Profile"  # 可选
base_url = "https://api.example.com"  # 必需
auth_token = "your-token-here"  # 必需
model = "model-name"  # 必需
small_fast_model = "fast-model"  # 可选
```

---

### 6. API 连接问题

#### 问题: `连接超时 / 连接被拒绝`
```bash
$ ccr switch my-profile
# 切换成功

$ claude-code
Error: Failed to connect to API: Connection timeout
```

**原因**: API 端点不可访问或配置错误

**解决方案**:
```bash
# 1. 检查当前 profile 配置
ccr current

# 2. 验证 base_url 是否正确
curl -I https://api.anthropic.com  # 应返回 200

# 3. 检查 auth_token 是否有效
# Claude: https://console.anthropic.com/settings/keys
# Codex: https://github.com/settings/tokens
# Gemini: https://makersuite.google.com/app/apikey

# 4. 检查网络代理设置
echo $HTTP_PROXY
echo $HTTPS_PROXY

# 5. 测试不同的 profile
ccr switch official-profile
```

---

### 7. 环境变量冲突

#### 问题: `环境变量被覆盖`
```bash
$ echo $ANTHROPIC_BASE_URL
https://api.custom-proxy.com

$ ccr switch official
# 切换后环境变量没变
```

**原因**: CCR 修改的是 `~/.claude/settings.json`，需要重启终端或重新加载环境变量

**解决方案**:
```bash
# 方案 1: 重启终端或新开窗口

# 方案 2: 重新加载 Claude Code 配置
# (具体方法取决于 Claude Code 的实现)
```

---

### 8. 历史记录和备份问题

#### 问题: `历史文件损坏`
```bash
$ ccr history
Error: 读取历史文件失败: invalid JSON
```

**原因**: `~/.ccr/history/<platform>.json` 文件格式错误

**解决方案**:
```bash
# 1. 备份损坏的文件
mv ~/.ccr/history/claude.json ~/.ccr/history/claude.json.corrupted

# 2. 创建新的空历史文件
echo "[]" > ~/.ccr/history/claude.json

# 3. 或删除后自动重建
rm ~/.ccr/history/claude.json
ccr switch any-profile  # 会自动创建新历史
```

---

### 9. 多用户环境问题

#### 问题: `配置在不同用户之间不同步`
```bash
# user1
$ ccr platform switch codex

# user2
$ ccr platform current
# 仍显示 claude
```

**原因**: CCR 配置存储在用户主目录 `~/.ccr/`，每个用户独立

**解决方案**:
```bash
# 方案 1: 使用共享配置目录
export CCR_ROOT=/shared/ccr
ccr claude profile init

# 方案 2: 导出/导入配置
# user1
ccr export -o /tmp/ccr-config.toml

# user2
ccr import /tmp/ccr-config.toml
```

---

### 10. 升级和迁移问题

#### 问题: `升级后配置丢失`
```bash
$ cargo install --git https://github.com/bahayonghang/ccr ccr
# 升级后

$ ccr list
Error: 配置文件未找到
```

**原因**: 配置文件路径或格式可能在新版本中变更

**解决方案**:
```bash
# 1. 检查备份
ls ~/.claude/backups/

# 2. 恢复配置
cp ~/.claude/backups/settings_*.json ~/.claude/settings.json

# 3. 或重新初始化
ccr init
ccr add  # 重新添加配置
```

---

## 调试技巧

### 启用调试日志
```bash
# 设置日志级别
export CCR_LOG_LEVEL=debug

# 运行命令查看详细日志
ccr platform switch codex

# 或一次性运行
CCR_LOG_LEVEL=debug ccr platform list

# 查看日志文件（UTC 按天轮转，保留14天）
tail -f ~/.ccr/logs/ccr.log.$(date -u +%Y-%m-%d)

# 查看所有日志文件
ls -lh ~/.ccr/logs/
```

### 检查配置文件
```bash
# 查看统一配置
cat ~/.ccr/config.toml

# 查看平台 profiles
cat ~/.ccr/platforms/claude/profiles.toml
cat ~/.ccr/platforms/codex/profiles.toml
cat ~/.ccr/platforms/gemini/profiles.toml

# 查看历史记录
cat ~/.ccr/history/claude.json

# 检查备份
ls -lh ~/.ccr/backups/
```

### 验证目录结构
```bash
# 查看完整目录树
tree ~/.ccr -L 3

# 或使用 find
find ~/.ccr -type f -o -type d | sort
```

### 手动清理
```bash
# 完全重置（谨慎操作！）
rm -rf ~/.ccr
rm ~/.claude/settings.json

# 重新初始化
ccr init
ccr claude profile init
ccr add official
```

---

## 获取帮助

### 内置帮助
```bash
# 查看所有命令
ccr --help

# 查看特定命令帮助
ccr platform --help
ccr switch --help

# 查看版本信息
ccr version
ccr --version
```

### 社区支持
- GitHub Issues: https://github.com/bahayonghang/ccr/issues
- 文档: https://ccr-docs.example.com (如果有)
- 示例配置: `docs/examples/`

### 报告问题
提交 Issue 时请包含:
1. CCR 版本: `ccr --version`（如需详细安装摘要，再补 `ccr version`）
2. 操作系统: `uname -a`
3. 配置文件 (去除敏感信息): `cat ~/.ccr/config.toml`
4. 错误日志: `CCR_LOG_LEVEL=debug ccr <command> 2>&1`
5. 重现步骤
