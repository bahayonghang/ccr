# 前端项目概述

CCR UI 的前端是一个基于 **Next.js 16 Beta** 构建的现代化 Web 应用，采用 App Router 架构，为用户提供直观、响应式的 CCR 配置管理界面。

## 🎯 项目目标

前端应用的主要目标是：

- **用户友好**：提供直观、易用的配置管理界面
- **实时交互**：支持实时命令执行和结果展示
- **响应式设计**：适配桌面端和移动端设备
- **极致性能**：利用 Next.js 16 和 Turbopack 实现 2-5x 构建速度提升
- **类型安全**：使用 TypeScript 确保代码质量
- **SEO 友好**：支持服务端渲染（SSR）和静态生成（SSG）

## 🏗️ 技术架构

### 核心技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Next.js | 16.0.0-canary.3 | React 全栈框架 |
| React | 19.0.0 | 用户界面库 |
| TypeScript | 5.6.3 | 类型安全的 JavaScript |
| Turbopack | 内置 | 下一代打包工具（默认） |
| Axios | 1.7.7 | HTTP 客户端 |
| Tailwind CSS | 3.4.14 | CSS 框架 |
| Lucide React | 0.454.0 | 图标库 |

### Next.js 16 新特性

- **Turbopack** - 默认打包器，2-5x 构建速度提升
- **文件系统缓存** - 开发模式缓存优化
- **React 19** - 支持最新 React 特性
- **App Router** - 基于文件系统的路由
- **Server Components** - 默认服务端组件
- **图像优化** - 自动 AVIF/WebP 格式转换

### 开发工具

- **ESLint** - 代码质量检查（Next.js 配置）
- **TypeScript** - 静态类型检查
- **PostCSS** - CSS 后处理器
- **Autoprefixer** - CSS 自动前缀

## 📁 项目结构

```
frontend/
├── public/                     # 静态资源
│   └── vite.svg               # 应用图标
├── src/                       # 源代码
│   ├── app/                   # Next.js App Router
│   │   ├── layout.tsx        # 根布局
│   │   ├── page.tsx          # 首页
│   │   ├── globals.css       # 全局样式
│   │   ├── configs/          # 配置管理页面
│   │   │   └── page.tsx
│   │   └── commands/         # 命令执行页面
│   │       └── page.tsx
│   ├── components/            # 可复用组件
│   │   ├── providers/        # Context Providers
│   │   │   └── ThemeProvider.tsx
│   │   ├── layout/           # 布局组件
│   │   │   ├── Navbar.tsx
│   │   │   └── CollapsibleSidebar.tsx
│   │   ├── sidebar/          # 侧边栏组件
│   │   │   ├── LeftSidebar.tsx
│   │   │   └── RightSidebar.tsx
│   │   ├── history/          # 历史记录组件
│   │   │   └── HistoryList.tsx
│   │   └── ui/               # 基础 UI 组件
│   │       └── ThemeToggle.tsx
│   └── lib/                   # 工具库
│       ├── api/              # API 客户端
│       │   └── client.ts     # HTTP 客户端配置
│       └── types/            # TypeScript 类型定义
│           └── index.ts      # 通用类型
├── package.json              # 项目配置
├── next.config.mjs           # Next.js 配置
├── tailwind.config.ts        # Tailwind 配置
└── tsconfig.json             # TypeScript 配置
```

## 🎨 设计系统

### 主题配置

应用支持深色和浅色两种主题模式，通过 CSS 变量实现主题切换：

```css
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f8fafc;
  --text-primary: #1e293b;
  --text-secondary: #64748b;
  --border-color: #e2e8f0;
  --accent-color: #3b82f6;
}

[data-theme="dark"] {
  --bg-primary: #0f172a;
  --bg-secondary: #1e293b;
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --border-color: #334155;
  --accent-color: #60a5fa;
}
```

### 响应式断点

使用 Tailwind CSS 的响应式断点系统：

- **sm**: 640px 及以上
- **md**: 768px 及以上  
- **lg**: 1024px 及以上
- **xl**: 1280px 及以上
- **2xl**: 1536px 及以上

## 🔄 状态管理

### 本地状态

使用 React 内置的 `useState` 和 `useEffect` 钩子管理组件本地状态：

```typescript
'use client' // Client Component

const [configs, setConfigs] = useState<Config[]>([]);
const [loading, setLoading] = useState(false);
const [error, setError] = useState<string | null>(null);
```

### 全局状态

使用 Context API 和自定义 Provider 管理主题等全局状态：

```typescript
// src/components/providers/ThemeProvider.tsx
'use client'

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState<'light' | 'dark'>('light')
  
  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light')
  }

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  )
}
```

## 🌐 路由配置

使用 Next.js App Router 基于文件系统的路由：

```
app/
├── layout.tsx          # 根布局（应用于所有页面）
├── page.tsx            # 首页 (/)
├── configs/
│   └── page.tsx        # 配置页面 (/configs)
└── commands/
    └── page.tsx        # 命令页面 (/commands)
```

### 路由结构

- `/` - 首页（重定向到 `/configs`）
- `/configs` - 配置管理页面
- `/commands` - 命令执行页面

### 布局嵌套

