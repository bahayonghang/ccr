# 🌐 CCR Desktop - Web 调试模式使用指南

## 🎯 什么是 Web 调试模式？

Web 调试模式是一个**不启动桌面窗口**的开发模式，同时运行：

- 🎨 **前端开发服务器** (Vite) - `http://localhost:5173`
- 📡 **后端 API 服务器** (CCR Web) - `http://localhost:3030`

**核心优势**：
- ✅ 在**浏览器**中调试，支持所有浏览器开发工具
- ✅ 适合**远程服务器**和**WSL**环境
- ✅ 无需图形界面，纯 Web 访问
- ✅ 完整的**前后端联调**，真实 API 调用
- ✅ 支持**热重载**，修改代码自动刷新

---

## 🚀 快速开始

### 启动 Web 调试模式

```bash
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web
```

**预期输出**：
```
▶ 启动 Web 调试模式...
  特性: 前端 + 后端 API 服务，无桌面窗口
  前端地址: http://localhost:5173
  后端 API: http://localhost:3030
  🌐 适用于 WSL/远程服务器调试

📡 启动后端 API 服务器 (端口 3030)...
🎨 启动前端开发服务器 (端口 5173)...

╔═══════════════════════════════════════════════════════════╗
║  🌐 Web 调试模式已启动                                   ║
║                                                           ║
║  🎨 前端界面: http://localhost:5173                       ║
║  📡 后端 API: http://localhost:3030                       ║
╚═══════════════════════════════════════════════════════════╝

✅ 后端服务器启动成功 (PID: 12345)
✅ 前端服务器启动成功 (PID: 12346)

🌐 在浏览器中访问: http://localhost:5173
```

### 使用方法

1. **打开浏览器** → 访问 http://localhost:5173
2. **F12 开发者工具** → 查看 Network 标签
3. **正常使用界面** → 所有操作会调用后端 API
4. **实时调试** → 前端修改自动热重载

---

## 📋 Web 调试模式命令

| 命令 | 功能 | 适用场景 |
|------|------|----------|
| `just dev-web` | 🚀 启动 Web 调试模式 | 开始调试 ⭐ |
| `just web-status` | 📊 查看服务状态 | 检查运行状态 |
| `just web-logs` | 📋 查看日志（最近 20 行） | 快速查看日志 |
| `just web-logs-follow` | 📊 实时跟踪日志 | 调试问题 |
| `just stop-web` | ⏹️ 停止所有服务 | 清理退出 |

---

## 🔧 详细使用教程

### 1. 启动服务

```bash
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web
```

**启动后会看到**：
- 前端服务器：Vite 在 5173 端口
- 后端服务器：CCR Web API 在 3030 端口
- 两个服务都在后台运行，有 PID 文件管理

### 2. 浏览器访问

打开任意浏览器，访问：**http://localhost:5173**

**你会看到**：
- 🎨 完整的 CCR Desktop 界面
- 🖱️ 滚轮功能正常（已修复）
- 📱 响应式设计
- 🔄 热重载支持

### 3. 调试 API 调用

在浏览器 DevTools 中：

1. **打开 Network 标签**
2. **执行配置操作**（如切换配置）
3. **查看 HTTP 请求**：
   ```
   Method: POST
   URL: http://localhost:3030/api/switch
   Status: 200 OK
   Response: {...}
   ```

### 4. 查看服务状态

```bash
just web-status
```

**输出示例**：
```
▶ Web 调试服务状态...

✅ 后端 API 服务器 (PID: 12345)
   📡 http://localhost:3030
✅ 前端开发服务器 (PID: 12346)
   🎨 http://localhost:5173

📊 端口监听状态:
   127.0.0.1:3030 → tcp
   127.0.0.1:5173 → tcp
```

### 5. 查看日志

```bash
# 查看最近日志
just web-logs

# 实时跟踪日志
just web-logs-follow
```

### 6. 停止服务

```bash
just stop-web
```

**或者在 `dev-web` 运行时按 `Ctrl+C`**

---

## 🆚 不同开发模式对比

| 特性 | `just dev` | `just dev-wsl` | `just dev-web` |
|------|------------|----------------|----------------|
| **界面** | Tauri 桌面窗口 | Tauri 桌面窗口 | 🌐 浏览器 |
| **环境** | 原生平台 | WSL 优化 | WSL/远程服务器 |
| **滚轮** | 可能有问题 | ✅ 已修复 | ✅ 原生支持 |
| **图形警告** | 有警告 | ✅ 已抑制 | 无（纯 Web） |
| **调试工具** | Tauri DevTools | Tauri DevTools | 🔧 浏览器 DevTools |
| **热重载** | ✅ 前端 | ✅ 前端 | ✅ 前端 |
| **API 调用** | Tauri Commands | Tauri Commands | 📡 HTTP API |
| **远程访问** | ❌ 本地窗口 | ❌ 本地窗口 | ✅ 支持 |
| **适用场景** | 本地开发 | WSL 开发 | 调试/远程/CI |

