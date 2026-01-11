<template>
  <div class="mb-6 flex items-center justify-between">
    <!-- 搜索框 -->
    <div class="relative flex-1 max-w-md">
      <Search
        class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4"
        :style="{ color: 'var(--text-muted)' }"
        aria-hidden="true"
      />
      <input
        v-model="searchQuery"
        type="text"
        :placeholder="$t('common.search')"
        :aria-label="$t('common.search')"
        class="w-full pl-10 pr-4 py-2 rounded-xl text-sm focus:outline-none focus:ring-2"
        :style="{
          border: '1px solid var(--border-color)',
          background: 'var(--bg-primary)',
          color: 'var(--text-primary)',
          '--tw-ring-color': 'var(--accent-primary)'
        }"
      >
    </div>

    <!-- 操作按钮 -->
    <div class="flex items-center gap-3">
      <!-- 刷新按钮 -->
      <button
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200 hover:opacity-80"
        :style="{
          background: 'var(--bg-secondary)',
          color: 'var(--text-secondary)',
          border: '1px solid var(--border-color)'
        }"
        :disabled="loading"
        :aria-label="$t('common.refresh')"
        @click="$emit('refresh')"
      >
        <RefreshCw
          :class="['w-4 h-4', loading ? 'animate-spin' : '']"
          aria-hidden="true"
        />
        {{ $t('common.refresh') }}
      </button>

      <!-- 添加按钮 -->
      <button
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200 hover:opacity-90"
        :style="{
          background: 'var(--accent-primary)',
          color: '#fff',
          border: '1px solid var(--accent-primary)'
        }"
        :aria-label="$t('common.add')"
        @click="$emit('add-command')"
      >
        <Plus
          class="w-4 h-4"
          aria-hidden="true"
        />
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
</script>
