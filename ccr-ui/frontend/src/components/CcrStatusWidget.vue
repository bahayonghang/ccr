<template>
  <div class="flex items-center gap-3 bg-bg-secondary/40 p-2 rounded-2xl border border-white/10 backdrop-blur-md shadow-sm transition-all duration-300">
    <!-- Loading State -->
    <div
      v-if="loading"
      class="flex items-center gap-3 px-4 py-2 rounded-xl bg-white/50 dark:bg-white/5 border border-white/20"
    >
      <Loader2 class="w-4 h-4 animate-spin text-accent-primary" />
      <div class="flex flex-col">
        <span class="text-[10px] uppercase text-text-muted font-bold">STATUS</span>
        <span class="text-xs font-bold text-text-secondary">Checking...</span>
      </div>
    </div>

    <!-- Installed State -->
    <div
      v-else-if="installed && version"
      class="flex items-center gap-3 px-4 py-2 rounded-xl bg-white/50 dark:bg-white/5 border border-white/20"
    >
      <div class="w-8 h-8 rounded-lg bg-green-500/10 flex items-center justify-center">
        <CheckCircle2 class="w-5 h-5 text-green-500" />
      </div>
      <div class="flex flex-col">
        <span class="text-[10px] uppercase text-text-muted font-bold">CCR INSTALLED</span>
        <span class="text-sm font-bold text-green-600 dark:text-green-400 font-mono">{{ version }}</span>
      </div>
    </div>

    <!-- Not Installed / Error State -->
    <div
      v-else
      class="flex items-center gap-3 px-4 py-2 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 transition-colors"
    >
      <div class="w-8 h-8 rounded-lg bg-red-100 dark:bg-red-800/30 flex items-center justify-center">
        <AlertTriangle class="w-5 h-5 text-red-600 dark:text-red-400" />
      </div>
      <div class="flex flex-col mr-2">
        <span class="text-[10px] uppercase text-red-600/70 dark:text-red-400/70 font-bold">CCR MISSING</span>
        <span class="text-xs font-bold text-red-700 dark:text-red-300">Not Detected</span>
      </div>
      <button 
        class="px-3 py-1.5 text-xs font-bold text-white bg-red-600 hover:bg-red-700 dark:bg-red-500 dark:hover:bg-red-600 rounded-lg transition-colors flex items-center gap-1.5 shadow-sm"
        @click="showInstallInfo"
      >
        <Download class="w-3.5 h-3.5" />
        Install
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Loader2, CheckCircle2, AlertTriangle, Download } from 'lucide-vue-next'
import { getVersion } from '@/api/client'

const loading = ref(true)
const installed = ref(false)
const version = ref('')

const checkStatus = async () => {
  loading.value = true
  try {
    const info = await getVersion()
    if (info && info.current_version) {
      version.value = info.current_version
      installed.value = true
    } else {
      installed.value = false
    }
  } catch (error) {
    console.warn('CCR check failed:', error)
    installed.value = false
  } finally {
    loading.value = false
  }
}

const showInstallInfo = () => {
  // Simple alert for now, can be upgraded to a modal
  window.open('https://github.com/liyonghang/ccr', '_blank')
}

onMounted(() => {
  checkStatus()
})
</script>
