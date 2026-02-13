# 后端 API 文档

本文档详细介绍 CCR UI 后端提供的所有 REST API 接口。

> **📢 重要更新**: v1.2.0 版本已从 Actix Web 迁移到 Axum。API 接口保持向后兼容，但内部实现已优化。详见 [Axum 迁移说明](./MIGRATION_AXUM.md)。

## 📋 API 概览

### 基础信息

- **基础 URL**: `http://127.0.0.1:38081/api` (v3.16+ 端口修改为 38081)
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

## 💰 预算管理接口 (Budget API)

管理 AI 模型调用的预算和成本限制。

### 获取预算配置

获取当前的预算配置。

**接口信息**
- **URL**: `/budget`
- **方法**: `GET`

**响应示例**
```json
{
  "enabled": true,
  "daily_limit": 50.00,
  "weekly_limit": 200.00,
  "monthly_limit": 500.00,
  "current_usage": {
    "daily": 12.34,
    "weekly": 85.67,
    "monthly": 127.45
  },
  "alerts": {
    "daily_threshold": 80,
    "weekly_threshold": 80,
    "monthly_threshold": 90
  }
}
```

### 更新预算配置

更新预算限制和警告阈值。

**接口信息**
- **URL**: `/budget`
- **方法**: `PUT`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "enabled": true,
  "daily_limit": 100.00,
  "weekly_limit": 500.00,
  "monthly_limit": 1500.00,
  "alerts": {
    "daily_threshold": 80,
    "weekly_threshold": 80,
    "monthly_threshold": 90
  }
}
```

### 检查预算状态

检查当前使用是否超出预算限制。

**接口信息**
- **URL**: `/budget/check`
- **方法**: `GET`

**响应示例**
```json
{
  "within_budget": true,
  "warnings": [],
  "usage_percentage": {
    "daily": 24.68,
    "weekly": 42.84,
    "monthly": 25.49
  }
}
```

## 💲 价格查询接口 (Pricing API)

查询 AI 模型的实时价格信息。

### 获取模型价格列表

获取所有支持模型的价格信息。

**接口信息**
- **URL**: `/pricing`
- **方法**: `GET`

**响应示例**
```json
{
  "models": [
    {
      "model_id": "claude-3-5-sonnet-20241022",
      "model_name": "Claude 3.5 Sonnet",
      "provider": "anthropic",
      "pricing": {
        "input_per_million": 3.00,
        "output_per_million": 15.00,
        "cache_write_per_million": 3.75,
        "cache_read_per_million": 0.30
      },
      "context_window": 200000,
      "max_output": 8192
    }
  ],
  "last_updated": "2024-12-22T10:00:00Z"
}
```

### 计算成本估算

根据 Token 数量估算调用成本。

**接口信息**
- **URL**: `/pricing/estimate`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "model": "claude-3-5-sonnet-20241022",
  "input_tokens": 10000,
  "output_tokens": 5000,
  "cache_read_tokens": 2000,
  "cache_write_tokens": 1000
}
```

**响应示例**
```json
{
  "model": "claude-3-5-sonnet-20241022",
  "breakdown": {
    "input_cost": 0.03,
    "output_cost": 0.075,
    "cache_read_cost": 0.0006,
    "cache_write_cost": 0.00375
  },
  "total_cost": 0.10935,
  "currency": "USD"
}
```

## 📊 用量统计接口 (Usage API)

查询详细的 API 用量和使用统计。

### 获取用量概览

获取指定时间范围的用量统计。

**接口信息**
- **URL**: `/usage`
- **方法**: `GET`

**查询参数**
- `range` (可选): 时间范围 (`today`/`week`/`month`)
- `model` (可选): 按模型筛选

**响应示例**
```json
{
  "total_requests": 1234,
  "total_tokens": {
    "input": 15200000,
    "output": 8300000,
    "cache_read": 3100000,
    "cache_write": 1500000
  },
  "by_model": {
    "claude-3-5-sonnet-20241022": {
      "requests": 850,
      "input_tokens": 10000000,
      "output_tokens": 5000000
    }
  },
  "by_project": {
    "/path/to/project": {
      "requests": 450,
      "total_cost": 45.00
    }
  },
  "cache_efficiency": 72.45
}
```

