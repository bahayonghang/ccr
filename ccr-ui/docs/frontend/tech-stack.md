# 前端技术栈详解

CCR UI 前端采用现代化的技术栈，基于 Next.js 16 Beta 构建，提供高性能、类型安全的用户界面。

## 🎯 核心框架

### Next.js 16 Beta

**版本**: 16.0.0-canary.3

**选择理由**:
- **Turbopack**: 默认打包器，提供 2-5x 构建速度提升
- **App Router**: 基于文件系统的现代路由
- **Server Components**: 默认服务端组件，减少客户端 JavaScript
- **React 19 支持**: 支持最新 React 特性

**核心特性**:
```typescript
// app/layout.tsx - App Router 布局
export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="zh-CN">
      <body className={inter.className}>
        <Providers>
          {children}
        </Providers>
      </body>
    </html>
  )
}
```

### React 19

**版本**: 19.0.0

**新特性**:
- **Actions**: 简化表单处理
- **use() Hook**: 异步数据获取
- **Optimistic Updates**: 乐观更新
- **Server Components**: 服务端渲染组件

**使用示例**:
```typescript
// 使用 React 19 Actions
function ConfigForm() {
  async function updateConfig(formData: FormData) {
    'use server'
    
    const config = {
      name: formData.get('name'),
      value: formData.get('value'),
    }
    
    await saveConfig(config)
  }

  return (
    <form action={updateConfig}>
      <input name="name" />
      <input name="value" />
      <button type="submit">保存</button>
    </form>
  )
}
```

## 🎨 样式和 UI

### Tailwind CSS

**版本**: 3.4.14

**配置**:
```typescript
// tailwind.config.ts
import type { Config } from 'tailwindcss'

const config: Config = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          500: '#3b82f6',
          900: '#1e3a8a',
        },
      },
    },
  },
  plugins: [],
}
```

**优势**:
- **原子化 CSS**: 快速构建界面
- **响应式设计**: 内置断点系统
- **暗色模式**: 原生支持
- **自定义主题**: 灵活的设计系统

### Lucide React

**版本**: 0.454.0

**图标系统**:
```typescript
import { Settings, User, Database } from 'lucide-react'

function Navigation() {
  return (
    <nav className="flex space-x-4">
      <button className="flex items-center space-x-2">
        <Settings className="w-4 h-4" />
        <span>设置</span>
      </button>
      <button className="flex items-center space-x-2">
        <User className="w-4 h-4" />
        <span>用户</span>
      </button>
    </nav>
  )
}
```

## 🔧 开发工具

### TypeScript

**版本**: 5.6.3

**配置**:
```json
{
  "compilerOptions": {
    "target": "ES2017",
    "lib": ["dom", "dom.iterable", "es6"],
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
  }
}
```

**类型定义**:
```typescript
// types/api.ts
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  message?: string
}

export interface ConfigItem {
  name: string
  description: string
  base_url: string
  auth_token: string
  model: string
  provider: string
  account: string
  tags: string[]
  is_current: boolean
  is_default: boolean
}
```

### ESLint

**版本**: 内置 Next.js 配置

**配置**:
```json
{
  "extends": ["next/core-web-vitals"],
  "rules": {
    "@typescript-eslint/no-unused-vars": "error",
    "@typescript-eslint/no-explicit-any": "warn",
    "react-hooks/exhaustive-deps": "warn"
  }
}
```

### Turbopack

**特性**:
- **增量编译**: 只重新编译变更的文件
- **并行处理**: 多核 CPU 优化
- **内存缓存**: 智能缓存策略
- **热更新**: 毫秒级更新

**性能对比**:
```
Webpack 5:     ~3-5s 启动时间
Turbopack:     ~500ms 启动时间
提升:          6-10x 更快
```

## 🌐 HTTP 客户端

### Axios

**版本**: 1.7.7

**配置**:
```typescript
// lib/client.ts
import axios from 'axios'

const client = axios.create({
  baseURL: '/api',
  timeout: 600000, // 10 分钟
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器
client.interceptors.request.use(
  (config) => {
    console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`)
    return config
  },
  (error) => Promise.reject(error)
)

// 响应拦截器
client.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error.response?.data || error.message)
    return Promise.reject(error)
  }
)
```

## 🎭 代码高亮

### React Syntax Highlighter

**版本**: 15.5.0

**使用**:
```typescript
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism'

function CodeBlock({ code, language }: { code: string; language: string }) {
  return (
    <SyntaxHighlighter
      language={language}
      style={tomorrow}
      customStyle={{
        margin: 0,
        borderRadius: '0.5rem',
      }}
    >
      {code}
    </SyntaxHighlighter>
  )
}
```

## 🏗️ 构建工具

### PostCSS

**版本**: 8.4.31

**配置**:
```javascript
// postcss.config.mjs
const config = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}

export default config
```

### 构建优化

**代码分割**:
```typescript
// 动态导入
const ConfigEditor = dynamic(() => import('@/components/ConfigEditor'), {
  loading: () => <div>加载中...</div>,
  ssr: false,
})
```

**图像优化**:
```typescript
import Image from 'next/image'

function Logo() {
  return (
    <Image
      src="/logo.svg"
      alt="CCR UI"
      width={120}
      height={40}
      priority
    />
  )
}
```

## 📊 性能监控

### Web Vitals

```typescript
// app/layout.tsx
import { Analytics } from '@vercel/analytics/react'

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="zh-CN">
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  )
}
```

### 性能指标

- **FCP**: < 1.8s
- **LCP**: < 2.5s
- **FID**: < 100ms
- **CLS**: < 0.1

## 🔄 状态管理

### React Context

```typescript
// contexts/ConfigContext.tsx
interface ConfigContextType {
  configs: ConfigItem[]
  currentConfig: ConfigItem | null
  loading: boolean
  switchConfig: (name: string) => Promise<void>
}

const ConfigContext = createContext<ConfigContextType | undefined>(undefined)

export function ConfigProvider({ children }: { children: React.ReactNode }) {
  const [configs, setConfigs] = useState<ConfigItem[]>([])
  const [loading, setLoading] = useState(false)

  const switchConfig = async (name: string) => {
    setLoading(true)
    try {
      await client.post('/switch', { config_name: name })
      // 更新状态
    } finally {
      setLoading(false)
    }
  }

  return (
    <ConfigContext.Provider value={{ configs, loading, switchConfig }}>
      {children}
    </ConfigContext.Provider>
  )
}
```

## 📱 响应式设计

### 断点系统

```typescript
// Tailwind 断点
const breakpoints = {
  sm: '640px',   // 手机横屏
  md: '768px',   // 平板
  lg: '1024px',  // 笔记本
  xl: '1280px',  // 桌面
  '2xl': '1536px' // 大屏
}
```

### 自适应组件

```typescript
function ResponsiveGrid({ children }: { children: React.ReactNode }) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {children}
    </div>
  )
}
```

## 🔒 安全性

### CSP 配置

```typescript
// next.config.mjs
const nextConfig = {
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'Content-Security-Policy',
            value: "default-src 'self'; script-src 'self' 'unsafe-eval';"
          }
        ]
      }
    ]
  }
}
```

### 环境变量

```bash
# .env.local
NEXT_PUBLIC_API_URL=http://localhost:8081
NEXT_PUBLIC_APP_ENV=development
```

## 📚 相关文档

- [开发指南](/frontend/development)
- [组件文档](/frontend/components)
- [API 接口](/frontend/api)
- [样式指南](/frontend/styling)