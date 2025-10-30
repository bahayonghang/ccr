# 项目结构

本文档详细介绍 CCR UI 项目的整体结构和各个目录的作用。

## 📁 整体项目结构

```
ccr-ui/
├── backend/                    # Rust 后端服务
│   ├── src/                   # 源代码
│   │   ├── main.rs           # 应用入口点
│   │   ├── config_reader.rs  # CCR 配置文件读取
│   │   ├── models.rs         # 数据模型定义
│   │   ├── claude_config_manager.rs  # Claude 配置管理
│   │   ├── markdown_manager.rs       # Markdown 文件管理
│   │   ├── plugins_manager.rs        # 插件管理
│   │   ├── settings_manager.rs       # 设置管理
│   │   ├── handlers/         # HTTP 请求处理器
│   │   │   ├── mod.rs
│   │   │   ├── config.rs     # 配置相关接口
│   │   │   ├── command.rs    # 命令执行接口
│   │   │   ├── system.rs     # 系统信息接口
│   │   │   ├── version.rs    # 版本管理接口
│   │   │   ├── mcp.rs        # MCP 服务器管理
│   │   │   ├── agents.rs     # Agent 管理
│   │   │   ├── plugins.rs    # 插件管理
│   │   │   └── slash_commands.rs # 斜杠命令管理
│   │   └── executor/         # 命令执行器
│   │       ├── mod.rs
│   │       └── cli_executor.rs # CLI 命令执行
│   ├── Cargo.toml            # Rust 项目配置
│   ├── examples/             # 示例配置文件
│   │   └── settings.example.json
│   └── README.md             # 后端说明文档
├── frontend/                  # Vue 3 + Vite 前端应用
│   ├── public/               # 静态资源
│   │   └── vite.svg         # 应用图标
│   ├── src/                 # 源代码
│   │   ├── main.ts          # 应用入口
│   │   ├── App.vue          # 根组件
│   │   ├── views/           # 页面组件
│   │   │   ├── HomeView.vue        # Dashboard 首页
│   │   │   ├── ConfigsView.vue     # 配置管理
│   │   │   ├── CommandsView.vue    # 命令执行
│   │   │   ├── McpView.vue         # MCP 服务器管理
│   │   │   ├── AgentsView.vue      # Agent 管理
│   │   │   ├── PluginsView.vue     # 插件管理
│   │   │   ├── SlashCommandsView.vue # 斜杠命令管理
│   │   │   ├── SyncView.vue        # 云同步
│   │   │   ├── StatsView.vue       # 统计分析
│   │   │   └── ConverterView.vue   # 配置转换器
│   │   ├── components/      # 可复用组件
│   │   │   ├── MainLayout.vue      # 主布局
│   │   │   ├── Navbar.vue          # 导航栏
│   │   │   ├── CollapsibleSidebar.vue # 侧边栏
│   │   │   ├── RightSidebar.vue    # 右侧栏
│   │   │   ├── StatusHeader.vue    # 状态头部
│   │   │   ├── HistoryList.vue     # 历史记录
│   │   │   ├── VersionManager.vue  # 版本管理器
│   │   │   ├── ThemeToggle.vue     # 主题切换
│   │   │   ├── UpdateModal.vue     # 更新对话框
│   │   │   └── ConfigCard.vue      # 配置卡片
│   │   ├── router/          # Vue Router 配置
│   │   │   └── index.ts
│   │   ├── stores/          # Pinia 状态管理
│   │   │   ├── config.ts
│   │   │   ├── theme.ts
│   │   │   └── system.ts
│   │   ├── api/             # API 客户端
│   │   │   └── client.ts
│   │   ├── types/           # TypeScript 类型定义
│   │   │   └── index.ts
│   │   ├── styles/          # 全局样式
│   │   │   └── main.css
│   │   └── utils/           # 工具函数
│   │       └── helpers.ts
│   ├── package.json        # Node.js 项目配置
│   ├── vite.config.ts      # Vite 配置
│   ├── tailwind.config.js  # Tailwind CSS 配置
│   ├── postcss.config.js   # PostCSS 配置
│   ├── tsconfig.json       # TypeScript 配置
│   ├── .eslintrc.cjs       # ESLint 配置
│   └── README.md           # 前端说明文档
├── docs/                   # 项目文档
│   ├── .vitepress/         # VitePress 配置
│   │   └── config.ts
│   ├── backend/            # 后端文档
│   │   ├── api.md         # API 接口文档
│   │   └── architecture.md # 架构设计文档
│   ├── frontend/           # 前端文档
│   │   ├── api.md         # API 调用文档
│   │   ├── development.md # 开发指南
│   │   └── overview.md    # 前端概览
│   ├── guide/              # 用户指南
│   │   ├── getting-started.md # 快速开始
│   │   └── project-structure.md # 项目结构
│   ├── index.md            # 文档首页
│   ├── contributing.md     # 贡献指南
│   ├── faq.md             # 常见问题
│   ├── package.json       # 文档构建配置
│   └── public/            # 文档静态资源
│       ├── favicon.ico
│       └── logo.svg
├── clean-logs.sh           # 日志清理脚本
├── justfile               # Just 任务配置
├── .gitignore             # Git 忽略文件
├── ARCHITECTURE.md        # 架构说明
└── README.md              # 项目说明
├── docs/                    # 项目文档 (VitePress)
│   ├── .vitepress/         # VitePress 配置
│   │   └── config.ts       # 文档站点配置
│   ├── guide/              # 使用指南
│   ├── frontend/           # 前端文档
│   ├── backend/            # 后端文档
│   ├── index.md            # 文档首页
│   └── package.json        # 文档项目配置
├── justfile                # Just 命令配置
├── README.md               # 项目主说明文档
└── ARCHITECTURE.md         # 架构设计文档
```

