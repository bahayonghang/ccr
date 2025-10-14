# 后端开发指南

本文档提供 CCR UI 后端开发的完整指南，包括环境搭建、开发流程、代码规范和最佳实践。

## 🚀 快速开始

### 系统要求

**必需软件**:
- **Rust**: 1.75+ (推荐使用 rustup 安装)
- **PostgreSQL**: 14+ 
- **Git**: 2.30+
- **Docker**: 20.10+ (可选，用于容器化部署)

**推荐工具**:
- **IDE**: VS Code + rust-analyzer 插件
- **数据库工具**: pgAdmin 或 DBeaver
- **API 测试**: Postman 或 Insomnia
- **性能分析**: cargo-flamegraph

### 环境搭建

#### 1. 安装 Rust

```bash
# 安装 rustup (Rust 工具链管理器)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.bashrc

# 验证安装
rustc --version
cargo --version

# 安装必要组件
rustup component add rustfmt clippy
```

#### 2. 安装开发工具

```bash
# 安装 cargo 扩展工具
cargo install cargo-watch      # 文件监控和自动重启
cargo install cargo-edit       # 依赖管理
cargo install sqlx-cli         # 数据库迁移工具
cargo install cargo-audit      # 安全审计
cargo install cargo-outdated   # 依赖更新检查
```

#### 3. 数据库设置

**PostgreSQL 安装** (Ubuntu/Debian):
```bash
# 安装 PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib

# 启动服务
sudo systemctl start postgresql
sudo systemctl enable postgresql

# 创建数据库用户
sudo -u postgres createuser --interactive ccr_user
sudo -u postgres createdb ccr_ui_db -O ccr_user

# 设置密码
sudo -u postgres psql -c "ALTER USER ccr_user PASSWORD 'your_password';"
```

**使用 Docker** (推荐):
```bash
# 启动 PostgreSQL 容器
docker run --name ccr-postgres \
  -e POSTGRES_DB=ccr_ui_db \
  -e POSTGRES_USER=ccr_user \
  -e POSTGRES_PASSWORD=your_password \
  -p 5432:5432 \
  -d postgres:15

# 验证连接
docker exec -it ccr-postgres psql -U ccr_user -d ccr_ui_db -c "SELECT version();"
```

#### 4. 项目初始化

```bash
# 克隆项目
git clone https://github.com/your-org/ccr-ui-backend.git
cd ccr-ui-backend

# 复制环境配置
cp .env.example .env

# 编辑配置文件
nano .env
```

**环境配置** (`.env`):
```env
# 数据库配置
DATABASE_URL=postgresql://ccr_user:your_password@localhost:5432/ccr_ui_db

# 服务器配置
HOST=0.0.0.0
PORT=8080

# 日志级别
RUST_LOG=debug

# CORS 配置
CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000

# JWT 密钥 (如果使用认证)
JWT_SECRET=your-super-secret-key-here

# MCP 服务器配置
MCP_SERVERS_DIR=./mcp-servers
MCP_TIMEOUT=30000
```

#### 5. 数据库迁移

```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# 运行迁移
sqlx database create
sqlx migrate run

# 验证迁移
sqlx migrate info
```

#### 6. 启动开发服务器

```bash
# 安装依赖并启动
cargo run

# 或使用 cargo-watch 进行热重载
cargo watch -x run

# 验证服务
curl http://localhost:8080/api/system/status
```

## 🏗️ 项目结构

