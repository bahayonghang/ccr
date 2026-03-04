<template>
  <div class="space-y-6">
    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
      导入 / 导出
    </h2>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- 导出 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          导出配置
        </h3>
        <div class="space-y-4">
          <label class="flex items-center">
            <input
              v-model="exportOptions.include_plaintext_keys"
              type="checkbox"
              class="w-4 h-4 text-blue-600 border-gray-300 rounded"
            >
            <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
              包含明文 API Key (危险)
            </span>
          </label>
          <label class="flex items-center">
            <input
              v-model="exportOptions.providers_only"
              type="checkbox"
              class="w-4 h-4 text-blue-600 border-gray-300 rounded"
            >
            <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
              仅导出提供商
            </span>
          </label>
          <button
            class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            @click="handleExport"
          >
            导出 JSON
          </button>
        </div>
      </div>

      <!-- 导入 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          导入配置
        </h3>
        <div class="space-y-4">
          <div class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-4">
            <input
              ref="importFileInput"
              type="file"
              accept=".json"
              class="hidden"
              @change="handleFileSelect"
            >
            <button
              class="w-full text-center text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
              @click="($refs.importFileInput as HTMLInputElement).click()"
            >
              点击选择 JSON 文件
            </button>
          </div>
          <div
            v-if="importPreview"
            class="text-sm text-gray-600 dark:text-gray-400"
          >
            <p>新提供商: {{ importPreview.new_providers }}</p>
            <p>新账号: {{ importPreview.new_accounts }}</p>
            <p>冲突项: {{ importPreview.conflicting_providers + importPreview.conflicting_accounts }}</p>
          </div>
          <select
            v-model="importConflictStrategy"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          >
            <option value="skip">
              跳过冲突项
            </option>
            <option value="overwrite">
              覆盖冲突项
            </option>
          </select>
          <button
            :disabled="!importData"
            class="w-full px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50"
            @click="handleImport"
          >
            执行导入
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  exportCheckinConfig,
  previewCheckinImport,
  importCheckinConfig,
} from '@/api'
import type { ExportData, ImportPreviewResponse } from '@/types/checkin'

const emit = defineEmits<{
  (e: 'refresh'): void
}>()

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
    const data = await exportCheckinConfig(exportOptions.value)
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `checkin-config-${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    alert('导出失败: ' + (e.message || '未知错误'))
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
    importPreview.value = await previewCheckinImport(data)
  } catch (e: any) {
    alert('解析文件失败: ' + (e.message || '未知错误'))
    importData.value = null
    importPreview.value = null
  }
}

// 执行导入
const handleImport = async () => {
  if (!importData.value) return

  try {
    const result = await importCheckinConfig({
      data: importData.value,
      options: { conflict_strategy: importConflictStrategy.value },
    })
    alert(`导入完成: 提供商 ${result.providers_imported} 个, 账号 ${result.accounts_imported} 个`)
    importData.value = null
    importPreview.value = null
    emit('refresh')
  } catch (e: any) {
    alert('导入失败: ' + (e.message || '未知错误'))
  }
}
</script>
