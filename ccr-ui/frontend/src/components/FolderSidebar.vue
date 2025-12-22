<template>
  <div
    :class="themeClasses.container"
    :style="themeStyles.container"
  >
    <h3
      :class="themeClasses.title"
      :style="themeStyles.title"
    >
      {{ $t('slashCommands.folders.title') }}
      <span
        :class="themeClasses.count"
        :style="themeStyles.count"
      >{{ stats.total }}</span>
    </h3>

    <div class="space-y-1">
      <div
        v-for="folder in folders"
        :key="folder.value"
        :class="folderClass(folder.value)"
        :style="folderStyle(folder.value)"
        @click="handleFolderClick(folder.value)"
        @mouseenter="handleMouseEnter"
        @mouseleave="handleMouseLeave"
      >
        <component
          :is="folder.icon"
          :class="iconClass(folder.value)"
          :style="iconStyle(folder.value)"
        />
        <span class="flex-1 truncate">{{ folder.label }}</span>
        <span
          :class="themeClasses.folderCount"
          :style="themeStyles.folderCount"
        >
          {{ folder.count }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface FolderOption {
  label: string
  value: string
  icon: any
  count: number
}

interface Stats {
  total: number
  enabled: number
  disabled: number
  byFolder: Record<string, number>
}

// Props
interface Props {
  folders: FolderOption[]
  selectedFolder: string
  stats: Stats
  theme: 'claude-code' | 'css-variable'
}

const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'folder-selected', folder: string): void
}

const emit = defineEmits<Emits>()

// 计算属性
const themeClasses = computed(() => {
  if (props.theme === 'claude-code') {
    return {
      container: 'glass-effect rounded-2xl p-4 border border-white/20 shadow-sm sticky top-6 self-start hidden lg:block w-64',
      title: 'text-xs font-bold text-guofeng-text-muted uppercase tracking-wider mb-3 px-2 flex items-center justify-between',
      count: 'bg-guofeng-bg-tertiary px-1.5 py-0.5 rounded text-[10px]',
      folderCount: 'text-xs px-1.5 py-0.5 rounded-md transition-colors'
    }
  } else {
    return {
      container: '',
      title: '',
      folderCount: ''
    }
  }
})

const themeStyles = computed(() => {
  if (props.theme === 'claude-code') {
    return {
      container: {},
      title: {},
      count: {},
      folderCount: {}
    }
  } else {
    return {
      container: {
        width: '180px',
        background: 'var(--bg-secondary)',
        borderRight: '1px solid var(--border-color)',
        padding: '12px 8px',
        overflowY: 'auto' as const,
        height: 'calc(100vh - 40px)',
        flexShrink: 0
      },
      title: {
        color: 'var(--text-muted)',
        fontSize: '11px',
        fontWeight: '600',
        marginBottom: '8px',
        marginLeft: '8px',
        textTransform: 'uppercase' as const,
        letterSpacing: '0.5px'
      },
      count: {},
      folderCount: {}
    }
  }
})

// 方法
const folderClass = (folderValue: string) => {
  if (props.theme === 'claude-code') {
    return `flex items-center gap-2 px-3 py-2 rounded-xl cursor-pointer text-sm transition-all duration-200 group ${
      props.selectedFolder === folderValue
        ? 'bg-guofeng-amber/10 text-guofeng-amber font-medium shadow-sm border border-guofeng-amber/20'
        : 'text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary hover:text-guofeng-text-primary'
    }`
  }
  return ''
}

const folderStyle = (folderValue: string) => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      padding: '6px 8px',
      borderRadius: '6px',
      cursor: 'pointer',
      marginBottom: '4px',
      background: props.selectedFolder === folderValue ? 'var(--accent-primary)' : 'transparent',
      color: props.selectedFolder === folderValue ? '#fff' : 'var(--text-primary)',
      display: 'flex',
      alignItems: 'center',
      gap: '6px',
      transition: 'all 0.2s',
      fontSize: '13px'
    }
  }
}

const iconClass = (folderValue: string) => {
  if (props.theme === 'claude-code') {
    return `w-4 h-4 transition-transform group-hover:scale-110 ${
      props.selectedFolder === folderValue ? 'text-guofeng-amber' : 'text-guofeng-text-muted'
    }`
  }
  return 'w-3.5 h-3.5'
}

const iconStyle = (_folderValue: string) => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {}
  }
}


const handleFolderClick = (folderValue: string) => {
  emit('folder-selected', folderValue)
}

const handleMouseEnter = (e: MouseEvent) => {
  const target = e.currentTarget as HTMLElement
  const folderValue = target.getAttribute('data-folder')

  if (props.theme !== 'claude-code' && folderValue !== props.selectedFolder) {
    target.style.background = 'var(--bg-tertiary)'
  }
}

const handleMouseLeave = (e: MouseEvent) => {
  const target = e.currentTarget as HTMLElement
  const folderValue = target.getAttribute('data-folder')

  if (props.theme !== 'claude-code' && folderValue !== props.selectedFolder) {
    target.style.background = 'transparent'
  }
}
</script>