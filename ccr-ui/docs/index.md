---
layout: home

hero:
  name: "CCR UI"
  text: "全栈配置控制台"
  tagline: "Vue 3 + Axum + Tauri · 为 CCR 提供可视化与桌面体验"
  image:
    src: /logo.svg
    alt: CCR UI Logo
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 查看源码
      link: https://github.com/bahayonghang/ccr

features:
  - icon: 🚀
    title: 现代全栈
    details: 前端 Vue 3.5 + Vite + TypeScript + Tailwind，后端 Rust 2024 + Axum（workspace 成员），一致依赖。
  - icon: 🖥️
    title: 多种界面
    details: Web 模式与 Tauri 桌面模式自动切换，同一前端体验。
  - icon: ⚙️
    title: 全量配置能力
    details: 可视化查看/切换/验证/历史/备份，覆盖全部 CCR CLI 命令。
  - icon: ☁️
    title: 多目录同步
    details: WebDAV 目录注册、启用/禁用、单目录/批量 push·pull·status，自动过滤备份与锁。
  - icon: 🔌
    title: 平台与系统
    details: 支持 Claude、Codex、Gemini CLI、Qwen、IFLOW 等平台概览，系统健康检查与日志辅助。
  - icon: 🧰
    title: 开发者友好
    details: 内置 just 任务、组件与 API 客户端、VitePress 文档，便于二开和集成。
---

## 项目简介
CCR UI 为 CCR 提供图形化与桌面化控制台：配置管理、命令执行、多目录同步、平台信息与系统监控一站式收拢。默认工作在 `~/.ccr/ccr-ui/` 或源码路径，Tauri 桌面模式自动切换调用方式（invoke/HTTP）。

### 前端技术栈（v3.6.2）
- Vue 3.5 + Vite 7 + TypeScript 5.7
- Vue Router 4.4，Pinia 2.2
- Tailwind CSS 3.4，Lucide 图标，Axios

### 后端技术栈（v3.6.2）
- Rust 2024 Edition，Axum 0.8（workspace 成员）
- Tokio / Serde / Tower，统一依赖版本
- 通过子进程调用 CCR，可选托管前端静态文件

### 主要能力
- Dashboard：状态卡片、快捷入口、系统信息
- 配置/历史/验证/备份：与 CLI 对齐的全量操作
- 命令执行：可视化运行全部 CCR 命令并流式显示输出
- 同步：WebDAV 多目录注册、启用/禁用、单目录或全量 push/pull/status
- 平台与系统：平台列表/当前/切换、健康检查、日志级别辅助
- 主题与响应式：深浅色切换，桌面与移动端适配

## 快速开始

```bash
# 推荐：直接用 CLI 自动拉起或下载 UI
ccr ui

# 仓库开发
git clone https://github.com/bahayonghang/ccr.git
cd ccr/ccr-ui
just s                 # 前后端一键开发
# 或手动：cargo run --manifest-path backend/Cargo.toml -- --port 8081
#        (另一个终端) cd frontend && npm install && npm run dev
```

## 文档导航
- [快速开始](/guide/getting-started)
- [项目结构](/guide/project-structure)
- [Tauri 桌面](/guide/tauri)
- [前端参考](/reference/frontend/overview)
- [后端参考](/reference/backend/architecture)
- [贡献指南](/contributing)
- [FAQ](/faq)
