<template>
  <div
    class="rounded-lg p-4"
    :style="{
      background: 'var(--bg-tertiary)',
      border: '1px solid var(--border-color)'
    }"
  >
    <!-- 版本信息头部 -->
    <div class="flex items-center justify-between mb-3">
      <span
        class="text-xs font-semibold uppercase tracking-wider"
        :style="{ color: 'var(--text-secondary)' }"
      >
        版本管理
      </span>
      <Zap class="w-4 h-4" :style="{ color: 'var(--accent-primary)' }" />
    </div>

    <!-- 当前版本 -->
    <div v-if="versionInfo" class="mb-3">
      <div class="text-xs mb-1" :style="{ color: 'var(--text-muted)' }">当前版本</div>
      <div
        class="text-2xl font-bold font-mono tracking-wide"
        :style="{ color: 'var(--accent-primary)' }"
      >
        v{{ versionInfo.current_version }}
      </div>
    </div>

    <!-- 更新信息 -->
    <div
      v-if="updateInfo && updateInfo.has_update"
      class="mb-3 p-2.5 rounded-lg"
      :style="{
        background: 'rgba(16, 185, 129, 0.1)',
        border: '1px solid var(--accent-success)'
      }"
    >
      <div class="flex items-center justify-between mb-1.5">
        <div class="flex items-center space-x-1.5">
          <span
            class="w-1.5 h-1.5 rounded-full animate-pulse"
            :style="{
              background: 'var(--accent-success)',
              boxShadow: '0 0 10px var(--glow-success)'
            }"
          />
          <span
            class="text-xs font-semibold"
            :style="{ color: 'var(--accent-success)' }"
          >
            发现新版本
          </span>
        </div>
        <span
          class="text-sm font-bold font-mono"
          :style="{ color: 'var(--accent-success)' }"
        >
          v{{ updateInfo.latest_version }}
        </span>
      </div>
      <a
        v-if="updateInfo.release_url"
        :href="updateInfo.release_url"
        target="_blank"
        rel="noopener noreferrer"
        class="text-xs underline hover:no-underline"
        :style="{ color: 'var(--accent-success)' }"
      >
        查看更新日志 →
      </a>
    </div>

    <div
      v-if="updateInfo && !updateInfo.has_update"
      class="mb-3 text-xs text-center py-1.5"
      :style="{ color: 'var(--text-muted)' }"
    >
      ✓ 已是最新版本
    </div>

    <!-- 操作按钮 -->
    <div class="grid grid-cols-2 gap-2">
      <button
        :disabled="isCheckingUpdate"
        class="px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-1.5 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
        :style="{
          background: 'var(--bg-secondary)',
          color: 'var(--text-primary)',
          border: '1px solid var(--border-color)'
        }"
        @click="handleCheckUpdate"
      >
        <RefreshCw :class="['w-3.5 h-3.5', { 'animate-spin': isCheckingUpdate }]" />
        <span>{{ isCheckingUpdate ? '检查中' : '检查更新' }}</span>
      </button>

      <button
        class="px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-1.5 text-white hover:scale-105"
        :class="{ 'animate-pulse-subtle': updateInfo?.has_update }"
        :style="{
          background: updateInfo?.has_update
            ? 'linear-gradient(135deg, var(--accent-success), var(--accent-primary))'
            : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
          boxShadow: updateInfo?.has_update
            ? '0 0 20px var(--glow-success)'
            : '0 0 20px var(--glow-primary)'
        }"
        @click="handleOpenUpdateModal"
      >
        <Zap class="w-3.5 h-3.5" />
        <span>立即更新</span>
      </button>
    </div>

    <!-- 更新对话框 -->
    <UpdateModal
      :is-open="showUpdateModal"
      :stage="updateStage"
      :output="updateOutput"
      :error="updateError"
      @close="handleCloseUpdateModal"
      @confirm="handleConfirmUpdate"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { RefreshCw, Zap } from 'lucide-vue-next'
import { getVersion, checkUpdate, updateCCR } from '@/api/client'
import type { VersionInfo, UpdateCheckResponse } from '@/types'
import UpdateModal from './UpdateModal.vue'

const versionInfo = ref<VersionInfo | null>(null)
const updateInfo = ref<UpdateCheckResponse | null>(null)
const isCheckingUpdate = ref(false)
const showUpdateModal = ref(false)
const updateStage = ref<'confirm' | 'updating' | 'success' | 'error'>('confirm')
const updateOutput = ref('')
const updateError = ref('')

onMounted(() => {
  loadVersionInfo()
})

const loadVersionInfo = async () => {
  try {
    const data = await getVersion()
    versionInfo.value = data
  } catch (err) {
    console.error('Failed to load version info:', err)
  }
}

const handleCheckUpdate = async () => {
  isCheckingUpdate.value = true
  try {
    const data = await checkUpdate()
    updateInfo.value = data
  } catch (err) {
    console.error('Failed to check for updates:', err)
  } finally {
    isCheckingUpdate.value = false
  }
}

const handleOpenUpdateModal = () => {
  updateStage.value = 'confirm'
  updateOutput.value = ''
  updateError.value = ''
  showUpdateModal.value = true
}

const handleConfirmUpdate = async () => {
  updateStage.value = 'updating'
  updateOutput.value = '开始更新 CCR...\n'

  try {
    const result = await updateCCR()

    if (result.success) {
      updateOutput.value = result.output || '更新完成！'
      updateStage.value = 'success'
      setTimeout(() => {
        loadVersionInfo()
        updateInfo.value = null
      }, 1000)
    } else {
      updateOutput.value = result.output || ''
      updateError.value = result.error || '更新失败'
      updateStage.value = 'error'
    }
  } catch (err) {
    console.error('Failed to update CCR:', err)
    updateError.value = err instanceof Error ? err.message : '更新过程中出现错误'
    updateStage.value = 'error'
  }
}

const handleCloseUpdateModal = () => {
  showUpdateModal.value = false
}
</script>
