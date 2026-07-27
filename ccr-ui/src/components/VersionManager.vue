<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getVersion, checkUpdate, updateCCR } from '@/api'
import type { VersionInfo } from '@/types/generated/system/VersionInfo'
import { logger } from '@/utils/logger'
import UpdateModal from './UpdateModal.vue'

interface UpdateResult {
  success: boolean
  output?: string
  error?: string
}

const { t } = useI18n()
const versionInfo = ref<VersionInfo | null>(null)
const updateInfo = ref<VersionInfo | null>(null)
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
    logger.error('Failed to load version info:', err)
  }
}

const handleCheckUpdate = async () => {
  isCheckingUpdate.value = true
  try {
    const data = await checkUpdate()
    updateInfo.value = data
  } catch (err) {
    logger.error('Failed to check for updates:', err)
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
  updateOutput.value = t('common.updateModal.outputStart')

  try {
    const result = await updateCCR<UpdateResult>()

    if (result.success) {
      updateOutput.value = result.output || t('common.updateModal.outputCompleted')
      updateStage.value = 'success'
      setTimeout(() => {
        loadVersionInfo()
        updateInfo.value = null
      }, 1000)
    } else {
      updateOutput.value = result.output || ''
      updateError.value = result.error || t('common.updateModal.outputError')
      updateStage.value = 'error'
    }
  } catch (err) {
    logger.error('Failed to update CCR:', err)
    updateError.value = err instanceof Error ? err.message : t('common.updateModal.outputUnexpectedError')
    updateStage.value = 'error'
  }
}

const handleCloseUpdateModal = () => {
  showUpdateModal.value = false
}
</script>

<template>
  <div
    class="rounded-lg p-4"
    :style="{
      background: 'var(--bg-tertiary)',
      border: '1px solid var(--border-color)'
    }"
  >
    <div class="flex items-center justify-between mb-3">
      <span
        class="text-xs font-semibold uppercase tracking-wider"
        :style="{ color: 'var(--text-secondary)' }"
      >
        {{ t('common.versionManager.title') }}
      </span>
      <SIcon
        name="Zap"
        size="w-4 h-4"
        :style="{ color: 'var(--accent-primary)' }"
      />
    </div>

    <!-- 当前版本 -->
    <div
      v-if="versionInfo"
      class="mb-3"
    >
      <div
        class="text-xs mb-1"
        :style="{ color: 'var(--text-muted)' }"
      >
        {{ t('common.versionManager.currentVersion') }}
      </div>
      <div
        class="text-2xl font-bold font-mono tracking-wide"
        :style="{ color: 'var(--accent-primary)' }"
      >
        {{ t('common.versionPrefix') }}{{ versionInfo.current }}
      </div>
    </div>

    <!-- 更新信息 -->
    <div
      v-if="updateInfo && updateInfo.update_available"
      class="mb-3 p-2.5 rounded-lg"
      :style="{
        background: 'rgba(var(--color-success-rgb), 0.1)',
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
            {{ t('common.versionManager.updateAvailable') }}
          </span>
        </div>
        <span
          class="text-sm font-bold font-mono"
          :style="{ color: 'var(--accent-success)' }"
        >
          {{ t('common.versionPrefix') }}{{ updateInfo.latest ?? updateInfo.current }}
        </span>
      </div>
    </div>

    <div
      v-if="updateInfo && !updateInfo.update_available"
      class="mb-3 text-xs text-center py-1.5 inline-flex w-full items-center justify-center gap-1.5"
      :style="{ color: 'var(--text-muted)' }"
    >
      <SIcon
        name="Check"
        size="w-3.5 h-3.5"
      />
      <span>{{ t('common.versionManager.upToDate') }}</span>
    </div>

    <!-- 操作按钮 -->
    <div class="grid grid-cols-2 gap-2">
      <button
        :disabled="isCheckingUpdate"
        class="px-3 py-2 rounded-lg font-semibold text-xs transition-transform flex items-center justify-center space-x-1.5 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
        :style="{
          background: 'var(--bg-secondary)',
          color: 'var(--text-primary)',
          border: '1px solid var(--border-color)'
        }"
        @click="handleCheckUpdate"
      >
        <SIcon
          name="RefreshCw"
          size="w-3.5 h-3.5"
          :class="['w-3.5 h-3.5', { 'animate-spin': isCheckingUpdate }]"
        />
        <span>{{ isCheckingUpdate ? t('common.versionManager.checking') : t('ccrControl.checkUpdate') }}</span>
      </button>

      <button
        class="px-3 py-2 rounded-lg font-semibold text-xs transition-transform flex items-center justify-center space-x-1.5 text-white hover:scale-105"
        :class="{ 'animate-pulse-subtle': updateInfo?.update_available }"
        :style="{
          background: updateInfo?.update_available
            ? 'linear-gradient(135deg, var(--accent-success), var(--accent-primary))'
            : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
          boxShadow: updateInfo?.update_available
            ? '0 0 20px var(--glow-success)'
            : '0 0 20px var(--glow-primary)'
        }"
        @click="handleOpenUpdateModal"
      >
        <SIcon
          name="Zap"
          size="w-3.5 h-3.5"
        />
        <span>{{ t('ccrControl.updateNow') }}</span>
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
