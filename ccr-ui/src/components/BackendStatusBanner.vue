<script setup lang="ts">
import { computed } from 'vue'
import { AlertTriangle } from 'lucide-vue-next'
import { useBackendHealth } from '@/composables/useBackendHealth'
import { isTauriEnvironment } from '@/api'

const { status, errorMessage, checkHealth } = useBackendHealth()
const isTauri = isTauriEnvironment()

const shouldShow = computed(() => isTauri && status.value === 'error')
</script>

<template>
  <div
    v-if="shouldShow"
    class="mx-6 mt-4 rounded-xl border border-red-200/80 bg-red-50/80 px-4 py-3 text-sm text-red-800 shadow-sm dark:border-red-800/70 dark:bg-red-900/30 dark:text-red-200"
  >
    <div class="flex items-start gap-3">
      <AlertTriangle class="mt-0.5 h-4 w-4" />
      <div class="flex-1">
        <div class="font-semibold">
          后端连接失败，请检查桌面后端是否启动
        </div>
        <div class="mt-1 text-xs opacity-80">
          {{ errorMessage || '无法访问本地后端服务' }}
        </div>
      </div>
      <button
        class="shrink-0 rounded-lg border border-red-200 bg-white/80 px-3 py-1 text-xs font-semibold text-red-700 transition hover:bg-white dark:border-red-700/60 dark:bg-red-900/30 dark:text-red-200"
        @click="checkHealth"
      >
        重试
      </button>
    </div>
  </div>
</template>
