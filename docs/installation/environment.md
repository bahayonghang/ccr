# 环境变量

CCR 管理 Claude Code 使用的环境变量。本文档详细说明这些环境变量的作用和管理方式。

## 🌍 环境变量列表

### ANTHROPIC_BASE_URL

**类型**: 字符串（URL）  
**必填**: ✅  
**说明**: Claude API 的基础 URL

**示例**:
```bash
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
export ANTHROPIC_BASE_URL="https://api.anyrouter.ai/v1"
export ANTHROPIC_BASE_URL="http://localhost:8000"
```

**作用**:
- 指定 API 端点地址
- Claude Code 会向此地址发送请求
- 支持官方 API 和第三方代理

---

### ANTHROPIC_AUTH_TOKEN

**类型**: 字符串  
**必填**: ✅  
**说明**: API 认证令牌

**示例**:
```bash
export ANTHROPIC_AUTH_TOKEN="sk-ant-api03-your-key-here"
```

**作用**:
- API 请求的身份认证
- 作为 HTTP Header 中的 `x-api-key`
- 必须与 base_url 对应的 API 匹配

**安全提示**:
- 🔒 不要在公共场合泄露
- 👁️ CCR 会自动掩码显示
- 📝 历史记录中自动脱敏

---

### ANTHROPIC_MODEL

**类型**: 字符串  
**必填**: ❌ 可选  
**说明**: 默认使用的模型名称

**示例**:
```bash
export ANTHROPIC_MODEL="claude-sonnet-4-5-20250929"
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"
export ANTHROPIC_MODEL="claude-3-opus-20240229"
```

**作用**:
- 指定默认模型
- Claude Code 会优先使用此模型
- 如果不设置，使用 Claude Code 的默认模型

**常用模型**:
| 模型名称 | 说明 |
|---------|------|
| `claude-sonnet-4-5-20250929` | Sonnet 4.5（最新）|
| `claude-3-5-sonnet-20241022` | Sonnet 3.5 |
| `claude-3-opus-20240229` | Opus 3（最强）|
| `claude-3-haiku-20240307` | Haiku 3（最快）|

---

### ANTHROPIC_SMALL_FAST_MODEL

**类型**: 字符串  
**必填**: ❌ 可选  
**说明**: 用于轻量级任务的快速小模型

**示例**:
```bash
export ANTHROPIC_SMALL_FAST_MODEL="claude-3-5-haiku-20241022"
```

**作用**:
- 用于代码补全等快速响应场景
- 提高轻量级任务的响应速度
- 降低成本

**使用场景**:
- 代码自动补全
- 简单问答
- 快速语法检查

## 🔄 CCR 如何管理环境变量

### 1. 读取配置

```rust
// 从 ~/.ccs_config.toml 读取
let section = config.get_section("anthropic")?;

// 提取字段
let base_url = section.base_url.as_ref().unwrap();
let auth_token = section.auth_token.as_ref().unwrap();
```

### 2. 更新 settings.json

```rust
// 清空旧的 ANTHROPIC_* 变量
settings.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));

// 设置新变量
settings.env.insert("ANTHROPIC_BASE_URL".into(), base_url.clone());
settings.env.insert("ANTHROPIC_AUTH_TOKEN".into(), auth_token.clone());
if let Some(model) = &section.model {
    settings.env.insert("ANTHROPIC_MODEL".into(), model.clone());
}
```

### 3. 原子写入

```rust
// 写入 ~/.claude/settings.json
settings_manager.save_atomic(&settings)?;
```

**结果**:
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-api03-...",
    "ANTHROPIC_MODEL": "claude-sonnet-4-5-20250929",
    "ANTHROPIC_SMALL_FAST_MODEL": "claude-3-5-haiku-20241022"
  }
}
```

## 🔍 查看当前环境变量

### 使用 CCR

```bash
ccr current
```

输出：
```
Claude Code 环境变量状态:

  ANTHROPIC_BASE_URL: https://api.anthropic.com
  ANTHROPIC_AUTH_TOKEN: sk-a...here
  ANTHROPIC_MODEL: claude-sonnet-4-5-20250929
  ANTHROPIC_SMALL_FAST_MODEL: claude-3-5-haiku-20241022
```

### 直接查看 settings.json

```bash
cat ~/.claude/settings.json | jq .env
```

### 在 Shell 中查看

```bash
# 注意：这些是写在 settings.json 中的
# 不是 Shell 环境变量

# 查看 settings.json
env | grep ANTHROPIC  # 可能看不到（因为不是 shell 环境变量）

# 正确方式
ccr current
```

## ⚙️ 手动管理环境变量

### 临时覆盖

如果需要临时覆盖配置（不推荐）：

```bash
# 设置 shell 环境变量（会覆盖 settings.json）
export ANTHROPIC_BASE_URL="https://api.temporary.com"
export ANTHROPIC_AUTH_TOKEN="temp-token"

# Claude Code 会优先使用环境变量
# 重启 shell 后恢复
```

### 永久设置（不推荐）

```bash
# 在 ~/.bashrc 或 ~/.zshrc 中
export ANTHROPIC_BASE_URL="https://api.example.com"
export ANTHROPIC_AUTH_TOKEN="your-token"

# 不推荐原因：
# 1. 失去 CCR 的管理功能
# 2. 无法使用配置切换
# 3. 无历史记录
# 4. 无自动备份
```

**推荐做法**: 使用 CCR 管理配置，而非手动设置环境变量

## 🔄 环境变量优先级

Claude Code 读取配置的优先级：

```
1. Shell 环境变量（最高优先级）
   export ANTHROPIC_BASE_URL="..."
   
2. settings.json 中的 env 对象
   { "env": { "ANTHROPIC_BASE_URL": "..." } }
   
3. Claude Code 默认配置（最低优先级）
```

**建议**:
- 使用 CCR 管理 settings.json（优先级 2）
- 不要手动设置 shell 环境变量（优先级 1）
- 这样配置更容易管理和追踪

## 🧪 环境变量测试

### 验证环境变量

```bash
# 使用 CCR 验证
ccr validate

# 检查特定变量
ccr current | grep ANTHROPIC_BASE_URL
```

### 测试不同配置

```bash
# 切换到配置 A
ccr switch anthropic
ccr current

# 切换到配置 B
ccr switch anyrouter
ccr current

# 对比环境变量变化
ccr history --limit 2
```

## 🔗 相关文档

- [配置文件格式](/installation/configuration)
- [settings.json 管理](/api/settings)
- [故障排除](/installation/troubleshooting)

