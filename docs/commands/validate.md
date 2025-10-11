# validate - 验证配置

验证配置和设置的完整性,确保所有必需字段都已正确配置。

## 用法

```bash
ccr validate
# 或使用别名
ccr check
```

## 检查项目

- 配置文件格式是否正确
- 所有配置段的完整性
- Claude Code 设置文件是否存在且有效
- 必需的环境变量是否设置

## 验证规则

每个配置需要包含：

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `base_url` | String | 是 | API 端点地址 |
| `auth_token` | String | 是 | 认证令牌 |
| `model` | String | 是 | 默认模型 |
| `description` | String | 否 | 配置描述 |
| `small_fast_model` | String | 否 | 小型快速模型 |

## 示例输出

### 验证成功

```
Validating Configuration
════════════════════════════════════════════════════════════════
Configuration File: /home/user/.ccs_config.toml
────────────────────────────────────────────────────────────────
✓ Configuration file exists
✓ Configuration file is valid TOML
✓ All configurations are complete:
  ✓ anthropic - All required fields present
  ✓ anyrouter - All required fields present
✓ Settings file exists
✓ Environment variables are properly set

✓ Validation passed
```

### 验证失败

```
Validating Configuration
════════════════════════════════════════════════════════════════
Configuration File: /home/user/.ccs_config.toml
────────────────────────────────────────────────────────────────
✓ Configuration file exists
✓ Configuration file is valid TOML
✗ Configuration 'incomplete' is missing required fields:
  ✗ Missing: auth_token
  ✗ Missing: model
⚠ Settings file not found at ~/.claude/settings.json

✗ Validation failed with 3 errors
```

## 使用场景

### 添加新配置后

编辑配置文件后验证：

```bash
vim ~/.ccs_config.toml
ccr validate
```

### 排查问题

当切换失败时,验证配置完整性：

```bash
ccr validate
```

### 定期检查

定期验证配置状态：

```bash
# 添加到 crontab(每天检查)
0 9 * * * ccr validate || echo "CCR configuration invalid!"
```

## 验证内容详解

### 1. 配置文件格式

检查 `~/.ccs_config.toml` 是否为有效的 TOML 格式：

```toml
# 正确格式
[config_name]
base_url = "https://api.example.com"

# 错误格式(缺少引号)
[config_name]
base_url = https://api.example.com
```

### 2. 必需字段

确保每个配置包含所有必需字段：

```toml
# 完整配置 ✓
[complete]
description = "Complete Config"
base_url = "https://api.example.com"
auth_token = "sk-xxx"
model = "claude-sonnet-4-5-20250929"

# 不完整配置 ✗
[incomplete]
base_url = "https://api.example.com"
# 缺少 auth_token 和 model
```

### 3. URL 格式

验证 `base_url` 是否为有效的 URL：

```toml
# 正确 ✓
base_url = "https://api.anthropic.com"
base_url = "http://localhost:8000"

# 错误 ✗
base_url = "not-a-url"
base_url = "api.anthropic.com"  # 缺少协议
```

### 4. 设置文件

检查 `~/.claude/settings.json` 是否存在且格式正确：

```json
{
  "environmentVariables": {
    "ANTHROPIC_BASE_URL": "...",
    "ANTHROPIC_AUTH_TOKEN": "...",
    "ANTHROPIC_MODEL": "..."
  }
}
```

## 修复常见问题

### 问题：配置文件格式错误

**解决：**
```bash
# 检查 TOML 语法
cat ~/.ccs_config.toml

# 重新初始化(会备份)
ccr init --force
```

### 问题：缺少必需字段

**解决：**
```bash
# 编辑配置文件
vim ~/.ccs_config.toml

# 添加缺少的字段
[config_name]
base_url = "https://api.example.com"
auth_token = "your-token"      # 添加
model = "claude-sonnet-4-5-20250929"  # 添加
```

### 问题：设置文件不存在

**解决：**
```bash
# 切换任意配置会自动创建
ccr switch anthropic
```

## 自动化验证

### Git 钩子

在提交前自动验证配置：

```bash
# .git/hooks/pre-commit
#!/bin/bash
if ! ccr validate; then
  echo "CCR configuration validation failed!"
  exit 1
fi
```

### CI/CD 集成

在部署流程中验证配置：

```yaml
# .github/workflows/validate.yml
- name: Validate CCR Config
  run: |
    ccr validate
```

## 相关命令

- [init](./init) - 重新初始化配置
- [list](./list) - 查看所有配置状态
- [switch](./switch) - 切换到有效配置
