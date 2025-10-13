# 🌐 CCR Desktop - Web 调试模式完整指南

## 🎯 新功能：Web 调试模式

**已完成**：为您创建了专门的 Web 调试模式，支持**前端+后端联调**，**无需桌面窗口**！

---

## ✨ 功能特点

### 🌟 主要优势

| 特点 | 说明 | 优势 |
|------|------|------|
| 🌐 **浏览器调试** | 在 Chrome/Edge 中运行 | 强大的开发工具 |
| 📡 **真实 API** | 前端调用后端 HTTP API | 完整的前后端联调 |
| 🔥 **热重载** | 前端代码自动刷新 | 高效开发体验 |
| 🌍 **远程友好** | 支持远程访问 | WSL/服务器调试 |
| ⚡ **轻量级** | 无需图形界面 | 资源占用更低 |

### 🆚 与桌面模式对比

| 特性 | 桌面模式 | Web 调试模式 |
|------|----------|--------------|
| 界面 | Tauri 原生窗口 | 浏览器页面 |
| 调试工具 | Tauri DevTools | Chrome DevTools ⭐ |
| API 调用 | Tauri invoke | HTTP fetch ⭐ |
| 远程访问 | ❌ 不支持 | ✅ 支持 |
| WSL 兼容性 | 需要图形界面 | 完美支持 ⭐ |
| 滚轮功能 | 已修复 | 原生支持 ✅ |

---

## 🚀 快速开始

### 1. 启动 Web 调试模式

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
║  🎨 前端界面: http://localhost:5173                       ║
║  📡 后端 API: http://localhost:3030                       ║
╚═══════════════════════════════════════════════════════════╝

✅ 后端服务器启动成功 (PID: xxxxx)
✅ 前端服务器启动成功 (PID: xxxxx)

🌐 在浏览器中访问: http://localhost:5173
💡 停止服务使用: just stop-web
```

### 2. 在浏览器中打开

访问：**http://localhost:5173**

### 3. 验证运行模式

F12 打开开发者工具，在 Console 中应该看到：
```
🔧 运行模式: 🌐 Web 浏览器
✅ Web API 连接正常
🔧 API 配置信息:
   运行模式: 🌐 Web 浏览器
   后端地址: http://localhost:3030
   前端地址: http://localhost:5173
🖱️ 初始化滚轮事件修复...
✅ 滚轮事件修复已启用
```

### 4. 界面标识

在应用顶部导航栏，您会看到：
- 桌面模式：`🖥️ 桌面` (紫色徽章)
- Web 模式：`🌐 Web` (绿色闪烁徽章)

---

## 📋 所有调试命令

| 命令 | 功能 | 使用场景 |
|------|------|----------|
| `just dev-web` | 🚀 启动 Web 调试模式 | 开始 Web 调试 ⭐ |
| `just web-status` | 📊 查看服务状态 | 检查运行状态 |
| `just web-logs` | 📋 查看日志 | 快速查看输出 |
| `just web-logs-follow` | 📊 实时日志 | 持续监控 |
| `just stop-web` | ⏹️ 停止所有服务 | 清理退出 |

---

## 🔧 调试工作流

### 典型调试流程

```bash
# 1. 启动 Web 调试模式
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web

# 2. 在浏览器中打开
# http://localhost:5173

# 3. F12 → Network 标签
# 观察 API 调用：
# - GET http://localhost:3030/api/configs
# - POST http://localhost:3030/api/switch
# - 等等...

# 4. 修改前端代码（自动热重载）
# 编辑 src-ui/src/ 下的 .vue/.ts/.css 文件

# 5. 查看实时日志（新终端窗口）
just web-logs-follow

# 6. 停止服务
just stop-web
```

### 前后端联调测试

1. **修改前端** → 自动热重载 ✅
2. **修改后端** → 需要重启后端：
   ```bash
   just stop-web
   just dev-web
   ```

---

## 🌍 适用场景

### ✅ 特别适合

1. **WSL 开发** - 避免图形界面复杂性
2. **远程服务器** - SSH 连接的开发环境
3. **API 调试** - 专门测试接口逻辑
4. **CI/CD 测试** - 自动化测试环境
5. **团队协作** - 共享调试环境
6. **移动端测试** - 手机浏览器访问

### 🔧 调试任务

- **接口联调** - 观察 HTTP 请求/响应
- **状态管理** - Vue DevTools 调试组件状态
- **性能分析** - Chrome 性能分析工具
- **错误排查** - 完整的 JavaScript 调用堆栈
- **响应式测试** - 模拟不同设备尺寸

---

## 📊 技术实现

### 自动 API 适配

```typescript
// 业务代码保持不变
const configs = await listConfigs()

// 适配器自动选择:
// Tauri 模式: invoke('list_configs')
// Web 模式: fetch('http://localhost:3030/api/configs')
```

### 环境自动检测

```typescript
function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

// 🖥️ Tauri 环境: __TAURI__ 对象存在
// 🌐 Web 环境: 纯浏览器，无 __TAURI__ 对象
```

### 双模式支持

**前端** (端口 5173):
- Vue 3 + TypeScript + Vite
- 自动热重载
- 响应式设计

**后端** (端口 3030):
- CCR Web 服务器
- REST API 接口
- 与 Tauri Commands 功能一致

---

## 🛠️ 高级配置

### 自定义端口

编辑 `justfile` 中的端口配置：

```makefile
# 后端端口 (默认 3030)
cd .. && nohup ccr web -p 8080

