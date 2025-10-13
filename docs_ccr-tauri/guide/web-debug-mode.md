# Web 调试模式

本小姐给你详细介绍 CCR Desktop 的 Web 调试模式！这可是特别适合远程开发和 WSL 环境的强大功能呢！(￣▽￣)ゞ

## 什么是 Web 调试模式？

Web 调试模式是 CCR Desktop 的**双模式运行**特性之一，它允许你在**没有桌面窗口**的情况下运行应用，通过浏览器访问完整的功能。

### 两种运行模式对比

| 特性 | 桌面模式 | Web 调试模式 |
|-----|---------|-------------|
| 运行方式 | Tauri 窗口 | 浏览器 |
| 前后端通信 | IPC (进程间通信) | HTTP API |
| 适用场景 | 日常使用 | 远程开发、WSL、调试 |
| DevTools | Tauri DevTools | 浏览器 DevTools |
| 性能 | 原生性能 | 网络延迟 |
| 依赖 | 图形界面 | 仅需浏览器 |

## 🚀 快速开始

### 启动 Web 调试模式

```bash
cd ccr-tauri
just dev-web
```

启动后你会看到：

```
╔═══════════════════════════════════════════════════════════╗
║  🌐 Web 调试模式已启动                                   ║
║                                                           ║
║  🎨 前端界面: http://localhost:5173                       ║
║  📡 后端 API: http://localhost:3030                       ║
║                                                           ║
║  💡 使用方法:                                              ║
║    1. 在浏览器中打开 http://localhost:5173                ║
║    2. F12 打开 DevTools 查看 Network 请求                 ║
║    3. 前端调用后端 API 进行配置操作                       ║
║                                                           ║
║  📋 实时日志:                                              ║
║    tail -f /tmp/ccr-web.log    (后端 API 日志)            ║
║    tail -f /tmp/vite-dev.log   (前端构建日志)             ║
║                                                           ║
║  ⏹️  停止服务: just stop-web                              ║
╚═══════════════════════════════════════════════════════════╝
```

### 访问应用

在浏览器中打开 http://localhost:5173（开发环境地址），你会看到完整的 CCR Desktop 界面！

## 🛠️ 管理 Web 服务

### 查看服务状态

```bash
just web-status
```

输出示例：
```
▶ Web 调试服务状态...

✅ 后端 API 服务器 (PID: 12345)
   📡 http://localhost:3030
✅ 前端开发服务器 (PID: 12346)
   🎨 http://localhost:5173

📊 端口监听状态:
   0.0.0.0:3030 → tcp
   127.0.0.1:5173 → tcp
```

### 查看日志

```bash
# 查看最近的日志
just web-logs

# 实时跟踪日志
just web-logs-follow
```

### 停止服务

```bash
just stop-web
```

## 📐 架构设计

### 系统架构

```
┌─────────────────────────────────────────────┐
│           浏览器 (http://localhost:5173)     │
│  ┌─────────────────────────────────────┐    │
│  │       Vue 3 前端                     │    │
│  │  • UI 组件                           │    │
│  │  • API 适配器 (HTTP 模式)            │    │
│  └────────────┬────────────────────────┘    │
└───────────────┼─────────────────────────────┘
                │ HTTP Request/Response
                ▼
┌───────────────────────────────────────────────┐
│  CCR Web API 服务器 (localhost:3030)          │
│  ┌─────────────────────────────────────┐     │
│  │  • 11 个 API 端点                    │     │
│  │  • 完整的 CRUD 操作                  │     │
│  │  • 调用 CCR 核心库                   │     │
│  └────────────┬────────────────────────┘     │
└───────────────┼───────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────┐
│  CCR 核心库 (Rust)                            │
│  • ConfigService                              │
│  • SettingsService                            │
│  • HistoryService                             │
│  • 文件操作                                   │
└───────────────────────────────────────────────┘
```

### API 适配器

前端的 `api/index.ts` 实现了智能适配，根据运行环境自动选择通信方式：

```typescript
// 检测运行环境
const isTauriEnvironment = window.__TAURI__ !== undefined

export async function listConfigs(): Promise<Config[]> {
  if (isTauriEnvironment) {
    // 桌面模式：使用 Tauri IPC
    return await invoke<Config[]>('list_configs')
  } else {
    // Web 模式：调用 HTTP API
    const response = await fetch('http://localhost:3030/api/configs')
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    const data = await response.json()
    return data.configs
  }
}
```

