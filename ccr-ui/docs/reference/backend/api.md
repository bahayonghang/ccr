# 后端 API 文档

本文档详细介绍 CCR UI 后端提供的所有 REST API 接口。

> **📢 重要更新**: v1.2.0 版本已从 Actix Web 迁移到 Axum。API 接口保持向后兼容，但内部实现已优化。详见 [Axum 迁移说明](./MIGRATION_AXUM.md)。

## 📋 API 概览

### 基础信息

- **基础 URL**: `http://127.0.0.1:8081/api`
- **协议**: HTTP/1.1
- **数据格式**: JSON
- **字符编码**: UTF-8
- **超时时间**: 600 秒（10分钟，支持长时间编译更新）

### 通用响应格式

所有 API 响应都遵循统一的格式：

```json
{
  "success": boolean,
  "data": any | null,
  "message": string | null
}
```

### HTTP 状态码

| 状态码 | 说明 | 使用场景 |
|--------|------|----------|
| 200 | OK | 请求成功 |
| 400 | Bad Request | 请求参数错误 |
| 404 | Not Found | 资源不存在 |
| 500 | Internal Server Error | 服务器内部错误 |
| 408 | Request Timeout | 请求超时 |

### 功能模块概览

CCR UI 后端提供以下主要功能模块：

- **配置管理** - CCR 配置文件的增删改查和切换
- **命令执行** - 执行 CCR CLI 命令
- **系统信息** - 获取系统状态和资源使用情况
- **版本管理** - 检查和更新 CCR 版本
- **统计分析** - API 使用统计和成本追踪（新增）
- **MCP 服务器管理** - 管理 Claude MCP 服务器配置
- **斜杠命令管理** - 管理自定义斜杠命令
- **Agent 管理** - 管理 AI Agent 配置
- **插件管理** - 管理系统插件
- **历史记录** - 查看配置变更历史

## 🔧 配置管理接口

### 获取配置列表

获取所有可用的 CCR 配置。

**接口信息**
- **URL**: `/configs`
- **方法**: `GET`
- **认证**: 无需认证

**请求示例**
```bash
curl -X GET http://127.0.0.1:8081/api/configs
```

**响应示例**
```json
{
  "success": true,
  "data": {
    "current_config": "default",
    "default_config": "default",
    "configs": [
      {
        "name": "default",
        "description": "Default configuration",
        "base_url": "https://api.anthropic.com",
        "auth_token": "sk-ant-***",
        "model": "claude-3-5-sonnet-20241022",
        "small_fast_model": "claude-3-5-haiku-20241022",
        "is_current": true,
        "is_default": true,
        "provider": "anthropic",
        "provider_type": "anthropic",
        "account": "personal",
        "tags": ["default", "personal"]
      }
    ]
  },
  "message": null
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `current_config` | string | 当前激活的配置名称 |
| `default_config` | string | 默认配置名称 |
| `configs` | array | 配置列表 |
| `configs[].name` | string | 配置名称 |
| `configs[].description` | string | 配置描述 |
| `configs[].base_url` | string | API 基础 URL |
| `configs[].auth_token` | string | 认证令牌（已脱敏） |
| `configs[].model` | string | 主要模型 |
| `configs[].small_fast_model` | string | 快速模型 |
| `configs[].is_current` | boolean | 是否为当前配置 |
| `configs[].is_default` | boolean | 是否为默认配置 |
| `configs[].provider` | string | 提供商 |
| `configs[].provider_type` | string | 提供商类型 |
| `configs[].account` | string | 账户名称 |
| `configs[].tags` | array | 标签列表 |

### 切换配置

切换到指定的 CCR 配置。

**接口信息**
- **URL**: `/switch`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "config_name": "work"
}
```

**参数说明**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `config_name` | string | 是 | 要切换到的配置名称 |

**请求示例**
```bash
curl -X POST http://127.0.0.1:8081/api/switch \
  -H "Content-Type: application/json" \
  -d '{"config_name": "work"}'
```

