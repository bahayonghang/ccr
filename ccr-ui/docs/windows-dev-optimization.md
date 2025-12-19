# Windows 并行启动优化说明

## 📝 优化内容

优化了 `just dev` 命令在 Windows 平台下的行为，从"打开2个新窗口"改为"在当前窗口内后台运行"。

## 🔄 变更文件

1. **新增文件**: `ccr-ui/scripts/dev-parallel-windows.ps1`
   - 专门的 Windows 并行启动脚本
   - 使用 PowerShell 后台作业（`Start-Job`）管理后端进程
   
2. **修改文件**: `ccr-ui/justfile`
   - 简化 `_dev-parallel-windows` 函数，调用上述脚本

## ⚡ 新旧行为对比

### 旧行为（优化前）
```powershell
# 使用 Start-Process 打开新 PowerShell 窗口
Start-Process powershell -WindowStyle Normal  # 后端窗口
Start-Process powershell -WindowStyle Normal  # 前端窗口
```
**表现**：
- ❌ 会弹出 **2 个新的 PowerShell 窗口**
- ❌ 窗口管理繁琐
- ✅ 日志分离清晰
- ✅ 可独立关闭

### 新行为（优化后）
```powershell
# 使用后台作业在当前窗口运行
$backendJob = Start-Job { ... }  # 后台运行
bun run dev                       # 前台运行
```
**表现**：
- ✅ **不打开新窗口**，在当前终端运行
- ✅ 前端日志实时可见
- ✅ `Ctrl+C` 一键停止前后端
- ✅ 类似 Linux/macOS 体验
- ⚠️ 后端日志默认不显示（写入 `logs/backend-console.log`）

## 🛠️ 使用方式

**没有变化！** 仍然使用相同命令：

```bash
# 启动开发环境
just dev
# 或简写
just s
```

## 📊 运行效果

```
🚀 启动开发环境（后台并行模式）...

🦀 启动后端服务器（后台作业）...
✅ 后端已在后台启动 (Job ID: 1)
   后端日志: logs/backend-console.log

⏳ 等待后端就绪 (http://127.0.0.1:38081/health)...
✅ 后端就绪

⚛️  启动前端服务器（前台运行，实时日志可见）...
   前端日志: logs/frontend.log

💡 提示: 按 Ctrl+C 退出时会自动停止后端服务器
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  VITE v5.x.x  ready in xxx ms
  ➜  Local:   http://localhost:5173/
  ➜  Network: http://192.168.x.x:5173/
```

## 🔍 查看后端日志

由于后端在后台运行，如需查看实时日志：

### 方法 1：查看日志文件
```powershell
Get-Content logs/backend-console.log -Tail 50 -Wait
```

### 方法 2：只启动后端（前台运行）
```bash
just dev-backend
```

## 🐛 故障排查

### 如果后端启动失败
1. 查看日志：`cat logs/backend-console.log`
2. 检查端口占用：`netstat -ano | findstr :38081`

### 如果前端无法连接后端
1. 确认后端健康检查：`curl http://127.0.0.1:38081/health`
2. 查看后端日志确认服务已启动

### 如果需要强制清理进程
```bash
just dev-clean
```

## 🎯 技术细节

### PowerShell 后台作业 vs Start-Process

| 特性 | Start-Process | Start-Job |
|------|---------------|-----------|
| 窗口行为 | 新窗口 | 后台运行 |
| 日志可见性 | 实时可见（独立窗口） | 需要 `Receive-Job` 查看 |
| 进程管理 | 需手动关闭窗口 | `Stop-Job` 自动清理 |
| 脚本集成 | 难以控制生命周期 | 易于 try-finally 管理 |

### 错误处理
脚本包含完善的错误处理：
- ✅ 后端启动失败自动退出
- ✅ 健康检查超时（90秒）自动退出
- ✅ `Ctrl+C` 触发 `finally` 清理后台作业
- ✅ 后端异常退出显示最近20行日志

## 📚 相关命令

```bash
# 只启动后端（前台运行，日志可见）
just dev-backend

# 只启动前端
just dev-frontend

# 清理残留进程
just dev-clean

# 完整启动（推荐）
just dev
```

## 🚀 下一步优化建议

如果需要更好的体验，可以考虑：

1. **使用 Windows Terminal**
   - 支持标签页管理
   - 可以将前后端运行在不同标签页

2. **添加日志查看命令**
   - `just logs-backend`: 实时查看后端日志
   - `just logs-frontend`: 实时查看前端日志

3. **进程监控**
   - 使用 `Get-Job` 查看后台作业状态
   - 集成到 `just status` 命令

---

**优化时间**: 2025-12-18  
**影响范围**: Windows 平台开发体验  
**兼容性**: 完全向下兼容，无需修改工作流程
