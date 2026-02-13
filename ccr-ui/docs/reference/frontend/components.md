# 前端组件文档

CCR UI 前端基于 **Vue 3.5 + Vite 7.1** 构建，采用 Composition API 和 TypeScript 开发。组件库结合了原子化设计（Atomic Design）思想，分为基础 UI 组件、业务功能组件和布局组件。

## 🏗️ 组件架构

### 目录结构

```
src/components/
├── ui/                 # 基础 UI 组件 (原子级)
│   ├── Button.vue
│   ├── Input.vue
│   ├── Card.vue
│   ├── Badge.vue
│   └── ...
├── common/             # 通用业务组件
│   ├── TerminalOutput.vue
│   ├── LoadingOverlay.vue
│   └── ...
├── layout/             # 布局组件
│   ├── Navbar.vue
│   ├── Sidebar.vue
│   └── ...
└── features/           # 特定功能组件
    ├── ConfigCard.vue
    ├── McpSyncPanel.vue
    └── ...
```

## 🎨 基础 UI 组件

### Button 组件

**文件**: `src/components/ui/Button.vue`

通用按钮组件，支持多种变体和尺寸。

```vue
<template>
  <button 
    :class="[
      'inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50',
      variants[variant],
      sizes[size],
      className
    ]"
    :disabled="disabled || loading"
    v-bind="$attrs"
  >
    <Spinner v-if="loading" class="mr-2 h-4 w-4" />
    <slot />
  </button>
</template>

<script setup lang="ts">
import Spinner from './Spinner.vue'

interface Props {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive'
  size?: 'sm' | 'md' | 'lg' | 'icon'
  loading?: boolean
  disabled?: boolean
  className?: string
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  loading: false,
  disabled: false
})

// ... variants and sizes definitions in script
</script>
```

**使用示例**:
```vue
<Button variant="primary" @click="saveConfig">保存配置</Button>
<Button variant="outline" :loading="isSyncing">同步</Button>
```

### Card 组件

**文件**: `src/components/ui/Card.vue`

基础卡片容器，提供一致的背景、边框和阴影。

```vue
<template>
  <div :class="cn('rounded-lg border bg-card text-card-foreground shadow-sm', props.class)">
    <div v-if="$slots.header || title" class="flex flex-col space-y-1.5 p-6">
      <h3 v-if="title" class="text-2xl font-semibold leading-none tracking-tight">{{ title }}</h3>
      <p v-if="description" class="text-sm text-muted-foreground">{{ description }}</p>
      <slot name="header" />
    </div>
    <div :class="cn('p-6 pt-0', contentClass)">
      <slot />
    </div>
    <div v-if="$slots.footer" class="flex items-center p-6 pt-0">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { cn } from '@/lib/utils'

interface Props {
  title?: string
  description?: string
  class?: string
  contentClass?: string
}

const props = defineProps<Props>()
</script>
```

## 🚀 业务功能组件

### ConfigCard (配置卡片)

**文件**: `src/components/ConfigCard.vue`

展示当个配置项的详细信息，包含状态指示器和常用操作。

**主要功能**:
- 显示配置名称、模型、提供商
- 状态徽章 (当前使用、默认)
- 快捷操作: 切换、编辑、删除
- 标签展示

### McpSyncPanel (WebDAV 同步面板)

**文件**: `src/components/McpSyncPanel.vue`

管理 WebDAV 同步状态和操作。

**功能**:
- 显示上次同步时间
- 触发全量 Push/Pull
- 查看同步日志
- 包含 `TerminalOutput` 组件显示同步过程

### TerminalOutput (终端输出)

**文件**: `src/components/common/TerminalOutput.vue`

模拟终端界面展示命令执行结果或日志。

**特性**:
- 支持 ANSI 颜色代码渲染
- 自动滚动到底部
- 复制输出内容
- 黑色背景高亮显示

```vue
<template>
  <div class="relative bg-zinc-950 rounded-lg overflow-hidden border border-zinc-800">
    <div class="flex justify-between items-center px-4 py-2 bg-zinc-900 border-b border-zinc-800">
      <span class="text-xs text-zinc-400">Terminal</span>
      <button @click="copyOutput" class="text-xs hover:text-white">Copy</button>
    </div>
    <pre class="p-4 overflow-x-auto text-sm font-mono text-zinc-300 max-h-[400px] overflow-y-auto custom-scrollbar">
      <code v-html="renderedOutput"></code>
    </pre>
  </div>
</template>
```

### ActivityHeatmap (活动热力图)

**文件**: `src/components/ActivityHeatmap.vue`

类似于 GitHub 的贡献图，可视化展示用户在不同日期的命令执行频率和令牌消耗。

## 📊 图表组件

基于 Chart.js 或 ECharts 封装的统计图表。

- **UsageStatsChart.vue**: 令牌和成本使用趋势图
- **TokenUsageChart.vue**: 模型使用分布饼图

## 🏗️ 布局组件

### MainLayout

**文件**: `src/components/MainLayout.vue`

应用的主要布局结构。

- 顶部 **Navbar**: 全局搜索、主题切换、通知
- 左侧 **Sidebar**: 导航菜单 (可折叠)
- 主内容区域: `<router-view>`
- 底部 **StatusHeader**: 后端连接状态

### 交互反馈

- **LoadingOverlay.vue**: 全局加载遮罩
- **ToastContainer.vue**: 全局消息通知容器
- **ConfirmModal.vue**: 危险操作确认对话框

