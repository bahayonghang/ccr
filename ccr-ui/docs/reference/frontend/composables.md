# Composables 详细文档

> **版本**: v3.16.2  
> **最后更新**: 2025-12-28  
> **框架**: Vue 3 Composition API

Composables（组合式函数）是 Vue 3 中用于封装和复用有状态逻辑的函数。CCR UI 前端包含 10 个核心 Composables。

## 📋 目录

- [概述](#概述)
- [Composables 列表](#composables-列表)
- [详细说明](#详细说明)
- [使用示例](#使用示例)
- [最佳实践](#最佳实践)

## 🎯 概述

### 什么是 Composables？

Composables 是以 `use` 开头的函数，用于：
- 封装可复用的状态逻辑
- 提供响应式数据和方法
- 简化组件代码，提高可维护性

### 核心优势

- ✅ **代码复用**：避免在多个组件中重复相同逻辑
- ✅ **逻辑分离**：将复杂逻辑从组件中抽离
- ✅ **类型安全**：TypeScript 支持，提供完整类型推导
- ✅ **测试友好**：独立函数，易于单元测试

## 📚 Composables 列表

| Composable | 文件 | 用途 |
|------------|------|------|
| **useAccessibility** | `useAccessibility.ts` | 无障碍辅助功能 |
| **useAgents** | `useAgents.ts` | Agents 管理逻辑 |
| **useApi** | `useApi.ts` | API 调用封装 |
| **useCcrControl** | `useCcrControl.ts` | CCR 控制面板 |
| **usePlatformMcp** | `usePlatformMcp.ts` | 平台 MCP 管理 |
| **usePlatformPlugins** | `usePlatformPlugins.ts` | 平台插件管理 |
| **useSkills** | `useSkills.ts` | 技能管理 |
| **useSkillsCache** | `useSkillsCache.ts` | 技能缓存 |
| **useStream** | `useStream.ts` | 流式数据处理 |
| **useWebSocket** | `useWebSocket.ts` | WebSocket 连接 |

## 🔍 详细说明

### 1. useAccessibility

**路径**: `src/composables/useAccessibility.ts`

**用途**: 提供无障碍辅助功能，包括键盘导航、屏幕阅读器支持等。

**导出**:
```typescript
interface UseAccessibility {
  // 焦点管理
  focusFirst: () => void
  focusLast: () => void
  focusNext: () => void
  focusPrevious: () => void
  
  // 键盘事件处理
  handleKeyDown: (event: KeyboardEvent) => void
  
  // ARIA 属性生成
  getAriaLabel: (text: string) => string
  getAriaDescribedBy: (id: string) => string
}
```

**使用场景**:
- 模态框焦点trap
- 列表键盘导航
- 表单无障碍增强

**示例**:
```vue
<script setup lang="ts">
import { useAccessibility } from '@/composables/useAccessibility'

const { focusFirst, handleKeyDown } = useAccessibility()

// 模态框打开时聚焦第一个元素
onMounted(() => {
  focusFirst()
})
</script>
```

### 2. useAgents

**路径**: `src/composables/useAgents.ts`

**用途**: 封装 Agents 管理的核心逻辑，支持多平台。

**导出**:
```typescript
interface UseAgents {
  // 状态
  agents: Ref<Agent[]>
  loading: Ref<boolean>
  error: Ref<string | null>
  
  // 方法
  fetchAgents: (platform?: string) => Promise<void>
  createAgent: (agent: AgentCreate) => Promise<void>
  updateAgent: (name: string, agent: AgentUpdate) => Promise<void>
  deleteAgent: (name: string) => Promise<void>
  toggleAgent: (name: string, enabled: boolean) => Promise<void>
  
  // 实用方法
  getAgentByName: (name: string) => Agent | undefined
  filterAgentsByFolder: (folder: string) => Agent[]
}
```

**使用场景**:
- AgentsView 页面
- Agent 选择器组件
- Agent 管理表单

**示例**:
```vue
<script setup lang="ts">
import { useAgents } from '@/composables/useAgents'

const { agents, loading, fetchAgents, deleteAgent } = useAgents()

onMounted(async () => {
  await fetchAgents('claude')
})

const handleDelete = async (name: string) => {
  if (confirm('确认删除？')) {
    await deleteAgent(name)
  }
}
</script>

<template>
  <div v-if="loading">加载中...</div>
  <div v-else>
    <div v-for="agent in agents" :key="agent.name">
      {{ agent.name }}
      <button @click="handleDelete(agent.name)">删除</button>
    </div>
  </div>
</template>
```

### 3. useApi

**路径**: `src/composables/useApi.ts`

**用途**: 统一的 API 调用封装，提供加载状态、错误处理等。

**导出**:
```typescript
interface UseApi<T> {
  // 状态
  data: Ref<T | null>
  loading: Ref<boolean>
  error: Ref<Error | null>
  
  // 方法
  execute: (...args: any[]) => Promise<T>
  reset: () => void
}

function useApi<T>(
  apiFunction: (...args: any[]) => Promise<T>,
  options?: {
    immediate?: boolean
    onSuccess?: (data: T) => void
    onError?: (error: Error) => void
  }
): UseApi<T>
```

**使用场景**:
- 封装任何 API 调用
- 自动管理加载状态
- 统一错误处理

**示例**:
```vue
<script setup lang="ts">
import { useApi } from '@/composables/useApi'
import { getConfigs } from '@/api/client'

const { data: configs, loading, error, execute } = useApi(getConfigs, {
  immediate: true,
  onError: (err) => {
    console.error('获取配置失败:', err)
  }
})

const refresh = () => {
  execute()
}
</script>

<template>
  <div v-if="loading">加载中...</div>
  <div v-else-if="error">错误: {{ error.message }}</div>
  <div v-else>
    <button @click="refresh">刷新</button>
    <!-- 显示配置列表 -->
  </div>
</template>
```

### 4. useCcrControl

**路径**: `src/composables/useCcrControl.ts`

**用途**: CCR 控制面板，封装配置切换、历史记录等操作。

**导出**:
```typescript
interface UseCcrControl {
  // 状态
  currentConfig: Ref<string | null>
  configs: Ref<Config[]>
  history: Ref<HistoryEntry[]>
  
  // 配置操作
  switchConfig: (name: string) => Promise<void>
  validateConfig: (name: string) => Promise<boolean>
  
  // 历史操作
  fetchHistory: () => Promise<void>
  rollback: (historyId: string) => Promise<void>
  
  // 实用方法
  isCurrentConfig: (name: string) => boolean
}
```

**使用场景**:
- CcrControlView 页面
- 配置切换组件
- 历史记录展示

**示例**:
```vue
<script setup lang="ts">
import { useCcrControl } from '@/composables/useCcrControl'

const {
  currentConfig,
  configs,
  switchConfig,
  isCurrentConfig
} = useCcrControl()

const handleSwitch = async (name: string) => {
  await switchConfig(name)
  alert('切换成功！')
}
</script>

<template>
  <div v-for="config in configs" :key="config.name">
    <span :class="{ active: isCurrentConfig(config.name) }">
      {{ config.name }}
    </span>
    <button @click="handleSwitch(config.name)">切换</button>
  </div>
</template>
```

### 5. usePlatformMcp

**路径**: `src/composables/usePlatformMcp.ts`

**用途**: 平台特定的 MCP 服务器管理逻辑。

**导出**:
```typescript
interface UsePlatformMcp {
  // 状态
  servers: Ref<McpServer[]>
  loading: Ref<boolean>
  
  // 方法
  fetchServers: (platform: string) => Promise<void>
  addServer: (platform: string, server: McpServerCreate) => Promise<void>
  updateServer: (platform: string, name: string, server: McpServerUpdate) => Promise<void>
  deleteServer: (platform: string, name: string) => Promise<void>
  toggleServer: (platform: string, name: string, disabled: boolean) => Promise<void>
}
```

**使用场景**:
- PlatformMcpView 通用视图
- Codex/Gemini/Qwen MCP 页面

**示例**:
```vue
<script setup lang="ts">
import { usePlatformMcp } from '@/composables/usePlatformMcp'

const props = defineProps<{
  platform: string
}>()

const { servers, fetchServers, toggleServer } = usePlatformMcp()

onMounted(() => {
  fetchServers(props.platform)
})

const handleToggle = (name: string, disabled: boolean) => {
  toggleServer(props.platform, name, !disabled)
}
</script>
```

### 6. usePlatformPlugins

**路径**: `src/composables/usePlatformPlugins.ts`

**用途**: 平台特定的插件管理逻辑。

**导出**:
```typescript
interface UsePlatformPlugins {
  // 状态
  plugins: Ref<Plugin[]>
  loading: Ref<boolean>
  
  // 方法
  fetchPlugins: (platform: string) => Promise<void>
  installPlugin: (platform: string, pluginId: string) => Promise<void>
  uninstallPlugin: (platform: string, pluginId: string) => Promise<void>
  togglePlugin: (platform: string, pluginId: string, enabled: boolean) => Promise<void>
  configurePlugin: (platform: string, pluginId: string, config: any) => Promise<void>
}
```

**使用场景**:
- PlatformPluginsView 通用视图
- 插件管理页面

### 7. useSkills

**路径**: `src/composables/useSkills.ts`

**用途**: 技能管理，支持技能搜索、仓库管理等。

**导出**:
```typescript
interface UseSkills {
  // 状态
  skills: Ref<Skill[]>
  repositories: Ref<Repository[]>
  loading: Ref<boolean>
  
  // 技能操作
  searchSkills: (query: string) => Promise<void>
  installSkill: (skillId: string) => Promise<void>
  uninstallSkill: (skillId: string) => Promise<void>
  updateSkill: (skillId: string) => Promise<void>
  
  // 仓库操作
  addRepository: (url: string) => Promise<void>
  removeRepository: (id: string) => Promise<void>
  refreshRepository: (id: string) => Promise<void>
}
```

**使用场景**:
- SkillsView 页面
- MarketView 技能市场

**示例**:
```vue
<script setup lang="ts">
import { useSkills } from '@/composables/useSkills'

const { skills, searchSkills, installSkill } = useSkills()

const searchQuery = ref('')

const handleSearch = async () => {
  await searchSkills(searchQuery.value)
}

const handleInstall = async (skillId: string) => {
  await installSkill(skillId)
  alert('安装成功！')
}
</script>

<template>
  <input v-model="searchQuery" @keyup.enter="handleSearch" />
  <button @click="handleSearch">搜索</button>
  
  <div v-for="skill in skills" :key="skill.id">
    {{ skill.name }}
    <button @click="handleInstall(skill.id)">安装</button>
  </div>
</template>
```

### 8. useSkillsCache

**路径**: `src/composables/useSkillsCache.ts`

**用途**: 技能数据缓存，提升性能和用户体验。

**导出**:
```typescript
interface UseSkillsCache {
  // 缓存操作
  getCachedSkills: (query: string) => Skill[] | null
  setCachedSkills: (query: string, skills: Skill[]) => void
  clearCache: () => void
  
  // 缓存状态
  cacheSize: Ref<number>
  cacheHitRate: Ref<number>
}
```

**使用场景**:
- 与 useSkills 配合使用
- 减少重复 API 请求

**示例**:
```typescript
import { useSkills } from '@/composables/useSkills'
import { useSkillsCache } from '@/composables/useSkillsCache'

const { skills, searchSkills } = useSkills()
const { getCachedSkills, setCachedSkills } = useSkillsCache()

const searchWithCache = async (query: string) => {
  // 先查缓存
  const cached = getCachedSkills(query)
  if (cached) {
    skills.value = cached
    return
  }
  
  // 缓存未命中，请求 API
  await searchSkills(query)
  setCachedSkills(query, skills.value)
}
```

### 9. useStream

**路径**: `src/composables/useStream.ts`

**用途**: 处理流式数据，用于命令执行、日志流等场景。

**导出**:
```typescript
interface UseStream {
  // 状态
  data: Ref<string[]>
  isStreaming: Ref<boolean>
  error: Ref<Error | null>
  
  // 方法
  startStream: (url: string) => Promise<void>
  stopStream: () => void
  clearData: () => void
  
  // 事件处理
  onData: (callback: (chunk: string) => void) => void
  onComplete: (callback: () => void) => void
  onError: (callback: (error: Error) => void) => void
}
```

**使用场景**:
- CommandsView 命令执行
- 日志实时查看

**示例**:
```vue
<script setup lang="ts">
import { useStream } from '@/composables/useStream'

const { data, isStreaming, startStream, stopStream } = useStream()

const executeCommand = async (command: string) => {
  await startStream(`/api/command/execute?cmd=${command}`)
}

const handleStop = () => {
  stopStream()
}
</script>

<template>
  <button @click="executeCommand('ccr list')">执行命令</button>
  <button v-if="isStreaming" @click="handleStop">停止</button>
  
  <div class="terminal">
    <div v-for="(line, index) in data" :key="index">
      {{ line }}
    </div>
  </div>
</template>
```

### 10. useWebSocket

**路径**: `src/composables/useWebSocket.ts`

**用途**: WebSocket 连接管理，用于实时通信。

**导出**:
```typescript
interface UseWebSocket {
  // 状态
  isConnected: Ref<boolean>
  error: Ref<Error | null>
  messages: Ref<any[]>
  
  // 连接管理
  connect: (url: string) => void
  disconnect: () => void
  reconnect: () => void
  
  // 消息发送
  send: (data: any) => void
  sendJson: (data: object) => void
  
  // 事件监听
  onMessage: (callback: (data: any) => void) => void
  onOpen: (callback: () => void) => void
  onClose: (callback: () => void) => void
  onError: (callback: (error: Error) => void) => void
}
```

**使用场景**:
- 实时日志流
- 命令执行进度
- 系统状态更新

**示例**:
```vue
<script setup lang="ts">
import { useWebSocket } from '@/composables/useWebSocket'

const { 
  isConnected, 
  messages, 
  connect, 
  disconnect, 
  send 
} = useWebSocket()

onMounted(() => {
  connect('ws://localhost:38081/ws')
})

onUnmounted(() => {
  disconnect()
})

const sendMessage = () => {
  send({ type: 'ping' })
}
</script>

<template>
  <div :class="{ connected: isConnected, disconnected: !isConnected }">
    {{ isConnected ? '已连接' : '未连接' }}
  </div>
  
  <button @click="sendMessage">发送消息</button>
  
  <div v-for="(msg, index) in messages" :key="index">
    {{ JSON.stringify(msg) }}
  </div>
</template>
```

## 💡 使用示例

### 组合多个 Composables

```vue
<script setup lang="ts">
import { useAgents } from '@/composables/useAgents'
import { useApi } from '@/composables/useApi'
import { useAccessibility } from '@/composables/useAccessibility'

// 使用 useAgents 获取数据
const { agents, loading, fetchAgents } = useAgents()

// 使用 useApi 执行其他 API 调用
const { execute: deleteAgent } = useApi(async (name: string) => {
  // API 调用逻辑
}, {
  onSuccess: () => {
    fetchAgents() // 删除成功后刷新列表
  }
})

// 使用 useAccessibility 增强无障碍
const { handleKeyDown } = useAccessibility()

onMounted(() => {
  fetchAgents()
})
</script>
```

### 自定义 Composable

基于现有 Composables 创建自定义逻辑：

```typescript
// src/composables/useAgentForm.ts
import { ref, computed } from 'vue'
import { useAgents } from './useAgents'

export function useAgentForm() {
  const { createAgent, updateAgent } = useAgents()
  
  const form = ref({
    name: '',
    model: 'claude-3-5-sonnet-20241022',
    tools: [],
    systemPrompt: ''
  })
  
  const isValid = computed(() => {
    return form.value.name.length > 0 && 
           form.value.systemPrompt.length > 0
  })
  
  const submit = async () => {
    if (!isValid.value) return
    await createAgent(form.value)
    reset()
  }
  
  const reset = () => {
    form.value = {
      name: '',
      model: 'claude-3-5-sonnet-20241022',
      tools: [],
      systemPrompt: ''
    }
  }
  
  return {
    form,
    isValid,
    submit,
    reset
  }
}
```

## 📏 最佳实践

### 1. 命名规范

✅ **推荐**:
```typescript
// 以 use 开头
export function useAgents() { ... }
export function usePlatformMcp() { ... }

// 返回对象，包含状态和方法
return {
  // 状态
  agents,
  loading,
  error,
  
  // 方法
  fetchAgents,
  createAgent
}
```

❌ **不推荐**:
```typescript
// 不以 use 开头
export function agentsHelper() { ... }

// 返回数组（难以扩展）
return [agents, fetchAgents]
```

### 2. 响应式数据

✅ **推荐**:
```typescript
export function useAgents() {
  // 使用 ref 或 reactive 创建响应式数据
  const agents = ref<Agent[]>([])
  const loading = ref(false)
  
  return {
    agents,
    loading
  }
}
```

❌ **不推荐**:
```typescript
export function useAgents() {
  // 返回普通变量（非响应式）
  let agents = []
  let loading = false
  
  return {
    agents,
    loading
  }
}
```

### 3. 错误处理

✅ **推荐**:
```typescript
export function useAgents() {
  const error = ref<Error | null>(null)
  
  const fetchAgents = async () => {
    try {
      error.value = null
      // API 调用
    } catch (err) {
      error.value = err as Error
      console.error('获取 Agents 失败:', err)
    }
  }
  
  return {
    error,
    fetchAgents
  }
}
```

### 4. 生命周期

✅ **推荐**:
```typescript
export function useWebSocket(url: string) {
  const socket = ref<WebSocket | null>(null)
  
  // 在 Composable 内部管理生命周期
  onMounted(() => {
    socket.value = new WebSocket(url)
  })
  
  onUnmounted(() => {
    socket.value?.close()
  })
  
  return {
    socket
  }
}
```

### 5. 参数传递

✅ **推荐**:
```typescript
// 支持响应式参数
export function useAgents(platform: Ref<string>) {
  const agents = ref<Agent[]>([])
  
  // 监听参数变化
  watch(platform, async (newPlatform) => {
    await fetchAgents(newPlatform)
  }, { immediate: true })
  
  return {
    agents
  }
}
```

### 6. TypeScript 类型

✅ **推荐**:
```typescript
interface UseAgentsReturn {
  agents: Ref<Agent[]>
  loading: Ref<boolean>
  fetchAgents: () => Promise<void>
}

export function useAgents(): UseAgentsReturn {
  // 实现
}
```

### 7. 依赖注入

使用 `provide/inject` 在组件树中共享 Composable 状态：

```typescript
// 在父组件提供
import { provide } from 'vue'
import { useAgents } from '@/composables/useAgents'

const agentsApi = useAgents()
provide('agentsApi', agentsApi)
```

```typescript
// 在子组件注入
import { inject } from 'vue'

const agentsApi = inject('agentsApi')
```

## 🔗 相关文档

- [Vue 3 Composition API](https://vuejs.org/guide/reusability/composables.html)
- [前端开发指南](../development.md)
- [组件文档](./components.md)
- [API 客户端文档](./api.md)

---

**浮浮酱温馨提示**：Composables 是 Vue 3 的精华所在，充分利用它们可以让代码更优雅、更易维护！(´｡• ᵕ •｡`) ♡