## 🔧 后端结构详解

### 核心文件

| 文件 | 作用 | 说明 |
|------|------|------|
| `main.rs` | 应用入口 | 启动 HTTP 服务器，配置路由和中间件 |
| `models.rs` | 数据模型 | 定义请求/响应的数据结构 |
| `config_reader.rs` | 配置读取 | 读取和解析配置文件 |

### 处理器模块 (handlers/)

```
handlers/
├── mod.rs          # 模块导出
├── config.rs       # 配置管理接口
│   ├── GET /api/configs           # 获取配置列表
│   ├── POST /api/configs/switch   # 切换配置
│   └── POST /api/configs/validate # 验证配置
├── command.rs      # 命令执行接口
│   ├── POST /api/commands/execute # 执行命令
│   └── GET /api/commands/list     # 获取命令列表
└── system.rs       # 系统信息接口
    └── GET /api/system/info       # 获取系统信息
```

### 执行器模块 (executor/)

```
executor/
├── mod.rs          # 模块导出
└── cli_executor.rs # CLI 命令执行器
    ├── execute_ccr_command()      # 执行 CCR 命令
    ├── execute_arbitrary_command() # 执行任意命令
    └── 超时处理、错误处理等功能
```

## ⚛️ 前端结构详解 (Vue 3 + Vite)

### Vue 应用结构

```
src/
├── main.ts                # 应用入口
├── App.vue                # 根组件
├── views/                 # 页面组件
├── configs/               # 配置管理路由
│   └── page.tsx          # 配置页面 (/configs)
└── commands/              # 命令执行路由
    └── page.tsx          # 命令页面 (/commands)
```

### 组件架构

```
src/components/
├── providers/             # Context Providers
│   └── ThemeProvider.tsx # 主题 Provider
├── layout/               # 布局组件
│   ├── Navbar.tsx       # 顶部导航栏
│   └── CollapsibleSidebar.tsx # 可折叠侧边栏
├── sidebar/              # 侧边栏组件
│   ├── LeftSidebar.tsx  # 左侧边栏
│   └── RightSidebar.tsx # 右侧边栏
├── history/              # 历史记录组件
│   └── HistoryList.tsx  # 历史列表
└── ui/                   # 基础 UI 组件
    └── ThemeToggle.tsx  # 主题切换按钮
```

### 库和工具

```
src/lib/
├── api/                  # API 客户端
│   └── client.ts        # HTTP 客户端配置
│       ├── Axios 实例配置
│       ├── 请求/响应拦截器
│       ├── 错误处理
│       └── API 路由代理
└── types/                # TypeScript 类型定义
    └── index.ts         # 通用类型定义
```

### 路由与页面

Vue Router 配置路由：

| 路由路径 | 组件 | 描述 |
|---------|---------|------|
| `/` | `HomeView.vue` | Dashboard 首页 |
| `/configs` | `ConfigsView.vue` | 配置管理页面 |
| `/commands` | `CommandsView.vue` | 命令执行页面 |
| `/mcp` | `McpView.vue` | MCP 服务器管理 |
| `/agents` | `AgentsView.vue` | Agents 管理 |
| `/plugins` | `PluginsView.vue` | 插件管理 |
| `/sync` | `SyncView.vue` | 云同步 |
| `/stats` | `StatsView.vue` | 统计分析 |

## 📚 文档结构

### VitePress 配置

```
docs/.vitepress/
├── config.ts          # 站点配置
│   ├── 导航栏配置
│   ├── 侧边栏配置
│   ├── 主题配置
│   └── 搜索配置
└── theme/             # 自定义主题 (可选)
    ├── index.ts       # 主题入口
    └── components/    # 自定义组件
```

### 文档内容