**成功响应**
```json
{
  "success": true,
  "data": "Switched to config: work",
  "message": null
}
```
  "data": null,
  "error": "Configuration 'invalid-config' not found"
}

## 🔄 MCP 服务器管理接口

### 获取 MCP 服务器列表

获取所有配置的 MCP 服务器。

**接口信息**
- **URL**: `/mcp`
- **方法**: `GET`
- **认证**: 无需认证

**请求示例**
```bash
curl -X GET http://127.0.0.1:8081/api/mcp
```

**响应示例**
```json
{
  "success": true,
  "data": {
    "servers": [
      {
        "name": "filesystem",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/files"],
        "env": {},
        "disabled": false
      }
    ]
  },
  "message": null
}
```

### 添加 MCP 服务器

添加新的 MCP 服务器配置。

**接口信息**
- **URL**: `/mcp`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "name": "filesystem",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/files"],
  "env": {
    "NODE_ENV": "production"
  },
  "disabled": false
}
```

### 更新 MCP 服务器

更新现有的 MCP 服务器配置。

**接口信息**
- **URL**: `/mcp/{name}`
- **方法**: `PUT`
- **Content-Type**: `application/json`

### 删除 MCP 服务器

删除指定的 MCP 服务器。

**接口信息**
- **URL**: `/mcp/{name}`
- **方法**: `DELETE`

## ⚡ 斜杠命令管理接口

### 获取斜杠命令列表

获取所有配置的斜杠命令。

**接口信息**
- **URL**: `/slash-commands`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "commands": [
      {
        "name": "git-commit",
        "description": "Generate git commit message",
        "command": "git log --oneline -10",
        "disabled": false,
        "folder": ""
      }
    ],
    "folders": ["utils", "development"]
  },
  "message": null
}
```

### 添加斜杠命令

添加新的斜杠命令。

**接口信息**
- **URL**: `/slash-commands`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "name": "git-status",
  "description": "Show git status",
  "command": "git status --porcelain",
  "args": [],
  "disabled": false
}
```

### 更新斜杠命令

更新现有的斜杠命令。

**接口信息**
- **URL**: `/slash-commands/{name}`
- **方法**: `PUT`

### 删除斜杠命令

删除指定的斜杠命令。

**接口信息**
- **URL**: `/slash-commands/{name}`
- **方法**: `DELETE`

## 🤖 Agent 管理接口

### 获取 Agent 列表

获取所有配置的 AI Agent。

**接口信息**
- **URL**: `/agents`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "agents": [
      {
        "name": "code-reviewer",
        "model": "claude-3-5-sonnet-20241022",
        "tools": ["filesystem", "bash"],
        "system_prompt": "You are a code reviewer...",
        "disabled": false,
        "folder": "development"
      }
    ],
    "folders": ["development", "writing"]
  },
  "message": null
}
```

### 添加 Agent

添加新的 AI Agent。

**接口信息**
- **URL**: `/agents`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "name": "code-reviewer",
  "model": "claude-3-5-sonnet-20241022",
  "tools": ["filesystem", "bash"],
  "system_prompt": "You are a helpful code reviewer.",
  "disabled": false
}
```

### 更新 Agent

更新现有的 AI Agent。

**接口信息**
- **URL**: `/agents/{name}`
- **方法**: `PUT`

### 删除 Agent

删除指定的 AI Agent。

**接口信息**
- **URL**: `/agents/{name}`
- **方法**: `DELETE`

## 🔌 插件管理接口

### 获取插件列表

获取所有已安装的插件。

**接口信息**
- **URL**: `/plugins`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "plugins": [
      {
        "id": "git-integration",
        "name": "Git Integration",
        "version": "1.0.0",
        "enabled": true,
        "config": {
          "auto_commit": false
        }
      }
    ]
  },
  "message": null
}
```

### 添加插件

安装新的插件。

**接口信息**
- **URL**: `/plugins`
- **方法**: `POST`

### 更新插件

更新插件配置。

**接口信息**
- **URL**: `/plugins/{id}`
- **方法**: `PUT`

