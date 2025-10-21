<template>
  <aside
    class="rounded-xl p-4 h-fit sticky top-5 transition-all duration-300 glass-effect"
    :class="collapsed ? 'w-16' : 'w-64'"
    :style="{
      border: '1px solid var(--border-color)',
      boxShadow: 'var(--shadow-small)'
    }"
  >
    <!-- 切换按钮 -->
    <div class="flex items-center justify-between mb-4">
      <div
        v-if="!collapsed"
        class="text-xs font-semibold uppercase tracking-wider"
        :style="{ color: 'var(--text-secondary)' }"
      >
        导航菜单
      </div>
      <button
        class="p-2 rounded-lg transition-all hover:scale-110"
        :style="{
          background: 'var(--bg-tertiary)',
          border: '1px solid var(--border-color)',
          color: 'var(--text-secondary)'
        }"
        :title="collapsed ? '展开菜单' : '收起菜单'"
        :aria-label="collapsed ? '展开菜单' : '收起菜单'"
        @click="toggleCollapsed"
      >
        <ChevronRight v-if="collapsed" class="w-4 h-4" />
        <ChevronLeft v-else class="w-4 h-4" />
      </button>
    </div>

    <!-- 导航链接 - 层级菜单 -->
    <nav class="space-y-2" aria-label="主导航">
      <div
        v-for="(group, groupIndex) in navigationGroups"
        :key="group.title"
        class="space-y-1"
      >
        <!-- 分隔线（折叠状态且非第一个分组） -->
        <div
          v-if="collapsed && groupIndex > 0"
          class="h-px mx-2 my-2"
          :style="{ background: 'var(--border-color)' }"
          aria-hidden="true"
        />

        <!-- 分组头部 -->
        <button
          class="w-full flex items-center rounded-lg transition-all hover:scale-[1.02]"
          :class="collapsed ? 'justify-center' : 'justify-between'"
          :style="{
            padding: collapsed ? '12px' : '12px 16px',
            background: hasActiveChild(group.items)
              ? 'linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(168, 85, 247, 0.2))'
              : 'var(--bg-tertiary)',
            border: `1px solid ${hasActiveChild(group.items) ? 'var(--accent-primary)' : 'var(--border-color)'}`,
            color: hasActiveChild(group.items) ? 'var(--accent-primary)' : 'var(--text-primary)'
          }"
          :title="collapsed ? group.title : undefined"
          :aria-expanded="!collapsed && expandedGroups[group.title]"
          :aria-label="`${group.title} 菜单组`"
          @click="!collapsed && toggleGroup(group.title)"
        >
          <div class="flex items-center" :class="collapsed ? '' : 'space-x-3'">
            <component :is="group.icon" class="w-5 h-5 flex-shrink-0" />
            <span v-if="!collapsed" class="font-semibold text-sm">{{ group.title }}</span>
          </div>
          <ChevronUp
            v-if="!collapsed && expandedGroups[group.title]"
            class="w-4 h-4"
          />
          <ChevronDown
            v-else-if="!collapsed"
            class="w-4 h-4"
          />
        </button>

        <!-- 子菜单项 - 仅在展开状态且非折叠时显示 -->
        <div
          v-if="!collapsed && expandedGroups[group.title]"
          class="ml-2 space-y-0 border-l-2"
          :style="{ borderColor: 'var(--border-color)' }"
        >
          <RouterLink
            v-for="(item, itemIndex) in group.items"
            :key="item.href"
            :to="item.href"
            class="flex items-center space-x-3 px-4 py-3 ml-2 rounded-lg transition-all duration-300 relative overflow-hidden group"
            :class="isActive(item.href) ? 'scale-[1.02]' : 'hover:translate-x-1'"
            :style="{
              marginTop: itemIndex > 0 ? '4px' : '0',
              marginBottom: '4px',
              background: isActive(item.href)
                ? 'linear-gradient(135deg, rgba(139, 92, 246, 0.95), rgba(168, 85, 247, 0.95))'
                : 'rgba(255, 255, 255, 0.05)',
              backdropFilter: 'blur(10px)',
              border: isActive(item.href)
                ? '2px solid rgba(139, 92, 246, 0.8)'
                : '1px solid rgba(139, 92, 246, 0.2)',
              boxShadow: isActive(item.href)
                ? '0 4px 20px rgba(139, 92, 246, 0.5), 0 0 30px rgba(168, 85, 247, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2)'
                : '0 2px 8px rgba(0, 0, 0, 0.1)',
              color: isActive(item.href) ? 'white' : 'var(--text-secondary)',
              fontWeight: isActive(item.href) ? '600' : '500',
              borderBottom: itemIndex < group.items.length - 1 && !isActive(item.href)
                ? '1px solid rgba(139, 92, 246, 0.1)'
                : undefined
            }"
            :aria-current="isActive(item.href) ? 'page' : undefined"
          >
            <!-- 液态玻璃效果叠加层 -->
            <span
              v-if="isActive(item.href)"
              class="absolute inset-0 rounded-lg pointer-events-none"
              :style="{
                background: 'linear-gradient(135deg, rgba(255, 255, 255, 0.15) 0%, rgba(255, 255, 255, 0.05) 50%, rgba(255, 255, 255, 0.15) 100%)',
                animation: 'shimmer 3s infinite'
              }"
              aria-hidden="true"
            />

            <!-- 左侧发光指示器 -->
            <span
              class="absolute left-0 top-0 w-1 h-full transition-all duration-300"
              :class="isActive(item.href) ? 'scale-y-100 opacity-100' : 'scale-y-0 opacity-0 group-hover:scale-y-75 group-hover:opacity-50'"
              :style="{
                background: isActive(item.href)
                  ? 'linear-gradient(to bottom, #8b5cf6, #a855f7, #c084fc)'
                  : 'rgba(139, 92, 246, 0.6)',
                boxShadow: isActive(item.href) ? '0 0 10px #8b5cf6' : undefined
              }"
              aria-hidden="true"
            />

            <!-- 图标 - 带动画 -->
            <component
              :is="item.icon"
              class="w-5 h-5 flex-shrink-0 transition-transform duration-300"
              :class="isActive(item.href) ? 'scale-110' : 'group-hover:scale-105'"
              :style="{
                filter: isActive(item.href) ? 'drop-shadow(0 0 4px rgba(255, 255, 255, 0.5))' : undefined
              }"
            />
            <span
              class="font-medium text-sm transition-all duration-300"
              :class="isActive(item.href) ? 'tracking-wide' : ''"
            >
              {{ item.name }}
            </span>

            <!-- 激活状态右侧光点 -->
            <span
              v-if="isActive(item.href)"
              class="ml-auto w-2 h-2 rounded-full animate-pulse"
              :style="{
                background: 'white',
                boxShadow: '0 0 8px white'
              }"
              aria-hidden="true"
            />
          </RouterLink>
        </div>

        <!-- 折叠状态下，显示子菜单作为独立项 -->
        <div v-if="collapsed" class="space-y-1">
          <RouterLink
            v-for="(item, itemIndex) in group.items"
            :key="item.href"
            :to="item.href"
            class="flex items-center justify-center px-4 py-3 rounded-lg transition-all duration-300 relative overflow-hidden group"
            :class="isActive(item.href) ? 'scale-110' : 'hover:scale-105'"
            :style="{
              background: isActive(item.href)
                ? 'linear-gradient(135deg, rgba(139, 92, 246, 0.95), rgba(168, 85, 247, 0.95))'
                : 'rgba(255, 255, 255, 0.05)',
              backdropFilter: 'blur(10px)',
              border: isActive(item.href)
                ? '2px solid rgba(139, 92, 246, 0.8)'
                : '1px solid rgba(139, 92, 246, 0.2)',
              boxShadow: isActive(item.href)
                ? '0 4px 20px rgba(139, 92, 246, 0.5), 0 0 30px rgba(168, 85, 247, 0.3)'
                : undefined,
              color: isActive(item.href) ? 'white' : 'var(--text-secondary)',
              marginTop: itemIndex > 0 ? '4px' : '0',
              borderBottom: itemIndex < group.items.length - 1 && !isActive(item.href)
                ? '1px solid rgba(139, 92, 246, 0.15)'
                : undefined
            }"
            :title="item.name"
            :aria-current="isActive(item.href) ? 'page' : undefined"
          >
            <!-- 液态玻璃效果 -->
            <span
              v-if="isActive(item.href)"
              class="absolute inset-0 rounded-lg pointer-events-none"
              :style="{
                background: 'linear-gradient(135deg, rgba(255, 255, 255, 0.15) 0%, rgba(255, 255, 255, 0.05) 50%, rgba(255, 255, 255, 0.15) 100%)',
                animation: 'shimmer 3s infinite'
              }"
              aria-hidden="true"
            />
            
            <component
              :is="item.icon"
              class="w-5 h-5 flex-shrink-0 transition-transform duration-300"
              :class="isActive(item.href) ? 'scale-110' : ''"
              :style="{
                filter: isActive(item.href) ? 'drop-shadow(0 0 4px rgba(255, 255, 255, 0.5))' : undefined
              }"
            />

            <!-- 激活指示器 -->
            <span
              v-if="isActive(item.href)"
              class="absolute bottom-0 left-1/2 transform -translate-x-1/2 w-1/2 h-0.5 animate-pulse"
              :style="{
                background: 'white',
                boxShadow: '0 0 8px white'
              }"
              aria-hidden="true"
            />
          </RouterLink>
        </div>
      </div>
    </nav>

    <!-- 收起状态提示 -->
    <div v-if="collapsed" class="mt-4 text-center">
      <button
        class="p-2 rounded-lg transition-all hover:scale-110"
        :style="{
          background: 'var(--bg-tertiary)',
          border: '1px solid var(--border-color)',
          color: 'var(--text-muted)'
        }"
        title="展开菜单"
        aria-label="展开菜单"
        @click="toggleCollapsed"
      >
        <Menu class="w-4 h-4" />
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import {
  Settings,
  Terminal,
  ChevronLeft,
  ChevronRight,
  Menu,
  ChevronDown,
  ChevronUp,
  Zap,
  Server,
  Command,
  Bot,
  Puzzle,
  Sparkles,
  Gem,
  Workflow,
  Cloud,
  Code2,
  ArrowLeftRight
} from 'lucide-vue-next'