# 前端端口 (默认 5173)
# 需要修改 vite.config.ts
```

### 远程访问配置

允许其他设备访问：

```typescript
// vite.config.ts
export default defineConfig({
  server: {
    host: '0.0.0.0',  // 监听所有网络接口
    port: 5173
  }
})
```

### 生产环境部署

```bash
# 构建生产版本
just ui-build

# 启动生产服务器
cd .. && ccr web -p 3030 &
cd src-ui && npx serve dist -p 5173
```

---

## 🔍 故障排查

### 常见问题及解决方案

#### Q1: 后端 API 连接失败

**症状**: Console 显示 "❌ Web API 连接失败"

**解决**:
```bash
# 检查后端是否启动
just web-status

# 查看后端日志
just web-logs

# 检查端口占用
ss -tuln | grep 3030

# 重启后端
just stop-web && just dev-web
```

#### Q2: 前端热重载不工作

**症状**: 修改代码后页面不刷新

**解决**:
```bash
# 重启前端服务器
just stop-web
just dev-web

# 或手动刷新浏览器
```

#### Q3: API 调用报错

**症状**: Network 标签显示 HTTP 错误

**解决**:
```bash
# 1. 检查 API 配置
# 在浏览器 Console 运行:
import { getApiConfig } from './api'
getApiConfig()

# 2. 测试 API 连接
import { testApiConnection } from './api'
await testApiConnection()

# 3. 查看详细日志
just web-logs-follow
```

---

## 📈 性能和监控

### 资源占用

| 服务 | CPU | 内存 | 描述 |
|------|-----|------|------|
| CCR Web 后端 | ~2% | ~30MB | Rust HTTP 服务器 |
| Vite 前端 | ~3% | ~80MB | Node.js 开发服务器 |
| **总计** | **~5%** | **~110MB** | 轻量级 |

### 监控命令

```bash
# 实时状态监控
watch -n 2 'just web-status'

# 查看资源占用
ps aux | grep -E "(ccr web|vite)" | grep -v grep

# 网络连接状态
ss -tuln | grep -E "(3030|5173)"
```

---

## 🎓 开发最佳实践

### ✅ 推荐工作流

1. **功能开发** → `just dev-wsl` (桌面体验)
2. **API 调试** → `just dev-web` (Web 调试) ⭐
3. **UI 设计** → `just ui-dev` (纯前端)
4. **生产测试** → `just release` (打包)

### 🔧 调试技巧

1. **Network 标签调试**
   - 查看 API 请求/响应
   - 检查请求参数
   - 分析响应时间

2. **Console 调试**
   - 查看运行模式：`getRunMode()`
   - 测试连接：`testApiConnection()`
   - 查看配置：`getApiConfig()`

3. **Vue DevTools**
   - 组件状态调试
   - 响应式数据追踪
   - 性能分析

---

## 🎉 立即体验

### 启动命令

```bash
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web
```

### 测试步骤

1. **启动服务** → 等待成功消息
2. **浏览器打开** → http://localhost:5173  
3. **F12 开发工具** → 查看 Console 日志
4. **测试功能** → 配置切换、创建、删除等
5. **Network 调试** → 观察 API 调用
6. **滚轮测试** → 验证滚动功能

---

## 📚 相关文档

- **`WEB_DEBUG_MODE.md`** - 完整技术文档
- **`justfile`** - 所有可用命令
- **`src-ui/src/api/adapter.ts`** - API 适配器源码

---

## 💡 技术亮点

### 🔌 智能 API 适配器

自动检测运行环境，无缝切换调用方式：

```typescript
// 同一份代码，两种运行方式：

// 🖥️ 桌面模式
await invoke('list_configs')

// 🌐 Web 模式  
await fetch('http://localhost:3030/api/configs')

// 业务代码完全不变！
const configs = await listConfigs()
```

### 🎨 视觉化运行模式

界面顶部会显示当前运行模式：
- `🖥️ 桌面` - Tauri 原生模式（紫色徽章）
- `🌐 Web` - 浏览器调试模式（绿色闪烁徽章）

### 🔍 完整的调试支持

- ✅ 自动 API 连接测试
- ✅ 详细的 Console 日志
- ✅ 实时状态监控
- ✅ 分离的前后端日志

---

## 🎊 总结

现在您有了 **3 种不同的开发模式**：

| 模式 | 命令 | 用途 |
|------|------|------|
| 🖥️ **标准桌面** | `just dev` | 日常开发，原生体验 |
| 🪟 **WSL 桌面** | `just dev-wsl` | WSL 优化，滚轮修复 |
| 🌐 **Web 调试** | `just dev-web` | API 调试，远程访问 ⭐ |

**推荐使用 Web 调试模式**进行：
- 🔧 接口联调和调试
- 🌍 远程开发和协作
- 🧪 自动化测试
- 📱 移动端兼容性测试

---

## 🎮 立即开始

```bash
# 体验新的 Web 调试模式
cd ~/Documents/Github/ccr/ccr-tauri
just dev-web

# 然后在浏览器打开：http://localhost:5173
```

**享受强大的 Web 调试体验！** 🚀

---

*Web 调试模式创建完成 - 2025-10-13*  
*支持 Tauri 2.0 + Vue 3 + TypeScript 双模式架构*
