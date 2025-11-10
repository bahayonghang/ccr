# 后端部署指南

本文档详细介绍 CCR UI 后端服务的部署方案，包括本地部署、容器化部署、云平台部署和生产环境最佳实践。

> **📢 重要更新**: v1.2.0 版本已从 Actix Web 迁移到 Axum。部署流程基本相同，但构建产物更小、性能更优。详见 [Axum 迁移说明](./MIGRATION_AXUM.md)。

## 🚀 部署概览

### 部署架构

```mermaid
graph TB
    LB["Load Balancer<br/>(Nginx/Traefik)"]
    Backend["CCR UI Backend<br/>(Rust + Axum Server)"]
    DB["PostgreSQL<br/>(Primary Database)"]
    
    LB --> Backend
    Backend --> DB
    
    style LB fill:#e1f5fe
    style Backend fill:#f3e5f5
    style DB fill:#e8f5e9
```

### 支持的部署方式

- **本地部署**: 直接在服务器上运行
- **Docker 容器**: 单容器或 Docker Compose
- **Kubernetes**: 云原生容器编排
- **云平台**: AWS, GCP, Azure 等
- **边缘部署**: 轻量级边缘计算环境

## 🏠 本地部署

### 系统要求

**最低配置**:
- CPU: 1 核心
- 内存: 512MB
- 存储: 1GB
- 操作系统: Linux (Ubuntu 20.04+, CentOS 8+)

**推荐配置**:
- CPU: 2+ 核心
- 内存: 2GB+
- 存储: 10GB+
- 操作系统: Ubuntu 22.04 LTS

### 环境准备

#### 1. 安装系统依赖

**Ubuntu/Debian**:
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装必要工具
sudo apt install -y curl wget git build-essential pkg-config libssl-dev

# 安装 PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# 启动并启用 PostgreSQL
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**CentOS/RHEL**:
```bash
# 更新系统
sudo dnf update -y

# 安装开发工具
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y openssl-devel pkg-config

# 安装 PostgreSQL
sudo dnf install -y postgresql postgresql-server postgresql-contrib

# 初始化数据库
sudo postgresql-setup --initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### 2. 安装 Rust

```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境
source ~/.bashrc

# 验证安装
rustc --version
cargo --version
```

#### 3. 数据库设置

```bash
# 切换到 postgres 用户
sudo -u postgres psql

-- 在 PostgreSQL 中执行
CREATE USER ccr_user WITH PASSWORD 'secure_password';
CREATE DATABASE ccr_ui_db OWNER ccr_user;
GRANT ALL PRIVILEGES ON DATABASE ccr_ui_db TO ccr_user;
\q
```

#### 4. 应用部署

```bash
# 克隆项目
git clone https://github.com/your-org/ccr-ui-backend.git
cd ccr-ui-backend

# 配置环境变量
cp .env.example .env
nano .env
```

**生产环境配置** (`.env`):
```bash
# 数据库配置
DATABASE_URL=postgresql://ccr_user:secure_password@localhost:5432/ccr_ui_db

# 服务器配置
HOST=0.0.0.0
PORT=8080

# 日志配置
RUST_LOG=info

# 安全配置
JWT_SECRET=your-super-secure-jwt-secret-key-here
CORS_ORIGINS=https://your-domain.com

# 性能配置
MAX_CONNECTIONS=50
CONNECTION_TIMEOUT=30
IDLE_TIMEOUT=300
```

```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# 运行数据库迁移
sqlx database create
sqlx migrate run

# 构建应用
cargo build --release

# 创建系统用户
sudo useradd -r -s /bin/false ccr

# 复制二进制文件
sudo cp target/release/ccr-ui-backend /usr/local/bin/
sudo chown ccr:ccr /usr/local/bin/ccr-ui-backend
sudo chmod +x /usr/local/bin/ccr-ui-backend

# 创建配置目录
sudo mkdir -p /etc/ccr-ui
sudo cp .env /etc/ccr-ui/
sudo chown -R ccr:ccr /etc/ccr-ui
```

#### 5. 系统服务配置

**创建 systemd 服务** (`/etc/systemd/system/ccr-ui-backend.service`):
```ini
[Unit]
Description=CCR UI Backend Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=ccr
Group=ccr
WorkingDirectory=/etc/ccr-ui
EnvironmentFile=/etc/ccr-ui/.env
ExecStart=/usr/local/bin/ccr-ui-backend
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=ccr-ui-backend

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/etc/ccr-ui

# 资源限制
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

