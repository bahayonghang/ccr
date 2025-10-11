# web - 启动 Web 界面

启动内置的 Web 配置界面,通过浏览器管理配置。

## 用法

```bash
ccr web [OPTIONS]
```

## 选项

- `--port <PORT>`: 指定监听端口(默认：8080)
- `--no-browser`: 不自动打开浏览器

## 功能特性

- 图形化配置管理
- 实时配置切换
- 配置的添加/编辑/删除
- 查看操作历史
- 配置验证
- 备份清理

## 示例

```bash
# 使用默认端口 8080
ccr web

# 使用自定义端口
ccr web --port 3000

# 启动但不打开浏览器
ccr web --no-browser
```

## Web 界面功能

### 1. 配置管理

- **查看配置列表**：显示所有可用配置
- **切换配置**：点击按钮即可切换
- **添加配置**：通过表单添加新配置
- **编辑配置**：修改现有配置
- **删除配置**：删除不需要的配置

### 2. 实时状态

- **当前配置**：高亮显示当前使用的配置
- **配置状态**：实时显示配置完整性
- **环境变量**：查看当前环境变量设置

### 3. 历史记录

- **操作历史**：查看所有操作记录
- **过滤功能**：按类型过滤历史
- **详细信息**：查看每次操作的详细信息

### 4. 维护工具

- **配置验证**：一键验证所有配置
- **备份清理**：清理旧备份文件
- **导出/导入**：配置的导出和导入

## RESTful API

Web 界面提供完整的 RESTful API,支持编程访问。

### 获取所有配置

```http
GET http://localhost:8080/api/configs
```

**响应：**
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

### 切换配置

```http
POST http://localhost:8080/api/switch
Content-Type: application/json

{
  "config_name": "anyrouter"
}
```

**响应：**
```json
{
  "success": true,
  "message": "Configuration switched successfully"
}
```

### 获取操作历史

```http
GET http://localhost:8080/api/history?limit=20&type=switch
```

**查询参数：**
- `limit`: 返回记录数(默认：20)
- `type`: 操作类型过滤(可选)

### 验证配置

```http
POST http://localhost:8080/api/validate
```

**响应：**
```json
{
  "valid": true,
  "errors": [],
  "warnings": []
}
```

### 清理备份

```http
POST http://localhost:8080/api/clean
Content-Type: application/json

{
  "days": 7,
  "dry_run": false
}
```

**响应：**
```json
{
  "success": true,
  "deleted_count": 10,
  "freed_space": "5.2 MB"
}
```

### 添加配置

```http
POST http://localhost:8080/api/config
Content-Type: application/json

{
  "name": "newconfig",
  "description": "New Configuration",
  "base_url": "https://api.example.com",
  "auth_token": "your-token",
  "model": "claude-sonnet-4-5-20250929"
}
```

### 更新配置

```http
PUT http://localhost:8080/api/config/anthropic
Content-Type: application/json

{
  "description": "Updated Description",
  "auth_token": "new-token"
}
```

### 删除配置

```http
DELETE http://localhost:8080/api/config/oldconfig
```

## 使用场景

### 团队协作

在团队环境中使用 Web 界面统一管理配置：

```bash
# 在服务器上启动
ccr web --port 8080 --no-browser
```

团队成员可以通过浏览器访问 `http://server-ip:8080` 进行配置管理。

### 远程管理

通过 SSH 端口转发远程管理配置：

```bash
# 在远程服务器上
ccr web --no-browser

# 在本地
ssh -L 8080:localhost:8080 user@remote-server
# 访问 http://localhost:8080
```

### 自动化集成

使用 API 进行自动化操作：

```bash
# 使用 curl 切换配置
curl -X POST http://localhost:8080/api/switch \
  -H "Content-Type: application/json" \
  -d '{"config_name": "production"}'

# 使用 Python
import requests
response = requests.post(
    "http://localhost:8080/api/switch",
    json={"config_name": "production"}
)
print(response.json())
```

## 安全注意事项

::: warning 安全提示
Web 界面默认监听 `localhost`,仅本地访问。如需远程访问,请使用：
- SSH 端口转发
- VPN 连接
- 反向代理(nginx/caddy)+ HTTPS
- 防火墙限制访问 IP
:::

### 使用 nginx 反向代理

```nginx
server {
    listen 443 ssl;
    server_name ccr.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 停止 Web 服务

按 `Ctrl+C` 停止 Web 服务：

```
^C
Shutting down web server...
✓ Web server stopped
```

## 相关命令

- [switch](./switch) - 命令行切换配置
- [list](./list) - 命令行查看配置
- [history](./history) - 命令行查看历史
