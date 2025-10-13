# 后端 API 文档

本文档详细介绍 CCR UI 后端提供的所有 REST API 接口。

## 📋 API 概览

### 基础信息

- **基础 URL**: `http://127.0.0.1:8081/api`
- **协议**: HTTP/1.1
- **数据格式**: JSON
- **字符编码**: UTF-8
- **超时时间**: 30 秒

### 通用响应格式

所有 API 响应都遵循统一的格式：

```json
{
  "success": boolean,
  "data": any | null,
  "error": string | null
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
  "data": [
    {
      "name": "default",
      "path": "/home/user/.claude/configs/default.toml",
      "is_active": true
    },
    {
      "name": "work",
      "path": "/home/user/.claude/configs/work.toml",
      "is_active": false
    },
    {
      "name": "personal",
      "path": "/home/user/.claude/configs/personal.toml",
      "is_active": false
    }
  ],
  "error": null
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 配置名称 |
| `path` | string | 配置文件路径 |
| `is_active` | boolean | 是否为当前激活配置 |

**错误响应**
```json
{
  "success": false,
  "data": null,
  "error": "Failed to execute CCR command: ccr: command not found"
}
```

### 切换配置

切换到指定的 CCR 配置。

**接口信息**
- **URL**: `/configs/switch`
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
curl -X POST http://127.0.0.1:8081/api/configs/switch \
  -H "Content-Type: application/json" \
  -d '{"config_name": "work"}'
```

**成功响应**
```json
{
  "success": true,
  "data": "Switched to config: work",
  "error": null
}
```

**错误响应**
```json
{
  "success": false,
  "data": null,
  "error": "Configuration 'invalid-config' not found"
}
```

### 验证配置

验证所有 CCR 配置的有效性。

**接口信息**
- **URL**: `/configs/validate`
- **方法**: `POST`

**请求示例**
```bash
curl -X POST http://127.0.0.1:8081/api/configs/validate
```

**成功响应**
```json
{
  "success": true,
  "data": {
    "valid": true,
    "message": "All configurations are valid"
  },
  "error": null
}
```

**验证失败响应**
```json
{
  "success": true,
  "data": {
    "valid": false,
    "errors": [
      "Configuration 'work' has invalid API key",
      "Configuration 'personal' is missing required field 'model'"
    ]
  },
  "error": null
}
```

## ⚡ 命令执行接口

### 执行命令

执行指定的 CCR 命令并返回结果。

**接口信息**
- **URL**: `/commands/execute`
- **方法**: `POST`
- **Content-Type**: `application/json`
- **超时**: 30 秒

**请求参数**
```json
{
  "command": "ccr",
  "args": ["list", "--verbose"]
}
```

**参数说明**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `command` | string | 是 | 要执行的命令 |
| `args` | string[] | 否 | 命令参数数组 |

**请求示例**
```bash
curl -X POST http://127.0.0.1:8081/api/commands/execute \
  -H "Content-Type: application/json" \
  -d '{
    "command": "ccr",
    "args": ["list"]
  }'
```

**成功响应**
```json
{
  "success": true,
  "data": {
    "success": true,
    "stdout": "* default (/home/user/.claude/configs/default.toml)\n  work (/home/user/.claude/configs/work.toml)\n  personal (/home/user/.claude/configs/personal.toml)\n",
    "stderr": "",
    "exit_code": 0,
    "execution_time_ms": 156
  },
  "error": null
}
```

