<!-- -->
<template>
  <aside
    class="rounded-xl p-4 h-fit sticky top-5 transition-all duration-300 bg-black/20 dark:bg-black/40 backdrop-blur-xl shadow-2xl border border-white/10"
    :class="collapsed ? 'w-16' : 'w-64'"
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
        class="p-2 rounded-lg transition-transform hover:scale-110 glass-surface border border-white/20 text-white/70 hover:text-white"
        :title="collapsed ? '展开菜单' : '收起菜单'"
        :aria-label="collapsed ? '展开菜单' : '收起菜单'"
        @click="toggleCollapsed"
      >
        <SIcon
          v-if="collapsed"
          name="ChevronRight"
          size="w-4 h-4"
        />
        <SIcon
          v-else
          name="ChevronLeft"
          size="w-4 h-4"
        />
      </button>
    </div>

    <!-- 导航链接 - 层级菜单 -->
    <nav
      class="space-y-2"
      aria-label="主导航"
    >
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
          class="w-full flex items-center rounded-lg transition-transform hover:scale-[1.02]"
          :class="collapsed ? 'justify-center' : 'justify-between'"
          :style="{
            padding: collapsed ? '12px' : '12px 16px',
            background: hasActiveChild(group.items)
              ? 'rgba(255, 255, 255, 0.15)'
              : 'rgba(255, 255, 255, 0.05)',
            border: `1px solid ${hasActiveChild(group.items) ? 'rgba(244, 114, 182, 0.5)' : 'rgba(255, 255, 255, 0.1)'}`,
            color: hasActiveChild(group.items) ? 'var(--accent-primary)' : 'rgba(255, 255, 255, 0.9)'
          }"
          :title="collapsed ? group.title : undefined"
          :aria-expanded="!collapsed && expandedGroups[group.title]"
          :aria-label="`${group.title} 菜单组`"
          @click="!collapsed && toggleGroup(group.title)"
        >
          <div
            class="flex items-center"
            :class="collapsed ? '' : 'space-x-3'"
          >
            <SIcon
              :name="group.icon || ''"
              size="w-5 h-5"
              class="flex-shrink-0"
            />
            <span
              v-if="!collapsed"
              class="font-semibold text-sm"
            >{{ group.title }}</span>
          </div>
          <SIcon
            v-if="!collapsed && expandedGroups[group.title]"
            name="ChevronUp"
            size="w-4 h-4"
          />
          <SIcon
            v-else-if="!collapsed"
            name="ChevronDown"
            size="w-4 h-4"
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
            class="flex items-center space-x-3 px-4 py-3 ml-2 rounded-lg transition-[color,background-color,border-color,transform] duration-300 relative overflow-hidden group"
            :class="isActive(item.href) ? 'scale-[1.02] nav-item-active-glow' : 'hover:translate-x-1 nav-item-inactive'"
            :style="{
              marginTop: itemIndex > 0 ? '4px' : '0',
              marginBottom: '4px',
              color: isActive(item.href) ? 'white' : 'var(--text-secondary)',
              fontWeight: isActive(item.href) ? '600' : '500',
              borderBottom: itemIndex < group.items.length - 1 && !isActive(item.href)
                ? '1px solid rgba(var(--color-accent-secondary-rgb), 0.1)'
                : undefined
            }"
            :aria-current="isActive(item.href) ? 'page' : undefined"
          >
            <!-- 液态玻璃效果叠加层 -->
            <span
              v-if="isActive(item.href)"
              class="absolute inset-0 rounded-lg pointer-events-none"
              :style="{
                background: 'linear-gradient(135deg, rgba(255, 255, 255, 0.12) 0%, rgba(255, 255, 255, 0.05) 100%)'
              }"
              aria-hidden="true"
            />

            <!-- 左侧发光指示器 -->
            <span
              class="absolute left-0 top-0 w-1 h-full transition-[transform,opacity] duration-300"
              :class="isActive(item.href) ? 'scale-y-100 opacity-100' : 'scale-y-0 opacity-0 group-hover:scale-y-75 group-hover:opacity-50'"
              :style="{
                background: isActive(item.href)
                  ? 'linear-gradient(to bottom, var(--color-accent-secondary), var(--color-accent-secondary-hover))'
                  : 'rgba(var(--color-accent-secondary-rgb), 0.6)',
                boxShadow: isActive(item.href) ? '0 0 10px var(--color-accent-secondary)' : undefined
              }"
              aria-hidden="true"
            />

            <!-- 图标 - 带动画 -->
            <SIcon
              :name="item.icon || ''"
              size="w-5 h-5"
              class="flex-shrink-0 transition-transform duration-300"
              :class="isActive(item.href) ? 'scale-110' : 'group-hover:scale-105'"
              :style="{
                filter: isActive(item.href) ? 'drop-shadow(0 0 4px rgba(255, 255, 255, 0.5))' : undefined
              }"
            />
            <span
              class="font-medium text-sm transition-colors duration-300"
              :class="isActive(item.href) ? 'tracking-wide' : ''"
            >
              {{ item.name }}
            </span>

            <!-- 激活状态右侧光点 -->
            <span
              v-if="isActive(item.href)"
              class="ml-auto w-2 h-2 rounded-full"
              :style="{
                background: 'white',
                boxShadow: '0 0 8px white'
              }"
              aria-hidden="true"
            />
          </RouterLink>
        </div>

        <!-- 折叠状态下，显示子菜单作为独立项 -->
        <div
          v-if="collapsed"
          class="space-y-1"
        >
          <RouterLink
            v-for="(item, itemIndex) in group.items"
            :key="item.href"
            :to="item.href"
            class="flex items-center justify-center px-4 py-3 rounded-lg transition-[color,background-color,border-color,transform] duration-300 relative overflow-hidden group"
            :class="isActive(item.href) ? 'scale-110 nav-item-active-glow' : 'hover:scale-105 nav-item-inactive'"
            :style="{
              color: isActive(item.href) ? 'white' : 'var(--text-secondary)',
              marginTop: itemIndex > 0 ? '4px' : '0',
              borderBottom: itemIndex < group.items.length - 1 && !isActive(item.href)
                ? '1px solid rgba(var(--color-accent-secondary-rgb), 0.15)'
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
                background: 'linear-gradient(135deg, rgba(255, 255, 255, 0.12) 0%, rgba(255, 255, 255, 0.05) 100%)'
              }"
              aria-hidden="true"
            />
            
            <SIcon
              :name="item.icon || ''"
              size="w-5 h-5"
              class="flex-shrink-0 transition-transform duration-300"
              :class="isActive(item.href) ? 'scale-110' : ''"
              :style="{
                filter: isActive(item.href) ? 'drop-shadow(0 0 4px rgba(255, 255, 255, 0.5))' : undefined
              }"
            />

            <!-- 激活指示器 -->
            <span
              v-if="isActive(item.href)"
              class="absolute bottom-0 left-1/2 transform -translate-x-1/2 w-1/2 h-0.5"
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
    <div
      v-if="collapsed"
      class="mt-4 text-center"
    >
      <button
        class="p-2 rounded-lg transition-transform hover:scale-110 glass-surface border border-white/20 text-white/70 hover:text-white"
        title="展开菜单"
        aria-label="展开菜单"
        @click="toggleCollapsed"
      >
        <SIcon
          name="Menu"
          size="w-4 h-4"
        />
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, reactive, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import type { PlatformCapabilitiesResponse, PlatformModuleCapabilities } from '@/types/platform'
interface NavItem {
  name: string
  href: string
  icon: string
}

