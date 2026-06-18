<!-- -->
<template>
  <div class="space-y-6">
    <h2 class="text-xl font-semibold text-text-primary">
      {{ tt('导入 / 导出', 'Import / Export') }}
    </h2>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- 导出 -->
      <div class="bg-bg-surface border border-border-default rounded-lg shadow-sm p-6">
        <h3 class="text-lg font-semibold text-text-primary mb-4">
          {{ tt('导出配置', 'Export config') }}
        </h3>
        <div class="space-y-4">
          <label class="flex items-center">
            <input
              v-model="exportOptions.include_plaintext_keys"
              type="checkbox"
              class="w-4 h-4 text-accent-primary border-border-default rounded"
            >
            <span class="ml-2 text-sm text-text-secondary">
              {{ tt('包含明文 API Key (危险)', 'Include plaintext API keys (dangerous)') }}
            </span>
          </label>
          <label class="flex items-center">
            <input
              v-model="exportOptions.providers_only"
              type="checkbox"
              class="w-4 h-4 text-accent-primary border-border-default rounded"
            >
            <span class="ml-2 text-sm text-text-secondary">
              {{ tt('仅导出提供商', 'Export providers only') }}
            </span>
          </label>
          <button
            class="w-full px-4 py-2 bg-accent-primary hover:bg-accent-primary/90 text-text-inverted rounded-lg transition-colors"
            @click="handleExport"
          >
            {{ tt('导出 JSON', 'Export JSON') }}
          </button>
        </div>
      </div>

      <!-- 导入 -->
      <div class="bg-bg-surface border border-border-default rounded-lg shadow-sm p-6">
        <h3 class="text-lg font-semibold text-text-primary mb-4">
          {{ tt('导入配置', 'Import config') }}
        </h3>
        <div class="space-y-4">
          <div class="border-2 border-dashed border-border-default rounded-lg p-4">
            <input
              ref="importFileInput"
              type="file"
              accept=".json"
              class="hidden"
              @change="handleFileSelect"
            >
            <button
              class="w-full text-center text-text-muted hover:text-text-secondary"
              @click="($refs.importFileInput as HTMLInputElement).click()"
            >
              {{ tt('点击选择 JSON 文件', 'Click to choose a JSON file') }}
            </button>
          </div>
          <div
            v-if="importPreview"
            class="text-sm text-text-secondary"
          >
            <p>{{ `${tt('新提供商', 'New providers')}: ${importPreview.new_providers}` }}</p>
            <p>{{ `${tt('新账号', 'New accounts')}: ${importPreview.new_accounts}` }}</p>
            <p>{{ `${tt('冲突项', 'Conflicts')}: ${importPreview.conflicting_providers + importPreview.conflicting_accounts}` }}</p>
          </div>
          <select
            v-model="importConflictStrategy"
            class="w-full px-3 py-2 border border-border-default rounded-lg bg-bg-surface text-text-primary"
          >
            <option value="skip">
              {{ tt('跳过冲突项', 'Skip conflicts') }}
            </option>
            <option value="overwrite">
              {{ tt('覆盖冲突项', 'Overwrite conflicts') }}
            </option>
          </select>
          <button
            :disabled="!importData"
            class="w-full px-4 py-2 bg-accent-success hover:bg-accent-success/90 text-text-inverted rounded-lg transition-colors disabled:opacity-50"
            @click="handleImport"
          >
            {{ tt('执行导入', 'Run import') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  exportCheckinConfig,
  previewCheckinImport,
  importCheckinConfig,
} from '@/api'
import type { ExportData, ImportPreviewResponse, CheckinImportResult } from '@/types/checkin'
import { getErrorMessage } from '@/types/api'
import { useUIStore } from '@/stores/ui'

const emit = defineEmits<{
  (e: 'refresh'): void
}>()

const uiStore = useUIStore()
const { locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

// 导出选项
const exportOptions = ref({
  include_plaintext_keys: false,
  providers_only: false,
})

// 导入相关
const importData = ref<ExportData | null>(null)
const importPreview = ref<ImportPreviewResponse | null>(null)
const importConflictStrategy = ref<'skip' | 'overwrite'>('skip')

// 导出操作
const handleExport = async () => {
  try {
    const data = await exportCheckinConfig<ExportData>({ ...exportOptions.value })
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `checkin-config-${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: unknown) {
    uiStore.showError(`${tt('导出失败', 'Export failed')}: ${getErrorMessage(e, tt('未知错误', 'Unknown error'))}`)
  }
}

// 文件选择
const handleFileSelect = async (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  try {
    const text = await file.text()
    const data = JSON.parse(text) as ExportData
    importData.value = data
    importPreview.value = await previewCheckinImport<ImportPreviewResponse>(data)
  } catch (e: unknown) {
    uiStore.showError(`${tt('解析文件失败', 'Failed to parse file')}: ${getErrorMessage(e, tt('未知错误', 'Unknown error'))}`)
    importData.value = null
    importPreview.value = null
  }
}

// 执行导入
const handleImport = async () => {
  if (!importData.value) return

  try {
    const result = await importCheckinConfig<CheckinImportResult>(
      importData.value,
      { conflict_strategy: importConflictStrategy.value },
    )
    uiStore.showSuccess(
      isZh.value
        ? `导入完成: 提供商 ${result.providers_imported} 个, 账号 ${result.accounts_imported} 个`
        : `Import complete: ${result.providers_imported} providers, ${result.accounts_imported} accounts`
    )
    importData.value = null
    importPreview.value = null
    emit('refresh')
  } catch (e: unknown) {
    uiStore.showError(`${tt('导入失败', 'Import failed')}: ${getErrorMessage(e, tt('未知错误', 'Unknown error'))}`)
  }
}
</script>
