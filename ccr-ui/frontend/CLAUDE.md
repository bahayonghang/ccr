# CCR UI Frontend 模块指导文件

[根目录](../../CLAUDE.md) > [ccr-ui](../CLAUDE.md) > **frontend**

## Change Log
- **2025-12-16**: 按标准模板重新组织文档结构
- **2025-10-22 00:04:36 CST**: 初始前端模块文档创建

---

## 项目架构

### 模块职责

CCR UI Frontend 是基于 Vue.js 3 的现代化单页应用(SPA),提供可视化管理界面用于管理多个 AI CLI 工具。

**核心职责**:
1. **可视化仪表盘** - 统一管理多个 AI CLI 工具的界面
2. **配置管理** - 可视化编辑器用于配置文件编辑
3. **命令执行** - 带可视化反馈的 CCR 命令执行
4. **多平台支持** - Claude Code, Codex, Gemini CLI, Qwen, iFlow
5. **Liquid Glass 设计** - 现代玻璃拟态 UI 与流畅动画

**关键特性**:
- 响应式设计(移动端/平板/桌面)
- 暗黑/明亮主题支持
- 实时 API 通信
- Pinia 状态管理
- TypeScript 类型安全

### 架构层次

```
frontend/
├── Presentation Layer (展示层)
│   ├── Views/             - 页面组件(40+ 视图)
│   └── Components/        - 可复用组件(15+ 组件)
│
├── State Management (状态管理)
│   └── Stores/            - Pinia 状态存储
│
├── API Layer (API 层)
│   └── API Client/        - Axios HTTP 客户端
│
└── Routing Layer (路由层)
    └── Vue Router/        - 路由配置
```

**设计原则**:
- **组件化**: 功能拆分为可复用的小组件
- **单向数据流**: 从 Store → Component → User Action → Store
- **懒加载**: 按需加载路由组件降低初始加载时间
- **Composition API**: 使用 `<script setup>` 语法提升开发体验

---

## 项目技术栈

### 核心框架

| 技术 | 版本 | 用途 |
|------|------|------|
| **Vue.js** | 3.5.22 | 核心前端框架 |
| **Vue Router** | 4.4.5 | 路由管理 |
| **Pinia** | 2.2.6 | 状态管理 |
| **TypeScript** | 5.7.3 | 类型安全 |

### UI & 样式

| 技术 | 版本 | 用途 |
|------|------|------|
| **Tailwind CSS** | 3.4.17 | 原子化 CSS 框架 |
| **Lucide Vue Next** | 0.468.0 | 图标库 |

### 构建工具

| 技术 | 版本 | 用途 |
|------|------|------|
| **Vite** | 7.1.11 | 构建工具与开发服务器 |
| **@vitejs/plugin-vue** | 5.2.1 | Vite 的 Vue 插件 |
| **Vue TSC** | 2.2.0 | Vue TypeScript 编译器 |

### 开发工具

| 技术 | 版本 | 用途 |
|------|------|------|
| **ESLint** | 9.19.0 | 代码检查 |
| **Axios** | 1.7.9 | HTTP 客户端 |

---

## 项目模块划分

### 文件与文件夹布局