### 获取会话详情

获取指定会话的详细用量信息。

**接口信息**
- **URL**: `/usage/sessions/{session_id}`
- **方法**: `GET`

**响应示例**
```json
{
  "session_id": "sess_abc123",
  "created_at": "2024-12-22T08:00:00Z",
  "total_cost": 25.50,
  "requests": 45,
  "models_used": ["claude-3-5-sonnet-20241022"],
  "token_usage": {
    "input": 500000,
    "output": 250000,
    "cache_read": 100000
  }
}
```

## 🌐 平台管理接口 (Platform API)

管理多个 AI CLI 平台（Claude Code、Codex、Gemini CLI 等）。

### 获取平台列表

获取所有支持的平台。

**接口信息**
- **URL**: `/platform`
- **方法**: `GET`

**响应示例**
```json
{
  "platforms": [
    {
      "name": "claude",
      "display_name": "Claude Code",
      "enabled": true,
      "current": true,
      "config_path": "~/.claude/config.json",
      "version": "0.8.0"
    },
    {
      "name": "codex",
      "display_name": "Codex",
      "enabled": true,
      "current": false,
      "config_path": "~/.codex/config.json"
    }
  ],
  "current_platform": "claude"
}
```

### 切换平台

切换到指定平台。

**接口信息**
- **URL**: `/platform/switch`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "platform": "gemini"
}
```

### 获取当前平台

获取当前活跃的平台。

**接口信息**
- **URL**: `/platform/current`
- **方法**: `GET`

**响应示例**
```json
{
  "name": "claude",
  "display_name": "Claude Code",
  "config_path": "~/.claude/config.json",
  "version": "0.8.0"
}
```

### 初始化平台

初始化指定平台的配置。

**接口信息**
- **URL**: `/platform/{name}/init`
- **方法**: `POST`

## ☁️ WebDAV 同步接口 (Sync API)

管理 WebDAV 云端同步功能。

### 获取同步配置

获取 WebDAV 同步配置和文件夹列表。

**接口信息**
- **URL**: `/sync/config`
- **方法**: `GET`

**响应示例**
```json
{
  "webdav": {
    "url": "https://dav.jianguoyun.com/dav/",
    "username": "user@example.com",
    "connected": true
  },
  "folders": [
    {
      "name": "claude",
      "local_path": "~/.claude",
      "remote_path": "/ccr/claude",
      "enabled": true,
      "last_sync": "2024-12-22T10:00:00Z",
      "status": "synced"
    }
  ]
}
```

### 推送文件夹

推送本地文件到 WebDAV。

**接口信息**
- **URL**: `/sync/push`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "folder_name": "claude",
  "force": false
}
```

### 拉取文件夹

从 WebDAV 拉取文件到本地。

**接口信息**
- **URL**: `/sync/pull`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "folder_name": "claude",
  "force": false
}
```

### 检查同步状态

检查指定文件夹的同步状态。

**接口信息**
- **URL**: `/sync/status/{folder_name}`
- **方法**: `GET`

**响应示例**
```json
{
  "folder_name": "claude",
  "local_files": 45,
  "remote_files": 45,
  "conflicts": 0,
  "status": "synced",
  "last_sync": "2024-12-22T10:00:00Z"
}
```

### 批量同步

同步所有启用的文件夹。

**接口信息**
- **URL**: `/sync/batch`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "direction": "push",
  "force": false
}
```

## 🖥️ 系统信息接口 (System API)

查询系统状态和资源使用情况。

### 获取系统信息

获取完整的系统信息。

**接口信息**
- **URL**: `/system/info`
- **方法**: `GET`

**响应示例**
```json
{
  "version": "3.9.4",
  "platform": {
    "os": "Linux",
    "arch": "x86_64",
    "hostname": "dev-machine"
  },
  "runtime": {
    "uptime_seconds": 86400,
    "rust_version": "1.75.0"
  },
  "resources": {
    "cpu_usage": 25.5,
    "memory": {
      "total_mb": 16384,
      "used_mb": 8192,
      "available_mb": 8192
    },
    "disk": {
      "total_gb": 512,
      "used_gb": 256,
      "available_gb": 256
    }
  },
  "paths": {
    "config_dir": "~/.ccr",
    "data_dir": "~/.ccr/data",
    "cache_dir": "~/.ccr/cache"
  }
}
```