这种设计的优点：
- **业务代码无感知**: Vue 组件不需要知道运行模式
- **统一接口**: 所有 API 函数在两种模式下行为一致
- **易于维护**: 只需在一个地方维护适配逻辑
- **类型安全**: TypeScript 保证两种模式的类型一致

### HTTP API 端点

后端提供 11 个 RESTful API 端点：

| 方法 | 端点 | 功能 |
|-----|------|------|
| GET | `/api/configs` | 获取所有配置 |
| GET | `/api/configs/:name` | 获取单个配置 |
| POST | `/api/configs` | 创建配置 |
| PUT | `/api/configs/:name` | 更新配置 |
| DELETE | `/api/configs/:name` | 删除配置 |
| POST | `/api/configs/:name/switch` | 切换配置 |
| GET | `/api/configs/current` | 获取当前配置 |
| GET | `/api/history` | 获取操作历史 |
| POST | `/api/validate` | 验证配置 |
| POST | `/api/export` | 导出配置 |
| POST | `/api/import` | 导入配置 |

## 🎯 使用场景

### 1. WSL 环境开发

在 WSL 环境中，Tauri 窗口可能遇到性能问题或滚轮无法使用。Web 调试模式是完美的替代方案：

```bash
# 在 WSL 中启动 Web 模式
cd /mnt/c/Users/YourName/Projects/ccr/ccr-tauri
just dev-web

# 在 Windows 浏览器中访问
# http://localhost:5173
```

### 2. 远程服务器开发

通过 SSH 连接到远程服务器时，无需 X11 转发：

```bash
# SSH 到服务器
ssh user@remote-server

# 启动 Web 模式
cd ~/ccr/ccr-tauri
just dev-web

# 在本地浏览器访问 (需要端口转发)
# ssh -L 5173:localhost:5173 -L 3030:localhost:3030 user@remote-server
```

### 3. 前端开发调试

使用浏览器 DevTools 的完整功能：

- **Network 面板**: 查看所有 API 请求和响应
- **Console 面板**: 查看 Vue 应用的日志
- **Elements 面板**: 实时修改 CSS 和 DOM
- **Performance 面板**: 分析性能瓶颈
- **Vue DevTools**: 查看组件树和状态

### 4. API 测试

直接使用 curl 或 Postman 测试后端 API：

```bash
# 获取所有配置
curl http://localhost:3030/api/configs

# 切换配置
curl -X POST http://localhost:3030/api/configs/anthropic/switch

# 获取历史记录
curl http://localhost:3030/api/history
```

## 🐛 故障排查

### 问题: 服务启动失败

**诊断步骤**:

1. **检查服务状态**:
   ```bash
   just web-status
   ```

2. **查看日志**:
   ```bash
   # 后端日志
   tail -f /tmp/ccr-web.log
   
   # 前端日志
   tail -f /tmp/vite-dev.log
   ```

3. **检查端口占用**:
   ```bash
   # 检查 5173 (前端) 和 3030 (后端) 端口
   ss -tuln | grep -E "(5173|3030)"
   
   # 或使用 netstat
   netstat -tuln | grep -E "(5173|3030)"
   ```

4. **杀死占用端口的进程**:
   ```bash
   # 查找占用端口的进程
   lsof -i :5173
   lsof -i :3030
   
   # 杀死进程
   kill -9 <PID>
   ```

### 问题: 前端可以访问但 API 调用失败

**症状**: 浏览器可以打开 http://localhost:5173，但所有 API 请求返回 404 或网络错误。

**原因**: 后端 API 服务未启动或启动失败。

**解决方案**:

1. **检查后端日志**:
   ```bash
   tail -f /tmp/ccr-web.log
   ```

2. **确保 CCR 已安装**:
   ```bash
   which ccr
   # 应该输出: /home/user/.cargo/bin/ccr
   
   ccr --version
   # 应该输出版本号
   ```

3. **手动启动后端服务测试**:
   ```bash
   cd ccr  # 进入 CCR 根目录
   ccr web -p 3030
   ```

4. **检查 CCR 配置文件**:
   ```bash
   # 确保配置文件存在
   ls -la ~/.ccs_config.toml
   ```

