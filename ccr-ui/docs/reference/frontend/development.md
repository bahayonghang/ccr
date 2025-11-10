# 前端开发指南

本指南将帮助你了解 CCR UI 前端项目（基于 Vue 3.5 + Vite 7.1）的开发流程、编码规范和最佳实践。

## 🚀 开发环境设置

### 系统要求

- **Node.js**: >= 18.0.0
- **npm**: >= 9.0.0
- **操作系统**: Linux, macOS, Windows

### 1. 安装依赖

```bash
cd frontend
npm install
```

### 2. 启动开发服务器

```bash
# 使用 Vite 启动开发服务器
npm run dev

# 开发服务器将在 http://localhost:5173 启动
# Vite 提供极速冷启动和毫秒级 HMR
```

### 3. 代码检查

```bash
# 运行 ESLint
npm run lint

# TypeScript 类型检查
npm run type-check
```

### 4. 构建项目

```bash
# 生产构建
npm run build

# 预览生产构建
npm run preview
```

## 📦 技术栈

### 核心框架

- **Vue 3.5** - 渐进式 JavaScript 框架，使用 Composition API
- **Vite 7.1** - 下一代前端构建工具
- **TypeScript** - 类型安全的 JavaScript

### UI 和样式

- **Tailwind CSS** - 实用优先的 CSS 框架
- **Lucide Vue Next** - 现代图标库

### 路由和状态管理

- **Vue Router 4** - Vue 官方路由管理器
- **Pinia 2** - Vue 状态管理库

### HTTP 客户端

- **Axios** - HTTP 请求库，支持拦截器和超时配置

## 📝 编码规范

### Vue 3 特定规范

#### 1. 使用 `<script setup>` 语法

```vue
<script setup lang="ts">
// ✅ 推荐：使用 script setup
import { ref, computed } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value++
}
</script>

<template>
  <div>
    <p>Count: {{ count }}</p>
    <p>Doubled: {{ doubled }}</p>
    <button @click="increment">Increment</button>
  </div>
</template>
```

#### 2. Props 和 Emits 类型定义

```vue
<script setup lang="ts">
// Props 定义
interface Props {
  title: string
  count?: number
  items: string[]
}

const props = withDefaults(defineProps<Props>(), {
  count: 0
})

// Emits 定义
interface Emits {
  (e: 'update', value: number): void
  (e: 'delete'): void
}

const emit = defineEmits<Emits>()

function handleUpdate() {
  emit('update', props.count + 1)
}
</script>

<template>
  <div>
    <h2>{{ title }}</h2>
    <button @click="handleUpdate">Update</button>
  </div>
</template>
```

#### 3. 类型定义

优先使用 `interface` 而不是 `type`：

```typescript
// ✅ 推荐
interface Config {
  name: string
  path: string
  isActive: boolean
}

// ❌ 避免（除非需要联合类型）
type Config = {
  name: string
  path: string
  isActive: boolean
}
```

### Vue 组件规范

#### 1. 页面组件结构

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigStore } from '@/stores/config'
import ConfigList from '@/components/ConfigList.vue'

// 使用 Pinia store
const configStore = useConfigStore()

// 本地状态
const loading = ref(false)

// 生命周期
onMounted(async () => {
  loading.value = true
  await configStore.fetchConfigs()
  loading.value = false
})
</script>

<template>
  <div class="configs-page">
    <h1>Configurations</h1>
    <div v-if="loading">Loading...</div>
    <ConfigList v-else :configs="configStore.configs" />
  </div>
</template>

<style scoped>
.configs-page {
  padding: 2rem;
}
</style>
```

#### 2. 可复用组件结构

```vue
<script setup lang="ts">
import { computed } from 'vue'
import type { Config } from '@/types'

// Props
interface Props {
  config: Config
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
})

// Emits
const emit = defineEmits<{
  (e: 'select', id: string): void
  (e: 'delete', id: string): void
}>()

// 计算属性
const statusClass = computed(() => {
  return props.config.is_current
    ? 'bg-green-100 text-green-800'
    : 'bg-gray-100 text-gray-800'
})

// 方法
function handleSelect() {
  if (!props.disabled) {
    emit('select', props.config.name)
  }
}

function handleDelete() {
  emit('delete', props.config.name)
}
</script>