### 删除插件

卸载指定的插件。

**接口信息**
- **URL**: `/plugins/{id}`
- **方法**: `DELETE`

### 切换插件状态

启用或禁用插件。

**接口信息**
- **URL**: `/plugins/{id}/toggle`
- **方法**: `PUT`

## 📊 版本管理接口

### 获取版本信息

获取当前 CCR 版本信息。

**接口信息**
- **URL**: `/version`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "current_version": "0.8.0",
    "build_time": "2024-01-15T10:30:00Z",
    "git_commit": "abc123def"
  },
  "message": null
}
```

### 检查更新

检查是否有新版本可用。

**接口信息**
- **URL**: `/update/check`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "current_version": "0.8.0",
    "latest_version": "0.9.0",
    "has_update": true,
    "release_url": "https://github.com/user/ccr/releases/tag/v0.9.0",
    "release_notes": "Bug fixes and improvements",
    "published_at": "2024-01-20T12:00:00Z"
  },
  "message": null
}
```

### 执行更新

执行 CCR 更新。

**接口信息**
- **URL**: `/update/execute`
- **方法**: `POST`

## 🏥 健康检查接口

### 健康检查

检查服务器状态。

**接口信息**
- **URL**: `/health`
- **方法**: `GET`

**响应示例**
```
OK
```

## 📊 统计分析接口（新增）

### 获取成本概览

获取指定时间范围的成本统计概览。

**接口信息**
- **URL**: `/stats/cost`
- **方法**: `GET`
- **认证**: 无需认证

**查询参数**
- `range` (可选): 时间范围，可选值为 `today`、`week`、`month`，默认为 `today`

**请求示例**
```bash
curl -X GET "http://127.0.0.1:8081/api/stats/cost?range=month"
```

**响应示例**
```json
{
  "total_cost": 127.45,
  "record_count": 1234,
  "token_stats": {
    "total_input_tokens": 15200000,
    "total_output_tokens": 8300000,
    "total_cache_tokens": 3100000,
    "cache_efficiency": 72.45
  },
  "by_model": {
    "claude-3-5-sonnet-20241022": 85.20,
    "claude-3-5-haiku-20241022": 32.10,
    "claude-3-opus-20240229": 10.15
  },
  "by_project": {
    "/path/to/project-a": 45.00,
    "/path/to/project-b": 35.20,
    "/path/to/project-c": 28.00
  },
  "trend": [
    {
      "date": "2025-10-27",
      "cost": 12.3456,
      "count": 156
    },
    {
      "date": "2025-10-26",
      "cost": 8.9012,
      "count": 123
    }
  ]
}
```

### 获取今日成本

获取今日成本统计的快捷方式。

**接口信息**
- **URL**: `/stats/cost/today`
- **方法**: `GET`

**请求示例**
```bash
curl -X GET http://127.0.0.1:8081/api/stats/cost/today
```

### 获取本周成本

获取本周成本统计的快捷方式。

**接口信息**
- **URL**: `/stats/cost/week`
- **方法**: `GET`

### 获取本月成本

获取本月成本统计的快捷方式。

**接口信息**
- **URL**: `/stats/cost/month`
- **方法**: `GET`

### 获取成本趋势

获取成本趋势数据。

**接口信息**
- **URL**: `/stats/cost/trend`
- **方法**: `GET`

**查询参数**
- `range` (可选): 时间范围

**响应示例**
```json
[
  {
    "date": "2025-10-27",
    "cost": 12.3456,
    "count": 156
  },
  {
    "date": "2025-10-26",
    "cost": 8.9012,
    "count": 123
  }
]
```

### 按模型分组

获取按模型分组的成本统计。

**接口信息**
- **URL**: `/stats/cost/by-model`
- **方法**: `GET`

**查询参数**
- `range` (可选): 时间范围

**响应示例**
```json
{
  "claude-3-5-sonnet-20241022": 85.20,
  "claude-3-5-haiku-20241022": 32.10,
  "claude-3-opus-20240229": 10.15
}
```

### 按项目分组

