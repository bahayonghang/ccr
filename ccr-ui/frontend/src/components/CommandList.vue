<template>
  <div
    v-if="loading"
    class="text-center py-8"
  >
    <div class="inline-flex items-center gap-2">
      <RefreshCw class="w-4 h-4 animate-spin" />
      <span>{{ $t('common.loading') }}</span>
    </div>
  </div>

  <div
    v-else
    class="grid gap-4"
  >
    <div
      v-for="command in commands"
      :key="command.name"
      :class="cardClass"
      :style="cardStyle"
      @mouseenter="handleMouseEnter"
      @mouseleave="handleMouseLeave"
    >
      <div class="flex-1 min-w-0">
        <!-- 命令名称和状态 -->
        <div class="flex items-center justify-between mb-2">
          <h3
            :class="titleClass"
            :style="titleStyle"
          >
            {{ command.name }}
          </h3>
          <span
            :class="statusClass(command.enabled)"
            :style="statusStyle(command.enabled)"
          >
            {{ command.enabled ? $t('common.enabled') : $t('common.disabled') }}
          </span>
        </div>

        <!-- 命令内容 -->
        <div class="mb-2">
          <code
            :class="commandClass"
            :style="commandStyle"
          >
            {{ command.command }}
          </code>
        </div>

        <!-- 描述 -->
        <p
          :class="descriptionClass"
          :style="descriptionStyle"
        >
          {{ command.description }}
        </p>

        <!-- 文件夹标签 -->
        <div class="flex items-center gap-2 mt-3">
          <span
            :class="folderTagClass"
            :style="folderTagStyle"
          >
            {{ command.folder }}
          </span>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center gap-2 ml-4">
        <button
          :class="actionButtonClass"
          :style="actionButtonStyle"
          :title="command.enabled ? $t('common.disable') : $t('common.enable')"
          :aria-label="command.enabled ? $t('common.disable') : $t('common.enable')"
          @click="$emit('toggle', command.name)"
        >
          <Power class="w-4 h-4" />
        </button>
        <button
          :class="actionButtonClass"
          :style="actionButtonStyle"
          :title="$t('common.edit')"
          :aria-label="$t('common.edit')"
          @click="$emit('edit', command)"
        >
          <Edit class="w-4 h-4" />
        </button>
        <button
          :class="[actionButtonClass, deleteButtonClass]"
          :style="[actionButtonStyle, deleteButtonStyle]"
          :title="$t('common.delete')"
          @click="$emit('delete', command.name)"
        >
          <Trash2 class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RefreshCw, Power, Edit, Trash2 } from 'lucide-vue-next'

interface SlashCommand {
  name: string
  command: string
  description: string
  folder: string
  enabled: boolean
}

interface PlatformConfig {
  api: {
    list: () => Promise<{ commands: SlashCommand[], folders: string[] }>
    add: (cmd: any) => Promise<void>
    update: (name: string, cmd: any) => Promise<void>
    delete: (name: string) => Promise<void>
    toggle: (name: string) => Promise<void>
  }
  i18n: {
    prefix: string
  }
  theme: 'claude-code' | 'css-variable'
  route: {
    homePath: string
    module: string
  }
  platform: {
    name: string
    displayName: string
  }
  features: {
    breadcrumb: boolean
    glassEffect: boolean
  }
}

interface ThemeColors {
  module: string
  primary: string
  secondary: string
  muted: string
  bg: string
  bgSecondary: string
  bgTertiary: string
  accent: string
  accentBg: string
  accentBorder: string
}

// Props
interface Props {
  commands: SlashCommand[]
  loading: boolean
  selectedFolder: string
  theme: 'claude-code' | 'css-variable'
  themeColors: ThemeColors
  config: PlatformConfig
}

const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'edit', command: SlashCommand): void
  (e: 'delete', name: string): void
  (e: 'toggle', name: string): void
}

defineEmits<Emits>()

// 计算属性
const cardClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'glass-effect rounded-xl p-4 border border-white/20 shadow-sm hover:shadow-md transition-all duration-200 group'
  }
  return ''
})

const cardStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)',
      borderRadius: '8px',
      padding: '16px',
      transition: 'all 0.2s'
    }
  }
})

const titleClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'text-base font-semibold text-guofeng-text-primary'
  }
  return 'text-base font-semibold'
})

const titleStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-primary)'
    }
  }
})

const statusClass = (enabled: boolean) => {
  if (props.theme === 'claude-code') {
    return `px-2 py-1 rounded-full text-xs font-medium ${
      enabled ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
    }`
  }
  return ''
}

const statusStyle = (enabled: boolean) => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: enabled ? 'var(--success-bg)' : 'var(--error-bg)',
      color: enabled ? 'var(--success-text)' : 'var(--error-text)',
      fontSize: '12px'
    }
  }
}

const commandClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'block text-sm bg-guofeng-bg-tertiary text-guofeng-text-primary px-2 py-1 rounded font-mono'
  }
  return 'block text-sm'
})

const commandStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-tertiary)',
      color: 'var(--text-primary)',
      padding: '4px 8px',
      borderRadius: '4px',
      fontFamily: 'monospace',
      fontSize: '13px'
    }
  }
})

const descriptionClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'text-sm text-guofeng-text-secondary'
  }
  return 'text-sm'
})

const descriptionStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-secondary)',
      fontSize: '14px'
    }
  }
})

const folderTagClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'px-2 py-1 rounded-md text-xs bg-guofeng-amber/10 text-guofeng-amber'
  }
  return 'px-2 py-1 rounded-md text-xs'
})

const folderTagStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--accent-primary) / 0.1',
      color: 'var(--accent-primary)',
      fontSize: '12px'
    }
  }
})

const actionButtonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'p-2 rounded-lg text-guofeng-text-muted hover:text-guofeng-text-primary hover:bg-guofeng-bg-tertiary transition-colors'
  }
  return 'p-2 rounded-lg'
})

const actionButtonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-muted)',
      transition: 'all 0.2s'
    }
  }
})

const deleteButtonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'hover:text-red-600 hover:bg-red-50'
  }
  return ''
})

const deleteButtonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {}
  }
})

// 方法
const handleMouseEnter = (e: MouseEvent) => {
  if (props.theme !== 'claude-code') {
    const target = e.currentTarget as HTMLElement
    target.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1)'
  }
}

const handleMouseLeave = (e: MouseEvent) => {
  if (props.theme !== 'claude-code') {
    const target = e.currentTarget as HTMLElement
    target.style.boxShadow = 'none'
  }
}
</script>