---

## 🌟 适用场景

### ✅ 推荐使用 Web 调试模式的情况

1. **WSL 环境开发** - 避免图形界面问题
2. **远程服务器调试** - SSH 连接的服务器
3. **CI/CD 测试** - 自动化测试环境
4. **API 联调** - 专门测试前后端接口
5. **移动端测试** - 手机浏览器访问
6. **多人协作** - 团队成员远程访问

### 🔧 特别适合的调试任务

- **接口测试** - 查看 HTTP 请求/响应
- **状态调试** - 观察 Vue 组件状态变化
- **性能分析** - 使用浏览器性能工具
- **错误排查** - 完整的调用堆栈
- **响应式测试** - 模拟不同屏幕尺寸

---

## 🔌 后端 API 接口

CCR Web 服务器提供以下 API 接口：

### 基础接口

| 接口 | 方法 | 功能 |
|------|------|------|
| `/api/configs` | GET | 获取配置列表 |
| `/api/current` | GET | 获取当前配置 |
| `/api/switch` | POST | 切换配置 |
| `/api/history` | GET | 获取历史记录 |
| `/api/system` | GET | 获取系统信息 |

### 配置管理

| 接口 | 方法 | 功能 |
|------|------|------|
| `/api/config` | POST | 创建配置 |
| `/api/config/{name}` | PUT | 更新配置 |
| `/api/config/{name}` | DELETE | 删除配置 |
| `/api/validate` | POST | 验证配置 |

### 备份管理

| 接口 | 方法 | 功能 |
|------|------|------|
| `/api/backups` | GET | 列出备份 |
| `/api/restore` | POST | 恢复备份 |

**完整 API 文档**: 启动后访问 http://localhost:3030

---

## 📊 日志和监控

### 查看服务状态

```bash
just web-status
```

### 查看历史日志

```bash
just web-logs
```

### 实时跟踪日志

```bash
just web-logs-follow
```

**日志文件位置**：
- 后端日志：`/tmp/ccr-web.log`
- 前端日志：`/tmp/vite-dev.log`

---

## 🚀 开发工作流

### 典型的调试工作流

```bash
# 1. 启动 Web 调试模式
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web

# 2. 打开浏览器
# 访问 http://localhost:5173

# 3. 开始调试
# F12 → Network 标签
# 执行配置操作，观察 API 调用

# 4. 修改代码
# 编辑 src-ui/src/ 下的文件 → 自动热重载
# 编辑根目录 src/ 下的后端代码 → 重启后端

# 5. 查看日志（新终端）
just web-logs-follow

# 6. 停止服务
just stop-web
```

### 与 Tauri 桌面模式的差异

**Tauri Commands** → **HTTP API 调用**

```typescript
// Tauri 模式 (桌面窗口)
import { invoke } from '@tauri-apps/api/tauri'
await invoke('list_configs')

// Web 调试模式 (浏览器)
fetch('http://localhost:3030/api/configs')
  .then(res => res.json())
```

---

## 🌍 远程访问配置

### 在远程服务器上运行

```bash
# 1. SSH 到服务器
ssh user@your-server

# 2. 启动 Web 调试模式
cd /path/to/ccr/ccr-tauri
just dev-web

# 3. 在本地浏览器访问
# http://your-server-ip:5173
```

### 局域网访问

修改 Vite 配置以允许外部访问：

```typescript
// src-ui/vite.config.ts
export default defineConfig({
  server: {
    host: '0.0.0.0',  // 允许外部访问
    port: 5173
  }
})
```

然后其他设备可以访问：`http://你的IP:5173`

---

## 🔧 故障排查

### 常见问题

#### Q1: 后端服务器启动失败

```bash
# 查看后端日志
cat /tmp/ccr-web.log

# 常见原因：端口被占用
ss -tuln | grep 3030

# 解决：更换端口或杀死占用进程
```

#### Q2: 前端连接不到后端

```bash
# 检查两个服务是否都在运行
just web-status

# 检查防火墙（如果适用）
sudo ufw status
```

#### Q3: 热重载不工作

```bash
# 重启前端服务器
just stop-web
just dev-web
```

---

