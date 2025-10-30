# 前端项目概述

CCR UI 的前端是一个基于 **Vue 3.5 + Vite 7.1** 构建的现代化 Web 应用，采用 Vue Router 和 Pinia 状态管理，为用户提供直观、响应式的 CCR 配置管理界面。

## 🎯 项目目标

前端应用的主要目标是：

- **用户友好**：提供直观、易用的配置管理界面
- **实时交互**：支持实时命令执行和结果展示
- **响应式设计**：适配桌面端和移动端设备
- **极致性能**：利用 Vite 7.1 实现极速的开发服务器和构建性能
- **类型安全**：使用 TypeScript 5.7 确保代码质量
- **组件化开发**：Vue 3 Composition API 提供灵活的组件开发方式

## 🏗️ 技术架构

### 核心技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue | 3.5.22 | 渐进式 JavaScript 框架 |
| Vite | 7.1.11 | 下一代前端构建工具 |
| Vue Router | 4.4.5 | Vue 官方路由管理器 |
| Pinia | 2.2.6 | Vue 状态管理库 |
| TypeScript | 5.7.3 | 类型安全的 JavaScript 超集 |
| Axios | 1.7.9 | HTTP 客户端 |
| Tailwind CSS | 3.4.17 | 实用优先的 CSS 框架 |
| Lucide Vue Next | 0.468.0 | 现代化图标库 |

### Vite 7.1 核心特性

- **极速冷启动** - 原生 ESM 按需编译
- **HMR（热模块替换）** - 毫秒级的模块热更新
- **优化的构建** - 基于 Rollup 的生产优化
- **TypeScript 支持** - 开箱即用的 TS 支持
- **CSS 预处理** - 内置 PostCSS、Sass 等支持
- **资源优化** - 智能代码分割和懒加载

### 开发工具

- **ESLint** - 代码质量检查（Vue 配置）
- **Vue TSC** - Vue 模板类型检查
- **PostCSS** - CSS 后处理器
- **Autoprefixer** - CSS 自动前缀

## 📁 项目结构

```
frontend/
├── public/                     # 静态资源
│   └── vite.svg               # 应用图标
├── src/                       # 源代码
│   ├── main.ts                # 应用入口
│   ├── App.vue                # 根组件
│   │
│   ├── views/                 # 页面组件
│   │   ├── HomeView.vue       # Dashboard 首页
│   │   │
│   │   ├── ClaudeCodeView.vue # Claude Code 主页
│   │   ├── CodexView.vue      # Codex 主页
│   │   ├── GeminiCliView.vue  # Gemini CLI 主页
│   │   ├── QwenView.vue       # Qwen 主页
│   │   ├── IflowView.vue      # IFLOW 主页
│   │   │
│   │   ├── ConfigsView.vue    # 配置管理
│   │   ├── SyncView.vue       # 云同步
│   │   ├── McpView.vue        # MCP 服务器管理
│   │   ├── SlashCommandsView.vue # Slash Commands
│   │   ├── AgentsView.vue     # Agents 管理
│   │   ├── PluginsView.vue    # 插件管理
│   │   ├── CommandsView.vue   # 命令执行中心
│   │   ├── ConverterView.vue  # 配置转换器
│   │   ├── StatsView.vue      # 统计分析
│   │   │
│   │   ├── CodexMcpView.vue   # Codex MCP 子页面
│   │   ├── CodexProfilesView.vue # Codex Profiles
│   │   ├── GeminiMcpView.vue  # Gemini MCP 子页面
│   │   ├── QwenMcpView.vue    # Qwen MCP 子页面
│   │   └── IflowMcpView.vue   # IFLOW MCP 子页面
│   │
│   ├── components/            # 可复用组件
│   │   ├── MainLayout.vue     # 主布局
│   │   ├── Navbar.vue         # 顶部导航栏
│   │   ├── CollapsibleSidebar.vue # 侧边栏
│   │   ├── RightSidebar.vue   # 右侧边栏
│   │   ├── StatusHeader.vue   # 状态头部
│   │   ├── HistoryList.vue    # 历史记录列表
│   │   ├── VersionManager.vue # 版本管理器
│   │   ├── ThemeToggle.vue    # 主题切换
│   │   ├── UpdateModal.vue    # 更新对话框
│   │   ├── ConfigCard.vue     # 配置卡片
│   │   ├── Button.vue         # 按钮组件
│   │   ├── Card.vue           # 卡片组件
│   │   ├── Input.vue          # 输入框组件
│   │   └── Table.vue          # 表格组件
│   │
│   ├── router/                # 路由配置
│   │   └── index.ts           # Vue Router 配置
│   │
│   ├── stores/                # Pinia 状态管理
│   │   ├── config.ts          # 配置状态
│   │   ├── system.ts          # 系统状态
│   │   └── theme.ts           # 主题状态
│   │
│   ├── api/                   # API 客户端
│   │   └── client.ts          # HTTP 客户端和 API 调用
│   │
│   ├── types/                 # TypeScript 类型定义
│   │   └── index.ts           # 通用类型
│   │
│   ├── styles/                # 全局样式
│   │   └── main.css           # 主样式文件
│   │
│   └── utils/                 # 工具函数
│       └── helpers.ts         # 辅助函数
│
├── package.json              # 项目配置
├── vite.config.ts            # Vite 配置
├── tailwind.config.js        # Tailwind 配置
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

使用 Vue 3 的 Composition API 管理组件本地状态：

```vue
<script setup lang="ts">
import { ref } from 'vue'