**命令执行失败响应**
```json
{
  "success": true,
  "data": {
    "success": false,
    "stdout": "",
    "stderr": "Error: Configuration file not found\n",
    "exit_code": 1,
    "execution_time_ms": 89
  },
  "error": null
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | boolean | 命令是否执行成功 |
| `stdout` | string | 标准输出内容 |
| `stderr` | string | 标准错误输出内容 |
| `exit_code` | number \| null | 进程退出码 |
| `execution_time_ms` | number | 执行时间（毫秒） |

**超时响应**
```json
{
  "success": false,
  "data": null,
  "error": "Command timed out after 30s"
}
```

### 获取可用命令列表

获取所有可用的 CCR 命令及其描述。

**接口信息**
- **URL**: `/commands/list`
- **方法**: `GET`

**请求示例**
```bash
curl -X GET http://127.0.0.1:8081/api/commands/list
```

**响应示例**
```json
{
  "success": true,
  "data": [
    {
      "name": "list",
      "description": "List all available configurations",
      "usage": "ccr list [OPTIONS]",
      "examples": [
        "ccr list",
        "ccr list --verbose"
      ]
    },
    {
      "name": "switch",
      "description": "Switch to a different configuration",
      "usage": "ccr switch <CONFIG_NAME>",
      "examples": [
        "ccr switch work",
        "ccr switch personal"
      ]
    },
    {
      "name": "current",
      "description": "Show current active configuration",
      "usage": "ccr current",
      "examples": [
        "ccr current"
      ]
    }
  ],
  "error": null
}
```

## 🖥️ 系统信息接口

### 获取系统信息

获取服务器系统信息和 CCR 版本信息。

**接口信息**
- **URL**: `/system/info`
- **方法**: `GET`

**请求示例**
```bash
curl -X GET http://127.0.0.1:8081/api/system/info
```

**响应示例**
```json
{
  "success": true,
  "data": {
    "os": "linux",
    "arch": "x86_64",
    "cpu_count": 8,
    "username": "user",
    "ccr_version": "ccr 1.2.3",
    "server_version": "0.1.0",
    "uptime_seconds": 3600
  },
  "error": null
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `os` | string | 操作系统类型 |
| `arch` | string | 系统架构 |
| `cpu_count` | number | CPU 核心数 |
| `username` | string | 当前用户名 |
| `ccr_version` | string \| null | CCR 版本信息 |
| `server_version` | string | 服务器版本 |
| `uptime_seconds` | number | 服务器运行时间（秒） |

## 🔒 安全性

### 输入验证

所有输入参数都经过严格验证：

```rust
// 配置名称验证
fn validate_config_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Config name cannot be empty".to_string());
    }
    
    if name.len() > 100 {
        return Err("Config name too long".to_string());
    }
    
    // 只允许字母、数字、连字符和下划线
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("Config name contains invalid characters".to_string());
    }
    
    Ok(())
}

// 命令参数验证
fn validate_command_args(args: &[String]) -> Result<(), String> {
    for arg in args {
        // 防止命令注入
        if arg.contains(';') || arg.contains('|') || arg.contains('&') {
            return Err("Dangerous characters detected in arguments".to_string());
        }
        
        // 防止路径遍历
        if arg.contains("../") {
            return Err("Path traversal detected in arguments".to_string());
        }
    }
    
    Ok(())
}
```

### CORS 配置

```rust
use actix_cors::Cors;

fn configure_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:5173")  // 开发环境
        .allowed_origin("http://127.0.0.1:5173")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec!["Content-Type", "Authorization"])
        .max_age(3600)
}
```

### 速率限制

```rust
// 简单的内存速率限制器
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn is_allowed(&mut self, client_ip: &str) -> bool {
        let now = Instant::now();
        let requests = self.requests.entry(client_ip.to_string()).or_default();
        
        // 清理过期请求
        requests.retain(|&time| now.duration_since(time) < self.window);
        
        if requests.len() >= self.max_requests {
            false
        } else {
            requests.push(now);
            true
        }
    }
}
```

## 📊 错误处理

### 错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("CCR command failed: {0}")]
    CcrCommandError(String),
    
    #[error("Configuration not found: {0}")]
    ConfigNotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Command timeout")]
    CommandTimeout,
    
    #[error("System error: {0}")]
    SystemError(String),
}
```

### 错误响应映射

```rust
impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_message) = match self {
            ApiError::CcrCommandError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::ConfigNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::CommandTimeout => (StatusCode::REQUEST_TIMEOUT, "Request timeout".to_string()),
            ApiError::SystemError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        
        HttpResponse::build(status).json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(error_message),
        })
    }
}
```

## 📈 性能监控

### 请求日志

```rust
use actix_web::middleware::Logger;

// 在 main.rs 中配置
App::new()
    .wrap(Logger::new("%a %{User-Agent}i %r %s %b %D"))
```

### 执行时间统计

```rust
use std::time::Instant;