```bash
# 重新加载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start ccr-ui-backend

# 启用开机自启
sudo systemctl enable ccr-ui-backend

# 检查状态
sudo systemctl status ccr-ui-backend

# 查看日志
sudo journalctl -u ccr-ui-backend -f
```

### 反向代理配置

#### Nginx 配置

**安装 Nginx**:
```bash
sudo apt install -y nginx
```

**配置文件** (`/etc/nginx/sites-available/ccr-ui-backend`):
```nginx
upstream ccr_backend {
    server 127.0.0.1:8080;
    # 如果有多个实例，可以添加更多服务器
    # server 127.0.0.1:8081;
    # server 127.0.0.1:8082;
}

server {
    listen 80;
    server_name your-domain.com;
    
    # 重定向到 HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    # SSL 证书配置
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    
    # SSL 安全配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    
    # 安全头
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    
    # 日志配置
    access_log /var/log/nginx/ccr-ui-backend.access.log;
    error_log /var/log/nginx/ccr-ui-backend.error.log;
    
    # 代理配置
    location /api/ {
        proxy_pass http://ccr_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # 超时配置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # 缓冲配置
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
    }
    
    # 健康检查
    location /health {
        proxy_pass http://ccr_backend/api/system/status;
        access_log off;
    }
    
    # 静态文件 (如果有)
    location /static/ {
        alias /var/www/ccr-ui-static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

```bash
# 启用站点
sudo ln -s /etc/nginx/sites-available/ccr-ui-backend /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重新加载 Nginx
sudo systemctl reload nginx
```

## 🐳 Docker 部署

### 单容器部署

#### 1. 构建镜像

**Dockerfile**:
```dockerfile
# 多阶段构建
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 预构建依赖 (缓存优化)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 复制源代码
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN touch src/main.rs && cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false -u 1001 ccr

# 创建应用目录
RUN mkdir -p /app && chown ccr:ccr /app

# 复制二进制文件
COPY --from=builder /app/target/release/ccr-ui-backend /usr/local/bin/
RUN chmod +x /usr/local/bin/ccr-ui-backend

# 复制迁移文件
COPY --from=builder /app/migrations /app/migrations
RUN chown -R ccr:ccr /app

USER ccr
WORKDIR /app

EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/system/status || exit 1

CMD ["ccr-ui-backend"]
```

```bash
# 构建镜像
docker build -t ccr-ui-backend:latest .

# 运行容器
docker run -d \
  --name ccr-ui-backend \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://ccr_user:password@host.docker.internal:5432/ccr_ui_db \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ccr-ui-backend:latest
```

### Docker Compose 部署

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  # PostgreSQL 数据库
  postgres:
    image: postgres:15-alpine
    container_name: ccr-postgres
    environment:
      POSTGRES_DB: ccr_ui_db
      POSTGRES_USER: ccr_user
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-secure_password}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    networks:
      - ccr-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ccr_user -d ccr_ui_db"]
      interval: 30s
      timeout: 10s
      retries: 3

  # CCR UI 后端
  backend:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: ccr-ui-backend
    environment:
      DATABASE_URL: postgresql://ccr_user:${POSTGRES_PASSWORD:-secure_password}@postgres:5432/ccr_ui_db
      HOST: 0.0.0.0
      PORT: 8080
      RUST_LOG: ${RUST_LOG:-info}
      JWT_SECRET: ${JWT_SECRET}
      CORS_ORIGINS: ${CORS_ORIGINS:-http://localhost:3000}
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - ccr-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/system/status"]
      interval: 30s
      timeout: 10s
      retries: 3
    volumes:
      - ./logs:/app/logs

  # Nginx 反向代理
  nginx:
    image: nginx:alpine
    container_name: ccr-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
      - nginx_logs:/var/log/nginx
    depends_on:
      - backend
    networks:
      - ccr-network
    restart: unless-stopped

  # Redis (可选，用于缓存)
  redis:
    image: redis:7-alpine
    container_name: ccr-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    networks:
      - ccr-network
    restart: unless-stopped
    command: redis-server --appendonly yes

volumes:
  postgres_data:
  redis_data:
  nginx_logs:

networks:
  ccr-network:
    driver: bridge
```

**环境变量文件** (`.env`):
```bash
# 数据库配置
POSTGRES_PASSWORD=your_secure_database_password

# 应用配置
JWT_SECRET=your-super-secure-jwt-secret-key-here
CORS_ORIGINS=https://your-domain.com,https://www.your-domain.com

# 日志级别
RUST_LOG=info

# 其他配置
BACKUP_SCHEDULE=0 2 * * *
```

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f backend

