# Web API 参考

CCR 内置轻量级 REST API 服务器（通过 `ccr web` 启动），默认监听端口 **19527**。

> ⚠️ `ccr web` 是旧版轻量 API，主要用于脚本/CI 等编程访问场景。日常浏览器管理推荐使用 `ccr ui`。

## 启动服务

```bash
# 默认启动（0.0.0.0:19527）
ccr web

# 自定义端口
ccr web --port 9000

# 不自动打开浏览器
ccr web --no-browser
```

## API 端点一览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/` | 主页（Web 界面） |
| GET | `/api/configs` | 列出所有配置 |
| POST | `/api/switch` | 切换配置 |
| POST | `/api/config` | 添加配置 |
| PUT | `/api/config/:name` | 更新配置 |
| DELETE | `/api/config/:name` | 删除配置 |
| PATCH | `/api/config/:name/enable` | 启用配置 |
| PATCH | `/api/config/:name/disable` | 禁用配置 |
| GET | `/api/history` | 获取历史记录 |
| POST | `/api/validate` | 验证配置 |
| POST | `/api/clean` | 清理备份 |
| GET | `/api/settings` | 获取设置 |
| GET | `/api/settings/backups` | 获取备份列表 |
| POST | `/api/settings/restore` | 恢复设置 |
| POST | `/api/export` | 导出配置 |
| POST | `/api/import` | 导入配置 |
| GET | `/api/sync/status` | 获取同步状态 |
| POST | `/api/sync/config` | 设置同步配置 |
| POST | `/api/sync/push` | 推送到云端 |
| POST | `/api/sync/pull` | 从云端拉取 |

## 端点详情

### 配置管理

#### GET /api/configs

列出所有配置。

```bash
curl http://localhost:19527/api/configs
```

**响应示例：**

```json
{
  "configs": [
    {
      "name": "anthropic",
      "description": "Anthropic Official API",
      "base_url": "https://api.anthropic.com",
      "auth_token": "sk-a...key",
      "model": "claude-sonnet-4-5-20250929",
      "small_fast_model": "claude-3-5-haiku-20241022",
      "is_current": true,
      "is_complete": true
    }
  ]
}
```

#### POST /api/switch

切换当前配置。

```bash
curl -X POST http://localhost:19527/api/switch \
  -H "Content-Type: application/json" \
  -d '{"config_name": "anyrouter"}'
```

**请求体：**

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `config_name` | string | 是 | 目标配置名称 |

**响应示例：**

```json
{
  "success": true,
  "message": "Configuration switched successfully"
}
```

#### POST /api/config

添加新配置。

```bash
curl -X POST http://localhost:19527/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "name": "newconfig",
    "description": "New Configuration",
    "base_url": "https://api.example.com",
    "auth_token": "your-token",
    "model": "claude-sonnet-4-5-20250929"
  }'
```

#### PUT /api/config/:name

更新指定配置。

```bash
curl -X PUT http://localhost:19527/api/config/anthropic \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated Description",
    "auth_token": "new-token"
  }'
```

#### DELETE /api/config/:name

删除指定配置。

```bash
curl -X DELETE http://localhost:19527/api/config/oldconfig
```

#### PATCH /api/config/:name/enable

启用指定配置。

```bash
curl -X PATCH http://localhost:19527/api/config/myconfig/enable
```

#### PATCH /api/config/:name/disable

禁用指定配置。

```bash
curl -X PATCH http://localhost:19527/api/config/myconfig/disable
```

### 历史与验证

#### GET /api/history

获取操作历史记录。

```bash
curl http://localhost:19527/api/history?limit=20
```

**查询参数：**

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `limit` | number | 20 | 返回记录数 |
| `type` | string | - | 操作类型过滤 |

#### POST /api/validate

验证所有配置完整性。

```bash
curl -X POST http://localhost:19527/api/validate
```

**响应示例：**

```json
{
  "valid": true,
  "errors": [],
  "warnings": []
}
```

#### POST /api/clean

清理旧备份文件。

```bash
curl -X POST http://localhost:19527/api/clean \
  -H "Content-Type: application/json" \
  -d '{"days": 7, "dry_run": false}'
```

**请求体：**

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `days` | number | 是 | 保留天数 |
| `dry_run` | boolean | 否 | 仅预览不删除 |

### 设置管理

#### GET /api/settings

获取当前设置信息。

```bash
curl http://localhost:19527/api/settings
```

#### GET /api/settings/backups

获取设置备份列表。

```bash
curl http://localhost:19527/api/settings/backups
```

#### POST /api/settings/restore

从备份恢复设置。

```bash
curl -X POST http://localhost:19527/api/settings/restore \
  -H "Content-Type: application/json" \
  -d '{"backup_name": "settings_20250101_120000.json"}'
```

### 导入导出

#### POST /api/export

导出配置到文件。

```bash
curl -X POST http://localhost:19527/api/export
```

#### POST /api/import

从文件导入配置。

```bash
curl -X POST http://localhost:19527/api/import \
  -H "Content-Type: application/json" \
  -d '{"merge": true, "backup": true}'
```

### 云同步

#### GET /api/sync/status

获取 WebDAV 同步状态。

```bash
curl http://localhost:19527/api/sync/status
```

#### POST /api/sync/config

设置 WebDAV 同步配置。

```bash
curl -X POST http://localhost:19527/api/sync/config \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://dav.example.com/dav/",
    "username": "user",
    "password": "pass",
    "remote_path": "/ccr/"
  }'
```

#### POST /api/sync/push

推送本地配置到云端。

```bash
curl -X POST http://localhost:19527/api/sync/push
```

#### POST /api/sync/pull

从云端拉取配置到本地。

```bash
curl -X POST http://localhost:19527/api/sync/pull
```

## 安全注意事项

::: warning 安全提示
Web API 默认监听 `0.0.0.0:19527`。生产环境建议：
- 使用 `--host 127.0.0.1` 限制为本地访问
- 通过 SSH 端口转发或 VPN 进行远程访问
- 使用反向代理（nginx/caddy）+ HTTPS
- 防火墙限制访问 IP
:::

## 相关文档

- [web 命令](/reference/commands/web) - Web 服务器启动选项
- [ui 命令](/reference/commands/ui) - 推荐的全栈 Web 界面
- [架构设计](/reference/architecture) - 了解 CCR 分层架构
