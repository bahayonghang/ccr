<!-- -->
<template>
  <div class="sync-page">
    <main class="sync-shell">
      <PageHeaderCard
        :title="$t('sync.title')"
        :description="$t('sync.subtitle')"
        badge="Settings"
        icon="Cloud"
        tone="info"
      >
        <template #actions>
          <RouterLink
            to="/"
            class="sync-back-link"
          >
            <SIcon
              name="Home"
              size="w-4 h-4"
            />
            <span>{{ $t('sync.backHome') }}</span>
          </RouterLink>
        </template>
      </PageHeaderCard>

      <!-- Loading state -->
      <AsyncStatePanel
        v-if="loading"
        state="loading"
        :title="$t('common.loading')"
      />

      <!-- Error state -->
      <AsyncStatePanel
        v-else-if="error"
        state="error"
        :title="$t('sync.loadFailed')"
        :description="error"
      />

      <!-- 主要内容 -->
      <div
        v-else
        class="grid grid-cols-1 gap-6 lg:grid-cols-3"
      >
        <!-- 左侧主内容区 (2 columns) -->
        <div class="lg:col-span-2 space-y-6">
          <SyncSelectionPanel
            :add-custom-folder="addCustomFolder"
            :adding-custom="addingCustom"
            :applying="applying"
            :apply-selection="applySelection"
            :custom-folder="customFolder"
            :has-changes="hasChanges"
            :optional-items="optionalItems"
            :preset-config="presetItems.config"
            :toggle-item="toggleItem"
            :update-custom-field="updateCustomField"
            :update-optional-local-path="updateOptionalLocalPath"
            :update-optional-remote-path="updateOptionalRemotePath"
            :update-preset-local-path="updatePresetLocalPath"
          />

          <SyncEnabledFoldersPanel
            :folders="enabledFolders"
            :get-folder-status="getFolderStatus"
            :pull-folder="pullFolder"
            :push-folder="pushFolder"
            :refresh-folders="refreshFolders"
            :refreshing-folders="refreshingFolders"
            :remove-folder="removeFolder"
            :toggle-folder="toggleFolder"
          />

          <!-- Batch operations card -->
          <SyncBatchOperationsPanel
            :batch-operating="batchOperating"
            :folders-count="enabledFolderCount"
            :get-all-folders-status="getAllFoldersStatus"
            :pull-all-folders="pullAllFolders"
            :push-all-folders="pushAllFolders"
          />

          <SyncOperationOutputPanel
            :clear-output="clearOperationOutput"
            :output="operationOutput"
          />
        </div>

        <!-- 右侧信息区 (1 column) -->
        <SyncInfoSidebar
          :sync-status="syncStatus"
          @status-refresh="fetchSyncStatus"
        />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import {
  getSyncStatus,
  listSyncFolders,
  addSyncFolder,
  updateSyncFolder,
  deleteSyncFolder,
  pushSync,
  pullSync,
  pushSyncFolder,
  pullSyncFolder,
} from '@/api'
import SyncBatchOperationsPanel from '@/components/sync/SyncBatchOperationsPanel.vue'
import SyncEnabledFoldersPanel from '@/components/sync/SyncEnabledFoldersPanel.vue'
import SyncInfoSidebar from '@/components/sync/SyncInfoSidebar.vue'
import SyncOperationOutputPanel from '@/components/sync/SyncOperationOutputPanel.vue'
import SyncSelectionPanel from '@/components/sync/SyncSelectionPanel.vue'
import { logger } from '@/utils/logger'
import type { CustomSyncFolderForm, SyncManagedFolder, SyncManagedFolderRaw, SyncOperationResult, SyncSelectableItem, SyncStatusView } from '@/types/syncSelection'

const { t } = useI18n()

// 使用 Tauri invoke API


const asRecord = (value: unknown): Record<string, unknown> => {
  return typeof value === 'object' && value !== null ? (value as Record<string, unknown>) : {}
}