```
ccr-ui/frontend/
├── public/                           # 静态资源
│   └── favicon.ico
│
├── src/                              # 源代码目录
│   ├── main.ts                       # 应用入口
│   ├── App.vue                       # 根组件
│   │
│   ├── views/                        # 页面组件(路由视图)
│   │   ├── HomeView.vue              # 首页
│   │   ├── ConfigsView.vue           # CCR 配置管理
│   │   ├── CommandsView.vue          # 命令执行器
│   │   ├── ConverterView.vue         # 配置格式转换器
│   │   ├── SyncView.vue              # WebDAV 同步管理
│   │   │
│   │   ├── ClaudeCodeView.vue        # Claude Code 概览
│   │   ├── McpView.vue               # Claude MCP 服务器
│   │   ├── AgentsView.vue            # Claude Agents
│   │   ├── SlashCommandsView.vue     # Claude 斜杠命令
│   │   ├── PluginsView.vue           # Claude 插件
│   │   │
│   │   ├── CodexView.vue             # Codex 概览
│   │   ├── CodexProfilesView.vue     # Codex 配置文件
│   │   ├── CodexMcpView.vue          # Codex MCP 服务器
│   │   ├── CodexAgentsView.vue       # Codex Agents
│   │   ├── CodexSlashCommandsView.vue
│   │   ├── CodexPluginsView.vue
│   │   │
│   │   ├── GeminiCliView.vue         # Gemini CLI 概览
│   │   ├── GeminiMcpView.vue         # Gemini MCP 服务器
│   │   ├── GeminiAgentsView.vue
│   │   ├── GeminiSlashCommandsView.vue
│   │   ├── GeminiPluginsView.vue
│   │   │
│   │   ├── QwenView.vue              # Qwen 概览
│   │   ├── QwenMcpView.vue           # Qwen MCP 服务器
│   │   ├── QwenAgentsView.vue
│   │   ├── QwenSlashCommandsView.vue
│   │   ├── QwenPluginsView.vue
│   │   │
│   │   ├── IflowView.vue             # iFlow 概览
│   │   ├── IflowMcpView.vue          # iFlow MCP 服务器
│   │   └── ... (其他 iFlow 视图)
│   │
│   ├── components/                   # 可复用组件
│   │   ├── Button.vue                # 按钮组件
│   │   ├── Card.vue                  # 卡片组件
│   │   ├── Input.vue                 # 输入框组件
│   │   ├── Table.vue                 # 表格组件
│   │   ├── Navbar.vue                # 导航栏
│   │   ├── MainLayout.vue            # 主布局
│   │   ├── ThemeToggle.vue           # 主题切换
│   │   ├── ConfigCard.vue            # 配置卡片
│   │   ├── DetailField.vue           # 详情字段
│   │   ├── HistoryList.vue           # 历史记录列表
│   │   ├── UpdateModal.vue           # 更新弹窗
│   │   ├── VersionManager.vue        # 版本管理器
│   │   ├── StatusHeader.vue          # 状态头部
│   │   ├── RightSidebar.vue          # 右侧边栏
│   │   └── CollapsibleSidebar.vue    # 可折叠侧边栏
│   │
│   ├── router/                       # 路由配置
│   │   └── index.ts                  # 路由定义
│   │
│   ├── store/                        # 状态管理
│   │   ├── index.ts                  # Store 入口
│   │   └── theme.ts                  # 主题 Store
│   │
│   ├── api/                          # API 客户端
│   │   └── client.ts                 # Axios 客户端配置
│   │
│   ├── types/                        # TypeScript 类型定义
│   │   └── index.ts                  # 类型声明
│   │
│   └── styles/                       # 全局样式
│       └── index.css                 # 全局 CSS (Tailwind)
│
├── index.html                        # HTML 模板
├── package.json                      # NPM 依赖
├── tsconfig.json                     # TypeScript 配置
├── vite.config.ts                    # Vite 配置
├── tailwind.config.js                # Tailwind CSS 配置
├── postcss.config.js                 # PostCSS 配置
└── .gitignore                        # Git 忽略文件
```

### 核心入口点

| 入口文件 | 路径 | 职责 |
|----------|------|------|
| **应用入口** | `/src/main.ts` | 初始化 Vue 应用、Pinia、Router |
| **根组件** | `/src/App.vue` | 根组件，包含 RouterView |
| **路由配置** | `/src/router/index.ts` | 所有路由定义 |
| **API 客户端** | `/src/api/client.ts` | Axios 实例与 API 函数 |
| **主题 Store** | `/src/store/theme.ts` | 主题状态管理 |

---

## 项目业务模块

### 1. CCR 配置管理

**路由**: `/configs`
**视图**: `ConfigsView.vue`

**功能**:
- 列出所有 CCR 配置
- 创建/编辑/删除配置段
- 切换当前配置
- 导入/导出配置
- 查看操作历史

### 2. 命令执行器

**路由**: `/commands`
**视图**: `CommandsView.vue`

**功能**:
- 执行 CCR CLI 命令
- 显示命令输出
- 命令历史记录
- 常用命令快捷按钮

### 3. Claude Code 管理

**路由**: `/claude`, `/mcp`, `/agents`, `/slash-commands`, `/plugins`
**视图**: `ClaudeCodeView.vue`, `McpView.vue`, `AgentsView.vue`, 等

**功能**:
- MCP 服务器管理(列表/添加/编辑/删除/启用)
- Agents 管理
- 斜杠命令管理
- 插件管理
- 配置查看与编辑

### 4. Codex 管理

**路由**: `/codex`, `/codex/profiles`, `/codex/mcp`, 等
**视图**: `CodexView.vue`, `CodexProfilesView.vue`, 等

