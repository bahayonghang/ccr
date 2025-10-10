# list - 列出所有配置

列出所有可用配置，标记当前配置和验证状态。

## 用法

```bash
ccr list
# 或使用别名
ccr ls
```

## 输出信息

- 配置文件路径
- 默认配置名称
- 当前配置名称
- 每个配置的详细信息：
  - 配置名称和描述
  - Base URL
  - Token（脱敏显示）
  - 模型名称
  - 小型快速模型（如果有）
  - 配置状态（完整 / 不完整）

## 示例输出

```
Available Configurations
════════════════════════════════════════════════════════════════
Configuration File: /home/user/.ccs_config.toml
Default Config: anthropic
Current Config: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic Official API
    Base URL: https://api.anthropic.com
    Token: sk-a...key
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    Status: ✓ Configuration Complete
  anyrouter - AnyRouter Proxy Service
    Base URL: https://api.anyrouter.ai/v1
    Token: you...ken
    Model: claude-sonnet-4-5-20250929
    Status: ✓ Configuration Complete

✓ Found 2 configurations
```

## 输出说明

### 配置标记

- `▶` - 当前正在使用的配置
- 无标记 - 其他可用配置

### 状态指示

- `✓ Configuration Complete` - 配置完整，包含所有必需字段
- `⚠ Configuration Incomplete` - 配置不完整，缺少必需字段

### 令牌脱敏

为了安全，API 令牌会自动脱敏显示，仅显示首尾字符：

```
原始: sk-ant-api03-abc123def456ghi789jkl012mno345pqr678
显示: sk-a...r678
```

## 使用场景

### 查看可用配置

在切换配置前，先查看所有可用的配置：

```bash
ccr list
```

### 检查配置状态

快速检查哪些配置是完整的：

```bash
ccr list | grep "Complete"
```

### 查找配置文件位置

查看配置文件的完整路径：

```bash
ccr list | head -n 3
```

## 相关命令

- [current](./current) - 显示当前配置详情
- [validate](./validate) - 验证所有配置
- [switch](./switch) - 切换到其他配置