const toErrorMessage = (error: unknown, fallback = 'unknown error'): string => {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  const message = asRecord(asRecord(error).response).data
  if (typeof message === 'object' && message !== null && typeof (message as Record<string, unknown>).message === 'string') {
    return String((message as Record<string, unknown>).message)
  }
  return fallback
}

const normalizeManagedFolder = (entry: SyncManagedFolder | SyncManagedFolderRaw): SyncManagedFolder => {
  const raw = entry as SyncManagedFolderRaw
  return {
    name: raw.name || '',
    enabled: raw.enabled ?? true,
    description: raw.description,
    localPath: raw.localPath ?? raw.local_path ?? '',
    remotePath: raw.remotePath ?? raw.remote_path ?? '',
  }
}

const foldersEquivalent = (folder: SyncManagedFolder | undefined, item: SyncSelectableItem): boolean => {
  if (!folder) return false
  return folder.localPath === item.localPath
    && (item.remotePath.trim() === '' || folder.remotePath === item.remotePath)
    && folder.enabled
}

const formatOperationResult = (result: SyncOperationResult, fallback: string): string => {
  const output = result?.data?.output || result?.output
  if (output) return output

  const lines = [result?.message || fallback]
  if (typeof result?.total === 'number') {
    const successCount = result.successCount ?? result.success_count ?? 0
    lines.push(`${successCount}/${result.total} succeeded`)
  }
  for (const failure of result?.failed || []) {
    lines.push(`- ${failure.folder}: ${failure.message}`)
  }
  return lines.join('\n')
}

// 状态
const loading = ref(true)
const error = ref('')
const syncStatus = ref<SyncStatusView | null>(null)
const enabledFolders = ref<SyncManagedFolder[]>([])
const operationOutput = ref('')

// 操作状态
const refreshingFolders = ref(false)
const applying = ref(false)
const addingCustom = ref(false)
const batchOperating = ref(false)

// 预设项目配置
const presetItems = ref<{ config: SyncSelectableItem }>({
  config: {
    key: 'config',
    name: 'Platforms 平台配置',
    description: 'CCR 供应商配置（API地址、密钥等）',
    localPath: '~/.ccr/platforms/',
    remotePath: 'platforms',
    selected: true, // 必选
    required: true
  }
})

// 可选平台列表
const optionalItems = ref<SyncSelectableItem[]>([
  {
    key: 'claude',
    name: 'Claude Code',
    description: 'Anthropic Claude Code CLI 配置和数据',
    icon: 'Code2',
    localPath: '~/.claude/',
    remotePath: '',
    selected: false
  },
  {
    key: 'gemini',
    name: 'Antigravity CLI',
    description: 'Google Antigravity CLI 配置和数据（保留 gemini key 兼容）',
    icon: 'Cloud',
    localPath: '~/.gemini/',
    remotePath: '',
    selected: false
  }
])

// 自定义文件夹表单
const customFolder = ref<CustomSyncFolderForm>({
  name: '',
  localPath: '',
  remotePath: '',
  description: ''
})

// 计算是否有变更
const enabledFolderCount = computed(() => enabledFolders.value.filter(folder => folder.enabled).length)

const hasChanges = computed(() => {
  if (!foldersEquivalent(enabledFolders.value.find(folder => folder.name === presetItems.value.config.key), presetItems.value.config)) {
    return true
  }

  return optionalItems.value.some((item) => {
    if (!item.selected) return false
    return !foldersEquivalent(enabledFolders.value.find(folder => folder.name === item.key), item)
  })
})

// 切换选项
const toggleItem = (key: string) => {
  const item = optionalItems.value.find(i => i.key === key)
  if (item) {
    item.selected = !item.selected
  }
}

const updatePresetLocalPath = (value: string) => {
  presetItems.value.config.localPath = value
}

const updateOptionalLocalPath = (key: string, value: string) => {
  const item = optionalItems.value.find((entry) => entry.key === key)
  if (item) {
    item.localPath = value
  }
}

const updateOptionalRemotePath = (key: string, value: string) => {
  const item = optionalItems.value.find((entry) => entry.key === key)
  if (item) {
    item.remotePath = value
  }
}

