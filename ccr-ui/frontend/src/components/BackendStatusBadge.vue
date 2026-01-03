<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircle, AlertCircle, Loader2, Server } from 'lucide-vue-next'
import { useBackendHealth } from '@/composables/useBackendHealth'
import { isTauriEnvironment } from '@/api'

const { status, lastCheckedAt, checkHealth } = useBackendHealth()
const isTauri = isTauriEnvironment()

const label = computed(() => {
  if (!isTauri) return '后端：Web'
  if (status.value === 'checking' || status.value === 'unknown') return '后端：检测中'
  if (status.value === 'ok') return '后端：运行中'
  if (status.value === 'error') return '后端：不可用'
  return '后端：未知'
})

const badgeClass = computed(() => {
  if (!isTauri) return 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300'
  if (status.value === 'ok') return 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300'
  if (status.value === 'error') return 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300'
  return 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
})

const tooltip = computed(() => {
  if (!lastCheckedAt.value) return '点击重试'
  return `上次检测：${lastCheckedAt.value.toLocaleTimeString('zh-CN')}`
})
</script>

<template>
  <button
    v-if="isTauri"
    class="inline-flex items-center gap-2 px-2.5 py-1 rounded-lg text-xs font-medium transition-all hover:opacity-90"
    :class="badgeClass"
    :title="tooltip"
    @click="checkHealth"
  >
    <Server
      v-if="status === 'ok'"
      class="w-3.5 h-3.5"
    />
    <AlertCircle
      v-else-if="status === 'error'"
      class="w-3.5 h-3.5"
    />
    <Loader2
      v-else
      class="w-3.5 h-3.5 animate-spin"
    />
    <span>{{ label }}</span>
    <CheckCircle
      v-if="status === 'ok'"
      class="w-3.5 h-3.5"
    />
  </button>
</template>