### 问题: CORS 错误

**症状**: 浏览器控制台显示 CORS (Cross-Origin Resource Sharing) 错误。

**原因**: 前端 (localhost:5173) 和后端 (localhost:3030) 是不同的源。

**解决方案**: CCR Web API 服务器已经配置了 CORS，允许所有来源访问。如果仍然遇到问题：

1. **检查后端配置**:
   ```bash
   # 查看 ccr web 命令的 CORS 配置
   ccr web --help
   ```

2. **使用浏览器扩展** (临时方案):
   - 安装 "Allow CORS" 或类似扩展
   - 仅在开发时使用，生产环境不应该这样做

### 问题: 性能慢

**症状**: Web 模式下界面响应慢，API 请求时间长。

**原因**: HTTP 通信比 IPC 有额外的网络延迟。

**解决方案**:

1. **使用本地环境**: 确保前后端都在 localhost
2. **优化 API 调用**: 减少不必要的请求，使用缓存
3. **考虑使用桌面模式**: 如果性能是关键需求

## 💡 最佳实践

### 1. 开发工作流

推荐的开发工作流：

```bash
# 1. 启动 Web 调试模式
just dev-web

# 2. 在浏览器中打开应用
# http://localhost:5173

# 3. 打开另一个终端，实时查看日志
just web-logs-follow

# 4. 修改前端代码，自动热重载

# 5. 测试 API (如果需要)
curl http://localhost:3030/api/configs

# 6. 完成开发后停止服务
just stop-web
```

### 2. 调试技巧

- **使用 Vue DevTools**: 安装 Vue DevTools 浏览器扩展，查看组件状态和事件
- **Network 面板**: 查看 API 请求的详细信息，包括请求头、响应体、耗时等
- **Preserve log**: 在 DevTools 中启用 "Preserve log"，防止页面刷新时清空日志
- **断点调试**: 在 Sources 面板设置断点，单步调试 JavaScript 代码

### 3. 性能优化

- **缓存 API 响应**: 对于不常变化的数据，使用前端缓存
- **批量请求**: 合并多个 API 请求，减少网络开销
- **懒加载**: 使用 Vue 的异步组件和路由懒加载

### 4. 安全注意事项

::: warning 安全警告
Web 调试模式**仅用于开发环境**，不应该在生产环境中使用！

原因：
- 后端 API 没有身份验证
- CORS 配置允许所有来源
- 日志文件包含敏感信息
- 端口暴露在本地网络

如果需要远程访问，请使用 SSH 隧道或 VPN，而不是直接暴露端口。
:::

## 🆚 对比其他调试方式

### Web 调试 vs Tauri DevTools

| 特性 | Web 调试模式 | Tauri DevTools |
|-----|-------------|----------------|
| 环境要求 | 浏览器 | 图形界面 |
| WSL 支持 | ✅ 完美 | ⚠️ 可能有问题 |
| DevTools | 完整 | 有限 |
| 远程访问 | ✅ 简单 | ❌ 困难 |
| 性能 | 稍慢 | 原生 |
| API 测试 | ✅ 简单 | ❌ 需要工具 |

### Web 调试 vs 纯前端模式

CCR Desktop 也提供了 `just dev-web-frontend-only` 命令，只启动前端服务器，不启动后端 API。

| 特性 | Web 调试模式 | 纯前端模式 |
|-----|-------------|-----------|
| 前端预览 | ✅ | ✅ |
| API 功能 | ✅ 完整 | ❌ 不可用 |
| 配置操作 | ✅ | ❌ |
| 适用场景 | 完整测试 | UI/样式测试 |

## 📚 相关资源

- [快速开始指南](/guide/getting-started) - 基础安装和使用
- [开发指南](/development/getting-started) - 深入的开发知识
- [架构设计](/architecture/overview) - 系统架构和 API 适配器
- [WSL 环境问题](/troubleshooting/wsl-issues) - WSL 专项故障排查

---

**Made with ❤️ by 哈雷酱**

哼，Web 调试模式可是本小姐特别为 WSL 用户和远程开发者设计的呢！(￣▽￣)／
现在你可以在任何有浏览器的地方开发 CCR Desktop 了！
笨蛋你要是还不会用的话...(,,><,,)