# 停止服务
docker-compose down

# 重新构建并启动
docker-compose up -d --build
```

## ☸️ Kubernetes 部署

### 基础配置

#### 1. 命名空间

**namespace.yaml**:
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: ccr-ui
  labels:
    name: ccr-ui
```

#### 2. 配置映射

**configmap.yaml**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: ccr-ui-config
  namespace: ccr-ui
data:
  HOST: "0.0.0.0"
  PORT: "8080"
  RUST_LOG: "info"
  CORS_ORIGINS: "https://your-domain.com"
```

#### 3. 密钥

**secret.yaml**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: ccr-ui-secrets
  namespace: ccr-ui
type: Opaque
data:
  DATABASE_URL: cG9zdGdyZXNxbDovL2Njcl91c2VyOnBhc3N3b3JkQHBvc3RncmVzOjU0MzIvY2NyX3VpX2Ri  # base64 encoded
  JWT_SECRET: eW91ci1zdXBlci1zZWN1cmUtand0LXNlY3JldC1rZXktaGVyZQ==  # base64 encoded
```

#### 4. 部署配置

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ccr-ui-backend
  namespace: ccr-ui
  labels:
    app: ccr-ui-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ccr-ui-backend
  template:
    metadata:
      labels:
        app: ccr-ui-backend
    spec:
      containers:
      - name: backend
        image: ccr-ui-backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: HOST
          valueFrom:
            configMapKeyRef:
              name: ccr-ui-config
              key: HOST
        - name: PORT
          valueFrom:
            configMapKeyRef:
              name: ccr-ui-config
              key: PORT
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: ccr-ui-config
              key: RUST_LOG
        - name: CORS_ORIGINS
          valueFrom:
            configMapKeyRef:
              name: ccr-ui-config
              key: CORS_ORIGINS
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: ccr-ui-secrets
              key: DATABASE_URL
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: ccr-ui-secrets
              key: JWT_SECRET
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/system/status
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/system/status
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          runAsNonRoot: true
          runAsUser: 1001
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
```

#### 5. 服务配置

**service.yaml**:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: ccr-ui-backend-service
  namespace: ccr-ui
  labels:
    app: ccr-ui-backend
spec:
  selector:
    app: ccr-ui-backend
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP
```

#### 6. Ingress 配置

**ingress.yaml**:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ccr-ui-ingress
  namespace: ccr-ui
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
spec:
  tls:
  - hosts:
    - your-domain.com
    secretName: ccr-ui-tls
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: ccr-ui-backend-service
            port:
              number: 80
```

### 数据库部署

**postgres-deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: ccr-ui
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15
        env:
        - name: POSTGRES_DB
          value: ccr_ui_db
        - name: POSTGRES_USER
          value: ccr_user
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: ccr-ui
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: ccr-ui
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

### 部署命令

```bash
# 应用所有配置
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f secret.yaml
kubectl apply -f postgres-deployment.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f ingress.yaml

# 查看部署状态
kubectl get pods -n ccr-ui
kubectl get services -n ccr-ui
kubectl get ingress -n ccr-ui

# 查看日志
kubectl logs -f deployment/ccr-ui-backend -n ccr-ui

# 扩容
kubectl scale deployment ccr-ui-backend --replicas=5 -n ccr-ui
```

## ☁️ 云平台部署

### AWS 部署

#### 1. ECS 部署

**任务定义** (task-definition.json):
```json
{
  "family": "ccr-ui-backend",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::account:role/ecsTaskRole",
  "containerDefinitions": [
    {
      "name": "ccr-ui-backend",
      "image": "your-account.dkr.ecr.region.amazonaws.com/ccr-ui-backend:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "HOST",
          "value": "0.0.0.0"
        },
        {
          "name": "PORT",
          "value": "8080"
        },
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "DATABASE_URL",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:ccr-ui-db-url"
        },
        {
          "name": "JWT_SECRET",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:ccr-ui-jwt-secret"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/ccr-ui-backend",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": [
          "CMD-SHELL",
          "curl -f http://localhost:8080/api/system/status || exit 1"
        ],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    }
  ]
}
```

**部署脚本**:
```bash
#!/bin/bash

# 构建并推送镜像到 ECR
aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin your-account.dkr.ecr.us-west-2.amazonaws.com

docker build -t ccr-ui-backend .
docker tag ccr-ui-backend:latest your-account.dkr.ecr.us-west-2.amazonaws.com/ccr-ui-backend:latest
docker push your-account.dkr.ecr.us-west-2.amazonaws.com/ccr-ui-backend:latest

