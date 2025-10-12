#!/bin/bash
# CCR Tauri 前端开发服务器启动脚本

# 获取脚本所在目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# 进入 src-ui 目录
cd "$SCRIPT_DIR/src-ui" || exit 1

# 启动 Vite 开发服务器
exec npm run dev
