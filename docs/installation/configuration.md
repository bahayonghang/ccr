# 配置文件详解

CCR 使用 TOML 格式的配置文件来管理多个 API 配置。本文档详细介绍配置文件的结构、字段说明和最佳实践。

## 📁 配置文件位置

```bash
~/.ccs_config.toml
```

**说明**:
- 位于用户主目录
- 与 CCS (Shell 版本) 共享
- TOML 格式（易读易写）

## 📝 配置文件结构

### 完整示例

```toml
# ═══════════════════════════════════════
# CCR / CCS 配置文件
# ═══════════════════════════════════════

# ──────────── 全局设置 ────────────
default_config = "anthropic"    # 默认配置
current_config = "anthropic"    # 当前活跃配置

# ──────────── Anthropic 官方 API ────────────
[anthropic]
description = "Anthropic 官方 API - 最稳定的选择"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-your-api-key-here"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

# ──────────── AnyRouter 代理服务 ────────────
[anyrouter]
description = "AnyRouter AI 代理服务 - 支持多模型"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-api-key"
model = "claude-sonnet-4-5-20250929"
# small_fast_model 可选，不填则不设置

# ──────────── OpenRouter 服务 ────────────
[openrouter]
description = "OpenRouter - 统一 API 网关"
base_url = "https://openrouter.ai/api/v1"
auth_token = "sk-or-v1-your-key"
model = "anthropic/claude-sonnet-4-5"

# ──────────── 自建代理 ────────────
[custom-proxy]
description = "自建代理服务器"
base_url = "http://localhost:8000/v1"
auth_token = "custom-auth-token"
model = "claude-3-opus-20240229"

# ──────────── 测试环境 ────────────
[test]
description = "测试环境配置"
base_url = "https://api-test.example.com"
auth_token = "test-token-12345"
```

## 📋 字段说明

### 全局字段

#### default_config

**类型**: 字符串  
**必填**: ✅  
**说明**: 默认使用的配置名称

```toml
default_config = "anthropic"
```

**用途**:
- 系统首次启动时使用的配置
- 作为回退配置
- 标识推荐配置

#### current_config

**类型**: 字符串  
**必填**: ✅  
**说明**: 当前活跃的配置名称

```toml
current_config = "anyrouter"
```

**注意**:
- 由 CCR 自动管理
- 切换配置时自动更新
- 不建议手动修改

### 配置节字段

每个配置节（如 `[anthropic]`）支持以下字段：

#### description

**类型**: 字符串  
**必填**: ❌ 可选  
**说明**: 配置的描述信息

```toml
description = "Anthropic 官方 API - 最稳定的选择"
```

**最佳实践**:
- 简短描述配置用途
- 标注配置特点
- 方便在列表中识别

#### base_url

**类型**: 字符串（URL）  
**必填**: ✅ 必需  
**说明**: API 端点地址

```toml
base_url = "https://api.anthropic.com"
```

**格式要求**:
- 必须以 `http://` 或 `https://` 开头
- 不要在末尾添加 `/v1`（除非 API 要求）
- 不要添加尾部斜杠 `/`

**示例**:
```toml
# ✅ 正确
base_url = "https://api.anthropic.com"
base_url = "https://api.anyrouter.ai/v1"
base_url = "http://localhost:8000"

# ❌ 错误
base_url = "api.anthropic.com"           # 缺少协议
base_url = "https://api.anthropic.com/"  # 末尾斜杠（不推荐）
```

#### auth_token

**类型**: 字符串  
**必填**: ✅ 必需  
**说明**: API 认证令牌

```toml
auth_token = "sk-ant-api03-your-api-key-here"
```

**安全提示**:
- ⚠️ 不要提交到 Git（使用 .gitignore）
- 🔒 设置文件权限为 600
- 👁️ 在 CLI 输出时自动掩码

**Token 格式** (因 API 而异):
```toml
# Anthropic
auth_token = "sk-ant-api03-..."

# AnyRouter
auth_token = "your-anyrouter-key"

# OpenRouter
auth_token = "sk-or-v1-..."
```

#### model

**类型**: 字符串  
**必填**: ❌ 可选  
**说明**: 默认使用的模型名称

```toml
model = "claude-sonnet-4-5-20250929"
```

**常用模型**:
```toml
# Claude Sonnet 4.5 (推荐)
model = "claude-sonnet-4-5-20250929"

# Claude 3.5 Sonnet
model = "claude-3-5-sonnet-20241022"

# Claude 3 Opus
model = "claude-3-opus-20240229"

# Claude 3 Haiku
model = "claude-3-haiku-20240307"
```

**说明**:
- 如果不设置，Claude Code 使用默认模型
- 设置后作为 `ANTHROPIC_MODEL` 环境变量

#### small_fast_model

**类型**: 字符串  
**必填**: ❌ 可选  
**说明**: 用于轻量级任务的快速小模型