### 获取资源使用情况

获取实时资源使用情况。

**接口信息**
- **URL**: `/system/resources`
- **方法**: `GET`

**响应示例**
```json
{
  "cpu_usage": 25.5,
  "memory_usage_mb": 8192,
  "memory_available_mb": 8192,
  "disk_usage_gb": 256,
  "disk_available_gb": 256
}
```

## 🎯 技能管理接口 (Skills API)

管理自定义技能，支持基础 CRUD 操作。

### 获取技能列表

获取当前平台的已安装技能。

**接口信息**
- **URL**: `/skills`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": [
    {
      "name": "code-review",
      "description": "执行代码质量审查",
      "path": "~/.claude/commands/code-review",
      "instruction": "...",
      "metadata": {
        "author": "user",
        "version": "1.0.0",
        "tags": ["review", "quality"]
      },
      "is_remote": false
    }
  ]
}
```

### 创建技能

创建新的自定义技能。

**接口信息**
- **URL**: `/skills`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "name": "my-skill",
  "instruction": "技能指令内容..."
}
```

### 删除技能

删除指定名称的技能。

**接口信息**
- **URL**: `/skills/{name}`
- **方法**: `DELETE`

### 获取技能仓库列表

获取所有已配置的技能仓库。

**接口信息**
- **URL**: `/skills/repositories`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": [
    {
      "name": "official",
      "url": "https://github.com/org/skills-repo",
      "branch": "main",
      "description": "官方技能仓库",
      "skill_count": 25,
      "last_synced": "2025-01-15T10:00:00Z",
      "is_official": true
    }
  ]
}
```

### 添加技能仓库

添加新的技能仓库。

**接口信息**
- **URL**: `/skills/repositories`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "name": "my-repo",
  "url": "https://github.com/user/skills",
  "branch": "main",
  "description": "自定义技能仓库"
}
```

### 移除技能仓库

移除指定名称的技能仓库。

**接口信息**
- **URL**: `/skills/repositories/{name}`
- **方法**: `DELETE`

### 扫描技能仓库

扫描并列出仓库中的所有技能。

**接口信息**
- **URL**: `/skills/repositories/{name}/scan`
- **方法**: `GET`

## 🏪 统一技能中心接口 (SkillHub API) (v4.0+)

跨平台统一技能管理中心，支持多源安装、市场浏览和批量操作。

> 📖 **前端指南**：[技能管理详细指南](/guide/skills)

### 获取平台代理列表

获取所有支持的 AI CLI 平台及状态。

**接口信息**
- **URL**: `/skill_hub/agents`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": [
    {
      "id": "claude-code",
      "display_name": "Claude Code",
      "global_skills_dir": "~/.claude/commands",
      "detected": true,
      "installed_count": 12
    },
    {
      "id": "codex",
      "display_name": "Codex",
      "global_skills_dir": "~/.codex/commands",
      "detected": true,
      "installed_count": 5
    },
    {
      "id": "gemini",
      "display_name": "Gemini CLI",
      "global_skills_dir": "~/.gemini/commands",
      "detected": false,
      "installed_count": 0
    }
  ]
}
```

### 获取平台已安装技能

获取指定平台已安装的技能列表。

**接口信息**
- **URL**: `/skill_hub/agents/{agent}/skills`
- **方法**: `GET`

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `agent` | string | 平台标识（如 `claude-code`、`codex`） |

**响应示例**
```json
{
  "success": true,
  "data": [
    {
      "name": "code-review",
      "description": "代码审查技能",
      "skill_dir": "~/.claude/commands/code-review"
    }
  ]
}
```

### 市场热门技能

获取 skills.sh 市场热门技能列表。

**接口信息**
- **URL**: `/skill_hub/marketplace/trending`
- **方法**: `GET`

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 返回数量（默认 50） |
| `page` | number | 否 | 页码（默认 1） |

**响应示例**
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "package": "anthropics/skill-code-review",
        "owner": "anthropics",
        "repo": "skill-code-review",
        "skill": "code-review",
        "skills_sh_url": "https://skills.sh/anthropics/skill-code-review"
      }
    ],
    "total": 120,
    "cached": true
  }
}
```