interface NavItem {
  name: string
  href: string
  icon: any
}

interface NavGroup {
  title: string
  icon: any
  defaultExpanded: boolean
  items: NavItem[]
}

const route = useRoute()
const collapsed = ref(false)
const expandedGroups = reactive<Record<string, boolean>>({})

// 导航菜单结构
const navigationGroups: NavGroup[] = [
  {
    title: 'CCR 命令执行',
    icon: Terminal,
    defaultExpanded: false,
    items: [
      { name: 'CCR 命令', href: '/commands/ccr', icon: Zap },
      { name: 'Claude Code 命令', href: '/commands/claude-code', icon: Zap },
      { name: 'Claude 命令', href: '/commands/claude', icon: Zap },
      { name: 'Qwen 命令', href: '/commands/qwen', icon: Sparkles },
      { name: 'Gemini 命令', href: '/commands/gemini', icon: Gem },
      { name: 'IFLOW 命令', href: '/commands/iflow', icon: Workflow }
    ]
  },
  {
    title: 'Claude Code',
    icon: Zap,
    defaultExpanded: true,
    items: [
      { name: '配置管理', href: '/configs', icon: Settings },
      { name: '☁️ 云同步', href: '/sync', icon: Cloud },
      { name: 'MCP 管理', href: '/mcp', icon: Server },
      { name: 'Slash Commands', href: '/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/agents', icon: Bot },
      { name: '插件管理', href: '/plugins', icon: Puzzle }
    ]
  },
  {
    title: 'Qwen',
    icon: Sparkles,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/qwen/mcp', icon: Server },
      { name: 'Slash Commands', href: '/qwen/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/qwen/agents', icon: Bot },
      { name: '插件管理', href: '/qwen/plugins', icon: Puzzle }
    ]
  },
  {
    title: 'Gemini Cli',
    icon: Gem,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/gemini-cli/mcp', icon: Server },
      { name: 'Slash Commands', href: '/gemini-cli/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/gemini-cli/agents', icon: Bot },
      { name: '插件管理', href: '/gemini-cli/plugins', icon: Puzzle }
    ]
  },
  {
    title: 'IFLOW',
    icon: Workflow,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/iflow/mcp', icon: Server },
      { name: 'Slash Commands', href: '/iflow/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/iflow/agents', icon: Bot },
      { name: '插件管理', href: '/iflow/plugins', icon: Puzzle }
    ]
  },
  {
    title: 'Codex',
    icon: Code2,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/codex/mcp', icon: Server },
      { name: 'Profile 管理', href: '/codex/profiles', icon: Settings }
    ]
  },
  {
    title: '配置转换器',
    icon: ArrowLeftRight,
    defaultExpanded: false,
    items: [
      { name: 'CLI 配置转换', href: '/converter', icon: ArrowLeftRight }
    ]
  }
]

onMounted(() => {
  // 初始化展开状态
  navigationGroups.forEach(group => {
    expandedGroups[group.title] = group.defaultExpanded
  })

  // 从 localStorage 读取折叠状态
  const savedCollapsed = localStorage.getItem('ccr-sidebar-collapsed')
  if (savedCollapsed === 'true') {
    collapsed.value = true
  }

  // 从 localStorage 读取展开状态
  const savedExpanded = localStorage.getItem('ccr-sidebar-expanded')
  if (savedExpanded) {
    try {
      const parsed = JSON.parse(savedExpanded)
      Object.assign(expandedGroups, parsed)
    } catch (e) {
      // 忽略解析错误
    }
  }
})

const toggleCollapsed = () => {
  collapsed.value = !collapsed.value
  localStorage.setItem('ccr-sidebar-collapsed', String(collapsed.value))
}

const toggleGroup = (groupTitle: string) => {
  expandedGroups[groupTitle] = !expandedGroups[groupTitle]
  localStorage.setItem('ccr-sidebar-expanded', JSON.stringify(expandedGroups))
}

const isActive = (href: string): boolean => {
  return route.path === href
}

const hasActiveChild = (items: NavItem[]): boolean => {
  return items.some(item => isActive(item.href))
}
</script>
