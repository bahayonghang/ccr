# CCR UI - Frontend (Next.js 16 Beta)

> Claude Code Configuration Switcher - Modern Web Console with Next.js 16 Beta

基于 Next.js 16 Beta 构建的现代化 Web 控制台，用于管理 Claude Code 配置。

## ✨ 特性

### 🚀 性能优化
- **Turbopack**: 默认打包器，提供 2-5x 构建速度提升
- **React Compiler**: 自动组件记忆化，减少不必要的重新渲染
- **文件系统缓存**: 显著提升开发环境启动速度
- **图片优化**: 自动转换为 AVIF/WebP 格式
- **代码分割**: 按需加载，优化首屏加载时间

### 🎨 UI/UX 优化
- **响应式设计**: 完美适配桌面、平板和移动设备
- **暗色/明亮主题**: 支持主题切换，自动保存偏好
- **动态背景**: 优雅的渐变动画效果
- **玻璃态效果**: 现代化的模糊背景设计
- **流畅动画**: 所有交互均有平滑过渡效果
- **可访问性**: 遵循 WCAG 2.1 AA 标准

### 📦 技术栈
- **Next.js 16 Beta** - React 框架（App Router）
- **React 19** - UI 库
- **TypeScript** - 类型安全
- **Tailwind CSS** - 实用优先的 CSS 框架
- **Lucide React** - 现代图标库
- **Axios** - HTTP 客户端
- **React Syntax Highlighter** - 代码高亮

## 📁 项目结构

```
frontend-next/
├── src/
│   ├── app/                      # Next.js App Router
│   │   ├── layout.tsx            # 根布局
│   │   ├── page.tsx              # 首页（重定向）
│   │   ├── configs/              # 配置管理页面
│   │   │   └── page.tsx
│   │   ├── commands/             # 命令执行页面
│   │   │   └── page.tsx
│   │   └── globals.css           # 全局样式
│   │
│   ├── components/               # React 组件
│   │   ├── layout/               # 布局组件
│   │   │   ├── Navbar.tsx
│   │   │   └── CollapsibleSidebar.tsx
│   │   ├── sidebar/              # 侧边栏组件
│   │   │   ├── LeftSidebar.tsx
│   │   │   └── RightSidebar.tsx
│   │   ├── history/              # 历史记录组件
│   │   │   └── HistoryList.tsx
│   │   ├── ui/                   # UI 组件
│   │   │   └── ThemeToggle.tsx
│   │   └── providers/            # Provider 组件
│   │       └── ThemeProvider.tsx
│   │
│   └── lib/                      # 工具库
│       ├── api/                  # API 客户端
│       │   └── client.ts
│       └── types/                # TypeScript 类型定义
│           └── index.ts
│
├── public/                       # 静态资源
│   └── vite.svg
│
├── next.config.mjs               # Next.js 配置
├── tailwind.config.ts            # Tailwind CSS 配置
├── tsconfig.json                 # TypeScript 配置
├── postcss.config.mjs            # PostCSS 配置
├── .eslintrc.json                # ESLint 配置
├── .gitignore                    # Git 忽略文件
├── package.json                  # 项目依赖
└── README.md                     # 本文件
```

## 🚀 快速开始

### 系统要求

- **Node.js**: >= 20.9.0
- **npm/yarn/pnpm**: 任意包管理器

### 安装

```bash
# 1. 进入项目目录
cd ccr-ui/frontend-next

# 2. 安装依赖
npm install
# 或
yarn install
# 或
pnpm install
```

### 开发

```bash
# 启动开发服务器（默认端口 3000，启用 Turbopack）
npm run dev

# 浏览器访问
open http://localhost:3000
```

### 构建

```bash
# 生产构建
npm run build

# 启动生产服务器
npm run start
```

### 代码质量

```bash
# ESLint 检查
npm run lint

# TypeScript 类型检查
npm run type-check
```

## ⚙️ 配置

### 环境变量

在项目根目录创建 `.env.local` 文件：

```env
# API 基础 URL（开发环境）
NEXT_PUBLIC_API_URL=http://localhost:8081

# 其他环境变量...
```

### API 代理

开发环境下，API 请求会自动代理到后端服务器（配置在 `next.config.mjs`）：

```javascript
async rewrites() {
  return [
    {
      source: '/api/:path*',
      destination: 'http://localhost:8081/api/:path*',
    },
  ];
}
```

### 性能优化配置

`next.config.mjs` 中已启用以下优化：

- ✅ **Turbopack**: 默认启用
- ✅ **React Compiler**: 自动记忆化
- ✅ **文件系统缓存**: 开发环境缓存
- ✅ **SWC 压缩**: 生产构建优化
- ✅ **图片优化**: AVIF/WebP 支持