### 市场搜索

按关键词搜索市场技能。

**接口信息**
- **URL**: `/skill_hub/marketplace/search`
- **方法**: `GET`

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `q` | string | 是 | 搜索关键词 |
| `limit` | number | 否 | 返回数量（默认 20） |
| `page` | number | 否 | 页码（默认 1） |

### 刷新市场缓存

强制刷新 skills.sh 市场数据缓存。

**接口信息**
- **URL**: `/skill_hub/marketplace/refresh`
- **方法**: `POST`

### 安装技能

从市场安装技能到指定平台。

**接口信息**
- **URL**: `/skill_hub/install`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "package": "owner/repo",
  "agents": ["claude-code", "codex"],
  "force": false
}
```

**参数说明**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `package` | string | 是 | 包标识（owner/repo 格式） |
| `agents` | string[] | 否 | 目标平台列表（默认所有已检测平台） |
| `force` | boolean | 否 | 是否强制覆盖安装（默认 false） |

**响应示例**
```json
{
  "success": true,
  "data": {
    "results": [
      { "agent": "claude-code", "ok": true, "message": "Installed successfully" },
      { "agent": "codex", "ok": true, "message": "Installed successfully" }
    ]
  }
}
```

### 卸载技能

从指定平台卸载技能。

**接口信息**
- **URL**: `/skill_hub/remove`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "skill": "skill-name",
  "agents": ["claude-code"]
}
```

### 批量安装

一次安装多个技能包到目标平台。

**接口信息**
- **URL**: `/skill_hub/install/batch`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "packages": ["owner1/repo1", "owner2/repo2", "owner3/repo3"],
  "agents": ["claude-code", "codex"],
  "force": false
}
```

**响应示例**
```json
{
  "success": true,
  "data": {
    "total": 3,
    "successCount": 2,
    "failCount": 1,
    "results": [
      { "package": "owner1/repo1", "ok": true },
      { "package": "owner2/repo2", "ok": true },
      { "package": "owner3/repo3", "ok": false, "message": "Repository not found" }
    ]
  }
}
```

### GitHub URL 导入

从 GitHub 仓库 URL 导入技能。

**接口信息**
- **URL**: `/skill_hub/import/github`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "url": "https://github.com/owner/repo",
  "agents": ["claude-code", "gemini"],
  "force": false
}
```

**支持的 URL 格式**：
- `https://github.com/owner/repo` — 整个仓库
- `https://github.com/owner/repo/tree/branch/path` — 指定路径
- `owner/repo` — 简写格式

### 本地文件夹导入

从本地文件系统导入技能。

**接口信息**
- **URL**: `/skill_hub/import/local`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "sourcePath": "/path/to/skill-folder",
  "agents": ["claude-code"],
  "skillName": "my-custom-skill"
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `sourcePath` | string | 是 | 本地技能文件夹路径 |
| `agents` | string[] | 是 | 目标平台列表 |
| `skillName` | string | 否 | 自定义技能名（默认使用文件夹名） |

### npx 安装

通过 npx 执行 npm 包安装技能。

**接口信息**
- **URL**: `/skill_hub/import/npx`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "package": "@scope/skill-package",
  "agents": ["claude-code"],
  "global": false
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `package` | string | 是 | npm 包名 |
| `agents` | string[] | 是 | 目标平台列表 |
| `global` | boolean | 否 | 是否全局安装（默认 false） |

### npx 可用性检测

检测系统中 npx 是否可用。

**接口信息**
- **URL**: `/skill_hub/npx/status`
- **方法**: `GET`

**响应示例**
```json
{
  "success": true,
  "data": {
    "available": true,
    "version": "10.2.3",
    "path": "/usr/local/bin/npx"
  }
}
```

### 浏览文件夹