# 注册任务定义
aws ecs register-task-definition --cli-input-json file://task-definition.json

# 更新服务
aws ecs update-service \
  --cluster ccr-ui-cluster \
  --service ccr-ui-backend-service \
  --task-definition ccr-ui-backend:LATEST
```

#### 2. RDS 数据库

**Terraform 配置**:
```hcl
resource "aws_db_instance" "ccr_ui_db" {
  identifier = "ccr-ui-database"
  
  engine         = "postgres"
  engine_version = "15.4"
  instance_class = "db.t3.micro"
  
  allocated_storage     = 20
  max_allocated_storage = 100
  storage_type          = "gp2"
  storage_encrypted     = true
  
  db_name  = "ccr_ui_db"
  username = "ccr_user"
  password = var.db_password
  
  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.ccr_ui.name
  
  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  skip_final_snapshot = false
  final_snapshot_identifier = "ccr-ui-db-final-snapshot"
  
  tags = {
    Name = "CCR UI Database"
    Environment = "production"
  }
}
```

### Google Cloud Platform

#### 1. Cloud Run 部署

**cloudbuild.yaml**:
```yaml
steps:
  # 构建镜像
  - name: 'gcr.io/cloud-builders/docker'
    args: ['build', '-t', 'gcr.io/$PROJECT_ID/ccr-ui-backend:$COMMIT_SHA', '.']
  
  # 推送镜像
  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', 'gcr.io/$PROJECT_ID/ccr-ui-backend:$COMMIT_SHA']
  
  # 部署到 Cloud Run
  - name: 'gcr.io/cloud-builders/gcloud'
    args:
    - 'run'
    - 'deploy'
    - 'ccr-ui-backend'
    - '--image'
    - 'gcr.io/$PROJECT_ID/ccr-ui-backend:$COMMIT_SHA'
    - '--region'
    - 'us-central1'
    - '--platform'
    - 'managed'
    - '--allow-unauthenticated'
    - '--set-env-vars'
    - 'RUST_LOG=info'
    - '--set-secrets'
    - 'DATABASE_URL=ccr-ui-db-url:latest,JWT_SECRET=ccr-ui-jwt-secret:latest'

images:
  - 'gcr.io/$PROJECT_ID/ccr-ui-backend:$COMMIT_SHA'
```

#### 2. Cloud SQL 配置

```bash
# 创建 Cloud SQL 实例
gcloud sql instances create ccr-ui-db \
  --database-version=POSTGRES_15 \
  --tier=db-f1-micro \
  --region=us-central1 \
  --storage-type=SSD \
  --storage-size=10GB \
  --backup-start-time=03:00

# 创建数据库
gcloud sql databases create ccr_ui_db --instance=ccr-ui-db

# 创建用户
gcloud sql users create ccr_user --instance=ccr-ui-db --password=secure_password
```

## 🔧 生产环境优化

### 性能优化

#### 1. 连接池配置

```rust
// 生产环境连接池配置
pub async fn create_production_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(100)        // 根据负载调整
        .min_connections(20)         // 保持最小连接数
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)   // 连接健康检查
        .connect(database_url)
        .await
}
```

#### 2. 缓存配置

```rust
// Redis 缓存配置
use redis::{Client, Commands, Connection};

pub struct CacheService {
    client: Client,
}

impl CacheService {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
    
    pub async fn get_config(&self, name: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        let key = format!("config:{}", name);
        conn.get(&key)
    }
    
    pub async fn set_config(&self, name: &str, value: &str, ttl: usize) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        let key = format!("config:{}", name);
        conn.set_ex(&key, value, ttl)
    }
}
```

### 监控和日志

#### 1. 结构化日志

```rust
use tracing::{info, warn, error, instrument};
use serde_json::json;

#[instrument(skip(pool))]
pub async fn create_config_with_metrics(
    pool: &PgPool,
    request: &CreateConfigRequest,
) -> Result<ConfigItem, AppError> {
    let start_time = std::time::Instant::now();
    
    info!(
        config_name = %request.name,
        provider = %request.provider,
        "Creating new configuration"
    );
    
    match create_config_in_db(pool, request).await {
        Ok(config) => {
            let duration = start_time.elapsed();
            info!(
                config_name = %config.name,
                duration_ms = %duration.as_millis(),
                "Configuration created successfully"
            );
            Ok(config)
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!(
                config_name = %request.name,
                error = %e,
                duration_ms = %duration.as_millis(),
                "Failed to create configuration"
            );
            Err(e)
        }
    }
}
```

#### 2. 健康检查端点

```rust
use axum::{http::StatusCode, Json};
use serde_json::json;

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    version: String,
    uptime: u64,
    database: DatabaseHealth,
    cache: CacheHealth,
    memory_usage: MemoryUsage,
}

