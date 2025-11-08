<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Monitor, Globe } from 'lucide-vue-next'
import { isTauriEnvironment, TauriAPI } from '@/api'

const isTauri = ref(false)
const tauriVersion = ref<string | null>(null)

onMounted(async () => {
  isTauri.value = isTauriEnvironment()

  if (isTauri.value) {
    try {
      tauriVersion.value = await TauriAPI.getTauriVersion()
    } catch (error) {
      console.error('Failed to get Tauri version:', error)
    }
  }
})
</script>

<template>
  <div
    class="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition-all"
    :class="[
      isTauri
        ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
        : 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300'
    ]"
  >
    <Monitor v-if="isTauri" :size="16" class="opacity-70" />
    <Globe v-else :size="16" class="opacity-70" />

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