## 🛠️ 技能管理组件 (v4.0+)

技能管理模块采用页面组件 + 业务组件 + Composable 架构，支持懒加载优化。

> 📖 **功能指南**：[技能管理详细指南](/guide/skills)

### 页面组件

#### UnifiedSkillsView (技能管理主页)

**文件**: `src/views/skills/UnifiedSkillsView.vue`

技能管理的核心页面，采用两栏布局。

**功能**:
- 左侧筛选面板（平台、来源、分类、标签过滤）
- 右侧主内容区（统计卡片 + 三标签页切换）
- 已安装 / 市场 / 仓库三个标签页
- 移动端适配（侧滑抽屉式筛选）

**依赖 Composable**: `useUnifiedSkills`

```vue
<UnifiedSkillsView />
<!-- 路由: /skills -->
```

#### AddSkillView (添加技能页面)

**文件**: `src/views/skills/AddSkillView.vue`

提供市场浏览和手动多源安装两大区域。

**功能**:
- 市场热门浏览（搜索、排序、分页、批量选择）
- 手动安装（GitHub URL / 本地文件夹 / npx 三种来源标签页切换）
- 目标平台选择器（自动检测、快捷选择）
- 安装进度 Toast 反馈

```vue
<AddSkillView />
<!-- 路由: /skills/add -->
```

### 业务组件

#### SkillsFilterPanel (筛选面板)

**文件**: `src/components/skills/SkillsFilterPanel.vue`

桌面端左侧固定筛选面板，支持折叠。

**Props**:

| Prop | 类型 | 说明 |
|------|------|------|
| `modelValue` | `SkillFilters` | 筛选条件（v-model） |
| `platforms` | `PlatformSummary[]` | 平台列表 |
| `categories` | `string[]` | 可用分类 |
| `tags` | `string[]` | 可用标签 |
| `collapsed` | `boolean` | 是否折叠 |

#### SkillsStatsCards (统计卡片)

**文件**: `src/components/skills/SkillsStatsCards.vue`

展示已安装数量、市场可用数和活跃平台信息。

**Props**:

| Prop | 类型 | 说明 |
|------|------|------|
| `stats` | `SkillsStats` | 统计数据 |
| `platforms` | `PlatformSummary[]` | 平台列表 |
| `cached` | `boolean` | 市场数据是否缓存 |
| `activePlatform` | `Platform \| 'all'` | 当前选中平台 |

#### SkillsInstalledTab (已安装标签页)

**文件**: `src/components/skills/SkillsInstalledTab.vue`

已安装技能的列表视图，支持查看、编辑、删除操作。

**Events**: `edit`, `delete`, `click`

#### SkillsMarketplaceTab (市场标签页)

**文件**: `src/components/skills/SkillsMarketplaceTab.vue` _(懒加载)_

市场浏览、搜索和批量安装功能。

**Events**: `install`, `search`, `batch-install`

#### MarketplaceSkillCard (市场技能卡片)

**文件**: `src/components/skills/MarketplaceSkillCard.vue`

单个市场技能展示卡片，显示所有者、描述、星标、安装按钮。

**Props**:

| Prop | 类型 | 说明 |
|------|------|------|
| `item` | `MarketplaceItem` | 市场技能数据 |
| `isInstalled` | `boolean` | 是否已安装 |
| `isInstalling` | `boolean` | 是否安装中 |
| `batchMode` | `boolean` | 批量模式 |
| `isSelected` | `boolean` | 批量模式下是否选中 |

#### MarketplacePagination (市场分页)

**文件**: `src/components/skills/MarketplacePagination.vue`

市场列表分页组件。

### 模态框组件（均为懒加载）

| 组件 | 文件 | 说明 |
|------|------|------|
| `SkillInstallModal` | `SkillInstallModal.vue` | 安装确认模态框，选择目标平台 |
| `SkillDetailModal` | `SkillDetailModal.vue` | 技能详情查看和编辑模态框 |
| `SkillDeleteConfirmModal` | `SkillDeleteConfirmModal.vue` | 删除确认对话框 |
| `SkillOperationLogModal` | `SkillOperationLogModal.vue` | 操作日志查看模态框 |

### 反馈组件

#### SkillInstallToast (安装进度提示)

**文件**: `src/components/skills/SkillInstallToast.vue`

浮动 Toast 组件，实时显示安装进度和状态。

**Props**:

| Prop | 类型 | 说明 |
|------|------|------|
| `progress` | `InstallProgress \| null` | 安装进度状态 |

**进度阶段**: `idle` → `downloading` → `installing` → `done` / `error`

### Composable

#### useUnifiedSkills

**文件**: `src/composables/useUnifiedSkills.ts`

统一技能管理的核心状态和方法。

**提供的状态**:
- `platforms` — 平台列表
- `skills` / `filteredSkills` — 已安装技能（含筛选）
- `marketplaceItems` — 市场技能列表
- `filters` / `activeTab` — 筛选和标签页状态
- `stats` — 统计数据
- `installProgress` — 安装进度
- `npxStatus` — npx 可用性状态

**提供的方法**:
- `initialize()` / `refresh()` — 初始化和刷新
- `installSkill()` / `removeSkill()` — 安装/卸载
- `importFromGithub()` / `importFromLocal()` / `importViaNpx()` — 多源导入
- `batchInstall()` — 批量安装
- `fetchMarketplaceTrending()` / `searchMarketplace()` — 市场操作
- `checkNpxStatus()` / `browseFolder()` — 工具方法