打开系统文件夹选择对话框（仅 Tauri 桌面模式可用）。

**接口信息**
- **URL**: `/skill_hub/browse`
- **方法**: `POST`

**响应示例**
```json
{
  "success": true,
  "data": "/Users/user/my-skills/custom-skill"
}
```

## 📝 内置提示词接口 (Builtin Prompts API)

管理系统内置的提示词模板。

### 获取内置提示词列表

获取所有内置提示词模板。

**接口信息**
- **URL**: `/builtin-prompts`
- **方法**: `GET`

**响应示例**
```json
{
  "prompts": [
    {
      "id": "code-review",
      "name": "代码审查",
      "description": "审查代码质量和最佳实践",
      "category": "development",
      "template": "Please review the following code...",
      "variables": ["code", "language"]
    }
  ],
  "categories": ["development", "writing", "analysis"]
}
```

### 获取提示词详情

获取指定提示词的详细信息。

**接口信息**
- **URL**: `/builtin-prompts/{id}`
- **方法**: `GET`

### 渲染提示词

使用变量渲染提示词模板。

**接口信息**
- **URL**: `/builtin-prompts/{id}/render`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "variables": {
    "code": "function example() { return true; }",
    "language": "JavaScript"
  }
}
```

## 📅 会话管理接口 (Sessions API)

管理 AI 对话会话。

### 获取会话列表

获取所有会话记录。

**接口信息**
- **URL**: `/sessions`
- **方法**: `GET`

**查询参数**
- `limit` (可选): 返回数量，默认 50
- `offset` (可选): 分页偏移量

**响应示例**
```json
{
  "sessions": [
    {
      "id": "sess_abc123",
      "created_at": "2024-12-22T08:00:00Z",
      "model": "claude-3-5-sonnet-20241022",
      "total_cost": 25.50,
      "message_count": 10,
      "status": "active"
    }
  ],
  "total": 100,
  "limit": 50,
  "offset": 0
}
```

### 获取会话详情

获取指定会话的详细信息。

**接口信息**
- **URL**: `/sessions/{id}`
- **方法**: `GET`

### 删除会话

删除指定会话。

**接口信息**
- **URL**: `/sessions/{id}`
- **方法**: `DELETE`

## 🏥 提供商健康检查接口 (Provider Health API)

检查 AI 服务提供商的健康状态。

### 检查所有提供商

检查所有配置的提供商健康状态。

**接口信息**
- **URL**: `/provider-health`
- **方法**: `GET`

**响应示例**
```json
{
  "providers": [
    {
      "name": "anthropic",
      "status": "healthy",
      "response_time_ms": 250,
      "last_check": "2024-12-22T10:00:00Z",
      "errors": []
    },
    {
      "name": "openai",
      "status": "degraded",
      "response_time_ms": 1500,
      "last_check": "2024-12-22T10:00:00Z",
      "errors": ["High latency detected"]
    }
  ]
}
```

### 检查单个提供商

检查指定提供商的健康状态。

**接口信息**
- **URL**: `/provider-health/{provider}`
- **方法**: `GET`

**响应示例**
```json
{
  "name": "anthropic",
  "status": "healthy",
  "response_time_ms": 250,
  "api_endpoint": "https://api.anthropic.com",
  "last_check": "2024-12-22T10:00:00Z",
  "uptime_percentage": 99.95
}
```

## 📅 签到管理接口 (v4.0+)

管理 AI 中转站的签到功能，支持 30+ 内置提供商、多账号管理、WAF/CF 绕过、CDK 充值、OAuth 引导登录。

---

### 提供商管理

#### 获取提供商列表

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
      "auth_prefix": "Bearer",
      "enabled": true,
      "created_at": "2026-01-15T10:00:00Z"
    }
  ]
}
```

#### 创建自定义提供商

添加自定义中转站配置。

**接口信息**
- **URL**: `/checkin/providers`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 提供商名称 |
| `base_url` | string | 是 | 站点基础 URL |
| `checkin_path` | string | 否 | 签到路径（默认 `/api/user/checkin`） |
| `balance_path` | string | 是 | 余额查询路径 |
| `user_info_path` | string | 是 | 用户信息路径 |
| `auth_header` | string | 是 | 认证头名称 |
| `auth_prefix` | string | 是 | 认证前缀 |

