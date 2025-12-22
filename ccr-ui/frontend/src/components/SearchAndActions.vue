<template>
  <div class="mb-6 flex items-center justify-between">
    <!-- 搜索框 -->
    <div class="relative flex-1 max-w-md">
      <Search
        :class="searchIconClass"
        :style="searchIconStyle"
      />
      <input
        v-model="searchQuery"
        type="text"
        :placeholder="$t('common.search')"
        :class="searchInputClass"
        :style="searchInputStyle"
      >
    </div>

    <!-- 操作按钮 -->
    <div class="flex items-center gap-3">
      <!-- 刷新按钮 -->
      <button
        :class="buttonClass"
        :style="buttonStyle"
        :disabled="loading"
        @click="$emit('refresh')"
      >
        <RefreshCw :class="['w-4 h-4', loading ? 'animate-spin' : '']" />
        {{ $t('common.refresh') }}
      </button>

      <!-- 添加按钮 -->
      <button
        :class="[buttonClass, addButtonClass]"
        :style="[buttonStyle, addButtonStyle]"
        @click="$emit('add-command')"
      >
        <Plus class="w-4 h-4" />
        {{ $t('common.add') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Search, RefreshCw, Plus } from 'lucide-vue-next'

// Props
interface Props {
  modelValue: string
  loading: boolean
  theme: 'claude-code' | 'css-variable'
  themeColors: {
    bg: string
    bgSecondary: string
    bgTertiary: string
    primary: string
    secondary: string
    accent: string
    accentBg: string
  }
}

const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'update:modelValue', value: string): void
  (e: 'refresh'): void
  (e: 'add-command'): void
}

const emit = defineEmits<Emits>()

// 计算属性
const searchQuery = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const searchIconClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-guofeng-text-muted'
  }
  return 'absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4'
})

const searchIconStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-muted)'
    }
  }
})

const searchInputClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'w-full pl-10 pr-4 py-2 rounded-xl border border-guofeng-border bg-guofeng-bg-primary text-guofeng-text-primary placeholder-guofeng-text-muted focus:outline-none focus:ring-2 focus:ring-guofeng-amber focus:border-transparent'
  }
  return 'w-full pl-10 pr-4 py-2'
})

const searchInputStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      borderRadius: '8px',
      border: '1px solid var(--border-color)',
      background: 'var(--bg-primary)',
      color: 'var(--text-primary)',
      fontSize: '14px'
    }
  }
})

const buttonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200 border border-guofeng-border bg-guofeng-bg-secondary text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary hover:text-guofeng-text-primary hover:shadow-sm disabled:opacity-50 disabled:cursor-not-allowed'
  }
  return 'inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200'
})

const buttonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-secondary)',
      color: 'var(--text-secondary)',
      border: '1px solid var(--border-color)',
      fontSize: '14px'
    }
  }
})

const addButtonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'bg-guofeng-amber text-white hover:bg-guofeng-amber/90 border-guofeng-amber'
  }
  return ''
})

const addButtonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--accent-primary)',
      color: '#fff',
      borderColor: 'var(--accent-primary)'
    }
  }
})
</script>