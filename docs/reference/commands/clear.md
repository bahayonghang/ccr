# clear - 清理 CCR 配置

清空 Claude Code `settings.json` 中由 CCR 写入的环境变量，使其恢复到默认状态。

## 用法

```bash
ccr clear [OPTIONS]
```

## 选项

- `-f, --force`: 跳过确认提示，直接清理（危险操作）

## 功能特性

- 🧹 **清理环境变量** - 清除 `ANTHROPIC_*` 等平台相关环境变量
- 🔒 **安全确认** - 默认需要用户确认（除非使用 --force）
- 💾 **自动备份** - 清理前自动备份 `settings.json`
- 📋 **历史记录** - 记录清理操作到审计日志
- ⚠️ **影响说明** - 清楚告知清理后的影响

## 示例

### 基本用法

```bash
# 清理配置（需要确认）
ccr clear

# 强制清理（跳过确认）
ccr clear --force

# 使用短选项
ccr clear -f
```

## 示例输出

### 正常清理流程

```bash
$ ccr clear

清理 CCR 配置
══════════════════════════════════════════════════════════════

▶ 当前配置信息

平台: Claude
当前 Profile: anthropic
环境变量数量: 3

将要清理的环境变量:
  • ANTHROPIC_API_KEY
  • ANTHROPIC_BASE_URL
  • ANTHROPIC_MODEL

⚠ 警告
══════════════════════════════════════════════════════════════

清理后的影响:
  • Claude Code 将无法使用 API（直到重新配置）
  • settings.json 中的环境变量将被清空
  • 配置文件中的 profiles 不受影响

建议操作:
  • 清理前确保已导出配置备份
  • 清理后可运行 'ccr switch <name>' 重新应用配置

? 确认清理 CCR 配置? [y/N]: y

▶ 执行清理
══════════════════════════────────────────────────────────────

✓ 创建备份: ~/.claude/backups/settings.json.20250110_143022.bak
✓ 清理环境变量: 3 个
✓ 保存 settings.json

──────────────────────────────────────────────────────────────

✓ CCR 配置已清理

ℹ 后续操作:
  • 运行 'ccr list' 查看可用配置
  • 运行 'ccr switch <name>' 应用新配置
  • 或直接编辑 settings.json 手动配置
```

### 强制清理（--force）

```bash
$ ccr clear --force

清理 CCR 配置
══════════════════════════════════════════════════════════════

▶ 执行清理 (--force 模式)
⚠ 跳过确认，直接清理

✓ 创建备份: ~/.claude/backups/settings.json.20250110_143522.bak
✓ 清理环境变量: 3 个
✓ 保存 settings.json

──────────────────────────────────────────────────────────────

✓ CCR 配置已清理

⚠ 重要提示:
  • Claude Code 现在无法使用 API
  • 运行 'ccr switch <name>' 重新应用配置
```

### 无配置需要清理

```bash
$ ccr clear

清理 CCR 配置
══════════════════════════════────────────────────════════────

▶ 检查当前配置

ℹ settings.json 中没有 CCR 写入的环境变量

✓ 无需清理
```

### 取消清理

```bash
...
? 确认清理 CCR 配置? [y/N]: n

ℹ 已取消清理
```

## 使用场景

### 场景 1: 重置 Claude Code 配置

```bash
# 清理所有 CCR 配置
ccr clear

# 重新应用新配置
ccr switch new_config
```

### 场景 2: 故障排查

```bash
# 清理可能损坏的配置
ccr clear

# 验证清理结果
cat ~/.claude/settings.json

# 重新应用配置
ccr switch anthropic
```

### 场景 3: 切换工作环境

```bash
# 清空当前配置
ccr clear

# 切换到新环境
ccr platform switch codex
ccr switch codex_config
```

### 场景 4: 卸载 CCR 前的清理

```bash
# 导出配置备份
ccr export -o backup.toml

# 清理 CCR 配置
ccr clear --force

# 现在可以安全卸载 CCR
```

## 清理内容

### 清理的环境变量（根据平台不同）