**请求示例**
```json
{
  "name": "My Provider",
  "base_url": "https://example.com",
  "checkin_path": "/api/user/checkin",
  "balance_path": "/api/user/self",
  "user_info_path": "/api/user/self",
  "auth_header": "Authorization",
  "auth_prefix": "Bearer"
}
```

#### 获取内置提供商列表

获取系统预置的 30+ 中转站配置，按分类返回。

**接口信息**
- **URL**: `/checkin/providers/builtin`
- **方法**: `GET`

**响应示例**
```json
{
  "providers": [
    {
      "id": "builtin-anyrouter",
      "name": "AnyRouter",
      "description": "AnyRouter 中转站，支持多种模型",
      "domain": "anyrouter.top",
      "base_url": "https://anyrouter.top",
      "checkin_path": "/api/user/sign_in",
      "balance_path": "/api/user/self",
      "user_info_path": "/api/user/self",
      "auth_header": "Authorization",
      "auth_prefix": "Bearer",
      "supports_checkin": true,
      "requires_waf_bypass": true,
      "requires_cf_clearance": false,
      "checkin_bugged": false,
      "icon": "🛡️",
      "category": "waf_required",
      "cdk_config": null,
      "oauth_config": {
        "github_client_id": "abc123",
        "linuxdo_client_id": "def456",
        "oauth_state_path": "/api/oauth/state"
      }
    }
  ]
}
```

**提供商分类（category）**

| 分类 | 说明 | 数量 |
|------|------|------|
| `standard` | 标准 NewAPI 站点 | 24 |
| `waf_required` | 需阿里云 WAF 绕过 | 1 |
| `cf_required` | 需 Cloudflare Clearance 绕过 | 4 |
| `special` | 特殊签到机制 | 2 |

#### 添加内置提供商

将内置提供商添加到用户配置中。