const configs = ref<Config[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
</script>
```

### 全局状态（Pinia）

使用 Pinia 管理主题等全局状态：

```typescript
// src/stores/theme.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<'light' | 'dark'>('light')
  
  const toggleTheme = () => {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
  }

  return { theme, toggleTheme }
})
```

使用示例：

```vue
<script setup lang="ts">
import { useThemeStore } from '@/stores/theme'

const themeStore = useThemeStore()
</script>

<template>
  <button @click="themeStore.toggleTheme()">
    切换主题
  </button>
</template>
```

## 🌐 路由配置

使用 Vue Router 4 进行路由管理：

```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: () => import('@/components/MainLayout.vue'),
      children: [
        {
          path: '',
          name: 'home',
          component: () => import('@/views/HomeView.vue'),
          meta: { cache: true }
        },
        // CLI 工具主页
        {
          path: 'claude-code',
          name: 'claude-code',
          component: () => import('@/views/ClaudeCodeView.vue')
        },
        {
          path: 'codex',
          name: 'codex',
          component: () => import('@/views/CodexView.vue')
        },
        // ... 其他路由
      ]
    }
  ],
  scrollBehavior() {
    return { top: 0 }
  }
})
```

### 路由结构（三级导航）

**一级 - 首页 Dashboard**
- `/` - Dashboard 首页（展示所有功能模块）

**二级 - CLI 工具主页**
- `/claude-code` - Claude Code 主页
- `/codex` - Codex 主页
- `/gemini-cli` - Gemini CLI 主页
- `/qwen` - Qwen 主页
- `/iflow` - IFLOW 主页

**三级 - 功能页面**
- `/configs` - 配置管理（Claude Code）
- `/sync` - 云同步（Claude Code）
- `/mcp` - MCP 服务器管理
- `/slash-commands` - Slash Commands 管理
- `/agents` - Agents 管理
- `/plugins` - 插件管理
- `/commands` - 命令执行中心
- `/converter` - 配置转换器
- `/stats` - 统计分析

**子路由示例**
- `/codex/mcp` - Codex MCP 配置
- `/codex/profiles` - Codex Profiles
- `/gemini-cli/mcp` - Gemini MCP 配置
- `/qwen/mcp` - Qwen MCP 配置

### 路由守卫

```typescript
// 全局前置守卫
router.beforeEach((to, from, next) => {
  // 路由切换逻辑
  next()
})

// 全局后置钩子
router.afterEach((to, from) => {
  // 页面标题更新等
})
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

### 1. Dashboard 首页

- 8 个功能模块卡片展示
- 系统状态实时监控（CPU、内存、系统信息）
- 动态渐变背景效果
- 快速访问各个 CLI 工具主页

### 2. CLI 工具主页

每个 CLI 工具（Claude Code、Codex、Gemini、Qwen、IFLOW）都有独立的主页：
- 清晰的子功能模块展示
- 独特的渐变配色方案
- 子功能快速导航卡片
- 返回首页便捷按钮

### 3. 配置管理（Claude Code）

- 显示当前可用的 CCR 配置列表
- 支持配置切换操作
- 实时显示当前激活的配置
- 配置验证功能
- 配置分类筛选（官方中转/第三方模型/未分类）
- 历史记录查看和审计

### 4. 云同步（WebDAV）

- 配置上传/下载
- 同步状态检查
- 强制推送/拉取
- WebDAV 配置显示
- 自动同步功能

### 5. MCP 服务器管理

- MCP 服务器列表展示
- 添加/编辑/删除服务器
- 启用/禁用服务器
- STDIO 和 HTTP 协议支持
- 环境变量配置

### 6. Slash Commands 管理

- 自定义命令列表
- 命令添加/编辑/删除
- 文件夹组织
- 命令启用/禁用

