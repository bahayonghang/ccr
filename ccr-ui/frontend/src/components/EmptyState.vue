<template>
  <div class="text-center py-12">
    <div
      :class="iconClass"
      :style="iconStyle"
    >
      <Command class="w-12 h-12" />
    </div>

    <h3
      :class="titleClass"
      :style="titleStyle"
    >
      {{ titleText }}
    </h3>

    <p
      :class="descriptionClass"
      :style="descriptionStyle"
    >
      {{ descriptionText }}
    </p>

    <!-- 操作建议 -->
    <div
      v-if="hasSearchQuery"
      class="mt-6"
    >
      <button
        :class="buttonClass"
        :style="buttonStyle"
        @click="$emit('clear-search')"
      >
        {{ $t('common.clearSearch') }}
      </button>
    </div>

    <div
      v-else-if="hasFolderFilter"
      class="mt-6"
    >
      <button
        :class="buttonClass"
        :style="buttonStyle"
        @click="$emit('clear-filter')"
      >
        {{ $t('common.showAll') }}
      </button>
    </div>

    <div
      v-else
      class="mt-6"
    >
      <button
        :class="[buttonClass, addButtonClass]"
        :style="[buttonStyle, addButtonStyle]"
        @click="$emit('add-first')"
      >
        <Plus class="w-4 h-4 mr-2" />
        {{ $t('slashCommands.addFirst') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Command, Plus } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Props
interface Props {
  searchQuery: string
  selectedFolder: string
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
  (e: 'clear-search'): void
  (e: 'clear-filter'): void
  (e: 'add-first'): void
}

defineEmits<Emits>()

// 计算属性
const hasSearchQuery = computed(() => props.searchQuery.trim() !== '')
const hasFolderFilter = computed(() => props.selectedFolder !== 'all')

const titleText = computed(() => {
  if (hasSearchQuery.value) {
    return t('slashCommands.noSearchResults')
  } else if (hasFolderFilter.value) {
    return t('slashCommands.noCommandsInFolder')
  } else {
    return t('slashCommands.noCommands')
  }
})

const descriptionText = computed(() => {
  if (hasSearchQuery.value) {
    return t('slashCommands.tryDifferentSearch')
  } else if (hasFolderFilter.value) {
    return t('slashCommands.tryDifferentFolder')
  } else {
    return t('slashCommands.addFirstCommand')
  }
})

const iconClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'mx-auto w-16 h-16 flex items-center justify-center rounded-full bg-guofeng-bg-tertiary text-guofeng-text-muted mb-4'
  }
  return 'mx-auto w-16 h-16 flex items-center justify-center rounded-full mb-4'
})

const iconStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-tertiary)',
      color: 'var(--text-muted)'
    }
  }
})

const titleClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'text-lg font-medium text-guofeng-text-primary mb-2'
  }
  return 'text-lg font-medium mb-2'
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

const descriptionClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'text-guofeng-text-secondary max-w-md mx-auto'
  }
  return 'max-w-md mx-auto'
})

const descriptionStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-secondary)'
    }
  }
})

const buttonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'inline-flex items-center px-4 py-2 rounded-lg border border-guofeng-border bg-guofeng-bg-secondary text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary hover:text-guofeng-text-primary transition-colors'
  }
  return 'inline-flex items-center px-4 py-2 rounded-lg border'
})

const buttonStyle = computed(() => {
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