```typescript
// app/layout.tsx - 根布局
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN">
      <body>
        <ThemeProvider>
          <Navbar />
          {children}
        </ThemeProvider>
      </body>
    </html>
  )
}
```

## 📡 API 集成

### HTTP 客户端配置

使用 Axios 作为 HTTP 客户端，配置了请求和响应拦截器：

```typescript
const apiClient = axios.create({
  baseURL: 'http://127.0.0.1:8081/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器
apiClient.interceptors.request.use(
  (config) => {
    console.log('API Request:', config.method?.toUpperCase(), config.url);
    return config;
  },
  (error) => Promise.reject(error)
);

// 响应拦截器
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error.response?.data || error.message);
    return Promise.reject(error);
  }
);
```

### API 接口

主要的 API 接口包括：

- **配置管理**
  - `GET /configs` - 获取配置列表
  - `POST /configs/switch` - 切换配置
  - `POST /configs/validate` - 验证配置

- **命令执行**
  - `POST /commands/execute` - 执行命令
  - `GET /commands/list` - 获取可用命令列表

- **系统信息**
  - `GET /system/info` - 获取系统信息

## 🎯 核心功能

### 1. 配置管理

- 显示当前可用的 CCR 配置列表
- 支持配置切换操作
- 实时显示当前激活的配置
- 配置验证功能

### 2. 命令执行

- 可视化的命令输入界面
- 实时显示命令执行结果
- 支持命令历史记录
- 终端风格的输出显示

### 3. 用户界面

- 响应式导航栏
- 侧边栏菜单（移动端）
- 深色/浅色主题切换
- 加载状态和错误处理

## 🔧 开发工具配置

### Next.js 配置

```javascript
// next.config.mjs
/** @type {import('next').NextConfig} */
const nextConfig = {
  // Turbopack 是 Next.js 16 的默认打包器
  experimental: {
    // 启用文件系统缓存（Beta）
    turbopackFileSystemCacheForDev: true,
  },

  // Turbopack workspace root 配置
  turbopack: {
    root: process.cwd(),
  },

  // 生产构建优化
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production' ? {
      exclude: ['error', 'warn'],
    } : false,
  },

  // API 代理配置（开发环境）
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:8081/api/:path*',
      },
    ];
  },

  // 性能优化
  poweredByHeader: false,
  compress: true,

  // 图片优化
  images: {
    formats: ['image/avif', 'image/webp'],
  },
};

export default nextConfig;
```

### TypeScript 配置

```json
{
  "compilerOptions": {
    "target": "ES2017",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [
      {
        "name": "next"
      }
    ],
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

## 📈 性能优化

### 自动代码分割

Next.js 自动为每个路由进行代码分割：

```typescript
// app/configs/page.tsx
// 自动代码分割，只在访问时加载
export default function ConfigsPage() {
  return <ConfigManagement />
}

// app/commands/page.tsx  
// 独立的代码块
export default function CommandsPage() {
  return <CommandExecutor />
}
```

### Server Components

默认使用 Server Components 减少客户端 JavaScript：

```typescript
// 服务端组件（默认）
export default function ServerComponent() {
  const data = await fetchData() // 在服务器上执行
  return <div>{data}</div>
}

// 客户端组件（需要声明）
'use client'
export default function ClientComponent() {
  const [state, setState] = useState()
  return <div>{state}</div>
}
```

### Turbopack 性能

- **2-5x 更快的构建**：使用 Turbopack 替代 Webpack
- **增量编译**：只重新编译修改的文件
- **文件系统缓存**：开发模式下缓存编译结果
- **热更新优化**：更快的 HMR（热模块替换）

### 图像优化

- 自动 AVIF/WebP 格式转换
- 响应式图片生成
- 延迟加载
- 模糊占位符

## 🧪 测试策略

### 单元测试

使用 Vitest 进行单元测试：

```typescript
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import App from './App';

describe('App', () => {
  it('renders without crashing', () => {
    render(<App />);
    expect(screen.getByText('CCR UI')).toBeInTheDocument();
  });
});
```

### 集成测试

使用 Cypress 进行端到端测试：

```typescript
describe('Config Management', () => {
  it('should load and display configs', () => {
    cy.visit('/configs');
    cy.get('[data-testid="config-list"]').should('be.visible');
    cy.get('[data-testid="config-item"]').should('have.length.greaterThan', 0);
  });
});
```

## 🚀 构建和部署

### 开发环境

```bash
# 启动开发服务器（使用 Turbopack）
npm run dev

# 开发服务器运行在 http://localhost:3000
```

### 生产构建

```bash
# 构建生产版本
npm run build

# 启动生产服务器
npm run start
```

### 构建产物

```
.next/
├── cache/              # 构建缓存
├── server/             # 服务端代码
├── static/             # 静态资源
└── standalone/         # 独立部署包（可选）
```

### 部署选项

1. **Vercel**（推荐）- Next.js 官方平台
2. **Docker** - 容器化部署
3. **Node.js** - 传统服务器部署
4. **静态导出** - `next export`（受限功能）

## 📚 相关文档

- [技术栈详解](/frontend/tech-stack)
- [开发指南](/frontend/development)
- [组件文档](/frontend/components)
- [API 接口](/frontend/api)
- [样式指南](/frontend/styling)