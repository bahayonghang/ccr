---
layout: home

hero:
  name: "CCR UI"
  text: "现代化 AI 配置管理平台"
  tagline: "基于 Vue 3 + Rust 构建的全栈多 CLI 工具配置管理 Web 应用"
  image:
    src: /logo.svg
    alt: CCR UI Logo
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 查看源码
      link: https://github.com/your-username/ccr

features:
  - icon: ⚡
    title: 现代化技术栈
    details: 前端采用 Vue 3.5 + Vite 7.1 + TypeScript + Tailwind CSS，后端使用 Rust + Axum，提供极致的开发体验和运行性能
  - icon: 🎯
    title: 直观的用户界面
    details: 科技风格的玻璃拟态设计，支持深色/浅色主题，Dashboard 首页展示所有功能模块，一目了然
  - icon: 🔧
    title: 强大的配置管理
    details: 支持 CCR 配置的查看、切换、验证、云同步等操作，实时显示命令执行结果，历史记录追踪
  - icon: 🚀
    title: 多 CLI 工具支持
    details: 统一管理 Claude Code、Codex、Gemini CLI、Qwen、IFLOW 等多个 AI CLI 工具的配置和服务
  - icon: 📱
    title: 响应式设计
    details: 完全响应式设计，支持桌面端和移动端访问，流畅的动画效果和悬停交互
  - icon: 🛠️
    title: 开发友好
    details: 完整的开发工具链，支持 Vite 热重载、TypeScript 类型检查、ESLint + Prettier，提供最佳的开发体验
  - icon: ☁️
    title: 云端同步
    details: 支持 WebDAV 云端配置同步和自动备份，配置文件安全无忧
  - icon: 🔌
    title: 插件生态
    details: 支持 MCP 服务器、Agents、Plugins、Slash Commands 等丰富的扩展功能管理
  - icon: 📊
    title: 统计与成本分析
    details: 完整的 API 使用统计和成本追踪系统，实时监控成本、Token 使用、支持按时间/模型/项目多维度分析
---

## 项目特色

CCR UI 是一个现代化的全栈 Web 应用程序，专为多个 AI CLI 工具配置管理而设计。它结合了前端的用户友好界面和后端的高性能处理能力，为开发者提供了一个强大而直观的配置管理平台。

### 🎨 前端技术栈

- **Vue 3.5** - 渐进式 JavaScript 框架，提供响应式数据绑定和组件化开发
- **Vite 7.1** - 下一代前端构建工具，极速的开发服务器和构建性能
- **Vue Router 4.4** - Vue.js 官方路由管理器，支持嵌套路由和导航守卫
- **Pinia 2.2** - Vue 的状态管理库，类型安全且开发友好
- **TypeScript 5.7** - 类型安全的 JavaScript 超集
- **Tailwind CSS 3.4** - 实用优先的 CSS 框架
- **Lucide Vue Next** - 现代化图标库（Vue 版本）
- **Axios 1.7** - 强大的 HTTP 客户端

### ⚙️ 后端技术栈

- **Rust 2024 Edition** - 系统级编程语言，安全且高性能
- **Axum 0.7** - 现代化的异步 Web 框架
- **Tokio** - 异步运行时
- **Serde** - 序列化和反序列化框架
- **Tower** - 中间件和服务抽象

### 📋 核心功能

#### 🏠 Dashboard 首页
- **功能模块导航** - 8 个主要功能模块卡片展示
- **系统监控** - 实时显示 CPU、内存使用率
- **快速访问** - 一键跳转到各个 CLI 工具配置页面

#### 🔵 Claude Code 配置管理
- **配置管理** - 查看、切换、验证 CCR 配置
- **云同步** - WebDAV 云端配置同步
- **MCP 服务器** - Model Context Protocol 服务器管理
- **Slash Commands** - 自定义命令管理
- **Agents** - AI Agent 配置和工具绑定
- **插件管理** - 插件启用/禁用和配置

#### 🎯 多 CLI 工具支持
- **Codex** - MCP 服务器、Profiles、基础配置
- **Gemini CLI** - Google Gemini AI 配置管理
- **Qwen** - 阿里通义千问配置管理
- **IFLOW** - 内部工作流配置

#### 🛠️ 其他功能
- **命令执行中心** - 统一的 CLI 命令执行界面
- **配置转换器** - 跨 CLI 工具的配置格式转换
- **历史记录** - 完整的操作审计日志
- **实时输出** - 终端风格的命令输出显示

#### 📊 统计与成本分析
- **成本追踪** - 精确记录每次 API 调用的成本和 Token 使用
- **多维度统计** - 按时间范围（今日/本周/本月）、模型、项目分组统计
- **可视化仪表板** - 4 个概览卡片（总成本、API 调用、输入/输出 Token）
- **趋势分析** - 成本趋势图表和 Top 会话查询
- **数据导出** - 支持 JSON/CSV 格式导出统计报告
- **实时刷新** - 一键刷新最新数据
- **响应式设计** - 支持深色模式和移动端

### 🎨 设计特点

- **玻璃拟态风格** - 半透明背景 + 模糊效果
- **渐变配色** - 每个 CLI 工具独特的渐变色系
- **流畅动画** - 卡片浮起、箭头移动、渐变流动
- **响应式布局** - 适配桌面、平板、手机等各种设备

## 快速开始

```bash
# 克隆项目
git clone https://github.com/your-username/ccr.git
cd ccr/ccr-ui

# 使用 Just 快速启动（推荐）
just s

# 或者手动启动
cd backend && cargo run &
cd frontend && npm run dev
```

访问 `http://localhost:5173` 开始使用 CCR UI。

## 文档导航

- [快速开始](/guide/getting-started) - 了解如何安装和运行项目
- [项目结构](/guide/project-structure) - 详细的项目架构说明
- [前端文档](/frontend/overview) - Vue 3 前端开发指南
- [后端文档](/backend/architecture) - Rust 后端架构说明
- [贡献指南](/contributing) - 如何参与项目开发
- [FAQ](/faq) - 常见问题解答

## 功能预览

### 📊 Dashboard 首页

全新的 Dashboard 首页提供了：
- 系统状态实时监控（CPU、内存、系统信息）
- 9 个功能模块卡片，科技风格设计（新增统计分析）
- 动态渐变背景效果
- 一键快速访问各个功能

### 🔵 CLI 工具主页

每个 CLI 工具都有独立的主页：
- 清晰的功能分类展示
- 独特的配色方案
- 子功能快速导航
- 返回首页便捷按钮

### 🎯 功能页面

保留所有原有功能：
- 配置管理（列表、切换、验证）
- MCP 服务器管理
- Agents 配置
- 插件管理
- Slash Commands 管理
- 云同步功能
- **📊 统计分析**（新增）- 完整的成本追踪和使用统计仪表板

---

<div style="text-align: center; margin-top: 2rem; padding: 1rem; background: var(--vp-c-bg-soft); border-radius: 8px;">
  <p>🚀 <strong>开始探索 CCR UI 的强大功能吧！</strong></p>
  <p>现代化的配置管理平台，支持多个 AI CLI 工具，提供完整的配置管理和云端同步能力。</p>
  <p>如果你在使用过程中遇到任何问题，欢迎查看我们的 <a href="/faq">FAQ</a> 或提交 <a href="https://github.com/your-username/ccr/issues">Issue</a>。</p>
</div>
