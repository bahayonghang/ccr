# 前端技术栈详解

CCR UI 前端采用现代化的技术栈，基于 Vue 3.5 + Vite 7.1 构建，提供高性能、类型安全的用户界面。

## 🎯 核心框架

### Vue 3.5

**版本**: 3.5.22

**选择理由**:
- **Composition API**: 更灵活的逻辑组织和代码复用
- **性能优化**: 虚拟 DOM 优化，更小的打包体积
- **TypeScript 支持**: 原生 TypeScript 支持
- **响应式系统**: Proxy-based 响应式系统

**核心特性**:
```vue
<script setup lang="ts">
import { ref, computed, watch } from 'vue'

// 响应式状态
const count = ref(0)

// 计算属性
const doubled = computed(() => count.value * 2)

// 侦听器
watch(count, (newVal) => {
  console.log(`Count changed to: ${newVal}`)
})
</script>

<template>
  <div>
    <p>Count: {{ count }}</p>
    <p>Doubled: {{ doubled }}</p>
    <button @click="count++">Increment</button>
  </div>
</template>
```

### Vite 7.1

**版本**: 7.1.11

**选择理由**:
- **极速冷启动**: 原生 ESM 开发服务器
- **即时 HMR**: 毫秒级热模块替换
- **优化构建**: 基于 Rollup 的生产优化
- **插件生态**: 丰富的插件系统

**性能对比**:
```
传统打包工具:    ~3-5s 启动时间
Vite:            ~200-500ms 启动时间
提升:            6-15x 更快
```

**核心配置**:
```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    hmr: { overlay: true },
    proxy: {
      '/api': {
        target: 'http://localhost:8081',
        changeOrigin: true
      }
    }
  },
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

## 🎨 样式和 UI

### Tailwind CSS

**版本**: 3.4.17

**配置**:
```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
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
- **暗色模式**: 原生支持 `dark:` 前缀
- **自定义主题**: 灵活的设计系统

**使用示例**:
```vue
<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
      标题
    </h1>
    <p class="mt-2 text-gray-600 dark:text-gray-300">
      描述文本
    </p>
  </div>
</template>
```

### Lucide Vue Next

**版本**: 0.468.0

**图标系统**:
```vue
<script setup lang="ts">
import { Settings, User, Database } from 'lucide-vue-next'
</script>

<template>
  <nav class="flex space-x-4">
    <button class="flex items-center space-x-2">
      <Settings :size="16" />
      <span>设置</span>
    </button>
    <button class="flex items-center space-x-2">
      <User :size="16" />
      <span>用户</span>
    </button>
  </nav>
</template>
```

## 🔧 开发工具

### TypeScript

**版本**: 5.7.3

**配置**:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "strict": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "noEmit": true,
    "jsx": "preserve",
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "exclude": ["node_modules"]
}
```

**类型定义示例**:
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

export interface McpServer {
  name: string
  command: string
  args: string[]
  env: Record<string, string>
  disabled: boolean
}
```

### ESLint

**版本**: 9.19.0

**配置**:
```javascript
// eslint.config.js
import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import typescript from '@typescript-eslint/eslint-plugin'

export default [
  js.configs.recommended,
  ...vue.configs['flat/recommended'],
  {
    rules: {
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/no-unused-vars': 'error',
      '@typescript-eslint/no-explicit-any': 'warn'
    }
  }
]
```

### Vue TSC

**Vue 模板类型检查**:
```bash
# 运行类型检查
npm run type-check

# vite-plugin-checker 在开发时实时检查
```

## 🌐 路由管理

### Vue Router

**版本**: 4.4.5

**核心特性**:
- **嵌套路由**: 支持复杂的路由嵌套
- **路由守卫**: 导航守卫控制访问
- **懒加载**: 路由级别的代码分割
- **History 模式**: HTML5 History API

**配置示例**:
```typescript
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
          meta: { 
            title: 'Dashboard',
            cache: true 
          }
        },
        {
          path: 'configs',
          name: 'configs',
          component: () => import('@/views/ConfigsView.vue'),
          meta: { 
            title: '配置管理',
            requiresAuth: false 
          }
        }
      ]
    }
  ],
  scrollBehavior() {
    return { top: 0 }
  }
})

// 全局前置守卫
router.beforeEach((to, from, next) => {
  // 更新页面标题
  document.title = `${to.meta.title || 'CCR UI'} - CCR UI`
  next()
})

export default router
```

## 📦 状态管理

### Pinia

**版本**: 2.2.6

**核心特性**:
- **类型安全**: 完整的 TypeScript 支持
- **模块化**: 独立的 Store 模块
- **DevTools**: Vue DevTools 集成
- **简洁 API**: 类似 Composition API

**Store 定义**:
```typescript
// stores/config.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ConfigItem } from '@/types'

export const useConfigStore = defineStore('config', () => {
  // State
  const configs = ref<ConfigItem[]>([])
  const currentConfig = ref<ConfigItem | null>(null)
  const loading = ref(false)

  // Getters
  const configCount = computed(() => configs.value.length)
  const activeConfig = computed(() => 
    configs.value.find(c => c.is_current)
  )

  // Actions
  async function fetchConfigs() {
    loading.value = true
    try {
      const response = await listConfigs()
      configs.value = response.configs
      currentConfig.value = response.configs.find(c => c.is_current) || null
    } finally {
      loading.value = false
    }
  }

  async function switchConfig(name: string) {
    await switchConfigAPI(name)
    await fetchConfigs()
  }

  return {
    configs,
    currentConfig,
    loading,
    configCount,
    activeConfig,
    fetchConfigs,
    switchConfig
  }
})
```