<template>
  <div 
    class="config-item"
    :class="{ 'opacity-50': disabled }"
    @click="handleSelect"
  >
    <div class="flex items-center justify-between">
      <span :class="statusClass" class="px-2 py-1 rounded">
        {{ config.name }}
      </span>
      <button 
        @click.stop="handleDelete"
        class="text-red-600 hover:text-red-800"
      >
        Delete
      </button>
    </div>
  </div>
</template>

<style scoped>
.config-item {
  padding: 1rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
}

.config-item:hover {
  border-color: #3b82f6;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
</style>
```

#### 3. 组合式函数 (Composables)

将复杂的逻辑提取到组合式函数：

```typescript
// composables/useConfigs.ts
import { ref, computed } from 'vue'
import { listConfigs, switchConfig } from '@/api/client'
import type { ConfigItem } from '@/types'

export function useConfigs() {
  const configs = ref<ConfigItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const currentConfig = computed(() => 
    configs.value.find(c => c.is_current)
  )

  async function fetchConfigs() {
    loading.value = true
    error.value = null
    
    try {
      const response = await listConfigs()
      configs.value = response.configs
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
    } finally {
      loading.value = false
    }
  }

  async function switchTo(name: string) {
    loading.value = true
    try {
      await switchConfig(name)
      await fetchConfigs()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to switch'
    } finally {
      loading.value = false
    }
  }

  return {
    configs,
    loading,
    error,
    currentConfig,
    fetchConfigs,
    switchTo
  }
}
```

使用示例：

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useConfigs } from '@/composables/useConfigs'

const { configs, loading, error, fetchConfigs, switchTo } = useConfigs()

onMounted(() => {
  fetchConfigs()
})
</script>

<template>
  <div>
    <div v-if="loading">Loading...</div>
    <div v-else-if="error">Error: {{ error }}</div>
    <div v-else>
      <div v-for="config in configs" :key="config.name">
        <button @click="switchTo(config.name)">
          {{ config.name }}
        </button>
      </div>
    </div>
  </div>
</template>
```

### CSS 和样式规范

#### 1. Tailwind CSS 使用

优先使用 Tailwind CSS 类：

```vue
<template>
  <!-- ✅ 推荐 -->
  <div class="flex items-center justify-between p-4 bg-white rounded-lg shadow-md">
    <h3 class="text-lg font-semibold text-gray-900">Title</h3>
    <button class="px-4 py-2 text-white bg-blue-600 rounded hover:bg-blue-700">
      Action
    </button>
  </div>

  <!-- ❌ 避免内联样式 -->
  <div style="display: flex; padding: 16px">
    <!-- ... -->
  </div>
</template>
```

#### 2. 响应式设计

使用 Tailwind 的响应式前缀：

```vue
<template>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    <!-- 移动端单列，平板双列，桌面三列 -->
  </div>
</template>
```

#### 3. 深色模式支持

使用 `dark:` 前缀支持深色模式：

```vue
<template>
  <div class="bg-white dark:bg-gray-800 text-gray-900 dark:text-white">
    <!-- 内容 -->
  </div>
</template>
```

#### 4. Scoped 样式

使用 `scoped` 样式避免样式污染：

```vue
<style scoped>
.custom-component {
  /* 只应用于当前组件 */
}
</style>
```

## 🏗️ 项目架构

### 目录结构

```
src/
├── api/              # API 客户端
│   └── client.ts
├── assets/           # 静态资源
│   └── logo.svg
├── components/       # 可复用组件
│   ├── Button.vue
│   ├── Card.vue
│   └── ...
├── composables/      # 组合式函数
│   └── useConfigs.ts
├── layouts/          # 布局组件
│   └── MainLayout.vue
├── router/           # 路由配置
│   └── index.ts
├── stores/           # Pinia stores
│   ├── config.ts
│   └── theme.ts
├── styles/           # 全局样式
│   └── main.css
├── types/            # TypeScript 类型
│   └── index.ts
├── utils/            # 工具函数
│   └── helpers.ts
├── views/            # 页面组件
│   ├── HomeView.vue
│   └── ConfigsView.vue
├── App.vue           # 根组件
└── main.ts           # 应用入口
```

### 状态管理模式（Pinia）

#### 1. 定义 Store

```typescript
// stores/config.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listConfigs, switchConfig } from '@/api/client'
import type { ConfigItem } from '@/types'

export const useConfigStore = defineStore('config', () => {
  // State
  const configs = ref<ConfigItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const currentConfig = computed(() => 
    configs.value.find(c => c.is_current) || null
  )

  const configCount = computed(() => configs.value.length)

  // Actions
  async function fetchConfigs() {
    loading.value = true
    error.value = null
    try {
      const response = await listConfigs()
      configs.value = response.configs
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch configs'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function switchTo(name: string) {
    loading.value = true
    try {
      await switchConfig(name)
      await fetchConfigs()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to switch'
      throw err
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    configs,
    loading,
    error,
    // Getters
    currentConfig,
    configCount,
    // Actions
    fetchConfigs,
    switchTo,
    clearError
  }
})
```

#### 2. 使用 Store

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()

onMounted(() => {
  configStore.fetchConfigs()
})

async function handleSwitch(name: string) {
  try {
    await configStore.switchTo(name)
    alert('Config switched successfully!')
  } catch (err) {
    alert('Failed to switch config')
  }
}
</script>

<template>
  <div>
    <div v-if="configStore.loading">Loading...</div>
    <div v-else-if="configStore.error">
      Error: {{ configStore.error }}
      <button @click="configStore.clearError">Clear</button>
    </div>
    <div v-else>
      <p>Total: {{ configStore.configCount }}</p>
      <div v-for="config in configStore.configs" :key="config.name">
        <button @click="handleSwitch(config.name)">
          {{ config.name }}
        </button>
      </div>
    </div>
  </div>
</template>
```

### API 客户端设计

#### 1. 基础配置

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
      if (process.env.NODE_ENV === 'development') {
        console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`)
      }
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
```

#### 2. Vite 代理配置

```typescript
// vite.config.ts
export default defineConfig({
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8081',
        changeOrigin: true
      }
    }
  }
})
```

#### 3. API 函数封装

```typescript
// api/client.ts (续)
import type { ConfigListResponse } from '@/types'

