# API 路由修复说明

## 问题描述

在从 Actix Web 迁移到 Axum 时，后端路由路径与前端 API 调用不匹配，导致出现 404 错误。

## 问题原因

前端 API 客户端使用 `baseURL: '/api'`，因此实际请求路径为：
- 前端调用 `/history` → 实际请求 `/api/history`
- 前端调用 `/configs` → 实际请求 `/api/configs`

但后端错误地配置为：
- `/api/configs/history` ❌
- `/api/configs/switch` ❌
- `/api/version/check` ❌
- `/api/mcp/servers` ❌

## 修复内容

### 1. 路由路径修正

| 原错误路径 | 修正后路径 | 状态 |
|----------|----------|------|
| `/api/configs/switch` | `/api/switch` | ✅ |
| `/api/configs/validate` | `/api/validate` | ✅ |
| `/api/configs/clean` | `/api/clean` | ✅ |
| `/api/configs/export` | `/api/export` | ✅ |
| `/api/configs/import` | `/api/import` | ✅ |
| `/api/configs/history` | `/api/history` | ✅ |
| `/api/version/check` | `/api/version/check-update` | ✅ |
| `/api/mcp/servers` | `/api/mcp` | ✅ |
| `/api/mcp/servers/:name` | `/api/mcp/:name` | ✅ |

### 2. HTTP 方法修正

前端使用 `PUT` 方法进行更新操作，需要将后端的 `PATCH` 改为 `PUT`：

```rust
// 修改前
.route("/api/mcp/:name", patch(handlers::mcp::update_mcp_server))

// 修改后  
.route("/api/mcp/:name", put(handlers::mcp::update_mcp_server))
```

受影响的端点：
- `/api/mcp/:name` - PUT
- `/api/mcp/:name/toggle` - PUT
- `/api/slash-commands/:name` - PUT
- `/api/slash-commands/:name/toggle` - PUT
- `/api/agents/:name` - PUT
- `/api/agents/:name/toggle` - PUT
- `/api/plugins/:id` - PUT
- `/api/plugins/:id/toggle` - PUT
- `/api/configs/:name` - PUT

### 3. 添加必要的导入

```rust
use axum::{
    routing::{delete, get, patch, post, put},  // 添加 put
    Router,
};
```

## 验证结果

所有关键 API 端点测试通过：

```bash
✅ /api/history            - 历史记录
✅ /api/configs            - 配置列表
✅ /api/system             - 系统信息
✅ /api/version            - 版本信息
✅ /api/version/check-update - 检查更新
✅ /api/switch             - 切换配置
✅ /api/mcp                - MCP 服务器
✅ /api/slash-commands     - Slash 命令
✅ /api/agents             - 代理
✅ /api/plugins            - 插件
```

## 完整 API 路由列表

### 配置管理
```
GET    /api/configs          - 列出所有配置
POST   /api/switch           - 切换配置
GET    /api/validate         - 验证配置
POST   /api/clean            - 清理备份
POST   /api/export           - 导出配置
POST   /api/import           - 导入配置
GET    /api/history          - 操作历史
POST   /api/configs          - 添加配置
PUT    /api/configs/:name    - 更新配置
DELETE /api/configs/:name    - 删除配置
```

### 命令执行
```
POST   /api/command/execute        - 执行命令
GET    /api/command/list           - 命令列表
GET    /api/command/help/:command  - 命令帮助
```

### 系统和版本
```
GET    /health                     - 健康检查
GET    /api/system                 - 系统信息
GET    /api/version                - 版本信息
GET    /api/version/check-update   - 检查更新
POST   /api/version/update         - 执行更新
```

### MCP 服务器
```
GET    /api/mcp              - 列出服务器
POST   /api/mcp              - 添加服务器
PUT    /api/mcp/:name        - 更新服务器
DELETE /api/mcp/:name        - 删除服务器
PUT    /api/mcp/:name/toggle - 切换状态
```

### Slash 命令
```
GET    /api/slash-commands        - 列出命令
POST   /api/slash-commands        - 添加命令
PUT    /api/slash-commands/:name  - 更新命令
DELETE /api/slash-commands/:name  - 删除命令
PUT    /api/slash-commands/:name/toggle - 切换状态
```

### 代理
```
GET    /api/agents          - 列出代理
POST   /api/agents          - 添加代理
PUT    /api/agents/:name    - 更新代理
DELETE /api/agents/:name    - 删除代理
PUT    /api/agents/:name/toggle - 切换状态
```

### 插件
```
GET    /api/plugins       - 列出插件
POST   /api/plugins       - 添加插件
PUT    /api/plugins/:id   - 更新插件
DELETE /api/plugins/:id   - 删除插件
PUT    /api/plugins/:id/toggle - 切换状态
```

## 文件变更

**修改的文件：**
- `ccr-ui/backend/src/main.rs`
  - 修正所有路由路径
  - 将 PATCH 改为 PUT
  - 添加 put 导入
  - 更新启动日志

## 测试方法

```bash
# 启动服务器
cd ccr-ui/backend
cargo run --release -- --port 8082

# 测试关键端点
curl http://127.0.0.1:8082/health
curl http://127.0.0.1:8082/api/configs
curl http://127.0.0.1:8082/api/history
curl http://127.0.0.1:8082/api/version
```

## 影响范围

- ✅ 前端无需任何修改
- ✅ 所有 API 端点保持完全兼容
- ✅ 现有功能不受影响
- ✅ 编译和运行正常

---

**修复日期**: 2025-10-15  
**影响版本**: 0.2.0  
**状态**: ✅ 已修复并验证