**功能**:
- Codex 配置文件管理
- MCP 服务器管理
- Agents、斜杠命令、插件管理
- 基础配置编辑

### 5. Gemini CLI / Qwen / iFlow 管理

**路由**: `/gemini-cli/*`, `/qwen/*`, `/iflow/*`
**视图**: 对应平台的视图组件

**功能**:
- MCP 服务器管理
- Agents 管理
- 斜杠命令管理
- 插件管理
- 配置编辑

### 6. 配置转换器

**路由**: `/converter`
**视图**: `ConverterView.vue`

**功能**:
- Claude ↔ Codex 配置转换
- Claude ↔ Gemini 配置转换
- 配置格式验证

### 7. WebDAV 同步

**路由**: `/sync`
**视图**: `SyncView.vue`

**功能**:
- WebDAV 配置设置
- 多文件夹同步管理
- 推送/拉取操作
- 同步状态显示

---

## 项目代码风格与规范

### 命名约定

#### 组件命名
- **文件名**: PascalCase (如 `ConfigCard.vue`, `ThemeToggle.vue`)
- **组件名**: PascalCase (与文件名一致)
- **Props**: camelCase (如 `isActive`, `userName`)
- **Events**: kebab-case (如 `@update-config`, `@delete-item`)

#### 变量与函数命名
- **变量**: camelCase (如 `mcpServers`, `isLoading`)
- **常量**: SCREAMING_SNAKE_CASE (如 `API_BASE_URL`, `MAX_RETRY_COUNT`)
- **函数**: camelCase, 动词开头 (如 `fetchServers()`, `handleSubmit()`)
- **Composables**: use 开头 (如 `useTheme()`, `useApi()`)

#### 类型定义
- **Interface**: PascalCase (如 `McpServer`, `SystemInfo`)
- **Type Alias**: PascalCase (如 `CommandResult`, `ApiResponse`)
- **Enum**: PascalCase (如 `ConversionFormat`)

### 代码风格

#### Vue 组件结构

推荐顺序:
```vue
<script setup lang="ts">
// 1. Imports
import { ref, computed, onMounted } from 'vue'
import type { McpServer } from '@/types'
import { claudeApi } from '@/api/client'

// 2. Props & Emits
interface Props {
  title: string
  data: McpServer[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  update: [value: McpServer]
  delete: [id: string]
}>()

// 3. Reactive state
const loading = ref(false)
const error = ref<string | null>(null)

// 4. Computed properties
const filteredData = computed(() => {
  return props.data.filter(item => item.enabled)
})

// 5. Methods
const handleUpdate = (item: McpServer) => {
  emit('update', item)
}

// 6. Lifecycle hooks
onMounted(() => {
  // 初始化逻辑
})
</script>

<template>
  <!-- 模板内容 -->
</template>

<style scoped>
/* 组件样式(优先使用 Tailwind) */
</style>
```

#### Import 规则

按以下顺序分组导入:
```typescript
// 1. Vue 核心
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useThemeStore } from '@/store/theme'

// 2. 第三方库
import axios from 'axios'

// 3. 类型定义
import type { McpServer, Agent } from '@/types'

// 4. API 客户端
import { claudeApi, codexApi } from '@/api/client'

// 5. 组件
import Button from '@/components/Button.vue'
import Card from '@/components/Card.vue'

// 6. 工具函数
import { formatDate, maskToken } from '@/utils'
```

#### Tailwind CSS 使用规范

- **优先使用 Tailwind 工具类**,减少自定义 CSS
- **组件复用样式**:提取为可复用组件而非重复类名
- **响应式设计**:使用 `sm:`, `md:`, `lg:` 前缀
- **暗黑模式**:使用 `dark:` 前缀

```vue
<button class="
  px-6 py-3
  bg-gradient-to-r from-blue-500 to-purple-600
  hover:from-blue-600 hover:to-purple-700
  dark:from-blue-700 dark:to-purple-800
  text-white font-semibold
  rounded-xl shadow-lg
  transition-all duration-300
  hover:scale-105 active:scale-95
  sm:px-4 sm:py-2
">
  Click Me
</button>
```

#### 异常处理

API 调用统一使用 try-catch:
```typescript
const fetchServers = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await claudeApi.listMcpServers()
    mcpServers.value = response.data
  } catch (err) {
    error.value = err instanceof Error ? err.message : '未知错误'
    console.error('Failed to fetch servers:', err)
  } finally {
    loading.value = false
  }
}
```

#### 参数校验