export const listConfigs = async (): Promise<ConfigListResponse> => {
  const response = await api.get<ConfigListResponse>('/configs')
  return response.data
}

export const switchConfig = async (configName: string): Promise<string> => {
  const response = await api.post<string>('/switch', { config_name: configName })
  return response.data
}

export const validateConfigs = async (): Promise<string> => {
  const response = await api.get<string>('/validate')
  return response.data
}
```

## 🧪 测试策略

### 1. 单元测试

使用 Vitest 和 @vue/test-utils：

```typescript
// __tests__/ConfigCard.test.ts
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfigCard from '@/components/ConfigCard.vue'

describe('ConfigCard', () => {
  const mockConfig = {
    name: 'test-config',
    model: 'claude-3-5-sonnet-20241022',
    is_current: false,
    is_default: false
  }

  it('renders config name', () => {
    const wrapper = mount(ConfigCard, {
      props: { config: mockConfig }
    })
    expect(wrapper.text()).toContain('test-config')
  })

  it('emits select event when clicked', async () => {
    const wrapper = mount(ConfigCard, {
      props: { config: mockConfig }
    })
    
    await wrapper.find('button').trigger('click')
    expect(wrapper.emitted('select')).toBeTruthy()
    expect(wrapper.emitted('select')?.[0]).toEqual(['test-config'])
  })
})
```

### 2. 组合式函数测试

```typescript
// __tests__/useConfigs.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useConfigs } from '@/composables/useConfigs'
import * as apiClient from '@/api/client'

vi.mock('@/api/client')

