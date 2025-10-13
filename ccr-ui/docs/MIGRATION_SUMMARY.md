# Next.js 16 Beta 迁移文档更新总结

本文档总结了前端从 React + Vite 迁移到 Next.js 16 Beta 后的文档更新内容。

## 📋 更新的文档列表

### 1. ✅ frontend/overview.md（前端项目概述）

**主要更新**：

- **技术栈更新**
  - React 18.3.1 → React 19.0.0
  - Vite 7.1.9 → Next.js 16.0.0-canary.3 + Turbopack（内置）
  - React Router 6.27.0 → Next.js App Router（基于文件系统）

- **新增 Next.js 16 特性说明**
  - Turbopack：默认打包器，2-5x 构建速度提升
  - 文件系统缓存：开发模式缓存优化
  - React 19 支持
  - Server Components：默认服务端组件
  - 图像优化：自动 AVIF/WebP 格式转换

- **项目结构更新**
  - `src/` → `src/app/`（App Router）
  - `src/pages/` → `src/app/[route]/page.tsx`
  - `src/api/` → `src/lib/api/`
  - `src/types/` → `src/lib/types/`
  - 新增 `src/components/providers/`

- **路由系统更新**
  - 从 React Router 客户端路由改为 Next.js App Router
  - 基于文件系统的路由结构
  - 支持布局嵌套

- **配置文件更新**
  - `vite.config.ts` → `next.config.mjs`
  - 新增 Turbopack 配置
  - 新增 API 代理配置（rewrites）
  - 新增图像优化配置

- **性能优化更新**
  - 从 Vite 优化改为 Turbopack 优化
  - 新增 Server Components 说明
  - 新增自动代码分割说明
  - 更新构建和部署流程

### 2. ✅ frontend/development.md（开发指南）

**主要更新**：

- **开发环境设置**
  - 开发服务器端口：5173 → 3000
  - 启动命令：`npm run dev`（现在使用 Turbopack）
  - 新增 `npm run type-check` 命令

- **编码规范更新**
  - 新增 Next.js 特定规范
  - Server Components vs Client Components 区分
  - `'use client'` 指令说明
  - 组件类型定义更新（移除 `React.FC`）

- **组件规范更新**
  - 新增页面组件结构（App Router）
  - 新增布局组件结构
  - 更新客户端组件结构
  - 新增 metadata 导出说明

- **API 客户端设计更新**
  - 基础配置从 Vite 环境变量改为 Next.js 代理
  - `import.meta.env` → `process.env`
  - 新增 API 路由代理配置说明

- **项目架构更新**
  - 组件分层从 `src/` 改为 `src/app/` + `src/components/`
  - 状态管理保持不变（useState + Context API）
  - API 客户端路径更新

### 3. ✅ frontend/api.md（API 接口文档）

**主要更新**：

- **API 基础配置**
  - 基础 URL：从直接配置后端地址改为使用 `/api` 代理
  - 新增 Next.js rewrites 配置说明
  - 文件路径：`src/api/` → `src/lib/api/`

- **环境变量配置**
  - `.env` → `.env.local` / `.env.production`
  - `VITE_API_BASE_URL` → `NEXT_PUBLIC_API_URL`
  - 新增 Next.js 环境变量规则说明
    - `NEXT_PUBLIC_*` 前缀暴露给浏览器
    - 无前缀仅在服务器端可用

- **代理配置**
  - 新增 next.config.mjs 中的 rewrites 配置
  - 开发环境代理到 localhost:8081

### 4. ✅ guide/project-structure.md（项目结构）

**主要更新**：

- **整体项目结构**
  - 前端描述：React 前端应用 → Next.js 16 Beta 前端应用
  - 配置文件：vite.config.ts → next.config.mjs
  - Tailwind 配置：tailwind.config.js → tailwind.config.ts

- **前端结构详解**
  - 新增 App Router 结构说明
  - 组件架构更新为新的目录结构
  - 库和工具路径更新
  - 新增路由与页面对照表

- **配置文件更新**
  - package.json：更新依赖和脚本
  - next.config.mjs：完整配置示例
  - tailwind.config.ts：TypeScript 配置

- **开发和生产环境**
  - 开发服务器：localhost:5173 → localhost:3000
  - 前端描述：静态文件服务器 → Next.js 服务器（Node.js）

- **构建产物**
  - `frontend/dist/` → `frontend/.next/`
  - 新增缓存、服务端代码、静态资源等说明

## 🎯 关键技术变更点

### 1. 框架迁移

| 项目 | 之前 | 现在 |
|------|-----|-----|
| 框架 | React + Vite | Next.js 16 Beta |
| 打包工具 | Vite | Turbopack（内置） |
| 路由 | React Router | App Router |
| React 版本 | 18.3.1 | 19.0.0 |

### 2. 目录结构变化

```diff
frontend/src/
- ├── pages/              # 页面组件
- ├── App.tsx             # 根组件  
- ├── main.tsx            # 应用入口
+ ├── app/                # App Router
+ │   ├── layout.tsx      # 根布局
+ │   ├── page.tsx        # 首页
+ │   └── [route]/        # 路由页面
├── components/           # 组件
- ├── api/                # API
+ ├── lib/                # 工具库
+     ├── api/            # API
+     └── types/          # 类型
```

### 3. 配置文件变化

```diff
frontend/
- ├── vite.config.ts      # Vite 配置
+ ├── next.config.mjs     # Next.js 配置
- ├── tailwind.config.js  # Tailwind 配置
+ ├── tailwind.config.ts  # Tailwind 配置（TS）
├── tsconfig.json         # TypeScript 配置（更新）
└── package.json          # 依赖更新
```

### 4. 开发体验提升

- **构建速度**：Turbopack 提供 2-5x 构建速度提升
- **热更新**：更快的 HMR（热模块替换）
- **开发端口**：3000（Next.js 标准端口）
- **类型安全**：更好的 TypeScript 集成

### 5. 新功能支持

- **Server Components**：默认服务端渲染，减少客户端 JavaScript
- **API 路由代理**：通过 rewrites 实现开发环境代理
- **图像优化**：自动格式转换和响应式图片
- **文件系统缓存**：开发模式下缓存编译结果

## 📖 文档使用建议

### 开发者学习路径

1. **快速开始**：阅读 `frontend/overview.md` 了解整体架构
2. **开发指南**：查看 `frontend/development.md` 学习编码规范
3. **API 接口**：参考 `frontend/api.md` 了解 API 调用方式
4. **项目结构**：通过 `guide/project-structure.md` 熟悉目录组织

### 迁移参考

从 React + Vite 迁移到 Next.js 的开发者应重点关注：

1. **路由系统**：从 React Router 改为 App Router
2. **组件类型**：理解 Server Components vs Client Components
3. **API 调用**：使用 Next.js 的 API 代理而非直接配置后端 URL
4. **环境变量**：使用 `NEXT_PUBLIC_*` 前缀暴露给浏览器

## ✨ 未来改进建议

1. **性能监控**：添加 Next.js 性能监控文档
2. **部署指南**：详细的 Vercel、Docker 部署步骤
3. **测试策略**：Next.js 特定的测试方法
4. **最佳实践**：Server Components 最佳实践

## 📝 文档维护

- 文档版本：基于 Next.js 16.0.0-canary.3
- 更新日期：2025-10-13
- 维护人：AI Assistant
- 下次审查：Next.js 16 正式版发布时

---

**注意**：Next.js 16 目前处于 Beta 阶段，部分 API 可能在正式版中有所调整。请关注官方文档更新。