## 💡 高级用法

### 自定义端口

如果需要更改端口，可以修改 justfile：

```makefile
# 修改默认端口
_start-web-servers:
  # 后端：3030 → 8080
  cd .. && nohup ccr web -p 8080 ...
  
  # 前端：5173 → 3000
  # 需要修改 vite.config.ts
```

### 配合 ngrok 使用（公网访问）

```bash
# 1. 启动 Web 调试模式
just dev-web

# 2. 在另一个终端启动 ngrok
ngrok http 5173

# 3. 使用 ngrok 提供的公网地址访问
```

### Docker 化部署

```dockerfile
# Dockerfile.web-debug
FROM node:18-alpine
WORKDIR /app
COPY ccr-tauri/src-ui/package*.json ./
RUN npm install
COPY ccr-tauri/src-ui ./
EXPOSE 5173
CMD ["npm", "run", "dev"]
```

---

## 📝 开发说明

### 前端代码修改

编辑 `src-ui/src/` 下的任何文件：
- ✅ `.vue` 组件 → 自动热重载
- ✅ `.ts` 脚本 → 自动重新编译
- ✅ `.css` 样式 → 实时更新

### 后端代码修改

编辑根目录 `src/` 下的文件：
- ⚠️ 需要**重启后端服务器**
- 方法：`just stop-web && just dev-web`

### 添加新 API 接口

在根项目的 `src/web/handlers.rs` 中添加新接口：

```rust
// 添加新的 API 端点
pub async fn new_endpoint() -> impl IntoResponse {
    // 实现逻辑
    Json("success")
}
```

然后在前端调用：

```typescript
// src-ui/src/api/index.ts
export async function newApiCall() {
  const response = await fetch('http://localhost:3030/api/new-endpoint')
  return response.json()
}
```

---

## 🎯 实际使用案例

### 案例 1: API 接口测试

```bash
# 1. 启动服务
just dev-web

# 2. 测试后端 API
curl http://localhost:3030/api/configs | jq

# 3. 在浏览器中测试前端调用
# Network 标签中观察请求
```

### 案例 2: 移动端适配测试

```bash
# 1. 启动服务
just dev-web

# 2. 手机连接同一 WiFi

# 3. 手机浏览器访问
# http://你的电脑IP:5173

# 4. 测试触摸操作和响应式布局
```

### 案例 3: 团队协作调试

```bash
# 开发者 A（后端）
just dev-web

# 开发者 B（前端，远程）
# 访问 http://开发者A的IP:5173
# 共享调试环境
```

---

## 📊 监控和日志

### 实时状态监控

```bash
# 持续监控（另开终端）
watch -n 2 'just web-status'
```

### 性能监控

```bash
# CPU/内存使用情况
ps aux | grep -E "(ccr web|vite)" | grep -v grep

# 网络连接
ss -tuln | grep -E "(3030|5173)"

# 磁盘使用（日志文件）
ls -lh /tmp/ccr-web.log /tmp/vite-dev.log
```

---

## 🆚 与其他模式的选择建议

### 选择 `just dev`（标准桌面模式）
- ✅ 原生桌面体验
- ✅ Tauri 特有功能测试
- ❌ WSL 中可能有滚轮问题

### 选择 `just dev-wsl`（WSL 优化桌面）
- ✅ WSL 环境完美支持
- ✅ 滚轮问题已修复
- ✅ 真实桌面应用环境
- ❌ 依然需要图形界面

### 选择 `just dev-web`（Web 调试模式）⭐
- ✅ 无需图形界面
- ✅ 浏览器调试工具强大
- ✅ 远程访问友好
- ✅ 适合 CI/CD 环境
- ❌ 不能测试桌面特有功能
- ❌ 需要适配 Tauri Commands → HTTP API

---

## 🎓 总结

**Web 调试模式**是 CCR Desktop 开发的重要补充：

- 🌐 **纯 Web 化**：在浏览器中获得完整的应用体验
- 🔧 **调试友好**：利用浏览器强大的开发工具
- 🌍 **访问灵活**：支持远程访问和团队协作
- ⚡ **性能优秀**：无需图形渲染，资源占用更低

**推荐工作流**：
1. 日常开发：`just dev-wsl`（桌面体验）
2. 接口调试：`just dev-web`（Web 调试）
3. 纯前端：`just ui-dev`（UI 开发）

---

## 🎉 立即开始

```bash
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web
```

然后在浏览器打开：**http://localhost:5173**

**享受强大的 Web 调试体验吧！** 🚀

---

*Web 调试模式 - 为现代开发者设计* ✨