interface NavGroup {
  title: string
  icon: string
  defaultExpanded: boolean
  items: NavItem[]
  module?: string  // 添加模块标识
}

// Props: 接受一个可选的 module 参数来过滤菜单
interface Props {
  module?: string  // 'claude-code' | 'codex' | 'gemini-cli' | 'qwen' | 'iflow' | 'droid' | 'opencode' | 'commands' | 'converter'
}

const props = withDefaults(defineProps<Props>(), {
  module: undefined
})

const route = useRoute()
const collapsed = ref(false)
const expandedGroups = reactive<Record<string, boolean>>({})
const capabilities = ref<PlatformCapabilitiesResponse | null>(null)

type CapabilityKey = keyof PlatformModuleCapabilities

const moduleToPlatformKey = (module?: string): string | null => {
  if (!module) return null
  if (module === 'claude') return 'claude-code'
  return module
}

const featureForHref = (href: string): CapabilityKey | null => {
  if (href.includes('/mcp')) return 'mcp'
  if (href.includes('/agents')) return 'agents'
  if (href.includes('/slash-commands')) return 'slash_commands'
  if (href.includes('/plugins')) return 'plugins'
  if (href.includes('/skills')) return 'skills'
  if (href.includes('/usage')) return 'usage'
  if (href.includes('/profiles')) return 'profiles'
  if (href.includes('/auth')) return 'auth'
  if (href.includes('/config')) return 'config'
  if (href.startsWith('/commands')) return 'commands'
  return null
}

const isItemEnabled = (groupModule: string | undefined, href: string): boolean => {
  const platformKey = moduleToPlatformKey(groupModule)
  const feature = featureForHref(href)
  if (!platformKey || !feature) return true
  if (!capabilities.value) return true
  const platformCaps = capabilities.value.platforms?.[platformKey]
  if (!platformCaps) return false
  return Boolean(platformCaps[feature])
}

