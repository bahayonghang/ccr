# current - 显示当前配置

显示当前配置的详细状态,包括环境变量设置。

## 用法

```bash
ccr current
# 或使用别名
ccr status
ccr show
```

## 输出信息

- 配置名称和描述
- Base URL
- Auth Token(脱敏)
- 模型设置
- 环境变量配置状态

## 示例输出

```
Current Configuration
════════════════════════════════════════════════════════════════
Name: anthropic
Description: Anthropic Official API
────────────────────────────────────────────────────────────────
Configuration Details:
  Base URL: https://api.anthropic.com
  Auth Token: sk-a...key
  Model: claude-sonnet-4-5-20250929
  Small Fast Model: claude-3-5-haiku-20241022

Environment Variables:
  ANTHROPIC_BASE_URL: https://api.anthropic.com
  ANTHROPIC_AUTH_TOKEN: sk-a...key
  ANTHROPIC_MODEL: claude-sonnet-4-5-20250929
  ANTHROPIC_SMALL_FAST_MODEL: claude-3-5-haiku-20241022

Status: ✓ Configuration Active
```

## 使用场景

### 确认当前配置

在执行重要操作前,确认当前使用的配置：

```bash
ccr current
```

### 检查环境变量

查看 Claude Code 的环境变量设置：

```bash
ccr current | grep "ANTHROPIC_"
```

### 验证切换结果

切换配置后,确认切换成功：

```bash
ccr switch anyrouter
ccr current
```

## 与 list 的区别

- **list**: 显示所有可用配置的概览
- **current**: 仅显示当前配置的详细信息

## 相关命令

- [list](./list) - 查看所有配置
- [switch](./switch) - 切换配置
- [validate](./validate) - 验证当前配置