**在组件中使用**:
```vue
<script setup lang="ts">
import { useConfigStore } from '@/stores/config'
import { onMounted } from 'vue'

const configStore = useConfigStore()

onMounted(() => {
  configStore.fetchConfigs()
})

function handleSwitch(name: string) {
  configStore.switchConfig(name)
}
</script>

<template>
  <div>
    <div v-if="configStore.loading">加载中...</div>
    <div v-else>
      <p>配置数量: {{ configStore.configCount }}</p>
      <div v-for="config in configStore.configs" :key="config.name">
        <button @click="handleSwitch(config.name)">
          {{ config.name }}
        </button>
      </div>
    </div>
  </div>
</template>
```

## 🌐 HTTP 客户端

### Axios

**版本**: 1.7.9

**配置**:
```typescript
// api/client.ts
import axios, { type AxiosInstance } from 'axios'

const createApiClient = (): AxiosInstance => {
  const api = axios.create({
    baseURL: '/api',
    timeout: 600000, // 10分钟
    headers: {
      'Content-Type': 'application/json'
    }
  })

  // 请求拦截器
  api.interceptors.request.use(
    (config) => {
      console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`)
      return config
    },
    (error) => {
      console.error('[API] Request error:', error)
      return Promise.reject(error)
    }
  )

  // 响应拦截器
  api.interceptors.response.use(
    (response) => response,
    (error) => {
      console.error('[API] Response error:', error.response?.data || error.message)
      return Promise.reject(error)
    }
  )

  return api
}

export const api = createApiClient()

// API 函数示例
export const listConfigs = async () => {
  const response = await api.get<ConfigListResponse>('/configs')
  return response.data
}

export const switchConfig = async (configName: string) => {
  const response = await api.post<string>('/switch', { config_name: configName })
  return response.data
}
```

## 🏗️ 构建工具

### PostCSS

**版本**: 8.5.1

**配置**:
```javascript
// postcss.config.js
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

### 构建优化

**代码分割**:
```typescript
// 路由懒加载
{
  path: '/configs',
  component: () => import('@/views/ConfigsView.vue')
}

// 组件懒加载
import { defineAsyncComponent } from 'vue'
const HeavyComponent = defineAsyncComponent(() =>
  import('@/components/HeavyComponent.vue')
)
```

**Tree-shaking**:
- Vite 自动进行 Tree-shaking
- 只打包实际使用的代码
- 减小最终包体积

## 📊 性能监控

### Vue DevTools

- 组件树查看
- 状态管理调试 (Pinia)
- 路由导航追踪
- 性能分析

### 性能指标

- **FCP (首次内容绘制)**: < 1.8s
- **LCP (最大内容绘制)**: < 2.5s
- **FID (首次输入延迟)**: < 100ms
- **CLS (累积布局偏移)**: < 0.1

## 🔄 组件开发

### Composition API

```vue
<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'

// Props
const props = defineProps<{
  title: string
  count?: number
}>()

// Emits
const emit = defineEmits<{
  (e: 'update', value: number): void
  (e: 'delete'): void
}>()

// 响应式数据
const localCount = ref(props.count || 0)
const state = reactive({
  loading: false,
  error: null as string | null
})

// 计算属性
const doubleCount = computed(() => localCount.value * 2)

// 方法
function increment() {
  localCount.value++
  emit('update', localCount.value)
}

// 生命周期
onMounted(() => {
  console.log('Component mounted')
})

// 侦听器
watch(() => props.count, (newVal) => {
  if (newVal !== undefined) {
    localCount.value = newVal
  }
})
</script>

<template>
  <div class="component">
    <h2>{{ title }}</h2>
    <p>Count: {{ localCount }}</p>
    <p>Double: {{ doubleCount }}</p>
    <button @click="increment">Increment</button>
  </div>
</template>

<style scoped>
.component {
  padding: 1rem;
  border: 1px solid #ccc;
  border-radius: 0.5rem;
}
</style>
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

```vue
<template>
  <!-- 响应式网格 -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    <slot />
  </div>
  
  <!-- 响应式显示/隐藏 -->
  <div class="hidden md:block">
    <!-- 只在中等及以上屏幕显示 -->
  </div>
  
  <!-- 响应式字体大小 -->
  <h1 class="text-2xl md:text-3xl lg:text-4xl">
    响应式标题
  </h1>
</template>
```

## 🔒 安全性

### XSS 防护

Vue 3 默认对插值进行 HTML 转义：

```vue
<template>
  <!-- 安全：自动转义 -->
  <div>{{ userInput }}</div>
  
  <!-- 危险：绕过转义 -->
  <div v-html="rawHtml"></div>
</template>
```

### CSRF 防护

```typescript
// 为所有请求添加 CSRF Token
api.interceptors.request.use((config) => {
  const token = getCsrfToken()
  if (token) {
    config.headers['X-CSRF-Token'] = token
  }
  return config
})
```

## 📚 相关文档

- [Vue 3 官方文档](https://vuejs.org/)
- [Vite 官方文档](https://vitejs.dev/)
- [Vue Router 文档](https://router.vuejs.org/)
- [Pinia 文档](https://pinia.vuejs.org/)
- [Tailwind CSS 文档](https://tailwindcss.com/)
- [开发指南](/frontend/development)
- [组件文档](/frontend/components)
- [API 接口](/frontend/api)
