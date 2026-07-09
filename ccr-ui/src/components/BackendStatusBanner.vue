<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useBackendHealth } from '@/composables/useBackendHealth'
import { isTauriEnvironment } from '@/api/runtime/environment'

const { status, errorMessage, checkHealth, resume, pause } = useBackendHealth()
const isTauri = isTauriEnvironment()
const { t } = useI18n()

const shouldShow = computed(() => isTauri && status.value === 'error')

// 轮询生命周期绑定到 Banner，避免模块级常驻轮询
onMounted(() => resume())
onUnmounted(() => pause())
</script>

<template>
  <div
    v-if="shouldShow"
    class="mx-6 mt-4 rounded-xl border border-red-200/80 bg-red-50/80 px-4 py-3 text-sm text-red-800 shadow-sm dark:border-red-800/70 dark:bg-red-900/30 dark:text-red-200"
  >
    <div class="flex items-start gap-3">
      <SIcon
        name="AlertTriangle"
        size="h-4 w-4"
        class="mt-0.5"
      />
      <div class="flex-1">
        <div class="font-semibold">
          {{ t('common.backend.bannerTitle') }}
        </div>
        <div class="mt-1 text-xs opacity-80">
          {{ errorMessage || t('common.backend.bannerFallback') }}
        </div>
        <div class="mt-1 text-xs opacity-80">
          {{ t('common.backend.bannerHint') }}
        </div>
      </div>
      <button
        class="shrink-0 rounded-lg border border-red-200 bg-white/80 px-3 py-1 text-xs font-semibold text-red-700 transition hover:bg-white dark:border-red-700/60 dark:bg-red-900/30 dark:text-red-200"
        @click="checkHealth"
      >
        {{ t('common.backend.retry') }}
      </button>
    </div>
  </div>
</template>
