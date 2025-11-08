# CCR Frontend API Documentation

CCR 前端统一 API 文档

## 📋 目录

- [概述](#概述)
- [环境检测](#环境检测)
- [统一 API](#统一-api)
- [Tauri API](#tauri-api)
- [HTTP API](#http-api)
- [类型定义](#类型定义)

## 概述

CCR Frontend 提供统一的 API 层，自动检测运行环境（Web/Desktop），透明切换后端调用方式。

### 架构图

```
Frontend Component
    ↓
Unified API Layer (@/api/index.ts)
    ↓
├─ isTauriEnvironment() → true
│   ↓
│   Tauri API (@/api/tauri.ts)
│   ↓
│   invoke('command_name', { params })
│   ↓
│   Rust Backend (src-tauri/src/main.rs)
│   ↓
│   CCR Core Library
│
└─ isTauriEnvironment() → false
    ↓
    HTTP API (@/api/client.ts)
    ↓
    axios.get('/api/endpoint')
    ↓
    Axum Backend (ccr-ui/backend)
    ↓
    CCR Core Library
```

## 环境检测

### `isTauriEnvironment()`

检测当前是否在 Tauri 桌面应用环境中运行。

```typescript
import { isTauriEnvironment } from '@/api'

if (isTauriEnvironment()) {
  console.log('Running in Desktop mode')
} else {
  console.log('Running in Web mode')
}
```

**返回值**: `boolean`
- `true`: Tauri 桌面应用
- `false`: Web 浏览器

**实现原理**:
```typescript
export const isTauriEnvironment = (): boolean => {
  return '__TAURI__' in window
}
```

### `getEnvironmentName()`

获取当前运行环境的名称。

```typescript
import { getEnvironmentName } from '@/api'

const env = getEnvironmentName() // 'tauri' | 'web'
```

## 统一 API

### 配置管理

#### `listConfigs()`

列出所有配置项。

```typescript
import { listConfigs } from '@/api'

const response = await listConfigs()
console.log(response.configs)        // 配置对象
console.log(response.current_config) // 当前配置名
console.log(response.default_config) // 默认配置名
```

**返回类型**: `ConfigListResponse`

```typescript
interface ConfigListResponse {
  configs: Record<string, {
    description?: string
    base_url: string
    auth_token: string  // 在列表中被屏蔽
    model: string
    provider_type?: string
  }>
  current_config: string
  default_config: string
}
```

**环境差异**:
- **Desktop**: 调用 `TauriAPI.listProfiles()` → 转换格式
- **Web**: 调用 `HttpAPI.listConfigs()`

#### `switchConfig(configName: string)`

切换到指定配置。

```typescript
import { switchConfig } from '@/api'

const result = await switchConfig('anthropic')
console.log(result) // "Successfully switched to profile: anthropic"
```

**参数**:
- `configName`: 配置名称

**返回类型**: `string`

**环境差异**:
- **Desktop**: `TauriAPI.switchProfile(configName)`
- **Web**: `HttpAPI.switchConfig(configName)`

#### `validateConfigs()`

验证所有配置。

```typescript
import { validateConfigs } from '@/api'

const result = await validateConfigs()
console.log(result) // "All configurations are valid"
```

**返回类型**: `string`

### 历史管理

#### `getHistory()`

获取操作历史记录。

```typescript
import { getHistory } from '@/api'

const response = await getHistory()
console.log(response.entries) // 历史条目数组
```

**返回类型**: `HistoryResponse`

```typescript
interface HistoryResponse {
  entries: Array<{
    id: string
    timestamp: string
    operation: string
    from_config: string
    to_config: string
    actor: string
  }>
}
```

**环境差异**:
- **Desktop**: `TauriAPI.getHistory(100)` → 转换格式
- **Web**: `HttpAPI.getHistory()`

### 平台管理

#### `listPlatforms()`

列出所有支持的平台。

```typescript
import { listPlatforms } from '@/api'

const platforms = await listPlatforms()
console.log(platforms) // ['claude', 'codex', 'gemini']
```

**返回类型**: `string[]`

**环境差异**:
- **Desktop**: `TauriAPI.listPlatforms()` → 返回完整列表
- **Web**: 返回 `['claude']` (Web 版本仅支持 Claude)

#### `switchPlatform(platform: string)`

切换当前平台（仅 Desktop）。

```typescript
import { switchPlatform } from '@/api'

const result = await switchPlatform('codex')
console.log(result) // "Successfully switched to platform: codex"
```

**参数**:
- `platform`: 平台名称 (`'claude'` | `'codex'` | `'gemini'`)

**返回类型**: `string`

**环境差异**:
- **Desktop**: `TauriAPI.switchPlatform(platform)`
- **Web**: 抛出错误（不支持）

#### `getCurrentPlatform()`

获取当前平台。

```typescript
import { getCurrentPlatform } from '@/api'

const platform = await getCurrentPlatform()
console.log(platform) // 'claude'
```

**返回类型**: `string`

**环境差异**:
- **Desktop**: `TauriAPI.getCurrentPlatform()`
- **Web**: 返回 `'claude'`

## Tauri API

直接使用 Tauri 命令（仅 Desktop 环境）。

```typescript
import { TauriAPI } from '@/api/tauri'

// 检查环境
if (TauriAPI.isTauriEnvironment()) {
  // 调用 Tauri 命令
  const profiles = await TauriAPI.listProfiles()
  const version = await TauriAPI.getTauriVersion()
}
```

### 可用命令

#### 配置管理

```typescript
// 列出配置
const profiles: ProfileInfo[] = await TauriAPI.listProfiles()

// 切换配置
const result: string = await TauriAPI.switchProfile('anthropic')

// 获取当前配置
const current: string = await TauriAPI.getCurrentProfile()

// 验证配置
const valid: string = await TauriAPI.validateConfigs()
```

#### 历史记录

```typescript
// 获取历史（默认 100 条）
const history: HistoryEntry[] = await TauriAPI.getHistory(100)

// 清理历史（TODO）
const result: string = await TauriAPI.clearHistory()
```

#### 云同步（TODO）

```typescript
// 推送到云端
const result: string = await TauriAPI.syncPush(false)

// 从云端拉取
const result: string = await TauriAPI.syncPull(false)

// 同步状态
const status: string = await TauriAPI.syncStatus()
```

#### 平台管理

```typescript
// 列出平台
const platforms: string[] = await TauriAPI.listPlatforms()

// 切换平台
const result: string = await TauriAPI.switchPlatform('codex')

// 获取当前平台
const platform: string = await TauriAPI.getCurrentPlatform()
```

#### 工具函数

```typescript
// 检查 Tauri 环境
const isTauri: boolean = TauriAPI.isTauriEnvironment()

// 获取 Tauri 版本
const version: string | null = await TauriAPI.getTauriVersion()
```

### 类型定义

```typescript
interface ProfileInfo {
  name: string
  description: string
  base_url: string
  model: string
  is_current: boolean
  is_default: boolean
  provider: string | null
}

interface HistoryEntry {
  id: string
  timestamp: string    // RFC3339 格式
  operation: string    // Rust Debug 格式
  actor: string
}
```

## HTTP API

直接使用 HTTP API（Web 和 Desktop 都可用）。

```typescript
import * as HttpAPI from '@/api/client'

// 配置管理
const configs = await HttpAPI.listConfigs()
const result = await HttpAPI.switchConfig('anthropic')
const valid = await HttpAPI.validateConfigs()

// 历史记录
const history = await HttpAPI.getHistory()

// 系统信息
const sysInfo = await HttpAPI.getSystemInfo()
const version = await HttpAPI.getVersion()

// MCP 服务器
const mcpServers = await HttpAPI.listMcpServers()
await HttpAPI.addMcpServer(serverData)
await HttpAPI.updateMcpServer('server-name', serverData)
await HttpAPI.deleteMcpServer('server-name')
```

详细 HTTP API 文档请查看 `src/api/client.ts`。

## 类型定义

### 通用类型

```typescript
// API 响应包装
interface ApiResponse<T> {
  data?: T
  success?: boolean
  message?: string
}

// 配置项
interface ConfigItem {
  description?: string
  base_url: string
  auth_token: string
  model: string
  provider_type?: string
}

// 历史条目
interface HistoryEntry {
  id: string
  timestamp: string
  operation: string
  from_config: string
  to_config: string
  actor: string
}
```

### Tauri 特有类型

```typescript
// ProfileInfo（Tauri 返回格式）
interface ProfileInfo {
  name: string
  description: string
  base_url: string
  model: string
  is_current: boolean
  is_default: boolean
  provider: string | null
}
```

## 使用示例

### 示例 1: 配置管理页面

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listConfigs, switchConfig, isTauriEnvironment } from '@/api'
import type { ConfigItem } from '@/types'

const configs = ref<Record<string, ConfigItem>>({})
const currentConfig = ref<string>('')
const isTauri = ref(false)

onMounted(async () => {
  // 检测环境
  isTauri.value = isTauriEnvironment()

  // 加载配置（自动选择后端）
  const data = await listConfigs()
  configs.value = data.configs
  currentConfig.value = data.current_config
})

const handleSwitch = async (name: string) => {
  try {
    await switchConfig(name)
    alert('切换成功')

    // 重新加载
    const data = await listConfigs()
    currentConfig.value = data.current_config
  } catch (error) {
    alert(`切换失败: ${error}`)
  }
}
</script>

<template>
  <div>
    <p>运行环境: {{ isTauri ? '桌面应用' : 'Web 版本' }}</p>
    <ul>
      <li v-for="(config, name) in configs" :key="name">
        {{ name }}
        <button @click="handleSwitch(name)">切换</button>
      </li>
    </ul>
  </div>
</template>
```

### 示例 2: 直接使用 Tauri API

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { TauriAPI } from '@/api/tauri'

const tauriVersion = ref<string | null>(null)
const platforms = ref<string[]>([])

onMounted(async () => {
  if (TauriAPI.isTauriEnvironment()) {
    // 获取 Tauri 版本
    tauriVersion.value = await TauriAPI.getTauriVersion()

    // 获取平台列表
    platforms.value = await TauriAPI.listPlatforms()
  }
})
</script>

<template>
  <div v-if="tauriVersion">
    <p>Tauri 版本: {{ tauriVersion }}</p>
    <p>支持的平台: {{ platforms.join(', ') }}</p>
  </div>
</template>
```

### 示例 3: 组合多个 API

```typescript
import {
  listConfigs,
  switchConfig,
  validateConfigs,
  getHistory
} from '@/api'

async function performConfigSwitch(configName: string) {
  try {
    // 1. 验证配置
    await validateConfigs()

    // 2. 切换配置
    await switchConfig(configName)

    // 3. 重新加载配置列表
    const configs = await listConfigs()

    // 4. 加载历史记录
    const history = await getHistory()

    return {
      success: true,
      configs,
      history
    }
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}
```

## 错误处理

所有 API 调用都可能抛出错误，建议使用 try-catch：

```typescript
import { listConfigs } from '@/api'

try {
  const data = await listConfigs()
  console.log('成功:', data)
} catch (error) {
  if (error instanceof Error) {
    console.error('错误:', error.message)
  } else {
    console.error('未知错误:', error)
  }
}
```

Tauri API 会在控制台打印详细错误日志：

```typescript
// Tauri API 内部错误处理
try {
  const result = await invoke<string>('command_name')
  return result
} catch (error) {
  console.error('[Tauri] command_name error:', error)
  throw error
}
```

## 性能对比

| 操作 | Web (HTTP) | Desktop (Tauri) | 性能提升 |
|------|-----------|----------------|---------|
| `listConfigs` | ~30ms | <1ms | **50x** |
| `switchConfig` | ~50ms | <1ms | **50x** |
| `getHistory` | ~20ms | <1ms | **20x** |

Desktop 模式通过 Tauri invoke 直接调用 Rust 代码，避免了 HTTP 网络开销。

## 最佳实践

1. **始终使用统一 API**
   ```typescript
   // ✅ 推荐
   import { listConfigs } from '@/api'

   // ❌ 不推荐（除非有特殊需求）
   import { listConfigs } from '@/api/client'
   ```

2. **环境检测后再使用特定功能**
   ```typescript
   if (isTauriEnvironment()) {
     // Desktop 特有功能
     await switchPlatform('codex')
   }
   ```

3. **错误处理**
   ```typescript
   try {
     const result = await switchConfig(name)
   } catch (error) {
     // 处理错误
   }
   ```

4. **类型安全**
   ```typescript
   import type { ConfigListResponse } from '@/types'

   const data: ConfigListResponse = await listConfigs()
   ```

## 相关文档

- [开发文档](./README.dev.md) - 完整开发指南
- [类型定义](./src/types/index.ts) - TypeScript 类型
- [Tauri 命令实现](./src-tauri/src/main.rs) - Rust 后端代码

---

**最后更新**: 2025-11-08
**API 版本**: 2.5.0