获取按项目分组的成本统计。

**接口信息**
- **URL**: `/stats/cost/by-project`
- **方法**: `GET`

**查询参数**
- `range` (可选): 时间范围

**响应示例**
```json
{
  "/path/to/project-a": 45.00,
  "/path/to/project-b": 35.20,
  "/path/to/project-c": 28.00
}
```

### 获取 Top 会话

获取成本最高的会话列表。

**接口信息**
- **URL**: `/stats/cost/top-sessions`
- **方法**: `GET`

**查询参数**
- `limit` (可选): 返回的会话数量，默认为 10

**请求示例**
```bash
curl -X GET "http://127.0.0.1:8081/api/stats/cost/top-sessions?limit=20"
```

**响应示例**
```json
[
  {
    "session_id": "sess_abc123",
    "cost": 25.50
  },
  {
    "session_id": "sess_def456",
    "cost": 18.30
  }
]
```

### 获取统计摘要

获取快速统计摘要（今日/本周/本月成本）。

**接口信息**
- **URL**: `/stats/summary`
- **方法**: `GET`

**响应示例**
```json
{
  "today_cost": 12.34,
  "week_cost": 85.67,
  "month_cost": 127.45,
  "total_sessions": 1234
}
```

**数据说明**

- **total_cost**: 总成本（美元）
- **record_count**: API 调用次数
- **token_stats**: Token 使用统计
  - **total_input_tokens**: 输入 Token 总数
  - **total_output_tokens**: 输出 Token 总数
  - **total_cache_tokens**: Cache Token 总数
  - **cache_efficiency**: Cache 效率百分比
- **by_model**: 按模型分组的成本（模型名 → 成本）
- **by_project**: 按项目分组的成本（项目路径 → 成本）
- **trend**: 每日成本趋势数组
  - **date**: 日期（YYYY-MM-DD）
  - **cost**: 当日成本
  - **count**: 当日调用次数

## 📝 历史记录接口

### 获取历史记录

获取配置变更历史记录。

**接口信息**
- **URL**: `/history`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "entries": [
      {
        "id": "hist_001",
        "timestamp": "2024-01-15T10:30:00Z",
        "operation": "switch_config",
        "actor": "user",
        "from_config": "default",
        "to_config": "work",
        "changes": [
          {
            "key": "ANTHROPIC_API_KEY",
            "old_value": "sk-ant-***",
            "new_value": "sk-ant-***"
          }
        ]
      }
    ],
    "total": 1
  },
  "message": null
}
```

## 🧹 清理接口

### 清理备份文件

清理过期的备份文件。

**接口信息**
- **URL**: `/clean`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "days": 30,
  "dry_run": false
}
```

**响应示例**
```json
{
  "success": true,
  "data": {
    "deleted_count": 5,
    "skipped_count": 2,
    "total_size_mb": 12.5,
    "dry_run": false
  },
  "message": null
}
```

## 📤 导出接口

### 导出配置

导出当前配置。

**接口信息**
- **URL**: `/export`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "include_secrets": false
}
```

## 📥 导入接口

### 导入配置

导入配置文件。

**接口信息**
- **URL**: `/import`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "content": "配置文件内容",
  "mode": "merge",
  "backup": true
}
```

## 📅 签到管理接口 (v3.7+)

管理 AI 中转站的签到功能，支持多提供商、多账号管理。

### 获取提供商列表

获取所有已配置的签到提供商。

**接口信息**
- **URL**: `/checkin/providers`
- **方法**: `GET`

**响应示例**
```json
{
  "providers": [
    {
      "id": "provider_abc123",
      "name": "AnyRouter",
      "base_url": "https://anyrouter.top",
      "checkin_path": "/api/user/sign_in",
      "balance_path": "/api/user/self",
      "user_info_path": "/api/user/self",
      "auth_header": "Authorization",
      "auth_prefix": "Bearer ",
      "enabled": true,
      "created_at": "2024-12-22T10:00:00Z"
    }
  ]
}
```

### 获取内置提供商列表