// 完整的导航菜单结构
const allNavigationGroups: NavGroup[] = [
  {
    title: 'CCR 命令执行',
    icon: 'Terminal',
    defaultExpanded: false,
    module: 'commands',
    items: [
      { name: 'CCR 命令', href: '/commands/ccr', icon: 'Zap' },
      { name: 'Claude Code 命令', href: '/commands/claude-code', icon: 'Zap' },
      { name: 'Claude 命令', href: '/commands/claude', icon: 'Zap' },
      { name: 'Qwen 命令', href: '/commands/qwen', icon: 'Sparkles' },
      { name: 'Gemini 命令', href: '/commands/gemini', icon: 'Gem' },
      { name: 'IFLOW 命令', href: '/commands/iflow', icon: 'Workflow' }
    ]
  },
  {
    title: 'Claude Code',
    icon: 'Code2',
    defaultExpanded: true,
    module: 'claude-code',
    items: [
      { name: 'Profiles 配置', href: '/claude-code/profiles', icon: 'Settings' },
      { name: '☁️ 云同步', href: '/sync', icon: 'Cloud' },
      { name: 'MCP 服务器', href: '/mcp', icon: 'Server' },
      { name: 'Slash Commands', href: '/slash-commands', icon: 'Command' },
      { name: 'Agents', href: '/agents', icon: 'Bot' },
      { name: 'Skills', href: '/skills', icon: 'Book' },
      { name: 'Skill Hub', href: '/skills/hub', icon: 'Boxes' },
      { name: '插件管理', href: '/plugins', icon: 'Puzzle' }
    ]
  },
  {
    title: 'Codex',
    icon: 'Boxes',
    defaultExpanded: false,
    module: 'codex',
    items: [
      { name: '账号与 Auth', href: '/codex/auth', icon: 'KeyRound' },
      { name: 'Profiles 配置', href: '/codex/profiles', icon: 'Folders' },
      { name: 'CLI 设置', href: '/codex/settings', icon: 'SlidersHorizontal' },
      { name: 'MCP 服务器', href: '/codex/mcp', icon: 'Server' },
      { name: 'Slash Commands', href: '/codex/slash-commands', icon: 'Command' }
    ]
  },
  {
    title: 'Gemini CLI',
    icon: 'Sparkles',
    defaultExpanded: false,
    module: 'gemini-cli',
    items: [
      { name: 'MCP 服务器', href: '/gemini-cli/mcp', icon: 'Server' },
      { name: 'Agents', href: '/gemini-cli/agents', icon: 'Bot' },
      { name: 'Slash Commands', href: '/gemini-cli/slash-commands', icon: 'Command' },
      { name: '插件管理', href: '/gemini-cli/plugins', icon: 'Puzzle' }
    ]
  },
  {
    title: 'Qwen',
    icon: 'Zap',
    defaultExpanded: false,
    module: 'qwen',
    items: [
      { name: 'MCP 服务器', href: '/qwen/mcp', icon: 'Server' },
      { name: 'Agents', href: '/qwen/agents', icon: 'Bot' },
      { name: 'Slash Commands', href: '/qwen/slash-commands', icon: 'Command' },
      { name: '插件管理', href: '/qwen/plugins', icon: 'Puzzle' }
    ]
  },
  {
    title: 'iFlow',
    icon: 'Flame',
    defaultExpanded: false,
    module: 'iflow',
    items: [
      { name: 'MCP 服务器', href: '/iflow/mcp', icon: 'Server' },
      { name: 'Agents', href: '/iflow/agents', icon: 'Bot' },
      { name: 'Slash Commands', href: '/iflow/slash-commands', icon: 'Command' },
      { name: '插件管理', href: '/iflow/plugins', icon: 'Puzzle' }
    ]
  },
  {
    title: 'Factory Droid',
    icon: 'Bot',
    defaultExpanded: false,
    module: 'droid',
    items: [
      { name: 'MCP 服务器', href: '/droid/mcp', icon: 'Server' },
      { name: 'Agents', href: '/droid/agents', icon: 'Bot' },
      { name: 'Slash Commands', href: '/droid/slash-commands', icon: 'Command' },
      { name: '插件管理', href: '/droid/plugins', icon: 'Puzzle' }
    ]
  },
  {
    title: 'OpenCode',
    icon: 'Layers',
    defaultExpanded: false,
    module: 'opencode',
    items: [
      { name: 'Providers', href: '/opencode/providers', icon: 'Layers' },
      { name: 'MCP 服务器', href: '/opencode/mcp', icon: 'Server' },
      { name: 'Skills', href: '/skills', icon: 'Book' },
      { name: '插件管理', href: '/opencode/plugins', icon: 'Puzzle' }
    ]
  },
  {
    title: '配置转换器',
    icon: 'ArrowLeftRight',
    defaultExpanded: false,
    module: 'converter',
    items: [
      { name: 'CLI 配置转换', href: '/converter', icon: 'ArrowLeftRight' }
    ]
  }
]

// 根据 module prop 过滤导航菜单
const navigationGroups = computed(() => {
  if (!props.module) {
    // 如果没有指定 module，显示所有菜单
    return allNavigationGroups
      .map(group => ({
        ...group,
        items: group.items.filter(item => isItemEnabled(group.module, item.href))
      }))
      .filter(group => group.items.length > 0)
  }
  
  // 否则只显示指定 module 的菜单
  return allNavigationGroups
    .filter(group => group.module === props.module)
    .map(group => ({
      ...group,
      items: group.items.filter(item => isItemEnabled(group.module, item.href))
    }))
    .filter(group => group.items.length > 0)
})

onMounted(() => {
  // TODO: 平台能力检测待迁移到 Tauri command
  // 当前默认所有功能可用（capabilities = null 时 isItemEnabled 返回 true）

  // 初始化展开状态
  navigationGroups.value.forEach(group => {
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
