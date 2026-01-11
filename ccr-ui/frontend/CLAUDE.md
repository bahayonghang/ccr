# CCR UI Frontend 模块指导文件

[根目录](../../CLAUDE.md) > [ccr-ui](../CLAUDE.md) > **frontend**

## Change Log
- **2026-01-11**: 添加设计系统文档 (Neo-Terminal Design System)
- **2025-12-17**: 激进精简到 300 行以内，只保留核心架构和技术栈
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

## 设计系统 (Neo-Terminal Design System)

### 设计令牌 (Design Tokens)

设计系统基于 CSS 变量实现，支持暗色/亮色主题自动切换。

**核心文件**: `src/styles/tokens.css`

| 类别 | 变量前缀 | 示例 |
|------|----------|------|
| **背景色** | `--color-bg-*` | `--color-bg-base`, `--color-bg-elevated` |
| **文字色** | `--color-text-*` | `--color-text-primary`, `--color-text-muted` |
| **边框色** | `--color-border-*` | `--color-border-default`, `--color-border-accent` |
| **强调色** | `--color-accent-*` | `--color-accent-primary`, `--color-accent-secondary` |
| **功能色** | `--color-success/warning/danger/info` | 状态反馈色 |
| **间距** | `--space-*` | `--space-1` (4px) 到 `--space-32` (128px) |
| **圆角** | `--radius-*` | `--radius-sm` (4px) 到 `--radius-full` |
| **阴影** | `--shadow-*` | `--shadow-sm` 到 `--shadow-2xl` |
| **玻璃效果** | `--glass-*` | `--glass-bg-medium`, `--glass-blur-lg` |
| **动画** | `--duration-*`, `--ease-*` | 时长和缓动函数 |

### 状态组件

| 组件 | 路径 | 用途 |
|------|------|------|
| **Skeleton** | `components/common/Skeleton.vue` | 加载骨架屏 |
| **SkeletonCard** | `components/common/SkeletonCard.vue` | 卡片骨架屏 |
| **ErrorState** | `components/common/ErrorState.vue` | 错误状态展示 |
| **LoadingOverlay** | `components/common/LoadingOverlay.vue` | 加载遮罩层 |
| **EmptyState** | `components/common/EmptyState.vue` | 空状态展示 |

### 无障碍 (Accessibility)

**Composable**: `src/composables/useAccessibility.ts`

| 功能 | API | 说明 |
|------|-----|------|
| **焦点陷阱** | `useFocusTrap()` | 模态框焦点限制 |
| **Escape 关闭** | `useEscapeKey()` | 按 Esc 关闭组件 |
| **ARIA 工具** | `ariaUtils.*` | 生成 ARIA 属性 |
| **焦点管理** | `focusUtils.*` | 焦点保存/恢复 |
| **唯一 ID** | `useUniqueId()` | 生成 ARIA 关联 ID |

**无障碍规范**:
- 所有交互元素添加 `aria-label`
- 装饰性图标添加 `aria-hidden="true"`
- 列表使用 `role="listbox"` + `role="option"`
- 模态框使用焦点陷阱和 Escape 关闭

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
│   ├── views/                        # 页面组件 (40+ 路由视图)
│   │   ├── HomeView.vue              # 首页
│   │   ├── ConfigsView.vue           # CCR 配置管理
│   │   ├── CommandsView.vue          # 命令执行器
│   │   ├── ConverterView.vue         # 配置格式转换器
│   │   ├── SyncView.vue              # WebDAV 同步管理
│   │   ├── ClaudeCodeView.vue        # Claude Code 概览
│   │   ├── CodexView.vue             # Codex 概览
│   │   ├── GeminiCliView.vue         # Gemini CLI 概览
│   │   └── ... (其他平台视图)
│   │
│   ├── components/                   # 可复用组件 (15+)
│   │   ├── Button.vue                # 按钮组件
│   │   ├── Card.vue                  # 卡片组件
│   │   ├── Navbar.vue                # 导航栏
│   │   ├── MainLayout.vue            # 主布局
│   │   ├── ThemeToggle.vue           # 主题切换
│   │   └── ... (其他组件)
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

## 项目代码风格与规范

### 命名约定

#### 组件与文件命名
- **组件文件名**: `PascalCase` (如 `ConfigCard.vue`, `ThemeToggle.vue`)
- **组件名**: `PascalCase` (与文件名一致)
- **Props**: `camelCase` (如 `isActive`, `userName`)
- **Events**: `kebab-case` (如 `@update-config`, `@delete-item`)

#### 变量与函数命名
- **变量**: `camelCase` (如 `mcpServers`, `isLoading`)
- **常量**: `SCREAMING_SNAKE_CASE` (如 `API_BASE_URL`, `MAX_RETRY_COUNT`)
- **函数**: `camelCase`, 动词开头 (如 `fetchServers()`, `handleSubmit()`)
- **Composables**: `use` 开头 (如 `useTheme()`, `useApi()`)

#### 类型定义
- **Interface**: `PascalCase` (如 `McpServer`, `SystemInfo`)
- **Type Alias**: `PascalCase` (如 `CommandResult`, `ApiResponse`)
- **Enum**: `PascalCase` (如 `ConversionFormat`)

### 代码风格要点

- **组件结构**: Imports → Props/Emits → Reactive state → Computed → Methods → Lifecycle hooks
- **Composition API**: 使用 `<script setup lang="ts">` 语法
- **TypeScript**: 启用严格模式,为所有变量和函数参数添加类型
- **Tailwind CSS**: 优先使用工具类,减少自定义 CSS
- **响应式设计**: 使用 `sm:`, `md:`, `lg:` 前缀
- **暗黑模式**: 使用 `dark:` 前缀
- **注释**: 使用中文注释解释复杂逻辑
- **单一职责**: 每个组件只做一件事,保持组件精简

---

## 测试与质量

### 代码质量检查

```bash
# TypeScript 类型检查
npm run type-check

# ESLint 代码检查
npm run lint

# 自动修复问题
npm run lint -- --fix

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

---

## 项目构建、测试与运行

### 环境要求

- **Node.js**: 18.x 或更高版本
- **npm**: 9.x 或更高版本(或 yarn/pnpm)

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

### 环境变量

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

### 生产构建

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
```

---

## 文档目录

### 文档存储规范

- **模块文档**: `/ccr-ui/frontend/CLAUDE.md` (本文件)
- **上级文档**: `/ccr-ui/CLAUDE.md` (CCR UI 总览)
- **根文档**: `/CLAUDE.md` (项目总览)
- **后端文档**: `/ccr-ui/backend/CLAUDE.md` (后端 API 文档)

---

## 常见问题

### Q: 如何添加新的页面视图?

A:
1. 在 `src/views/` 创建新的 Vue 组件(如 `NewView.vue`)
2. 在 `src/router/index.ts` 添加路由
3. 在 `src/components/Navbar.vue` 添加导航链接

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

---

**本小姐精心整理的前端模块文档完成啦！Vue.js 3 + Vite + Tailwind CSS 的完美组合，这才是现代化前端的标准呢～(￣▽￣)／**