**Claude Code 平台:**
- `ANTHROPIC_API_KEY`
- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_MODEL`
- 其他 `ANTHROPIC_*` 变量

**Codex 平台:**
- `OPENAI_API_KEY`
- `OPENAI_BASE_URL`
- `OPENAI_MODEL`
- 其他 `OPENAI_*` 变量

**Gemini 平台:**
- `GOOGLE_API_KEY`
- `GOOGLE_BASE_URL`
- `GOOGLE_MODEL`
- 其他 `GOOGLE_*` 变量

### 不清理的内容

- ✅ CCR 配置文件（`~/.ccr/config.toml` 或 `~/.ccs_config.toml`）
- ✅ Profile 配置（`~/.ccr/platforms/*/profiles.toml`）
- ✅ 历史记录（`~/.ccr/history/`）
- ✅ 备份文件（`~/.ccr/backups/`）
- ✅ `settings.json` 中的其他配置项

## 清理前后对比

### 清理前的 settings.json

```json
{
  "env": {
    "ANTHROPIC_API_KEY": "sk-ant-xxx",
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_MODEL": "claude-3-5-sonnet-20241022",
    "OTHER_VAR": "value"
  },
  "other_settings": {
    ...
  }
}
```

### 清理后的 settings.json

```json
{
  "env": {
    "OTHER_VAR": "value"
  },
  "other_settings": {
    ...
  }
}
```

**注意：** 只清理 CCR 管理的环境变量，其他配置保持不变。

## 备份机制

### 自动备份

每次清理前都会自动备份 `settings.json`：

```bash
备份位置: ~/.claude/backups/settings.json.20250110_143022.bak
          或
          ~/.ccr/backups/claude/settings.json.20250110_143022.bak
```

### 备份文件命名

```
settings.json.<时间戳>.bak
```

时间戳格式：`YYYYMMDD_HHMMSS`

### 恢复备份

如果需要恢复：

```bash
# 找到最近的备份
ls -lt ~/.claude/backups/ | head -5

# 恢复备份
cp ~/.claude/backups/settings.json.20250110_143022.bak ~/.claude/settings.json

# 验证恢复
cat ~/.claude/settings.json
```

## 清理后的影响

### ⚠️ 立即影响

- ❌ Claude Code 无法连接 API（需要重新配置）
- ❌ 无法使用 `claude` 命令进行对话
- ✅ Claude Code 本身仍可运行（只是无法调用 API）

### ✅ 不影响

- ✅ CCR 配置文件仍然存在
- ✅ Profile 配置仍然可用
- ✅ 历史记录完整保留
- ✅ 可以随时重新应用配置

## 恢复配置

### 方法 1: 重新应用 Profile（推荐）

```bash
# 查看可用配置
ccr list

# 应用配置
ccr switch anthropic
```

### 方法 2: 从备份恢复

```bash
# 恢复最近的备份
cp ~/.claude/backups/settings.json.20250110_143022.bak ~/.claude/settings.json
```

### 方法 3: 手动编辑 settings.json

```bash
# 直接编辑
vim ~/.claude/settings.json

# 添加必要的环境变量
{
  "env": {
    "ANTHROPIC_API_KEY": "sk-ant-xxx",
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_MODEL": "claude-3-5-sonnet-20241022"
  }
}
```

## 常见问题

### Q: 清理后 Claude Code 还能用吗？

**A:** Claude Code 程序本身可以运行，但无法调用 API：
- ✅ 可以运行 `claude` 命令
- ❌ 无法进行 AI 对话（需要重新配置）

### Q: 清理会删除我的配置吗？

**A:** 不会，清理只影响 `settings.json`：
- ❌ 不会删除 CCR 配置文件
- ❌ 不会删除 Profile 配置
- ❌ 不会删除历史记录
- ✅ 只清空 `settings.json` 中的环境变量

### Q: 如何撤销清理操作？

**A:** 两种方法：
```bash
# 方法 1: 重新应用配置
ccr switch anthropic

# 方法 2: 从备份恢复
cp ~/.claude/backups/settings.json.*.bak ~/.claude/settings.json
```

### Q: 清理前需要做什么准备？

**A:** 建议步骤：
```bash
# 1. 导出配置备份（可选）
ccr export -o backup.toml

# 2. 记住当前配置名称
ccr current

# 3. 执行清理
ccr clear

# 4. 重新应用配置
ccr switch <之前的配置名称>
```

### Q: 清理和删除配置有什么区别？

**A:**
- **clear**：清空 `settings.json` 中的环境变量，不影响 CCR 配置文件
- **delete**：删除 CCR 配置文件中的特定 profile，不影响 `settings.json`

## 最佳实践

### 1. 清理前导出备份

```bash
# 养成习惯：清理前先导出
ccr export -o backup-before-clear.toml
ccr clear
```

### 2. 记住当前配置

```bash
# 清理前记下当前配置
ccr current  # 记住输出的配置名称

# 清理
ccr clear

# 恢复
ccr switch <刚才的配置名称>
```

### 3. 使用 force 模式（脚本中）

```bash
#!/bin/bash
# 自动化脚本中使用 --force 避免交互

ccr clear --force
ccr switch new_config
```

### 4. 清理后验证

```bash
# 清理后检查
cat ~/.claude/settings.json

# 确认环境变量已清除
ccr clear
```

## 注意事项

::: danger 危险操作
- 清理后 Claude Code 将无法使用 API
- 使用 `--force` 会立即清理，跳过确认
- 清理是不可逆的（除非从备份恢复）
:::

::: warning 注意
- 清理只影响 `settings.json`，不影响 CCR 配置
- 清理后需要重新运行 `ccr switch` 应用配置
- 自动备份文件会占用磁盘空间
:::

::: tip 建议
- 清理前使用 `ccr export` 导出配置
- 清理后立即重新应用配置
- 定期清理过期的备份文件
- 在自动化脚本中使用 `--force`
:::

## 相关命令

- [switch](./switch) - 应用配置到 settings.json
- [current](./current) - 查看当前配置
- [export](./export) - 导出配置备份
- [clean](./clean) - 清理过期备份文件
- [validate](./validate) - 验证配置完整性