表单验证示例:
```typescript
const validateForm = () => {
  errors.value = {}

  if (!formData.value.name) {
    errors.value.name = '名称不能为空'
  }

  if (!formData.value.command) {
    errors.value.command = '命令不能为空'
  }

  if (formData.value.name && formData.value.name.length < 3) {
    errors.value.name = '名称至少3个字符'
  }

  return Object.keys(errors.value).length === 0
}
```

### 其他规范

- **注释**: 使用中文注释解释复杂逻辑
- **TypeScript**: 启用严格模式,为所有变量和函数参数添加类型
- **单一职责**: 每个组件只做一件事,保持组件精简
- **Props 验证**: 使用 TypeScript 接口定义 Props
- **避免直接操作 DOM**: 使用 Vue 响应式系统

---

## 测试与质量

### 单元测试

(当前未配置单元测试框架,可扩展使用 Vitest)

**推荐测试框架**:
- **Vitest**: Vue 官方推荐的测试框架
- **@vue/test-utils**: Vue 组件测试工具

### 集成测试

(当前未配置,可扩展使用 Playwright 或 Cypress)

### 代码质量检查

#### TypeScript 类型检查

```bash
# 运行类型检查
npm run type-check

# 输出:编译错误和类型错误
```

#### ESLint 代码检查

```bash
# 运行 ESLint
npm run lint

# 自动修复问题
npm run lint -- --fix
```

#### 构建验证

```bash
# 生产构建(验证无构建错误)
npm run build

# 预览构建结果
npm run preview
```

### 质量目标

- ✅ **零 TypeScript 错误**: 所有代码通过类型检查
- ✅ **零 ESLint 警告**: 代码符合 ESLint 规则
- ✅ **成功构建**: 生产构建无错误
- 🚧 **单元测试覆盖率**: (待配置) 目标 80%+
- 🚧 **E2E 测试**: (待配置) 覆盖关键用户流程

---

## 项目构建、测试与运行

### 环境与配置

#### 环境要求

- **Node.js**: 18.x 或更高版本
- **npm**: 9.x 或更高版本(或 yarn/pnpm)

#### 环境变量

**.env.development** (开发环境):
```bash
VITE_API_BASE_URL=http://localhost:8081
VITE_APP_TITLE=CCR UI
```

**.env.production** (生产环境):
```bash
VITE_API_BASE_URL=/api
VITE_APP_TITLE=CCR UI
```

### 开发命令

```bash
# 安装依赖
npm install

# 启动开发服务器(端口 3000)
npm run dev

# 构建生产版本
npm run build

# 预览生产构建
npm run preview

# 类型检查
npm run type-check

# 代码检查
npm run lint

# 格式化代码
npm run format
```

### 构建流程

**开发模式**:
```bash
cd ccr-ui/frontend
npm run dev

# Vite 启动开发服务器
# ➜  Local:   http://localhost:3000/
# ➜  Network: use --host to expose
```

**生产构建**:
```bash
npm run build

# 输出到 dist/ 目录:
# dist/
# ├── index.html
# ├── assets/
# │   ├── index-[hash].js
# │   ├── index-[hash].css
# │   └── ...
# └── favicon.ico
```

### 部署指南

#### 本地部署

```bash
# 构建
npm run build

# 预览构建结果
npm run preview
```

#### 生产部署

1. **构建前端**:
   ```bash
   cd ccr-ui/frontend
   npm run build
   ```

2. **部署 dist/ 目录**到静态文件服务器:
   - Nginx
   - Apache
   - Vercel
   - Netlify

3. **配置反向代理**(如果 API 在不同端口):
   ```nginx
   location /api {
       proxy_pass http://localhost:8081;
       proxy_set_header Host $host;
       proxy_set_header X-Real-IP $remote_addr;
   }
   ```

---

## Git 工作流程

### 分支策略