```
ccr-ui-backend/
├── src/
│   ├── main.rs                 # 应用入口点
│   ├── lib.rs                  # 库根模块
│   ├── config/                 # 配置管理
│   │   ├── mod.rs
│   │   ├── app_config.rs       # 应用配置
│   │   └── database.rs         # 数据库配置
│   ├── handlers/               # HTTP 处理器
│   │   ├── mod.rs
│   │   ├── configs.rs          # 配置管理接口
│   │   ├── commands.rs         # 命令执行接口
│   │   ├── system.rs           # 系统状态接口
│   │   └── mcp.rs              # MCP 服务器接口
│   ├── services/               # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── config_service.rs   # 配置服务
│   │   ├── command_service.rs  # 命令服务
│   │   └── mcp_service.rs      # MCP 服务
│   ├── repositories/           # 数据访问层
│   │   ├── mod.rs
│   │   ├── config_repo.rs      # 配置仓库
│   │   └── command_repo.rs     # 命令历史仓库
│   ├── models/                 # 数据模型
│   │   ├── mod.rs
│   │   ├── config.rs           # 配置模型
│   │   ├── command.rs          # 命令模型
│   │   └── response.rs         # 响应模型
│   ├── middleware/             # 中间件
│   │   ├── mod.rs
│   │   ├── cors.rs             # CORS 处理
│   │   ├── logging.rs          # 请求日志
│   │   └── auth.rs             # 认证中间件
│   ├── utils/                  # 工具函数
│   │   ├── mod.rs
│   │   ├── validation.rs       # 数据验证
│   │   └── crypto.rs           # 加密工具
│   └── errors/                 # 错误处理
│       ├── mod.rs
│       └── app_error.rs        # 应用错误类型
├── migrations/                 # 数据库迁移
│   ├── 001_initial.sql
│   ├── 002_add_configs.sql
│   └── 003_add_commands.sql
├── tests/                      # 测试文件
│   ├── integration/
│   │   ├── configs_test.rs
│   │   └── commands_test.rs
│   └── common/
│       └── mod.rs
├── docker/                     # Docker 配置
│   ├── Dockerfile
│   └── docker-compose.yml
├── scripts/                    # 脚本文件
│   ├── setup.sh               # 环境设置脚本
│   └── deploy.sh              # 部署脚本
├── Cargo.toml                 # 项目配置
├── Cargo.lock                 # 依赖锁定
├── .env.example               # 环境变量示例
├── .gitignore                 # Git 忽略文件
├── README.md                  # 项目说明
└── justfile                   # Just 命令文件
```

## 🔧 开发工作流

### 日常开发

#### 1. 启动开发环境

```bash
# 启动数据库 (如果使用 Docker)
docker-compose up -d postgres

# 启动开发服务器
just dev
# 或
cargo watch -x run
```

#### 2. 代码开发

**创建新功能**:
```bash
# 创建功能分支
git checkout -b feature/new-api-endpoint

# 编写代码...

# 运行测试
just test

# 代码格式化
just fmt

# 代码检查
just lint
```

**添加新的 API 端点**:

1. **定义模型** (`src/models/`):
```rust
// src/models/user.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}
```

2. **创建仓库** (`src/repositories/`):
```rust
// src/repositories/user_repo.rs
use async_trait::async_trait;
use sqlx::PgPool;
use crate::models::user::{User, CreateUserRequest};
use crate::errors::AppError;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: &CreateUserRequest) -> Result<User, AppError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, AppError>;
    async fn find_all(&self) -> Result<Vec<User>, AppError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &CreateUserRequest) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email)
            VALUES ($1, $2)
            RETURNING *
            "#,
            user.username,
            user.email
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    // 其他方法实现...
}
```

3. **创建服务** (`src/services/`):
```rust
// src/services/user_service.rs
use std::sync::Arc;
use crate::repositories::user_repo::UserRepository;
use crate::models::user::{User, CreateUserRequest};
use crate::errors::AppError;

pub struct UserService {
    repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { repository }
    }
    
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AppError> {
        // 业务逻辑验证
        if request.username.is_empty() {
            return Err(AppError::Validation {
                field: "username".to_string(),
                message: "Username cannot be empty".to_string(),
            });
        }
        
        // 调用仓库
        self.repository.create(&request).await
    }
    
    pub async fn get_user(&self, id: i32) -> Result<Option<User>, AppError> {
        self.repository.find_by_id(id).await
    }
}
```

4. **创建处理器** (`src/handlers/`):
```rust
// src/handlers/users.rs
use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
    response::Json as ResponseJson,
};
use crate::models::user::{User, CreateUserRequest};
use crate::services::user_service::UserService;
use crate::errors::AppError;
use crate::AppState;

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<ResponseJson<User>, AppError> {
    let user = state.user_service.create_user(request).await?;
    Ok(ResponseJson(user))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<ResponseJson<Option<User>>, AppError> {
    let user = state.user_service.get_user(id).await?;
    Ok(ResponseJson(user))
}
```

