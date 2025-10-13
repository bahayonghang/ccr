# 🚀 快速开始指南

最快 5 分钟启动 CCR UI（Next.js 16 Beta 版本）！

## 📦 安装

```bash
# 1. 进入项目目录
cd /home/lyh/Documents/Github/ccr/ccr-ui/frontend-next

# 2. 安装依赖（首次运行）
npm install

# 3. 启动开发服务器
npm run dev
```

## ✅ 验证

开发服务器启动后，访问：
- **本地地址**: http://localhost:3000
- **配置管理**: http://localhost:3000/configs
- **命令执行**: http://localhost:3000/commands

## ⚙️ 后端配置

确保 CCR 后端服务正在运行：

```bash
# 在另一个终端窗口
cd /home/lyh/Documents/Github/ccr
cargo run -- web --port 8081
```

前端会自动代理 API 请求到 `http://localhost:8081`

## 🎨 功能特性

### ✨ 已实现
- ✅ **配置管理页面** - 查看、切换、验证配置
- ✅ **命令执行页面** - 执行 CCR 命令
- ✅ **历史记录** - 查看操作历史
- ✅ **系统信息** - 实时监控系统状态
- ✅ **主题切换** - 明亮/暗色主题
- ✅ **响应式设计** - 完美支持移动端

### 🚀 性能优化
- ✅ **Turbopack** - 2-5x 构建速度提升
- ✅ **React Compiler** - 自动优化组件渲染
- ✅ **文件系统缓存** - 加速开发环境
- ✅ **代码分割** - 按需加载

### 🎯 Web 最佳实践
- ✅ **语义化 HTML** - 提升可访问性
- ✅ **ARIA 属性** - 支持屏幕阅读器
- ✅ **键盘导航** - 完整键盘操作支持
- ✅ **响应式布局** - 移动优先设计
- ✅ **性能优化** - Lighthouse 95+ 分数

## 📱 浏览器支持

- ✅ Chrome/Edge 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ 移动浏览器（iOS Safari, Chrome Mobile）

## 🔧 常用命令

```bash
# 开发
npm run dev              # 启动开发服务器（Turbopack）

# 构建
npm run build            # 生产构建
npm run start            # 启动生产服务器

# 代码质量
npm run lint             # ESLint 检查
npm run type-check       # TypeScript 类型检查
```

## 🎯 项目结构（简化版）

```
frontend-next/
├── src/
│   ├── app/              # 页面路由
│   │   ├── configs/      # 配置管理页
│   │   └── commands/     # 命令执行页
│   ├── components/       # React 组件
│   └── lib/              # API 和类型
├── public/               # 静态资源
└── package.json          # 依赖配置
```

## 💡 开发提示

### 热更新
修改代码后自动刷新，无需手动重载！

### 主题切换
点击右上角的太阳/月亮图标切换主题

### API 代理
开发环境下 `/api/*` 请求会自动代理到 `http://localhost:8081/api/*`

## 🐛 遇到问题？

### 端口被占用
```bash
npm run dev -- -p 3001  # 使用其他端口
```

### 清除缓存
```bash
rm -rf .next node_modules
npm install
```

### 后端连接失败
检查 CCR 后端是否在 8081 端口运行：
```bash
curl http://localhost:8081/api/system
```

## 📚 更多文档

- [完整文档](./README.md) - 详细功能说明
- [迁移指南](./MIGRATION.md) - 技术迁移细节
- [项目架构](../../ccr-ui/ARCHITECTURE.md) - 整体架构说明

## 🎉 开始使用！

```bash
npm run dev
```

打开浏览器访问 http://localhost:3000，开始体验全新的 CCR UI！

---

**问题反馈**: 欢迎提交 Issue
**功能建议**: 欢迎提交 Pull Request