async fn execute_with_timing<F, T>(operation: F) -> (T, Duration)
where
    F: Future<Output = T>,
{
    let start = Instant::now();
    let result = operation.await;
    let duration = start.elapsed();
    (result, duration)
}
```

### 健康检查接口

```rust
#[actix_web::get("/health")]
async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}
```

## 🧪 API 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    
    #[actix_web::test]
    async fn test_get_configs() {
        let app = test::init_service(
            App::new().service(get_configs)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/configs")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
    
    #[actix_web::test]
    async fn test_switch_config() {
        let app = test::init_service(
            App::new().service(switch_config)
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/configs/switch")
            .set_json(&SwitchConfigRequest {
                config_name: "test".to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        // 根据实际情况验证响应
    }
}
```

### 集成测试

```bash
#!/bin/bash
# test_api.sh

BASE_URL="http://127.0.0.1:8081/api"

echo "Testing system info..."
curl -s "$BASE_URL/system/info" | jq .

echo "Testing config list..."
curl -s "$BASE_URL/configs" | jq .

echo "Testing command execution..."
curl -s -X POST "$BASE_URL/commands/execute" \
  -H "Content-Type: application/json" \
  -d '{"command": "ccr", "args": ["--version"]}' | jq .

echo "Testing config switch..."
curl -s -X POST "$BASE_URL/configs/switch" \
  -H "Content-Type: application/json" \
  -d '{"config_name": "default"}' | jq .
```

## 📚 使用示例

### JavaScript/TypeScript 客户端

```typescript
class CcrApiClient {
  private baseUrl: string;
  
  constructor(baseUrl: string = 'http://127.0.0.1:8081/api') {
    this.baseUrl = baseUrl;
  }
  
  async getConfigs(): Promise<Config[]> {
    const response = await fetch(`${this.baseUrl}/configs`);
    const data = await response.json();
    
    if (!data.success) {
      throw new Error(data.error);
    }
    
    return data.data;
  }
  
  async switchConfig(configName: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/configs/switch`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ config_name: configName }),
    });
    
    const data = await response.json();
    
    if (!data.success) {
      throw new Error(data.error);
    }
  }
  
  async executeCommand(command: string, args: string[] = []): Promise<CommandOutput> {
    const response = await fetch(`${this.baseUrl}/commands/execute`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ command, args }),
    });
    
    const data = await response.json();
    
    if (!data.success) {
      throw new Error(data.error);
    }
    
    return data.data;
  }
}
```

### Python 客户端

```python
import requests
from typing import List, Dict, Any, Optional

class CcrApiClient:
    def __init__(self, base_url: str = "http://127.0.0.1:8081/api"):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({"Content-Type": "application/json"})
    
    def get_configs(self) -> List[Dict[str, Any]]:
        response = self.session.get(f"{self.base_url}/configs")
        data = response.json()
        
        if not data["success"]:
            raise Exception(data["error"])
        
        return data["data"]
    
    def switch_config(self, config_name: str) -> None:
        response = self.session.post(
            f"{self.base_url}/configs/switch",
            json={"config_name": config_name}
        )
        data = response.json()
        
        if not data["success"]:
            raise Exception(data["error"])
    
    def execute_command(self, command: str, args: List[str] = None) -> Dict[str, Any]:
        if args is None:
            args = []
            
        response = self.session.post(
            f"{self.base_url}/commands/execute",
            json={"command": command, "args": args}
        )
        data = response.json()
        
        if not data["success"]:
            raise Exception(data["error"])
        
        return data["data"]
```

## 🚀 部署配置

### 环境变量

```bash
# 服务器配置
CCR_UI_PORT=8081
CCR_UI_HOST=127.0.0.1

# 日志级别
RUST_LOG=info

# CCR 命令路径（可选）
CCR_COMMAND_PATH=/usr/local/bin/ccr
```

### Docker 部署

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ccr-ui-backend /usr/local/bin/

EXPOSE 8081

CMD ["ccr-ui-backend", "--port", "8081"]
```

### Nginx 反向代理

```nginx
server {
    listen 80;
    server_name api.your-domain.com;
    
    location /api/ {
        proxy_pass http://127.0.0.1:8081/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 超时配置
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
}
```

这个 API 文档提供了完整的后端接口说明，包括请求格式、响应格式、错误处理、安全性考虑和使用示例，帮助前端开发者和其他系统集成者正确使用 CCR UI 的后端服务。