**接口信息**
- **URL**: `/checkin/providers/builtin/add`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "builtin_id": "builtin-wong"
}
```

#### 获取单个提供商

**接口信息**
- **URL**: `/checkin/providers/{id}`
- **方法**: `GET`

#### 更新提供商

**接口信息**
- **URL**: `/checkin/providers/{id}`
- **方法**: `PUT`
- **Content-Type**: `application/json`

#### 删除提供商

删除提供商及其下所有账号（级联删除）。

**接口信息**
- **URL**: `/checkin/providers/{id}`
- **方法**: `DELETE`

---

### 账号管理

#### 获取账号列表

获取所有签到账号。

**接口信息**
- **URL**: `/checkin/accounts`
- **方法**: `GET`

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `provider_id` | string | 否 | 按提供商筛选 |

**响应示例**
```json
{
  "accounts": [
    {
      "id": "acc_abc123",
      "provider_id": "provider_xyz",
      "provider_name": "Wong",
      "name": "主账号",
      "cookies_masked": "ses***; new***",
      "api_user": "user_123",
      "enabled": true,
      "created_at": "2026-01-15T10:00:00Z",
      "last_checkin_at": "2026-02-13T08:00:00Z",
      "last_balance_check_at": "2026-02-13T08:00:00Z",
      "latest_balance": 5000.0,
      "balance_currency": "积分",
      "total_quota": 10000.0,
      "total_consumed": 5000.0,
      "extra_config": null
    }
  ]
}
```

#### 创建签到账号

**接口信息**
- **URL**: `/checkin/accounts`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `provider_id` | string | 是 | 提供商 ID |
| `name` | string | 是 | 账号名称 |
| `cookies_json` | string | 是 | Cookie（JSON 格式，会加密存储） |
| `api_user` | string | 否 | API User 标识 |
| `extra_config` | object | 否 | 扩展配置（CDK 凭证等） |

**请求示例**
```json
{
  "provider_id": "provider_xyz",
  "name": "主账号",
  "cookies_json": "{\"session\": \"abc123\", \"new-api-user\": \"token\"}",
  "api_user": "user_123",
  "extra_config": null
}
```

#### 获取单个账号

**接口信息**
- **URL**: `/checkin/accounts/{id}`
- **方法**: `GET`

#### 获取账号 Dashboard

获取账号的签到统计、月历日历、余额趋势等详细数据。

**接口信息**
- **URL**: `/checkin/accounts/{id}/dashboard`
- **方法**: `GET`

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `year` | number | 否 | 日历年份（默认当前年） |
| `month` | number | 否 | 日历月份（默认当前月） |
| `days` | number | 否 | 趋势天数（默认 30） |

**响应示例**
```json
{
  "account": { "id": "acc_abc123", "name": "主账号", "..." : "..." },
  "streak": {
    "current_streak": 15,
    "longest_streak": 30,
    "total_check_in_days": 120
  },
  "calendar": {
    "days": [
      {
        "date": "2026-02-01",
        "is_checked_in": true,
        "income_increment": 1000.0,
        "current_balance": 5000.0
      }
    ]
  },
  "trend": {
    "data_points": [
      {
        "date": "2026-02-01",
        "total_quota": 10000.0,
        "income_increment": 1000.0,
        "current_balance": 5000.0,
        "is_checked_in": true
      }
    ]
  }
}
```

#### 更新账号

**接口信息**
- **URL**: `/checkin/accounts/{id}`
- **方法**: `PUT`
- **Content-Type**: `application/json`

#### 删除账号

**接口信息**
- **URL**: `/checkin/accounts/{id}`
- **方法**: `DELETE`

#### 获取解密 Cookie

获取账号的解密 Cookie（用于编辑）。

**接口信息**
- **URL**: `/checkin/accounts/{id}/cookies`
- **方法**: `GET`

**响应示例**
```json
{
  "cookies": "{\"session\": \"abc123\", \"new-api-user\": \"token\"}"
}
```

---

### 签到操作

#### 批量签到

执行批量签到，可指定账号列表。

**接口信息**
- **URL**: `/checkin/execute`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `account_ids` | string[] | 否 | 指定账号 ID 列表，不传则签到所有启用账号 |

**请求示例**
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
      "provider_name": "Wong",
      "status": "success",
      "message": "签到成功，获得 1000 积分",
      "reward_amount": 1000.0,
      "balance_after": 5000.0
    }
  ]
}
```

#### 单账号签到

**接口信息**
- **URL**: `/checkin/accounts/{id}/checkin`
- **方法**: `POST`

---

### CDK 充值

#### 执行 CDK 充值

获取 CDK 充值码并自动兑换。仅支持配置了 `cdk_config` 的提供商。

**接口信息**
- **URL**: `/checkin/accounts/{id}/topup`
- **方法**: `POST`

**响应示例**
```json
{
  "cdk_type": "runawaytime",
  "success": true,
  "message": "成功兑换 2 个充值码",
  "codes_found": 3,
  "codes_redeemed": 2,
  "failed_codes": ["INVALID_CODE"],
  "direct_reward": null
}
```

**CdkProviderConfig 结构**

| 字段 | 类型 | 说明 |
|------|------|------|
| `cdk_type` | string | CDK 类型（`runawaytime` / `b4u` / `x666`） |
| `cdk_source_url` | string | CDK 来源站点 URL |
| `topup_path` | string? | 充值路径（x666 为 null，直接到账） |
| `requires_cdk_cookies` | bool | 是否需要 CDK 站 Cookie |
| `requires_access_token` | bool | 是否需要 access_token（x666） |

---

### 余额查询

#### 查询余额

查询指定账号的当前余额。

**接口信息**
- **URL**: `/checkin/accounts/{id}/balance`
- **方法**: `POST`

**响应示例**
```json
{
  "balance": 5000.0,
  "currency": "积分",
  "total_quota": 10000.0,
  "total_consumed": 5000.0,
  "checked_at": "2026-02-13T10:00:00Z"
}
```

#### 获取余额历史

获取账号的历史余额快照。

**接口信息**
- **URL**: `/checkin/accounts/{id}/balance/history`
- **方法**: `GET`

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 返回数量（默认 30） |