## 🎨 样式系统

### CSS 变量

项目使用 CSS 变量实现主题系统，所有颜色和间距都可以通过变量控制：

```css
/* 明亮主题 */
:root[data-theme="light"] {
  --bg-primary: #f5f7fa;
  --text-primary: #1d1d1f;
  --accent-primary: #007aff;
  /* ... */
}

/* 暗色主题 */
:root[data-theme="dark"] {
  --bg-primary: #0a0e27;
  --text-primary: #e5e7eb;
  --accent-primary: #8b5cf6;
  /* ... */
}
```

### Tailwind CSS

使用 Tailwind 的工具类结合 CSS 变量：

```tsx
<div 
  className="rounded-xl p-4 glass-effect"
  style={{ 
    background: 'var(--bg-card)',
    border: '1px solid var(--border-color)' 
  }}
>
  {/* 内容 */}
</div>
```

## 📱 响应式设计

项目采用移动优先的响应式设计策略：

```tsx
// 响应式网格布局
<div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
  {/* 内容 */}
</div>

// 条件渲染
<span className="hidden sm:inline">桌面端显示</span>
```

## ♿ 可访问性

项目遵循 WCAG 2.1 AA 标准，包括：

- ✅ 语义化 HTML 标签
- ✅ ARIA 属性支持
- ✅ 键盘导航支持
- ✅ 屏幕阅读器优化
- ✅ 聚焦指示器
- ✅ 颜色对比度符合标准

示例：

```tsx
<button
  onClick={handleClick}
  aria-label="刷新数据"
  aria-pressed={isActive}
>
  <RefreshIcon aria-hidden="true" />
  <span className="sr-only">刷新</span>
</button>
```

## 🔧 开发指南

### 组件开发

1. **使用 'use client' 指令**（如果组件需要客户端交互）：
   ```tsx
   'use client';
   
   import { useState } from 'react';
   
   export default function MyComponent() {
     // ...
   }
   ```

2. **类型安全**：始终为 props 定义类型
   ```tsx
   interface MyComponentProps {
     title: string;
     onSubmit: (data: FormData) => void;
   }
   
   export default function MyComponent({ title, onSubmit }: MyComponentProps) {
     // ...
   }
   ```

3. **样式一致性**：使用 CSS 变量
   ```tsx
   <div style={{ color: 'var(--text-primary)' }}>
     {/* 内容 */}
   </div>
   ```

### API 调用

使用 `lib/api/client.ts` 中的封装方法：

```tsx
import { listConfigs, switchConfig } from '@/lib/api/client';

// 在组件中
const handleSwitch = async (name: string) => {
  try {
    await switchConfig(name);
    const configs = await listConfigs();
    // 处理数据
  } catch (error) {
    console.error('Failed:', error);
  }
};
```

## 📊 性能监控

### Core Web Vitals

Next.js 内置了性能监控，可以查看以下指标：

- **LCP** (Largest Contentful Paint): 最大内容绘制
- **FID** (First Input Delay): 首次输入延迟
- **CLS** (Cumulative Layout Shift): 累积布局偏移

在开发环境下，这些指标会在控制台输出。

## 🐛 故障排查

### 常见问题

1. **端口占用**
   ```bash
   # 修改端口
   npm run dev -- -p 3001
   ```

2. **清除缓存**
   ```bash
   rm -rf .next node_modules
   npm install
   npm run dev
   ```

3. **类型错误**
   ```bash
   npm run type-check
   ```

## 📝 待办事项

- [ ] 添加单元测试（Jest + React Testing Library）
- [ ] 添加 E2E 测试（Playwright）
- [ ] 实现配置导入/导出功能
- [ ] 添加国际化支持（i18n）
- [ ] 优化 SEO 元数据
- [ ] 添加 PWA 支持

## 🔄 迁移说明

从 React + Vite 迁移到 Next.js 16 Beta 的主要变化：

### 路由系统
- ✅ 从 React Router 迁移到 Next.js App Router
- ✅ 使用文件系统路由（`app/` 目录）

### 组件
- ✅ 所有组件保持原有样式和功能
- ✅ 添加 'use client' 指令支持客户端交互

### 样式
- ✅ CSS 变量系统完全保留
- ✅ Tailwind 配置保持一致
- ✅ 全局样式优化和增强

### 性能
- ✅ 启用 Turbopack 提升构建速度
- ✅ React Compiler 自动优化
- ✅ 图片和字体自动优化

## 📄 License

MIT License - 详见 [LICENSE](../../LICENSE) 文件

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**Built with ❤️ using Next.js 16 Beta**