5. **注册路由** (`src/main.rs`):
```rust
use axum::{routing::{get, post}, Router};

fn create_router() -> Router<AppState> {
    Router::new()
        // 现有路由...
        .route("/api/users", post(handlers::users::create_user))
        .route("/api/users/:id", get(handlers::users::get_user))
}
```

#### 3. 数据库迁移

**创建迁移**:
```bash
# 创建新迁移文件
sqlx migrate add create_users_table

# 编辑迁移文件 (migrations/xxx_create_users_table.sql)
```

**迁移文件示例**:
```sql
-- migrations/004_create_users_table.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- 创建触发器自动更新 updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
```

**运行迁移**:
```bash
# 运行迁移
sqlx migrate run

# 检查状态
sqlx migrate info

# 回滚 (如果需要)
sqlx migrate revert
```

#### 4. 测试开发

**单元测试**:
```rust
// src/services/user_service.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    // Mock 仓库
    struct MockUserRepository {
        users: Vec<User>,
    }
    
    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(&self, _request: &CreateUserRequest) -> Result<User, AppError> {
            Ok(User {
                id: 1,
                username: "test_user".to_string(),
                email: "test@example.com".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        
        async fn find_by_id(&self, _id: i32) -> Result<Option<User>, AppError> {
            Ok(self.users.first().cloned())
        }
        
        async fn find_all(&self) -> Result<Vec<User>, AppError> {
            Ok(self.users.clone())
        }
    }
    
    #[tokio::test]
    async fn test_create_user_success() {
        let mock_repo = Arc::new(MockUserRepository { users: vec![] });
        let service = UserService::new(mock_repo);
        
        let request = CreateUserRequest {
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let result = service.create_user(request).await;
        assert!(result.is_ok());
        
        let user = result.unwrap();
        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
    }
    
    #[tokio::test]
    async fn test_create_user_empty_username() {
        let mock_repo = Arc::new(MockUserRepository { users: vec![] });
        let service = UserService::new(mock_repo);
        
        let request = CreateUserRequest {
            username: "".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let result = service.create_user(request).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AppError::Validation { field, .. } => {
                assert_eq!(field, "username");
            }
            _ => panic!("Expected validation error"),
        }
    }
}
```

**集成测试**:
```rust
// tests/integration/users_test.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;

use ccr_ui_backend::{create_app, AppState};

#[tokio::test]
async fn test_create_user_endpoint() {
    // 设置测试数据库
    let app_state = AppState::new_for_test().await.unwrap();
    let app = create_app().with_state(app_state);
    
    // 创建请求
    let request = Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": "test_user",
                "email": "test@example.com"
            }).to_string()
        ))
        .unwrap();
    
    // 发送请求
    let response = app.oneshot(request).await.unwrap();
    
    // 验证响应
    assert_eq!(response.status(), StatusCode::OK);
    
    // 验证响应体
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(user["username"], "test_user");
    assert_eq!(user["email"], "test@example.com");
}
```

### 代码质量

#### 1. 代码格式化

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt -- --check
```

#### 2. 代码检查

```bash
# 运行 Clippy
cargo clippy

# 严格模式
cargo clippy -- -D warnings

# 修复可自动修复的问题
cargo clippy --fix
```

#### 3. 安全审计

```bash
# 安装 cargo-audit
cargo install cargo-audit

# 运行安全审计
cargo audit

# 检查依赖更新
cargo outdated
```

#### 4. 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html

# 查看报告
open tarpaulin-report.html
```

## 📋 代码规范

### 命名约定

**文件和模块**:
- 使用 snake_case: `user_service.rs`, `config_repo.rs`
- 模块名与文件名一致

**结构体和枚举**:
- 使用 PascalCase: `ConfigItem`, `AppError`
- 枚举变体使用 PascalCase: `Status::Active`

**函数和变量**:
- 使用 snake_case: `create_user()`, `user_name`
- 常量使用 SCREAMING_SNAKE_CASE: `MAX_CONNECTIONS`

**特征 (Traits)**:
- 使用 PascalCase: `UserRepository`, `Serialize`

### 文档注释