const updateCustomField = (field: keyof CustomSyncFolderForm, value: string) => {
  customFolder.value[field] = value
}

const clearOperationOutput = () => {
  operationOutput.value = ''
}

// 应用选择 - 将选中的项目注册或更新为同步文件夹
const applySelection = async () => {
  applying.value = true
  const failed: string[] = []
  let appliedCount = 0

  try {
    const selectedItems = [
      presetItems.value.config,
      ...optionalItems.value.filter(item => item.selected)
    ]

    for (const item of selectedItems) {
      const existingFolder = enabledFolders.value.find(f => f.name === item.key)

      try {
        if (existingFolder) {
          await updateSyncFolder(item.key, undefined, true, item.localPath, item.remotePath || '', item.description)
        } else {
          await addSyncFolder(item.key, item.localPath, item.remotePath || '', item.description)
        }
        appliedCount += 1
      } catch (err: unknown) {
        failed.push(`${item.name}: ${toErrorMessage(err)}`)
        logger.error(`同步文件夹 ${item.name} 应用失败:`, err)
      }
    }

    await refreshFolders()
    operationOutput.value = failed.length > 0
      ? `同步配置部分应用：${appliedCount} 个成功\n${failed.map(item => `- ${item}`).join('\n')}`
      : '同步配置已应用'
  } catch (err: unknown) {
    operationOutput.value = `应用失败：${toErrorMessage(err)}`
  } finally {
    applying.value = false
  }
}

// 添加自定义文件夹
const addCustomFolder = async () => {
  if (!customFolder.value.name || !customFolder.value.localPath) return

  addingCustom.value = true
  try {
    await addSyncFolder(
      customFolder.value.name,
      customFolder.value.localPath,
      customFolder.value.remotePath || '',
      customFolder.value.description || undefined
    )
    operationOutput.value = `${t('sync.messages.addSuccess')}: ${customFolder.value.name}`
    customFolder.value = { name: '', localPath: '', remotePath: '', description: '' }
    await refreshFolders()
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.addFailed')}: ${toErrorMessage(err)}`
  } finally {
    addingCustom.value = false
  }
}

// 获取同步状态
const fetchSyncStatus = async () => {
  try {
    syncStatus.value = await getSyncStatus<SyncStatusView>()
  } catch (err: unknown) {
    logger.error('Failed to fetch sync status:', err)
  }
}

// 获取文件夹列表
const fetchFolders = async () => {
  try {
    const response = await listSyncFolders<(SyncManagedFolder | SyncManagedFolderRaw)[] | { output?: string; data?: { output?: string } }>()
    if (Array.isArray(response)) {
      enabledFolders.value = response.map(normalizeManagedFolder).filter(folder => folder.name)
      return
    }

    const output = typeof response === 'string'
      ? response
      : (response?.output || response?.data?.output || '')

    if (output) {
      parseFoldersList(output)
    } else {
      enabledFolders.value = []
    }
  } catch (err: unknown) {
    logger.error('Failed to fetch folders:', err)
    enabledFolders.value = []
  }
}

// 解析文件夹列表输出
const parseFoldersList = (output: string) => {
  try {
    // TODO: 实现完整解析逻辑
    // 目前：检查是否有文件夹输出，如果没有则设置为空数组
    if (output.includes('暂无注册的同步文件夹') || output.includes('No registered sync folders')) {
      enabledFolders.value = []
      return
    }
    
    // 如果有文件夹，这里应该解析它们（后续实现）
    // 暂时设置为空数组
    enabledFolders.value = []
  } catch (err) {
    logger.error('Failed to parse folders list:', err)
    enabledFolders.value = []
  }
}

// 刷新文件夹列表
const refreshFolders = async () => {
  refreshingFolders.value = true
  try {
    await fetchFolders()
  } finally {
    refreshingFolders.value = false
  }
}

