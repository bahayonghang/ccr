# Web API 参考

CCR 提供完整的 RESTful API，用于 Web 界面和第三方集成。

## 🌐 API 概览

### 基础信息

| 项目 | 值 |
|------|-----|
| **协议** | HTTP/1.1 |
| **默认端口** | 8080 |
| **内容类型** | application/json |
| **字符编码** | UTF-8 |
| **服务器** | tiny_http |

### 启动服务器

```bash
# 使用默认端口
ccr web

# 指定端口
ccr web --port 3000
```

### 基础 URL

```
http://localhost:8080
```

## 📚 API 端点

### 静态资源

#### GET /

**描述**: 返回 Web 管理界面

**响应**: HTML 页面

```bash
curl http://localhost:8080/
```

---

## 🔧 配置管理 API

### GET /api/configs

**描述**: 获取所有配置列表

**请求**:
```bash
curl http://localhost:8080/api/configs
```

**响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "current_config": "anthropic",
    "default_config": "anthropic",
    "configs": [
      {
        "name": "anthropic",
        "description": "Anthropic 官方 API",
        "base_url": "https://api.anthropic.com",
        "auth_token": "sk-a...here",
        "model": "claude-sonnet-4-5-20250929",
        "small_fast_model": "claude-3-5-haiku-20241022",
        "is_current": true,
        "is_default": true
      },
      {
        "name": "anyrouter",
        "description": "AnyRouter 代理服务",
        "base_url": "https://api.anyrouter.ai/v1",
        "auth_token": "your...here",
        "model": "claude-sonnet-4-5-20250929",
        "small_fast_model": null,
        "is_current": false,
        "is_default": false
      }
    ]
  }
}
```

**说明**:
- `auth_token` 已自动掩码
- `is_current` 标识当前活跃配置
- `is_default` 标识默认配置

---

### POST /api/switch

**描述**: 切换到指定配置

**请求**:
```bash
curl -X POST http://localhost:8080/api/switch \
  -H "Content-Type: application/json" \
  -d '{"config_name": "anyrouter"}'
```

**请求体**:
```json
{
  "config_name": "anyrouter"
}
```

**响应** (200 OK):
```json
{
  "success": true,
  "data": "配置切换成功"
}
```

**错误响应** (500):
```json
{
  "success": false,
  "message": "配置节 'nonexistent' 不存在\n建议: 运行 'ccr list' 查看可用配置"
}
```

**执行操作**:
1. 验证目标配置
2. 备份当前设置
3. 更新 settings.json
4. 更新 current_config
5. 记录操作历史

---

### POST /api/config

**描述**: 添加新配置

**请求**:
```bash
curl -X POST http://localhost:8080/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-config",
    "description": "新配置",
    "base_url": "https://api.example.com",
    "auth_token": "your-token",
    "model": "claude-sonnet-4-5-20250929",
    "small_fast_model": "claude-3-5-haiku-20241022"
  }'
```

**请求体**:
```json
{
  "name": "new-config",
  "description": "新配置描述（可选）",
  "base_url": "https://api.example.com",
  "auth_token": "your-api-token",
  "model": "model-name（可选）",
  "small_fast_model": "small-model（可选）"
}
```

**响应** (200 OK):
```json
{
  "success": true,
  "data": "配置添加成功"
}
```

**错误响应**:
```json
{
  "success": false,
  "message": "验证失败: base_url 不能为空"
}
```

---

### PUT /api/config/{name}

**描述**: 更新现有配置

**请求**:
```bash
curl -X PUT http://localhost:8080/api/config/anyrouter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "anyrouter-updated",
    "description": "更新的描述",
    "base_url": "https://api.anyrouter.ai/v1",
    "auth_token": "new-token",
    "model": "claude-sonnet-4-5-20250929"
  }'