pub async fn comprehensive_health_check(
    State(state): State<AppState>
) -> Result<Json<HealthStatus>, StatusCode> {
    // 检查数据库
    let db_health = check_database_health(&state.db_pool).await;
    
    // 检查缓存
    let cache_health = check_cache_health(&state.cache_service).await;
    
    // 检查内存使用
    let memory_usage = get_memory_usage();
    
    let overall_status = if db_health.healthy && cache_health.healthy {
        "healthy"
    } else {
        "unhealthy"
    };
    
    Ok(Json(HealthStatus {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: get_uptime(),
        database: db_health,
        cache: cache_health,
        memory_usage,
    }))
}
```

### 安全配置

#### 1. 环境变量验证

```rust
use std::env;

pub struct SecurityConfig {
    pub jwt_secret: String,
    pub allowed_origins: Vec<String>,
    pub rate_limit: u32,
}

impl SecurityConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET"))?;
        
        if jwt_secret.len() < 32 {
            return Err(ConfigError::WeakJwtSecret);
        }
        
        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let rate_limit = env::var("RATE_LIMIT")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidRateLimit)?;
        
        Ok(Self {
            jwt_secret,
            allowed_origins: cors_origins,
            rate_limit,
        })
    }
}
```

#### 2. 速率限制

```rust
use tower_governor::{GovernorConfigBuilder, governor::middleware::NoOpMiddleware};
use std::time::Duration;

pub fn create_rate_limiter() -> GovernorConfigBuilder<PeerIpKeyExtractor, NoOpMiddleware> {
    GovernorConfigBuilder::default()
        .per_second(10)                    // 每秒 10 个请求
        .burst_size(20)                    // 突发 20 个请求
        .finish()
        .unwrap()
}
```

## 📊 监控和维护

### 备份策略

#### 1. 数据库备份

**备份脚本** (`backup.sh`):
```bash
#!/bin/bash

# 配置
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="ccr_ui_db"
DB_USER="ccr_user"
BACKUP_DIR="/var/backups/ccr-ui"
RETENTION_DAYS=30

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 生成备份文件名
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="$BACKUP_DIR/ccr_ui_db_$TIMESTAMP.sql"

# 执行备份
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
  --no-password --verbose --clean --no-acl --no-owner \
  > "$BACKUP_FILE"

# 压缩备份文件
gzip "$BACKUP_FILE"

# 清理旧备份
find "$BACKUP_DIR" -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: ${BACKUP_FILE}.gz"
```

#### 2. 自动备份 (Cron)

```bash
# 添加到 crontab
crontab -e

# 每天凌晨 2 点执行备份
0 2 * * * /usr/local/bin/backup.sh >> /var/log/ccr-ui-backup.log 2>&1
```

### 日志轮转

**logrotate 配置** (`/etc/logrotate.d/ccr-ui`):
```
/var/log/ccr-ui/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 ccr ccr
    postrotate
        systemctl reload ccr-ui-backend
    endscript
}
```

### 更新部署

**零停机更新脚本**:
```bash
#!/bin/bash

# 蓝绿部署脚本
set -e

NEW_VERSION=$1
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Deploying version: $NEW_VERSION"

# 拉取新镜像
docker pull ccr-ui-backend:$NEW_VERSION

# 启动新容器
docker run -d \
  --name ccr-ui-backend-new \
  --network ccr-network \
  -e DATABASE_URL="$DATABASE_URL" \
  -e JWT_SECRET="$JWT_SECRET" \
  ccr-ui-backend:$NEW_VERSION

# 等待新容器就绪
echo "Waiting for new container to be ready..."
for i in {1..30}; do
    if docker exec ccr-ui-backend-new curl -f http://localhost:8080/api/system/status; then
        echo "New container is ready"
        break
    fi
    sleep 2
done

# 更新负载均衡器配置
# (这里需要根据具体的负载均衡器实现)

# 停止旧容器
docker stop ccr-ui-backend || true
docker rm ccr-ui-backend || true

# 重命名新容器
docker rename ccr-ui-backend-new ccr-ui-backend

echo "Deployment completed successfully"
```

## 📚 相关文档

- [技术栈详解](/backend/tech-stack)
- [架构设计](/backend/architecture)
- [开发指南](/backend/development)
- [API 文档](/backend/api)
- [错误处理](/backend/error-handling)