获取系统预置的中转站配置。

**接口信息**
- **URL**: `/checkin/providers/builtin`
- **方法**: `GET`

**响应示例**
```json
{
  "providers": [
    {
      "id": "anyrouter",
      "name": "AnyRouter",
      "domain": "https://anyrouter.top",
      "base_url": "https://anyrouter.top",
      "checkin_path": "/api/user/sign_in",
      "balance_path": "/api/user/self",
      "user_info_path": "/api/user/self",
      "auth_header": "Authorization",
      "auth_prefix": "Bearer ",
      "icon": "🌐",
      "description": "AnyRouter 中转站，支持多种模型",
      "supports_checkin": true,
      "requires_waf_bypass": true,
      "checkin_bugged": false
    }
  ]
}
```

### 添加内置提供商

将内置提供商添加到用户配置中。

**接口信息**
- **URL**: `/checkin/providers/builtin/add`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "builtin_id": "anyrouter"
}
```

**响应示例**
```json
{
  "id": "provider_abc123",
  "name": "AnyRouter",
  "base_url": "https://anyrouter.top",
  "enabled": true,
  "created_at": "2024-12-22T10:00:00Z"
}
```

### 创建自定义提供商

添加自定义中转站配置。

**接口信息**
- **URL**: `/checkin/providers`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "name": "My Provider",
  "base_url": "https://example.com",
  "checkin_path": "/api/user/checkin",
  "balance_path": "/api/user/dashboard",
  "user_info_path": "/api/user/self",
  "auth_header": "Authorization",
  "auth_prefix": "Bearer "
}
```

### 获取账号列表

获取所有签到账号。

**接口信息**
- **URL**: `/checkin/accounts`
- **方法**: `GET`

**查询参数**
- `provider_id` (可选): 按提供商筛选

### 创建签到账号

为指定提供商添加账号。

**接口信息**
- **URL**: `/checkin/accounts`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "provider_id": "provider_abc123",
  "name": "主账号",
  "api_key": "sk-xxx...",
  "enabled": true
}
```

### 执行签到

执行批量签到或单个账号签到。

**接口信息**
- **URL**: `/checkin/execute`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**（可选）
```json
{
  "account_ids": ["acc_123", "acc_456"]
}
```

**响应示例**
```json
{
  "total": 3,
  "successful": 2,
  "failed": 1,
  "results": [
    {
      "account_id": "acc_123",
      "account_name": "主账号",
      "provider_name": "AnyRouter",
      "status": "Success",
      "message": "签到成功，获得 1000 积分"
    }
  ]
}
```

### 查询余额

查询指定账号的余额。

**接口信息**
- **URL**: `/checkin/accounts/{id}/balance`
- **方法**: `POST`

**响应示例**
```json
{
  "remaining_quota": 10000.50,
  "used_quota": 5000.25,
  "total_quota": 15000.75,
  "currency": "$",
  "usage_percentage": 33.33,
  "query_time": "2024-12-22T10:00:00Z"
}
```

### 获取签到记录

获取历史签到记录。

**接口信息**
- **URL**: `/checkin/records`
- **方法**: `GET`

**查询参数**
- `limit` (可选): 返回记录数量，默认 100

### 获取今日统计

获取今日签到统计数据。

**接口信息**
- **URL**: `/checkin/stats/today`
- **方法**: `GET`

**响应示例**
```json
{
  "total_accounts": 5,
  "checked_in_count": 3,
  "pending_count": 2,
  "failed_count": 0,
  "last_checkin_at": "2024-12-22T08:00:00Z"
}
```

### 导出签到配置

导出提供商和账号配置。

**接口信息**
- **URL**: `/checkin/export`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "include_plaintext_keys": false,
  "providers_only": false
}
```

### 导入签到配置

导入提供商和账号配置。

**接口信息**
- **URL**: `/checkin/import`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "data": { ... },
  "conflict_strategy": "skip"
}
```

**conflict_strategy 选项**
- `skip`: 跳过冲突项
- `overwrite`: 覆盖冲突项