**响应示例**
```json
{
  "history": [
    {
      "balance": 5000.0,
      "total_quota": 10000.0,
      "total_consumed": 5000.0,
      "checked_at": "2026-02-13T10:00:00Z"
    },
    {
      "balance": 4500.0,
      "total_quota": 9500.0,
      "total_consumed": 5000.0,
      "checked_at": "2026-02-12T10:00:00Z"
    }
  ]
}
```

---

### 签到记录

#### 获取所有记录

获取签到历史记录，支持分页和过滤。

**接口信息**
- **URL**: `/checkin/records`
- **方法**: `GET`

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 每页数量（默认 20） |
| `page` | number | 否 | 页码（默认 1） |
| `page_size` | number | 否 | 每页大小 |
| `status` | string | 否 | 按状态筛选（success/failed/already_checked_in） |
| `account_id` | string | 否 | 按账号筛选 |
| `provider_id` | string | 否 | 按提供商筛选 |
| `keyword` | string | 否 | 关键词搜索 |

#### 导出签到记录

以文件下载方式导出签到记录。

**接口信息**
- **URL**: `/checkin/records/export`
- **方法**: `GET`

#### 获取账号签到记录

获取指定账号的签到记录。

**接口信息**
- **URL**: `/checkin/accounts/{id}/records`
- **方法**: `GET`

---

### 统计

#### 获取今日统计

获取今日签到统计数据。

**接口信息**
- **URL**: `/checkin/stats/today`
- **方法**: `GET`

**响应示例**
```json
{
  "total_accounts": 10,
  "checked_in_count": 8,
  "pending_count": 2,
  "failed_count": 0,
  "last_checkin_at": "2026-02-13T08:00:00Z"
}
```

---

### 导入/导出

#### 导出签到配置

导出提供商和账号配置。

**接口信息**
- **URL**: `/checkin/export`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `include_plaintext_keys` | bool | 否 | 是否包含明文 Cookie（默认 false） |
| `providers_only` | bool | 否 | 是否只导出提供商（默认 false） |

**请求示例**
```json
{
  "include_plaintext_keys": false,
  "providers_only": false
}
```

#### 预览导入

上传导出数据预览将要导入的内容和冲突项。

**接口信息**
- **URL**: `/checkin/import/preview`
- **方法**: `POST`
- **Content-Type**: `application/json`

#### 执行导入

执行配置导入。

**接口信息**
- **URL**: `/checkin/import`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "data": { "...导出数据..." },
  "options": {
    "conflict_strategy": "skip"
  }
}
```

**conflict_strategy 选项**

| 策略 | 说明 |
|------|------|
| `skip` | 跳过冲突项，保留现有配置 |
| `overwrite` | 覆盖冲突项，使用导入的配置 |

---

### 连接测试

#### 测试账号连通性

验证账号配置是否正确、Cookie 是否有效。

**接口信息**
- **URL**: `/checkin/accounts/{id}/test`
- **方法**: `POST`

**响应示例**
```json
{
  "success": true,
  "message": "连接成功",
  "user_name": "test_user",
  "balance": 5000.0
}
```

---

### OAuth 引导登录

#### 获取 OAuth 授权 URL

获取 OAuth 授权链接，用于引导用户在浏览器中完成登录。

**接口信息**
- **URL**: `/checkin/oauth/authorize-url`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `provider_id` | string | 是 | 提供商 ID |
| `oauth_type` | string | 是 | OAuth 类型（`github` / `linuxdo`） |

**请求示例**
```json
{
  "provider_id": "provider_xyz",
  "oauth_type": "github"
}
```

**响应示例**
```json
{
  "authorize_url": "https://github.com/login/oauth/authorize?client_id=abc&state=xyz&scope=user:email",
  "extraction_guide": "授权完成后，请在浏览器中打开 DevTools → Application → Cookies，复制所有 Cookie 粘贴到下方输入框"
}
```

**OAuthProviderConfig 结构**

| 字段 | 类型 | 说明 |
|------|------|------|
| `github_client_id` | string? | GitHub OAuth client_id |
| `linuxdo_client_id` | string? | LinuxDo OAuth client_id |
| `oauth_state_path` | string | OAuth state 获取路径 |