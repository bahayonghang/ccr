<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { isTauriEnvironment, TauriRuntimeApi } from '@/api/runtime/environment'
import { logger } from '@/utils/logger'

const isTauri = ref(false)
const tauriVersion = ref<string | null>(null)

onMounted(async () => {
  isTauri.value = isTauriEnvironment()

  if (isTauri.value) {
    try {
      tauriVersion.value = await TauriRuntimeApi.getTauriVersion()
    } catch (error) {
      logger.error('Failed to get Tauri version:', error)
    }
  }
})
</script>

<template>
  <div
    class="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors"
    :class="[
      isTauri
        ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
        : 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300'
    ]"
  >
    <SIcon
      v-if="isTauri"
      name="Monitor"
      class="opacity-70"
      size="w-4 h-4"
    />
    <SIcon
      v-else
      name="Globe"
      class="opacity-70"
      size="w-4 h-4"
    />

    <span>
      {{ isTauri ? '桌面应用' : 'Web版本' }}
    </span>

    <span
      v-if="isTauri && tauriVersion"
      class="px-1.5 py-0.5 rounded bg-white/50 dark:bg-black/20 text-xs"
    >
      v{{ tauriVersion }}
    </span>
  </div>
</template>
