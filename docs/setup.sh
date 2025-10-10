#!/bin/bash
# CCR 文档快速启动脚本

set -e

echo "🚀 CCR 文档快速启动"
echo "===================="
echo ""

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 未安装 Node.js"
    echo "请先安装 Node.js: https://nodejs.org/"
    exit 1
fi

echo "✓ Node.js 版本: $(node --version)"

# 检查 npm
if ! command -v npm &> /dev/null; then
    echo "❌ 错误: 未安装 npm"
    exit 1
fi

echo "✓ npm 版本: $(npm --version)"
echo ""

# 安装依赖
echo "📦 安装依赖..."
npm install

echo ""
echo "✓ 依赖安装完成"
echo ""

# 启动选项
echo "请选择操作:"
echo "1) 启动开发服务器 (npm run dev)"
echo "2) 构建文档 (npm run build)"
echo "3) 预览构建结果 (npm run preview)"
echo "4) 退出"
echo ""
read -p "请输入选项 [1-4]: " choice

case $choice in
    1)
        echo ""
        echo "🚀 启动开发服务器..."
        echo "浏览器将自动打开 http://localhost:5173"
        echo "按 Ctrl+C 停止服务器"
        echo ""
        npm run dev
        ;;
    2)
        echo ""
        echo "🔨 构建文档..."
        npm run build
        echo ""
        echo "✓ 构建完成！"
        echo "产物位于: .vitepress/dist/"
        ;;
    3)
        echo ""
        echo "👀 预览构建结果..."
        npm run preview
        ;;
    4)
        echo "再见！"
        exit 0
        ;;
    *)
        echo "无效选项"
        exit 1
        ;;
esac