```
docs/
├── guide/             # 使用指南
│   ├── getting-started.md    # 快速开始
│   ├── project-structure.md # 项目结构
│   ├── development-setup.md # 开发环境
│   └── build-deploy.md      # 构建部署
├── frontend/          # 前端文档
│   ├── overview.md           # 项目概述
│   ├── tech-stack.md        # 技术栈
│   ├── development.md       # 开发指南
│   ├── components.md        # 组件文档
│   ├── api.md              # API 接口
│   ├── styling.md          # 样式指南
│   └── testing.md          # 测试指南
├── backend/           # 后端文档
│   ├── architecture.md      # 架构设计
│   ├── tech-stack.md       # 技术栈
│   ├── development.md      # 开发指南
│   ├── api.md             # API 文档
│   ├── models.md          # 数据模型
│   ├── error-handling.md  # 错误处理
│   └── deployment.md      # 部署指南
├── contributing.md    # 贡献指南
├── changelog.md       # 更新日志
├── faq.md            # 常见问题
└── index.md          # 文档首页
```

## 🛠️ 配置文件说明

### 后端配置

#### Cargo.toml
```toml
[package]
name = "ccr-ui-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.9"      # Web 框架
tokio = "1.42"         # 异步运行时
serde = "1.0"          # 序列化
anyhow = "1.0"         # 错误处理
# ... 其他依赖
```

### 前端配置

#### package.json
```json
{
  "name": "ccr-ui-frontend-next",
  "version": "0.1.0",
  "dependencies": {
    "next": "^16.0.0-canary.3",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "typescript": "^5.6.3"
  },
  "scripts": {
    "dev": "next dev --turbopack",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  }
}
```

#### next.config.mjs
```javascript
/** @type {import('next').NextConfig} */
export default {
  experimental: {
    turbopackFileSystemCacheForDev: true,
  },
  turbopack: {
    root: process.cwd(),
  },
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:8081/api/:path*',
      },
    ]
  },
  images: {
    formats: ['image/avif', 'image/webp'],
  },
}
```

#### tailwind.config.ts
```typescript
import type { Config } from 'tailwindcss'

const config: Config = {
  content: ['./src/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    extend: {}
  },
  plugins: []
}

export default config
```

## 🔄 数据流向

### 请求流程

```
用户操作 → 前端组件 → API 客户端 → 后端处理器 → CLI 执行器 → CCR 命令
                                                                    ↓
用户界面 ← 前端组件 ← API 响应 ← 后端响应 ← 命令结果 ← 命令输出
```

### 文件关系

```
前端页面组件 → 使用 → UI 组件
     ↓
调用 API 服务 → 通过 → HTTP 客户端
     ↓
请求后端接口 → 处理器 → 执行器 → CCR CLI
```

## 📦 构建产物

### 前端构建

```
frontend/dist/
├── assets/          # 构建后的资源
│   ├── *.js        # JavaScript 文件
│   ├── *.css       # CSS 文件
│   └── *.svg       # SVG 图标
├── static/           # 静态资源
│   ├── chunks/      # 客户端 JS 分块
│   ├── css/         # 样式文件
│   └── media/       # 图片等媒体资源
└── standalone/       # 独立部署包（可选）
```

### 后端构建

```
backend/target/release/
└── ccr-ui-backend    # 可执行文件
```

## 🚀 部署结构

### 开发环境

```
开发环境:
├── 前端开发服务器 (localhost:5173) - Vite + Vue 3
├── 后端开发服务器 (localhost:8081) - Axum (Rust)
└── 文档开发服务器 (localhost:5174) - VitePress
```

### 生产环境

```
生产环境:
├── 静态文件服务器 (Nginx/Caddy) - 前端 SPA
├── 后端 API 服务器 (Rust 二进制)
└── 文档站点 (静态部署)
```

## 📋 开发工作流

### 1. 新功能开发

```
1. 在 backend/src/models.rs 定义数据模型
2. 在 backend/src/handlers/ 添加 API 处理器
3. 在 frontend/src/types/ 定义前端类型
4. 在 frontend/src/api/ 添加 API 客户端
5. 在 frontend/src/components/ 开发 UI 组件
6. 在 frontend/src/pages/ 集成页面功能
7. 在 docs/ 更新相关文档
```

### 2. 测试流程

```
1. 后端单元测试: cargo test
2. 前端单元测试: npm test
3. 集成测试: 启动完整应用测试
4. 文档测试: 验证文档构建和链接
```

### 3. 部署流程

```
1. 后端构建: cargo build --release
2. 前端构建: npm run build
3. 文档构建: npm run docs:build
4. 部署到目标环境
```

这个项目结构设计遵循了前后端分离的最佳实践，每个模块职责清晰，便于开发、测试和维护。