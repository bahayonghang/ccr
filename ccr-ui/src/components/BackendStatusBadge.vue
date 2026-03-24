<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useBackendHealth } from '@/composables/useBackendHealth'
import { isTauriEnvironment } from '@/api/runtime/environment'

const { status, lastCheckedAt, checkHealth } = useBackendHealth()
const isTauri = isTauriEnvironment()
const { t, locale } = useI18n()

const label = computed(() => {
  if (!isTauri) return t('common.backend.web')
  if (status.value === 'checking' || status.value === 'unknown') return t('common.backend.checking')
  if (status.value === 'ok') return t('common.backend.ok')
  if (status.value === 'error') return t('common.backend.error')
  return t('common.backend.unknown')
})

const badgeClass = computed(() => {
  if (!isTauri) return 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300'
  if (status.value === 'ok') return 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300'
  if (status.value === 'error') return 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300'
  return 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
})

const tooltip = computed(() => {
  if (!lastCheckedAt.value) return t('common.backend.retryHint')
  const time = lastCheckedAt.value.toLocaleTimeString(locale.value === 'zh-CN' ? 'zh-CN' : 'en-US')
  return t('common.backend.lastChecked', { time })
})
</script>

<template>
  <button
    v-if="isTauri"
    class="inline-flex items-center gap-2 px-2.5 py-1 rounded-lg text-xs font-medium transition-opacity hover:opacity-90"
    :class="badgeClass"
    :title="tooltip"
    @click="checkHealth"
  >
    <SIcon
      v-if="status === 'ok'"
      name="Server"
      size="w-3.5 h-3.5"
    />
    <SIcon
      v-else-if="status === 'error'"
      name="AlertCircle"
      size="w-3.5 h-3.5"
    />
    <SIcon
      v-else
      name="Loader2"
      size="w-3.5 h-3.5"
      class="animate-spin"
    />
    <span>{{ label }}</span>
    <SIcon
      v-if="status === 'ok'"
      name="CheckCircle"
      size="w-3.5 h-3.5"
    />
  </button>
</template>
