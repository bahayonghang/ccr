---
layout: home

hero:
  name: "CCR Desktop"
  text: "Claude Code Configuration Switcher"
  tagline: 用 Tauri 打造的优雅配置管理工具 ⚡
  image:
    src: /logo.svg
    alt: CCR Desktop
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/harleyqing/ccr

features:
  - icon: 🎨
    title: 现代化界面
    details: Vue 3 + TypeScript + Vite 构建，提供优雅流畅的用户体验，支持深色/浅色主题切换

  - icon: 🚀
    title: 高性能
    details: Tauri 2.0 + Rust 后端，原生性能表现，启动快速，资源占用低

  - icon: 📦
    title: 小体积
    details: 相比 Electron 体积更小，安装包仅 10MB 左右，不打包 Chromium

  - icon: 🔒
    title: 安全可靠
    details: Tauri 安全模型，细粒度文件系统权限控制，保护用户数据安全

  - icon: 🔄
    title: 完整功能
    details: 复用 CCR 核心库，支持配置切换、导入导出、备份恢复等全部功能

  - icon: 🌍
    title: 跨平台
    details: 支持 macOS、Linux、Windows，一套代码多平台运行。WSL 环境特别优化

  - icon: 🌐
    title: Web 调试模式
    details: 支持纯 Web 模式运行，无需桌面窗口，适合远程开发和 WSL 调试

  - icon: 🎯
    title: TypeScript
    details: 完整的类型定义，开发体验极佳，减少运行时错误

  - icon: 🛠️
    title: 开发友好
    details: 热重载支持，Vue DevTools 集成，完善的开发工具链

  - icon: 📚
    title: 文档完善
    details: 详细的 API 文档、开发指南、最佳实践，助你快速上手
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: linear-gradient(135deg, #8b5cf6 0%, #a855f7 100%);

  --vp-home-hero-image-background-image: linear-gradient(-45deg, #8b5cf6 50%, #a855f7 50%);
  --vp-home-hero-image-filter: blur(44px);
}

@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(56px);
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px);
  }
}
</style>

## 核心特性

CCR Desktop 是 CCR (Claude Code Configuration Switcher) 的桌面版本，使用 Tauri 2.0 构建，提供了现代化的图形界面来管理 Claude Code 的配置文件。

### 🎯 主要功能

- **配置管理** - 查看、创建、编辑、删除配置
- **一键切换** - 快速切换不同的 Claude Code 配置
- **配置分类** - 官方中转、第三方模型、未分类三种类型
- **导入导出** - 支持配置文件的导入和导出
- **备份恢复** - 自动备份，支持恢复历史版本
- **历史记录** - 完整的操作历史追踪
- **系统信息** - 实时显示系统资源使用情况
- **双模式运行** - 支持桌面窗口模式和纯 Web 调试模式
- **WSL 优化** - 针对 WSL 环境的滚轮和图形优化

### 🌟 为什么选择 Tauri？

相比传统的 Electron 应用：

| 特性 | Tauri | Electron |
|-----|-------|----------|
| 安装包大小 | ~10MB | ~100MB+ |
| 内存占用 | ~50MB | ~200MB+ |
| 启动速度 | 极快 | 较慢 |
| 安全性 | 更高 | 中等 |
| 系统集成 | 原生 | 模拟 |

### 🚀 快速体验

::: code-group

```bash [桌面模式]
# 克隆仓库
git clone https://github.com/harleyqing/ccr.git
cd ccr/ccr-tauri

# 一键安装依赖
just setup

# 运行桌面应用 (推荐 macOS/Linux)
just dev

# WSL 环境优化启动
just dev-wsl
```

```bash [Web 调试模式]
# 克隆仓库
git clone https://github.com/harleyqing/ccr.git
cd ccr/ccr-tauri

# 一键安装依赖
just setup

# Web 模式运行 (无桌面窗口)
just dev-web

# 访问 http://localhost:5173
```

:::

## 技术栈

### 后端 (Rust)
- **Tauri 2.0** - 桌面应用框架
- **CCR Core** - 配置管理核心库
- **Serde** - 序列化/反序列化
- **Tokio** - 异步运行时

### 前端 (Vue 3)
- **Vue 3** - 渐进式 JavaScript 框架
- **TypeScript** - 类型安全
- **Vite** - 快速构建工具
- **Pinia** - 状态管理 (可选)

## 项目状态

<div style="display: flex; gap: 10px; margin-top: 20px;">
  <img src="https://img.shields.io/badge/version-1.1.2-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/tauri-2.0-purple.svg" alt="Tauri">
  <img src="https://img.shields.io/badge/vue-3.4+-green.svg" alt="Vue">
</div>

## 社区

- 💬 [讨论区](https://github.com/harleyqing/ccr/discussions)
- 🐛 [问题反馈](https://github.com/harleyqing/ccr/issues)
- 📖 [变更日志](https://github.com/harleyqing/ccr/releases)

## 许可证

MIT License © 2024 CCR Team

---

::: tip 提示
这是本小姐精心打造的桌面应用呢！(￣▽￣)／
有问题的话... 才、才不是说本小姐会帮你解决啦，只是不想看到你搞砸而已！(,,><,,)
:::
