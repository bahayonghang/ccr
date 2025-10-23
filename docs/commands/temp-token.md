# temp-token - 临时Token管理

临时覆盖当前配置的认证信息，无需修改永久配置文件。适用于临时测试免费Token、临时切换API端点等场景。

## 用法

```bash
ccr temp-token <action> [options]
```

## 子命令

### set - 设置并立即应用临时配置

临时覆盖当前 settings.json 中的 token、base_url、model 等字段。

**重要**: 临时配置会**立即应用**到 `~/.claude/settings.json`，无需再次 switch！

```bash
ccr temp-token set <token> [options]
```

**参数：**
- `<token>`: 临时使用的认证令牌（必需）

**选项：**
- `--base-url <URL>`: 临时 Base URL（可选）
- `--model <MODEL>`: 临时主模型（可选）

### show - 显示当前临时配置

查看当前设置的临时配置状态。

```bash
ccr temp-token show
```

### clear - 清除临时配置

删除所有临时配置覆盖，恢复使用永久配置。

```bash
ccr temp-token clear
```

## 工作原理

1. **立即应用**：`temp-token set` 直接修改 `~/.claude/settings.json`
2. **不保存临时文件**：不创建临时配置文件，直接写入 settings.json
3. **switch 恢复**：下次执行 `ccr switch` 时，会使用配置文件中的原始 token

## 使用场景

### 场景 1: 临时使用免费Token测试

```bash
# 1. 先切换到 duck 配置（使用配置文件中的 token）
ccr switch duck

# 2. 设置临时免费 token（立即生效）
ccr temp-token set sk-free-test-token-xxx

# 3. 现在 settings.json 中已经是临时 token 了，可以直接使用

# 4. 测试完成后，再次 switch 恢复到配置文件中的 token
ccr switch duck
```

### 场景 2: 临时切换API端点

```bash
# 1. 先切换到目标配置
ccr switch anyrouter

# 2. 设置临时 token 和 base_url（立即生效）
ccr temp-token set sk-temp-xxx \
  --base-url https://api.temp-provider.com/v1 \
  --model claude-3-sonnet

# 3. 现在 settings.json 已更新，可以直接使用

# 4. 测试完成后恢复
ccr switch anyrouter
```

### 场景 3: 短期测试不同模型

```bash
# 1. 先切换到目标配置
ccr switch anthropic

# 2. 只覆盖模型配置（立即生效）
ccr temp-token set sk-test-xxx --model claude-opus-4

# 3. 测试完成后恢复
ccr switch anthropic
```

## 输出示例

### 设置临时Token

```bash
$ ccr temp-token set sk-free-xxx

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  设置临时 Token
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎯 正在应用临时配置到当前设置...
✅ 临时配置已应用到当前设置

╔════════════════╤═══════════════════════════════════╗
║ 字段           │ 临时值                            ║
╠════════════════╪═══════════════════════════════════╣
║ Auth Token     │ sk-f...xxx                        ║
║ 创建时间       │ 2025-10-23 15:30:00               ║
╚════════════════╧═══════════════════════════════════╝

💡 提示:
   • 临时配置已立即应用到 settings.json
   • 下次 switch 时将使用配置文件中的原始 token
   • 临时配置不会修改 toml 配置文件
```

### 显示临时配置

```bash
$ ccr temp-token show

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  临时配置状态
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 当前有 2 个字段被临时覆盖

╔════════════════╤═══════════════════════════════════╗
║ 字段           │ 临时值                            ║
╠════════════════╪═══════════════════════════════════╣
║ Auth Token     │ sk-f...xxx                        ║
║ Base URL       │ https://api.temp.com              ║
║ 创建时间       │ 2025-10-23 15:30:00               ║
╚════════════════╧═══════════════════════════════════╝
```

### 切换配置恢复

```bash
$ ccr switch duck

步骤 1/5: 读取配置
...
步骤 3/5: 更新 Claude Code 设置
✅ Claude Code 设置已更新

💡 提示: 从 duck → duck ✓
🔄 建议重启 Claude Code 以确保配置完全生效
```

### 清除临时配置

```bash
$ ccr temp-token clear

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  清除临时配置
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 临时配置已清除

💡 现在将使用 toml 配置文件中的设置
```

## 注意事项

1. **不修改永久配置**：临时配置不会修改 `~/.ccs_config.toml`，只影响 `settings.json`
2. **立即生效**：`temp-token set` 直接修改当前 `settings.json`，无需再次 switch
3. **不保存临时文件**：不创建 `temp_override.json`，直接写入 settings.json
4. **switch 恢复**：下次执行 `ccr switch` 时，会使用配置文件中的原始 token
5. **部分覆盖**：可以只覆盖部分字段（如只覆盖 token，不覆盖 base_url）
6. **安全性**：临时 token 在显示时会自动脱敏，只显示前后几位

## 常见问题

### Q: 临时配置会持久化吗？

A: 不会。`ccr temp-token set` 直接修改 `~/.claude/settings.json`，不创建临时配置文件。配置立即生效，下次 `ccr switch` 时会恢复为配置文件中的值。

### Q: 如何知道当前是否使用了临时 token？

A: 执行 `ccr current` 查看当前 settings.json 的内容，对比配置文件中的 token 即可判断。

### Q: 临时配置会影响其他环境变量吗？

A: 不会。临时配置只覆盖你明确设置的字段（token、base_url、model 等），其他环境变量保持不变。

### Q: 为什么要先 switch 再设置临时 token？

A: 因为 `temp-token set` 直接修改当前 settings.json。你需要先用 `ccr switch` 确认当前在哪个配置，然后临时覆盖它的 token。这样下次 switch 时就能恢复到配置文件中的原始值。

### Q: 为什么不保存临时配置文件？

A: 设计为直接应用是为了简化流程：
- 设置后立即可用，无需等待下次 switch
- 不需要管理额外的临时配置文件
- 避免忘记清理临时配置的问题

### Q: 如果想持续使用某个 token 怎么办？

A: 如果需要持续使用某个 token，应该直接修改 `~/.ccs_config.toml` 文件中的永久配置，而不是使用临时配置。临时配置只适用于一次性测试场景。

## 相关命令

- [`ccr switch`](./switch.md) - 切换配置（恢复配置文件中的 token）
- [`ccr current`](./current.md) - 查看当前配置状态
- [`ccr list`](./list.md) - 列出所有可用配置

## 文件位置

- 永久配置文件：`~/.ccs_config.toml`
- Claude 设置文件：`~/.claude/settings.json`（临时 token 直接写入此文件）