// Delete folder
const removeFolder = async (name: string) => {
  if (!confirm(t('sync.messages.deleteConfirm', { name }))) {
    return
  }

  try {
    const result = await deleteSyncFolder<SyncOperationResult>(name)
    if (result?.success === false) {
      operationOutput.value = `${t('sync.messages.deleteFailed')}: ${result?.message || 'unknown error'}`
      return
    }

    operationOutput.value = `${t('sync.messages.deleteSuccess')}: ${name}`
    await refreshFolders()
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.deleteFailed')}: ${toErrorMessage(err)}`
  }
}

// Toggle folder status
const toggleFolder = async (name: string, currentEnabled: boolean) => {
  const actionText = currentEnabled ? t('sync.messages.disabled') : t('sync.messages.enabled')
  try {
    const result = await updateSyncFolder<SyncOperationResult>(name, undefined, !currentEnabled)
    if (result?.success === false) {
      operationOutput.value = `${t('sync.messages.toggleFailed')}: ${result?.message || 'unknown error'}`
      return
    }

    operationOutput.value = t('sync.messages.toggleSuccess', { action: actionText }) + `: ${name}`
    await refreshFolders()
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.toggleFailed')}: ${toErrorMessage(err)}`
  }
}

// Upload folder
const pushFolder = async (name: string) => {
  try {
    const result = await pushSyncFolder<SyncOperationResult>(name, false)
    operationOutput.value = `[${name}] ${formatOperationResult(result, '上传完成')}`
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.uploadFailed')}: ${toErrorMessage(err)}`
  }
}

// Download folder
const pullFolder = async (name: string) => {
  try {
    const result = await pullSyncFolder<SyncOperationResult>(name, false)
    operationOutput.value = `[${name}] ${formatOperationResult(result, '下载完成')}`
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.downloadFailed')}: ${toErrorMessage(err)}`
  }
}

// Get folder status
const getFolderStatus = async (name: string) => {
  try {
    // TODO: 当前 Tauri API 暂无按文件夹状态查询，回退为全局状态
    const status = await getSyncStatus<SyncStatusView>()
    operationOutput.value = `[${name}] ${JSON.stringify(status, null, 2)}`
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.statusFailed')}: ${toErrorMessage(err)}`
  }
}

// Batch upload
const pushAllFolders = async () => {
  batchOperating.value = true
  try {
    const result = await pushSync<SyncOperationResult>(false)
    operationOutput.value = formatOperationResult(result, '批量上传完成')
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.batchUploadFailed')}: ${toErrorMessage(err)}`
  } finally {
    batchOperating.value = false
  }
}

// Batch download
const pullAllFolders = async () => {
  batchOperating.value = true
  try {
    const result = await pullSync<SyncOperationResult>(false)
    operationOutput.value = formatOperationResult(result, '批量下载完成')
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.batchDownloadFailed')}: ${toErrorMessage(err)}`
  } finally {
    batchOperating.value = false
  }
}

// Batch view status
const getAllFoldersStatus = async () => {
  batchOperating.value = true
  try {
    const status = await getSyncStatus<SyncStatusView>()
    operationOutput.value = JSON.stringify(status, null, 2)
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.statusFailed')}: ${toErrorMessage(err)}`
  } finally {
    batchOperating.value = false
  }
}

// 初始化
onMounted(async () => {
  loading.value = true
  try {
    await Promise.all([
      fetchSyncStatus(),
      fetchFolders()
    ])
  } catch (err: unknown) {
    error.value = toErrorMessage(err, t('sync.loadFailed'))
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.sync-page {
  @apply px-4 py-4 sm:px-6 sm:py-6;
}

.sync-shell {
  @apply mx-auto flex max-w-[1440px] flex-col gap-5;
}

.sync-back-link {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/60 px-4 py-2 text-sm font-medium text-text-secondary transition-colors duration-200;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
  backdrop-filter: blur(14px);
}

.sync-back-link:hover {
  @apply border-accent-info/25 text-text-primary;

  background-color: rgb(var(--color-bg-surface-rgb) / 70%);
}
</style>
