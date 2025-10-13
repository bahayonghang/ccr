# 🚀 部署和运行指南

## ✅ 问题解决记录

### 已解决的问题

#### 1. Next.js Workspace Root 警告 ✓

**问题**:
```
Warning: Next.js inferred your workspace root, but it may not be correct.
We detected multiple lockfiles...
```

**解决方案**:
在 `next.config.mjs` 中添加 `turbopack.root` 配置：

```javascript
turbopack: {
  root: process.cwd(),
}
```

#### 2. 端口占用问题 ✓

**问题**:
```
Error: Os { code: 98, kind: AddrInUse, message: "Address already in use" }
```

**解决方案**:
清理占用的端口：

```bash
# 清理后端端口
lsof -ti:8081 | xargs kill -9

# 清理前端端口
lsof -ti:3000 | xargs kill -9
```

## 🌐 服务访问

### 开发环境

- **前端**: http://localhost:3000
- **后端 API**: http://localhost:8081
- **系统信息 API**: http://localhost:8081/api/system
- **配置列表 API**: http://localhost:8081/api/configs

### 快速启动

```bash
# 方式 1: 使用 just（推荐）
cd /home/lyh/Documents/Github/ccr/ccr-ui
just dev

# 方式 2: 手动启动
# 终端 1 - 后端
cd backend && cargo run

# 终端 2 - 前端
cd frontend && npm run dev
```

## 🔍 健康检查

### 测试后端

```bash
curl http://localhost:8081/api/system
```

**预期输出**:
```json
{
  "success": true,
  "data": {
    "hostname": "...",
    "os": "linux",
    "cpu_cores": 24,
    ...
  }
}
```

### 测试前端

```bash
curl -I http://localhost:3000
```

**预期输出**:
```
HTTP/1.1 200 OK
Content-Type: text/html
...
```

## 🛠️ 常见问题

### 1. 端口被占用

**症状**: `Address already in use`

**解决**:
```bash
# 查找占用进程
lsof -ti:8081
lsof -ti:3000

# 终止进程
kill -9 <PID>

# 或一键清理
cd /home/lyh/Documents/Github/ccr/ccr-ui
just clean-ports  # 如果 justfile 中有此命令
```

### 2. 前端无法连接后端

**检查清单**:
- [ ] 后端是否在 8081 端口运行？
- [ ] 防火墙是否阻止了连接？
- [ ] API 代理配置是否正确？

**验证 API 代理**:
检查 `next.config.mjs` 中的 `rewrites` 配置：

```javascript
async rewrites() {
  return [
    {
      source: '/api/:path*',
      destination: 'http://localhost:8081/api/:path*',
    },
  ];
}
```

### 3. 构建警告

**已解决的警告**:
- ✅ Next.js workspace root 警告
- ✅ Invalid config keys 警告
- ✅ Viewport configuration 警告

**当前构建状态**:
```
✓ Compiled successfully in 6.0s
✓ No warnings
```

## 📊 性能指标

### 构建时间

- **开发模式启动**: ~269ms
- **生产构建**: ~6s
- **热更新**: ~50ms

### 包大小

- **Next.js**: ~350KB (已优化)
- **静态页面**: 预渲染 5 个路由

## 🔒 生产部署

### 构建生产版本

```bash
# 使用 just
cd /home/lyh/Documents/Github/ccr/ccr-ui
just build

# 或手动构建
cd frontend && npm run build
cd backend && cargo build --release
```

### 生产服务器

```bash
# 前端
cd frontend && npm run start

# 后端
cd backend && ./target/release/ccr-ui-backend
```

## 📝 环境变量

### 前端 (.env.local)

```env
# API 基础 URL（开发环境）
NEXT_PUBLIC_API_URL=http://localhost:8081

# 生产环境
# NEXT_PUBLIC_API_URL=https://your-api-domain.com
```

### 后端

后端默认配置在代码中，无需额外环境变量。

## 🎉 验证成功

运行以下命令验证一切正常：

```bash
# 1. 清理端口
lsof -ti:8081 | xargs kill -9 2>/dev/null
lsof -ti:3000 | xargs kill -9 2>/dev/null

# 2. 启动服务
cd /home/lyh/Documents/Github/ccr/ccr-ui
just dev

# 3. 在浏览器中访问
# http://localhost:3000

# 4. 检查 API
curl http://localhost:8081/api/system
```

**预期结果**:
- ✅ 前端页面加载成功
- ✅ 后端 API 返回 JSON 数据
- ✅ 无构建警告
- ✅ 无运行时错误

---

**最后更新**: 2025-10-13
**状态**: ✅ 所有问题已解决

