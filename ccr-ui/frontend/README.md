# CCR UI - Frontend (Vue.js 3)

> AI CLI Configuration Manager - Modern Web Console with Vue.js 3 and Liquid Glass Design

基于 Vue.js 3 构建的现代化 Web 控制台，用于管理多种 AI CLI 工具配置。

## ✨ 特性

- **现代化 UI**: 采用液态玻璃（Liquid Glass）设计风格，结合 Material Design 原则
- **响应式设计**: 支持桌面端和移动端访问
- **主题系统**: 支持亮色/暗色主题切换
- **模块化架构**: 组件化开发，易于维护和扩展
- **TypeScript**: 完整的类型安全
- **API 集成**: 与后端 API 完整集成

## 🚀 快速开始

### 环境要求

- Node.js 18.x 或更高版本
- npm 或 yarn

### 安装与运行

1. **安装依赖**:

```bash
npm install
```

2. **开发模式**:

```bash
npm run dev
```

应用将在 `http://localhost:3000` 上运行

3. **构建生产版本**:

```bash
npm run build
```

## 🏗️ 项目结构

```
src/
├── assets/           # 静态资源
├── components/       # 可复用组件
├── views/            # 页面视图
├── router/           # 路由配置
├── store/            # Pinia 状态管理
├── styles/           # 全局样式
├── api/              # API 客户端
├── types/            # TypeScript 类型定义
└── utils/            # 工具函数
```

## 🎨 设计风格

### 液态玻璃设计 (Liquid Glass)

- **背景**: 渐变背景配合动态模糊效果
- **卡片**: 使用 `backdrop-filter: blur()` 实现玻璃态效果
- **动画**: 流畅的过渡和悬停效果
- **色彩**: 采用 CSS 变量系统实现主题切换

### 主题系统

- **亮色主题**: 以蓝紫色为主色调
- **暗色主题**: 深色背景配柔和高亮
- **CSS 变量**: 统一管理颜色、阴影、边框等样式变量

## 📊 功能模块

- **首页**: 系统概览和模块导航
- **Claude Code**: 配置管理、云同步、MCP 服务器、Agents、插件
- **Codex**: MCP 服务器、Profiles、基础配置
- **Gemini CLI**: 配置管理和工具集成
- **Qwen**: 阿里通义千问配置管理
- **IFLOW**: 工作流配置管理
- **命令中心**: 统一的 CLI 命令执行界面
- **配置转换器**: 跨平台配置格式转换
- **云同步**: WebDAV 云端配置同步

## 🛠️ 技术栈

- **框架**: Vue.js 3 (Composition API)
- **路由**: Vue Router 4
- **状态管理**: Pinia
- **HTTP 客户端**: Axios
- **样式**: Tailwind CSS + 自定义 CSS
- **图标**: Lucide Icons
- **构建工具**: Vite
- **类型检查**: TypeScript

## 🔧 开发

### 添加新页面

1. 在 `src/views/` 中创建页面组件
2. 在 `src/router/index.ts` 中添加路由配置

### 添加新组件

1. 在 `src/components/` 中创建组件
2. 按照液态玻璃设计原则编写样式

## 📱 响应式设计

项目采用响应式设计：
- 使用 Tailwind CSS 的响应式类
- 在移动设备上优化触摸体验
- 减少移动设备上的动画以提升性能

## 🌙 主题切换

主题系统通过 CSS 变量实现：
- 亮色/暗色主题无缝切换
- localStorage 持久化主题选择
- 通过 Pinia 统一管理主题状态

## 🚀 部署

构建完成后，将 `dist/` 目录中的文件部署到静态服务器。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

Made with ❤️ using Vue.js 3, TypeScript, and Liquid Glass Design