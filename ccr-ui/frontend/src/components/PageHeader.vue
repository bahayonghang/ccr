<template>
  <div class="flex items-center justify-between mb-6">
    <div class="flex items-center gap-4">
      <h2
        :class="titleClass"
        :style="titleStyle"
      >
        <Command class="inline-block w-7 h-7 mr-2" />
        {{ title }}
      </h2>
      <span
        :class="badgeClass"
        :style="badgeStyle"
      >
        {{ count }}/{{ total }}
      </span>
    </div>

    <div class="flex items-center gap-3">
      <RouterLink
        :to="homePath"
        :class="linkClass"
        :style="linkStyle"
      >
        <Home class="w-4 h-4" />
        <span>{{ $t('common.backToHome') }}</span>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Command, Home } from 'lucide-vue-next'

// Props
interface Props {
  title: string
  subtitle?: string
  count: number
  total: number
  homePath: string
  theme: 'claude-code' | 'css-variable'
  themeColors: {
    module: string
    primary: string
    secondary: string
    bg: string
    bgSecondary: string
    accent: string
    accentBg: string
  }
}

const props = defineProps<Props>()

// 计算属性
const titleClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'text-2xl font-bold text-guofeng-text-primary'
  }
  return 'text-2xl font-bold'
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

const badgeClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'px-3 py-1 rounded-full text-sm font-medium bg-guofeng-amber text-white'
  }
  return 'px-3 py-1 rounded-full text-sm font-medium'
})

const badgeStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--accent-primary)',
      color: '#fff'
    }
  }
})

const linkClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors text-guofeng-text-secondary hover:text-guofeng-text-primary bg-guofeng-bg-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border'
  }
  return 'inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors'
})

const linkStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-secondary)',
      color: 'var(--text-secondary)',
      border: '1px solid var(--border-color)'
    }
  }
})
</script>