```toml
small_fast_model = "claude-3-5-haiku-20241022"
```

**用途**:
- 代码补全
- 快速问答
- 简单任务

**推荐设置**:
```toml
# 性能优先
small_fast_model = "claude-3-5-haiku-20241022"

# 如果不需要可以不设置
# small_fast_model = ""
```

## 🎯 配置模板

### Anthropic 官方 API

```toml
[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-YOUR_KEY_HERE"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"
```

### AnyRouter 代理

```toml
[anyrouter]
description = "AnyRouter AI 代理服务"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "YOUR_ANYROUTER_KEY"
model = "claude-sonnet-4-5-20250929"
```

### OpenRouter

```toml
[openrouter]
description = "OpenRouter 统一网关"
base_url = "https://openrouter.ai/api/v1"
auth_token = "sk-or-v1-YOUR_KEY"
model = "anthropic/claude-sonnet-4-5"
```

### 自建代理

```toml
[custom-proxy]
description = "自建代理服务器"
base_url = "http://localhost:8000"
auth_token = "custom-auth-token"
model = "claude-3-opus-20240229"
```

### 最小配置

```toml
[minimal]
base_url = "https://api.example.com"
auth_token = "your-token"
# description, model, small_fast_model 都是可选的
```

## 🔧 配置管理

### 添加新配置

#### 方式 1: 手动编辑

```bash
vim ~/.ccs_config.toml
```

添加新的配置节：
```toml
[new-config]
description = "新配置"
base_url = "https://api.new.com"
auth_token = "new-token"
```

保存后验证：
```bash
ccr validate
ccr list
```

#### 方式 2: 通过 Web 界面

```bash
ccr web
```

在浏览器中点击"➕ 添加配置"

### 修改配置

#### 方式 1: 手动编辑

```bash
vim ~/.ccs_config.toml
```

修改后验证：
```bash
ccr validate
```

#### 方式 2: 通过 Web 界面

在配置卡片上点击"编辑"按钮

### 删除配置

#### 方式 1: 手动编辑

从 `~/.ccs_config.toml` 中删除对应的配置节

#### 方式 2: 通过 Web 界面

在配置卡片上点击"删除"按钮

**限制**:
- ⚠️ 不能删除当前配置
- ⚠️ 不能删除默认配置

## 🛡️ 安全最佳实践

### 1. 文件权限

```bash
# 设置配置文件权限为 644（所有者读写，其他人只读）
chmod 644 ~/.ccs_config.toml

# 或者更严格（仅所有者读写）
chmod 600 ~/.ccs_config.toml
```

### 2. Git 忽略

如果配置文件在 Git 仓库中：

```bash
# .gitignore
.ccs_config.toml
**/ccs_config.toml
```

### 3. 环境变量替代

对于 CI/CD 环境，考虑使用环境变量：

```bash
export CCR_AUTH_TOKEN="$SECRET_TOKEN"
```

然后在配置文件中引用（未来功能）：
```toml
auth_token = "${CCR_AUTH_TOKEN}"
```

### 4. 密钥管理

**不推荐**:
```toml
# ❌ 明文硬编码
auth_token = "sk-ant-api03-real-key-12345"
```

**推荐**:
```bash
# ✅ 使用环境变量或密钥管理工具
# 1. 从密钥管理器获取
TOKEN=$(pass show anthropic/api-key)

# 2. 写入配置文件
sed -i "s/YOUR_KEY_HERE/$TOKEN/" ~/.ccs_config.toml

# 3. 或使用配置管理工具
```

## 📊 配置验证

### 验证规则

```rust
pub fn validate(&self) -> Result<()> {
    // 1. base_url 必须存在
    let base_url = self.base_url.as_ref()
        .ok_or(CcrError::ValidationError("base_url 不能为空"))?;
    
    // 2. base_url 格式检查
    if !base_url.starts_with("http://") && 
       !base_url.starts_with("https://") {
        return Err(CcrError::ValidationError(
            "base_url 必须以 http:// 或 https:// 开头"
        ));
    }
    
    // 3. auth_token 必须存在
    let auth_token = self.auth_token.as_ref()
        .ok_or(CcrError::ValidationError("auth_token 不能为空"))?;
    
    // 4. auth_token 不能为空字符串
    if auth_token.trim().is_empty() {
        return Err(CcrError::ValidationError("auth_token 不能为空"));
    }
    
    Ok(())
}
```

### 验证命令

```bash
ccr validate
```

**输出示例**:
```
配置验证报告

▶ 验证配置文件
✓ 配置文件存在

  ✓ anthropic
  ✓ anyrouter
  ✗ broken-config - base_url 不能为空

⚠ 配置节验证: 2 个通过, 1 个失败
```

## 🔗 相关文档

- [安装指南](/installation/)
- [环境变量](/installation/environment)
- [validate 命令](/commands/validate)
- [配置管理 API](/api/config)