### 7. Agents 管理

- Agent 列表展示
- Agent 配置编辑
- 工具绑定管理
- 模型选择

### 8. 插件管理

- 插件列表展示
- 插件启用/禁用
- 插件配置编辑

### 9. 命令执行中心

- 可视化的命令输入界面
- 实时显示命令执行结果
- 支持 6 个 CLI 工具的命令
- 终端风格的输出显示

### 10. 配置转换器

- 跨 CLI 工具的配置格式转换
- 支持 MCP、Slash Commands、Agents 转换
- JSON/TOML 双格式支持

### 11. 用户界面

- 响应式导航栏
- 可折叠侧边栏菜单
- 深色/浅色主题切换
- 加载状态和错误处理
- 玻璃拟态设计风格
- 流畅动画效果

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

Vue Router 支持路由级别的代码分割：

```typescript
// 路由懒加载
const routes = [
  {
    path: '/configs',
    // 只在访问时加载，自动代码分割
    component: () => import('@/views/ConfigsView.vue')
  },
  {
    path: '/commands',
    // 独立的代码块
    component: () => import('@/views/CommandsView.vue')
  }
]
```

### 组件懒加载

使用 `defineAsyncComponent` 进行组件级别的懒加载：

```typescript
import { defineAsyncComponent } from 'vue'

// 异步组件
const AsyncComp = defineAsyncComponent(() =>
  import('./components/HeavyComponent.vue')
)
```

### Vite 构建优化

- **极速冷启动**：原生 ESM，无需打包即可启动
- **按需编译**：只编译当前访问的代码
- **智能依赖预构建**：使用 esbuild 预构建依赖
- **增量更新**：HMR 只更新变更的模块
- **代码分割**：自动分割 vendor 和业务代码

### 资源优化

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'vue-vendor': ['vue', 'vue-router', 'pinia'],
          'ui-vendor': ['lucide-vue-next'],
          'http-vendor': ['axios']
        }
      }
    }
  }
})
```

## 🧪 测试策略

### 单元测试

使用 Vitest + @vue/test-utils 进行单元测试：

```typescript
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import App from './App.vue'

describe('App', () => {
  it('renders without crashing', () => {
    const wrapper = mount(App)
    expect(wrapper.text()).toContain('CCR UI')
  })
})
```

### 组件测试

```typescript
import { mount } from '@vue/test-utils'
import ConfigCard from '@/components/ConfigCard.vue'

describe('ConfigCard', () => {
  it('displays config information', () => {
    const wrapper = mount(ConfigCard, {
      props: {
        config: {
          name: 'default',
          model: 'claude-3-5-sonnet-20241022'
        }
      }
    })
    expect(wrapper.text()).toContain('default')
  })
})
```

### 端到端测试

使用 Playwright 或 Cypress 进行 E2E 测试：

```typescript
import { test, expect } from '@playwright/test'

test('should load and display configs', async ({ page }) => {
  await page.goto('/configs')
  await expect(page.locator('[data-testid="config-list"]')).toBeVisible()
  const items = await page.locator('[data-testid="config-item"]').count()
  expect(items).toBeGreaterThan(0)
})
```

## 🚀 构建和部署

### 开发环境

```bash
# 启动开发服务器（使用 Vite）
npm run dev

# 开发服务器运行在 http://localhost:5173
# 支持热模块替换（HMR），极速冷启动
```

### 生产构建

```bash
# 构建生产版本
npm run build

# 预览生产构建
npm run preview
```

### 构建产物

```
dist/
├── assets/             # 静态资源（JS、CSS、图片等）
│   ├── index-[hash].js
│   ├── index-[hash].css
│   └── *.svg
└── index.html          # 入口 HTML 文件
```

### 构建优化特性

- **Tree-shaking**：自动移除未使用的代码
- **代码分割**：按需加载，减小初始加载体积
- **资源压缩**：自动压缩 JS、CSS 和图片
- **哈希命名**：文件名包含内容哈希，便于缓存
- **Legacy 支持**：可选的传统浏览器支持

### 部署选项

1. **静态托管**（推荐）- Vercel、Netlify、Cloudflare Pages
2. **Docker** - 容器化部署
3. **Nginx** - 传统 Web 服务器
4. **Node.js** - 使用 Express 等框架提供静态文件服务

### Docker 部署示例

```dockerfile
FROM nginx:alpine
COPY dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## 📚 相关文档

- [技术栈详解](/frontend/tech-stack)
- [开发指南](/frontend/development)
- [组件文档](/frontend/components)
- [API 接口](/frontend/api)
- [样式指南](/frontend/styling)