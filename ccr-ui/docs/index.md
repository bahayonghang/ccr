---
layout: home

hero:
  name: "CCR UI"
  text: "现代化配置管理平台"
  tagline: "基于 React + Rust 构建的全栈 CCR 配置管理 Web 应用"
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
    details: 前端采用 React 18 + TypeScript + Vite，后端使用 Rust + Actix Web，提供极致的开发体验和运行性能
  - icon: 🎯
    title: 直观的用户界面
    details: 精心设计的 UI 界面，支持深色模式，提供配置管理、命令执行等核心功能的可视化操作
  - icon: 🔧
    title: 强大的配置管理
    details: 支持 CCR 配置的查看、切换、验证等操作，实时显示命令执行结果，提升配置管理效率
  - icon: 🚀
    title: 高性能架构
    details: 前后端分离架构，支持并发处理，内置缓存机制，确保应用的高性能和稳定性
  - icon: 📱
    title: 响应式设计
    details: 完全响应式设计，支持桌面端和移动端访问，随时随地管理你的 CCR 配置
  - icon: 🛠️
    title: 开发友好
    details: 完整的开发工具链，支持热重载、TypeScript、ESLint 等，提供最佳的开发体验
---

## 项目特色

CCR UI 是一个现代化的全栈 Web 应用程序，专为 CCR (Claude Code Configuration Switcher) 配置管理而设计。它结合了前端的用户友好界面和后端的高性能处理能力，为开发者提供了一个强大而直观的配置管理平台。

### 🎨 前端技术栈

- **React 18** - 现代化的用户界面框架
- **TypeScript** - 类型安全的 JavaScript 超集
- **Vite** - 极速的前端构建工具
- **Tailwind CSS** - 实用优先的 CSS 框架
- **React Router** - 声明式路由管理
- **Axios** - 强大的 HTTP 客户端

### ⚙️ 后端技术栈

- **Rust** - 系统级编程语言，安全且高性能
- **Actix Web** - 高性能的异步 Web 框架
- **Tokio** - 异步运行时
- **Serde** - 序列化和反序列化框架
- **Clap** - 命令行参数解析

### 📋 核心功能

- **配置管理** - 查看、切换、验证 CCR 配置
- **命令执行** - 可视化执行 CCR 命令并查看结果
- **实时输出** - 终端风格的命令输出显示
- **多页面导航** - 不同功能模块间的便捷切换
- **响应式设计** - 适配各种屏幕尺寸

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
- [前端文档](/frontend/overview) - React 前端开发指南
- [后端文档](/backend/architecture) - Rust 后端架构说明
- [贡献指南](/contributing) - 如何参与项目开发

---

<div style="text-align: center; margin-top: 2rem; padding: 1rem; background: var(--vp-c-bg-soft); border-radius: 8px;">
  <p>🚀 <strong>开始探索 CCR UI 的强大功能吧！</strong></p>
  <p>如果你在使用过程中遇到任何问题，欢迎查看我们的 <a href="/faq">FAQ</a> 或提交 <a href="https://github.com/your-username/ccr/issues">Issue</a>。</p>
</div>