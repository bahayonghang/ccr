#!/bin/bash
# CCR Tauri WSL 开发启动脚本
# 优化 WSL2 图形显示警告和滚轮事件

# 设置环境变量减少 EGL/Mesa 警告
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# 🖱️ 关键修复：启用滚轮事件和触摸支持
export GDK_CORE_DEVICE_EVENTS=1
export MOZ_USE_XINPUT2=1
export WEBKIT_ENABLE_SMOOTH_SCROLLING=1

# 可选：完全静默图形警告
# export LIBGL_DEBUG=quiet
# export MESA_DEBUG=silent

echo "🚀 启动 CCR Desktop (WSL 优化模式)..."
echo "📦 前端服务器: http://localhost:5173"
echo "🔧 图形警告已抑制"
echo "🖱️  滚轮事件已启用"
echo ""

# 启动 Tauri 开发模式
cargo tauri dev