describe('useConfigs', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('fetches configs successfully', async () => {
    const mockConfigs = [
      { name: 'config1', model: 'model1', is_current: true },
      { name: 'config2', model: 'model2', is_current: false }
    ]

    vi.mocked(apiClient.listConfigs).mockResolvedValue({
      configs: mockConfigs,
      current_config: 'config1',
      default_config: 'config1'
    })

    const { configs, fetchConfigs } = useConfigs()
    await fetchConfigs()

    expect(configs.value).toEqual(mockConfigs)
  })
})
```

### 3. E2E 测试

使用 Playwright：

```typescript
// e2e/config-management.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Config Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/configs')
  })

  test('should display config list', async ({ page }) => {
    await expect(page.locator('[data-testid="config-list"]')).toBeVisible()
    const items = await page.locator('[data-testid="config-item"]').count()
    expect(items).toBeGreaterThan(0)
  })

  test('should switch config', async ({ page }) => {
    await page.locator('[data-testid="config-item"]').first().click()
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible()
  })
})
```

## 🔧 开发工具

### 1. VS Code 配置

推荐的 VS Code 扩展：

```json
// .vscode/extensions.json
{
  "recommendations": [
    "Vue.volar",
    "Vue.vscode-typescript-vue-plugin",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "dbaeumer.vscode-eslint"
  ]
}
```

工作区设置：

```json
// .vscode/settings.json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "[vue]": {
    "editor.defaultFormatter": "Vue.volar"
  },
  "tailwindCSS.experimental.classRegex": [
    ["clsx\\(([^)]*)\\)", "(?:'|\"|`)([^']*)(?:'|\"|`)"]
  ]
}
```

### 2. 调试配置

```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Launch Chrome",
      "request": "launch",
      "type": "chrome",
      "url": "http://localhost:5173",
      "webRoot": "${workspaceFolder}/frontend/src"
    }
  ]
}
```

## 📈 性能优化

### 1. 路由懒加载

```typescript
// router/index.ts
const routes = [
  {
    path: '/configs',
    component: () => import('@/views/ConfigsView.vue') // 懒加载
  },
  {
    path: '/commands',
    component: () => import('@/views/CommandsView.vue')
  }
]
```

### 2. 组件懒加载

```vue
<script setup lang="ts">
import { defineAsyncComponent } from 'vue'

// 异步组件
const HeavyComponent = defineAsyncComponent(() =>
  import('@/components/HeavyComponent.vue')
)
</script>

<template>
  <Suspense>
    <template #default>
      <HeavyComponent />
    </template>
    <template #fallback>
      <div>Loading...</div>
    </template>
  </Suspense>
</template>
```

### 3. 使用 v-memo 优化渲染

```vue
<template>
  <div v-for="item in list" :key="item.id" v-memo="[item.id, item.value]">
    <!-- 只有 item.id 或 item.value 变化时才重新渲染 -->
    {{ item.name }}
  </div>
</template>
```

### 4. 虚拟列表

对于大量数据的列表，使用虚拟滚动：

```bash
npm install vue-virtual-scroller
```

```vue
<script setup lang="ts">
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

const items = ref([/* 大量数据 */])
</script>

<template>
  <RecycleScroller
    :items="items"
    :item-size="60"
    key-field="id"
    v-slot="{ item }"
  >
    <div class="item">
      {{ item.name }}
    </div>
  </RecycleScroller>
</template>
```

## 🚀 部署准备

### 1. 环境变量

```bash
# .env.production
VITE_API_BASE_URL=/api
VITE_APP_VERSION=1.0.0
```

### 2. 构建优化

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
    },
    chunkSizeWarningLimit: 1000
  }
})
```

## 📚 最佳实践

### 1. 组件设计原则

- **单一职责**：每个组件只负责一个功能
- **Props Down, Events Up**：通过 props 传递数据，通过 emits 传递事件
- **可复用性**：设计通用的组件接口
- **可测试性**：避免复杂的组件逻辑

### 2. 状态管理原则

- **最小化状态**：只存储必要的状态
- **单一数据源**：避免重复存储数据
- **合理使用 Store**：不是所有状态都需要全局管理

### 3. 性能优化原则

- **按需加载**：使用路由和组件懒加载
- **避免不必要的响应式**：使用 `shallowRef` 和 `shallowReactive`
- **合理使用计算属性**：缓存复杂计算结果

### 4. 代码组织原则

- **清晰的目录结构**：按功能而非类型组织代码
- **一致的命名规范**：组件使用 PascalCase，文件使用 kebab-case
- **合理的文件大小**：单个文件不超过 300 行

## 🔍 调试技巧

### 1. Vue DevTools

安装 Vue DevTools 浏览器扩展，用于：
- 查看组件树
- 检查组件状态
- 调试 Pinia stores
- 追踪路由变化
- 性能分析

### 2. 网络调试

使用浏览器开发者工具监控 API 请求：
- Network 面板查看请求详情
- 检查请求头和响应体
- 模拟慢速网络

### 3. 响应式调试

```vue
<script setup lang="ts">
import { ref, watchEffect } from 'vue'

const count = ref(0)

// 追踪响应式变化
watchEffect(() => {
  console.log('Count changed:', count.value)
})
</script>
```

## 📖 相关文档

- [技术栈详解](/frontend/tech-stack)
- [前端概述](/frontend/overview)
- [API 接口](/frontend/api)
- [组件文档](/frontend/components)
- [样式指南](/frontend/styling)