```rust
/// 用户服务，提供用户管理相关功能
/// 
/// # Examples
/// 
/// ```rust
/// let service = UserService::new(repository);
/// let user = service.create_user(request).await?;
/// ```
pub struct UserService {
    repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserService {
    /// 创建新用户
    /// 
    /// # Arguments
    /// 
    /// * `request` - 用户创建请求，包含用户名和邮箱
    /// 
    /// # Returns
    /// 
    /// 返回创建的用户信息，如果创建失败则返回错误
    /// 
    /// # Errors
    /// 
    /// * `AppError::Validation` - 当输入数据验证失败时
    /// * `AppError::Database` - 当数据库操作失败时
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AppError> {
        // 实现...
    }
}
```

### 错误处理

**使用 Result 类型**:
```rust
// 好的做法
pub async fn get_config(name: &str) -> Result<ConfigItem, AppError> {
    // 实现...
}

// 避免 panic
pub fn parse_number(s: &str) -> Result<i32, AppError> {
    s.parse().map_err(|_| AppError::InvalidInput("Invalid number".to_string()))
}
```

**错误传播**:
```rust
pub async fn complex_operation() -> Result<String, AppError> {
    let config = get_config("default").await?;  // 使用 ? 操作符
    let result = process_config(&config)?;
    Ok(result)
}
```

### 异步编程

**使用 async/await**:
```rust
// 好的做法
pub async fn fetch_data() -> Result<Data, AppError> {
    let response = reqwest::get("https://api.example.com/data").await?;
    let data = response.json().await?;
    Ok(data)
}

// 并发执行
pub async fn fetch_multiple() -> Result<Vec<Data>, AppError> {
    let futures = vec![
        fetch_data_from_source1(),
        fetch_data_from_source2(),
        fetch_data_from_source3(),
    ];
    
    let results = futures::future::try_join_all(futures).await?;
    Ok(results)
}
```

## 🔍 调试和性能分析

### 日志调试

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(pool))]
pub async fn create_config(
    pool: &PgPool,
    request: &CreateConfigRequest,
) -> Result<ConfigItem, AppError> {
    debug!("Creating config with name: {}", request.name);
    
    // 验证输入
    if request.name.is_empty() {
        warn!("Attempted to create config with empty name");
        return Err(AppError::Validation {
            field: "name".to_string(),
            message: "Name cannot be empty".to_string(),
        });
    }
    
    // 执行创建
    match create_config_in_db(pool, request).await {
        Ok(config) => {
            info!("Successfully created config: {}", config.name);
            Ok(config)
        }
        Err(e) => {
            error!("Failed to create config: {:?}", e);
            Err(e)
        }
    }
}
```

### 性能分析

**使用 flamegraph**:
```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin ccr-ui-backend

# 查看结果
open flamegraph.svg
```

**基准测试**:
```rust
// benches/config_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ccr_ui_backend::services::ConfigService;

fn benchmark_config_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = rt.block_on(async {
        // 设置测试服务
        ConfigService::new_for_test().await.unwrap()
    });
    
    c.bench_function("create_config", |b| {
        b.to_async(&rt).iter(|| async {
            let request = CreateConfigRequest {
                name: black_box("test_config".to_string()),
                // 其他字段...
            };
            
            service.create_config(request).await.unwrap()
        })
    });
}

criterion_group!(benches, benchmark_config_creation);
criterion_main!(benches);
```

## 🚀 部署准备

### 构建优化

**Release 构建**:
```bash
# 优化构建
cargo build --release

# 检查二进制大小
ls -lh target/release/ccr-ui-backend

# 使用 strip 减小大小
strip target/release/ccr-ui-backend
```

### Docker 构建

**多阶段 Dockerfile**:
```dockerfile
# 构建阶段
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false ccr

# 复制二进制文件
COPY --from=builder /app/target/release/ccr-ui-backend /usr/local/bin/

# 设置权限
RUN chown ccr:ccr /usr/local/bin/ccr-ui-backend

USER ccr
EXPOSE 8080

CMD ["ccr-ui-backend"]
```

### 环境配置

**生产环境配置**:
```env
# .env.production
DATABASE_URL=postgresql://ccr_user:secure_password@db:5432/ccr_ui_db
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
CORS_ORIGINS=https://your-domain.com
JWT_SECRET=your-production-secret-key
```

## 📚 相关文档

- [技术栈详解](/backend/tech-stack)
- [架构设计](/backend/architecture)
- [API 文档](/backend/api)
- [部署指南](/backend/deployment)
- [错误处理](/backend/error-handling)