- **main**: 主分支,生产环境代码
- **dev**: 开发分支,测试环境代码
- **feature/***: 功能分支
- **bugfix/***: Bug 修复分支

### 提交规范

遵循 Conventional Commits 规范:

```bash
# 功能开发
git commit -m "feat(前端): 添加 Codex 配置管理页面"

# Bug 修复
git commit -m "fix(前端): 修复暗黑模式下按钮颜色问题"

# 样式调整
git commit -m "style(前端): 优化 Liquid Glass 卡片样式"

# 重构
git commit -m "refactor(前端): 重构 API 客户端使用 Composables"

# 文档更新
git commit -m "docs(前端): 更新 README 添加部署说明"

# 性能优化
git commit -m "perf(前端): 实现路由懒加载降低初始加载时间"
```

### PR 流程

1. 从 `dev` 分支创建功能分支
2. 开发并提交代码
3. 推送到远程仓库
4. 创建 Pull Request
5. Code Review
6. 合并到 `dev` 分支
7. 测试通过后合并到 `main`

---

## 文档目录(重要)

### 文档存储规范

- **模块文档**: `/ccr-ui/frontend/CLAUDE.md` (本文件)
- **上级文档**: `/ccr-ui/CLAUDE.md` (CCR UI 总览)
- **根文档**: `/CLAUDE.md` (项目总览)
- **API 文档**: `/ccr-ui/backend/CLAUDE.md` (后端 API 文档)

### 相关文件列表

#### 源代码
- `/ccr-ui/frontend/src/main.ts` - 应用入口
- `/ccr-ui/frontend/src/App.vue` - 根组件
- `/ccr-ui/frontend/src/views/` - 页面组件(40+ 文件)
- `/ccr-ui/frontend/src/components/` - 可复用组件(15+ 文件)
- `/ccr-ui/frontend/src/router/index.ts` - 路由配置
- `/ccr-ui/frontend/src/store/` - Pinia 状态存储
- `/ccr-ui/frontend/src/api/client.ts` - API 客户端
- `/ccr-ui/frontend/src/types/index.ts` - TypeScript 类型

#### 配置文件
- `/ccr-ui/frontend/package.json` - NPM 依赖
- `/ccr-ui/frontend/vite.config.ts` - Vite 配置
- `/ccr-ui/frontend/tsconfig.json` - TypeScript 配置
- `/ccr-ui/frontend/tailwind.config.js` - Tailwind CSS 配置
- `/ccr-ui/frontend/.gitignore` - Git 忽略规则

#### 构建输出
- `/ccr-ui/frontend/dist/` - 生产构建输出(被忽略)
- `/ccr-ui/frontend/node_modules/` - 依赖包(被忽略)

### 外部链接

- **Vue.js 文档**: https://vuejs.org/
- **Vite 文档**: https://vitejs.dev/
- **Tailwind CSS 文档**: https://tailwindcss.com/
- **Pinia 文档**: https://pinia.vuejs.org/
- **TypeScript 文档**: https://www.typescriptlang.org/

---

## 常见问题(FAQ)

### Q: 如何添加新的页面视图?

A:
1. 在 `src/views/` 创建新的 Vue 组件(如 `NewView.vue`)
2. 在 `src/router/index.ts` 添加路由:
   ```typescript
   { path: '/new', component: () => import('@/views/NewView.vue') }
   ```
3. 在 `src/components/Navbar.vue` 添加导航链接

### Q: 如何调用后端 API?

A:
```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { claudeApi } from '@/api/client'

const mcpServers = ref([])
const loading = ref(false)

const fetchServers = async () => {
  loading.value = true
  try {
    const response = await claudeApi.listMcpServers()
    mcpServers.value = response.data
  } catch (error) {
    console.error('API 调用失败:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchServers()
})
</script>
```

### Q: 如何使用 Pinia Store?

A:
```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useThemeStore } from '@/store/theme'

const themeStore = useThemeStore()

// 访问状态
const isDark = computed(() => themeStore.isDark)

// 调用 actions
const toggleTheme = () => {
  themeStore.toggleTheme()
}
</script>
```

### Q: 如何自定义 Liquid Glass 主题?

A: 编辑 `tailwind.config.js`:
```javascript
theme: {
  extend: {
    colors: {
      primary: {
        50: '#f0f9ff',
        // ... 自定义颜色
      },
    },
    backdropBlur: {
      xs: '2px',
    },
  },
}
```

Liquid Glass 效果使用:
- `backdrop-blur` - 背景模糊
- `bg-white/10` - 半透明白色背景
- `border-white/20` - 半透明边框
- `rounded-2xl` - 圆角
- `shadow-xl` - 阴影

### Q: 开发环境端口冲突怎么办?

A: 修改 `vite.config.ts`:
```typescript
server: {
  port: 3001, // 改为其他端口
}
```

或启动时指定端口:
```bash
npm run dev -- --port 3001
```

### Q: 如何处理表单验证?

A: 参考"代码风格与规范 → 参数校验"章节的表单验证示例。

---

**本小姐精心整理的前端模块文档就到这里啦！这可是贵族级别的文档标准哦～(￣▽￣)／**