```

**路径参数**:
- `{name}` - 要更新的配置名称

**请求体**: 与 POST /api/config 相同

**响应** (200 OK):
```json
{
  "success": true,
  "data": "配置更新成功"
}
```

**说明**:
- 如果 `name` 字段与路径参数不同，会重命名配置
- 会自动更新 `current_config` 和 `default_config` 的引用

---

### DELETE /api/config/{name}

**描述**: 删除配置

**请求**:
```bash
curl -X DELETE http://localhost:8080/api/config/old-config
```

**路径参数**:
- `{name}` - 要删除的配置名称

**响应** (200 OK):
```json
{
  "success": true,
  "data": "配置删除成功"
}
```

**错误响应**:
```json
{
  "success": false,
  "message": "不能删除当前配置"
}
```

**限制**:
- ❌ 不能删除当前配置
- ❌ 不能删除默认配置

---

## 📜 历史记录 API

### GET /api/history

**描述**: 获取操作历史记录

**请求**:
```bash
curl http://localhost:8080/api/history
```

**响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "total": 50,
    "entries": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2025-01-10T14:30:22+08:00",
        "operation": "切换配置",
        "actor": "user",
        "from_config": "anthropic",
        "to_config": "anyrouter",
        "changes": [
          {
            "key": "ANTHROPIC_BASE_URL",
            "old_value": "https://api.anthropic.com",
            "new_value": "https://api.anyrouter.ai/v1"
          },
          {
            "key": "ANTHROPIC_AUTH_TOKEN",
            "old_value": "sk-a...here",
            "new_value": "your...here"
          }
        ]
      }
    ]
  }
}
```

**说明**:
- 默认返回最近 50 条记录
- `changes` 中的敏感值已掩码
- 按时间倒序排列

---

## ✅ 验证 API

### POST /api/validate

**描述**: 验证所有配置的完整性

**请求**:
```bash
curl -X POST http://localhost:8080/api/validate
```

**响应** (200 OK):
```json
{
  "success": true,
  "data": "验证通过"
}
```

**错误响应** (500):
```json
{
  "success": false,
  "message": "配置验证失败: base_url 不能为空"
}
```

---

## 🔌 JavaScript 集成

### Fetch API 示例

```javascript
// 基础请求
async function getConfigs() {
    const response = await fetch('/api/configs');
    const data = await response.json();
    
    if (data.success) {
        console.log('配置列表:', data.data.configs);
    } else {
        console.error('错误:', data.message);
    }
}

// 切换配置
async function switchConfig(configName) {
    const response = await fetch('/api/switch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ config_name: configName })
    });
    
    const data = await response.json();
    return data;
}

// 添加配置
async function addConfig(config) {
    const response = await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config)
    });
    
    const data = await response.json();
    return data;
}

// 更新配置
async function updateConfig(oldName, newConfig) {
    const response = await fetch(`/api/config/${oldName}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newConfig)
    });
    
    const data = await response.json();
    return data;
}

// 删除配置
async function deleteConfig(configName) {
    const response = await fetch(`/api/config/${configName}`, {
        method: 'DELETE'
    });
    
    const data = await response.json();
    return data;
}

// 获取历史
async function getHistory() {
    const response = await fetch('/api/history');
    const data = await response.json();
    return data;
}
```

### 错误处理

```javascript
async function safeApiCall(apiFunc) {
    try {
        const data = await apiFunc();
        
        if (data.success) {
            return { ok: true, data: data.data };
        } else {
            return { ok: false, error: data.message };
        }
    } catch (error) {
        return { ok: false, error: `网络错误: ${error.message}` };
    }
}

// 使用
const result = await safeApiCall(() => switchConfig('anyrouter'));
if (result.ok) {
    console.log('成功:', result.data);
} else {
    console.error('失败:', result.error);
}
```

## 🔒 安全考虑

### 1. 本地访问

**当前**: 服务器绑定到 `0.0.0.0`，可从局域网访问

**风险**: 未授权访问

**建议**:
- 仅在可信网络使用
- 考虑添加身份认证（未来功能）
- 使用防火墙限制访问

### 2. 敏感信息

**当前**: API Token 自动掩码

**实现**:
```rust
auth_token: ColorOutput::mask_sensitive(
    &section.auth_token.clone().unwrap_or_default()
)
// sk-ant-1234567890abcdef → sk-a...cdef
```

### 3. CSRF 保护

**当前**: 无 CSRF 保护

**说明**: CCR Web API 主要用于本地访问，CSRF 风险较低

**未来**: 可能添加 CSRF Token

## 📊 API 响应格式

### 成功响应

```rust
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,    // true
    data: Option<T>,  // 响应数据
    message: Option<String>,  // null
}
```

### 错误响应

```rust
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,    // false
    data: Option<T>,  // null
    message: Option<String>,  // 错误信息
}
```

### HTTP 状态码

| 状态码 | 说明 | 场景 |
|--------|------|------|
| 200 | 成功 | API 调用成功 |
| 404 | 未找到 | 路由不存在 |
| 500 | 服务器错误 | 配置错误、文件锁错误等 |

## 🔗 相关文档

- [API 概览](/api/)
- [前端集成](/api/frontend)
- [Web 命令](/commands/web)

