<template>
  <div class="unified-mcp-view">
    <UnifiedMcpCommandBar
      :all-platforms="ALL_PLATFORMS"
      :filter-keyword="filterKeyword"
      :filter-platform="filterPlatform"
      :filter-protocol="filterProtocol"
      :loading="loading"
      :platform-counts="platformCounts"
      :platform-meta="PLATFORM_META"
      :protocol-options="protocolOptions"
      :server-count="servers.length"
      @open-add="openAddForm()"
      @refresh="loadServers()"
      @update:filter-keyword="filterKeyword = $event"
      @update:filter-platform="filterPlatform = $event"
      @update:filter-protocol="filterProtocol = $event"
    />

    <UnifiedMcpServerGrid
      :error="error"
      :filtered-servers="filteredServers"
      :get-platform-color="getPlatformColor"
      :get-platform-label="getPlatformLabel"
      :has-active-filters="hasActiveFilters"
      :loading="loading"
      :servers="servers"
      :supports-feature="supportsFeature"
      @delete="handleDelete"
      @edit="openEditForm"
      @retry="loadServers()"
      @toggle="toggleServer"
    />

    <UnifiedMcpFormModal
      :show="showForm"
      :editing-server="editingServer"
      :all-platforms="ALL_PLATFORMS"
      :platform-meta="PLATFORM_META"
      :is-http-mode="isHttpMode"
      :form-data="formData"
      :arg-input="argInput"
      :env-key="envKey"
      :env-value="envValue"
      :header-key="headerKey"
      :header-value="headerValue"
      :include-tool-input="includeToolInput"
      :current-capability="currentCapability"
      :close-form="closeForm"
      :submit-form="submitForm"
      :set-http-mode="setHttpMode"
      :update-form-field="updateFormField"
      :update-arg-input="updateArgInput"
      :update-env-key="updateEnvKey"
      :update-env-value="updateEnvValue"
      :update-header-key="updateHeaderKey"
      :update-header-value="updateHeaderValue"
      :update-include-tool-input="updateIncludeToolInput"
      :add-env-var="addEnvVar"
      :remove-env-var="removeEnvVar"
      :add-header="addHeader"
      :remove-header="removeHeader"
    />

    <UnifiedMcpDeleteConfirmModal
      :show="showDeleteConfirm"
      :platform-label="getPlatformLabel(deletingServer?.platform ?? '')"
      :server-name="deletingServer?.name ?? ''"
      :close="closeDeleteConfirm"
      :confirm="confirmDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import UnifiedMcpCommandBar from '@/components/mcp/UnifiedMcpCommandBar.vue'
import UnifiedMcpDeleteConfirmModal from '@/components/mcp/UnifiedMcpDeleteConfirmModal.vue'
import UnifiedMcpFormModal from '@/components/mcp/UnifiedMcpFormModal.vue'
import UnifiedMcpServerGrid from '@/components/mcp/UnifiedMcpServerGrid.vue'
import { useUnifiedMcp } from '@/composables/useUnifiedMcp'
import type { UnifiedMcpRequest, UnifiedMcpServer, UnifiedMcpPlatform } from '@/types/unifiedMcp'

const {
  PLATFORM_META,
  ALL_PLATFORMS,
  servers,
  loading,
  error,
  filterPlatform,
  filterKeyword,
  filterProtocol,
  filteredServers,
  platformCounts,
  hasActiveFilters,
  showForm,
  editingServer,
  isHttpMode,
  formData,
  argInput,
  envKey,
  envValue,
  headerKey,
  headerValue,
  includeToolInput,
  currentCapability,
  loadServers,
  toggleServer,
  deleteServer,
  openAddForm,
  openEditForm,
  closeForm,
  submitForm,
  addEnvVar,
  removeEnvVar,
  addHeader,
  removeHeader,
  supportsFeature,
} = useUnifiedMcp()

// 删除确认
const showDeleteConfirm = ref(false)
const deletingServer = ref<UnifiedMcpServer | null>(null)

function handleDelete(server: UnifiedMcpServer) {
  deletingServer.value = server
  showDeleteConfirm.value = true
}

function closeDeleteConfirm() {
  showDeleteConfirm.value = false
}

async function confirmDelete() {
  if (deletingServer.value) {
    await deleteServer(deletingServer.value)
    closeDeleteConfirm()
    deletingServer.value = null
  }
}

function updateFormField(field: keyof UnifiedMcpRequest, value: unknown) {
  ;(formData.value as Record<keyof UnifiedMcpRequest, unknown>)[field] = value
}

function setHttpMode(value: boolean) {
  isHttpMode.value = value
}

function updateArgInput(value: string) {
  argInput.value = value
}

function updateEnvKey(value: string) {
  envKey.value = value
}

function updateEnvValue(value: string) {
  envValue.value = value
}

function updateHeaderKey(value: string) {
  headerKey.value = value
}

function updateHeaderValue(value: string) {
  headerValue.value = value
}

function updateIncludeToolInput(value: string) {
  includeToolInput.value = value
}

// 协议过滤选项
const protocolOptions = [
  { value: 'all' as const, label: '全部' },
  { value: 'stdio' as const, label: 'STDIO' },
  { value: 'http' as const, label: 'HTTP' },
]

// 辅助函数
function getPlatformColor(platform: string): string {
  return PLATFORM_META[platform as UnifiedMcpPlatform]?.color ?? '#6b7280'
}

function getPlatformLabel(platform: string): string {
  return PLATFORM_META[platform as UnifiedMcpPlatform]?.label ?? platform
}

onMounted(() => {
  loadServers()
})
</script>

<style scoped>
/* ============ Layout ============ */
.unified-mcp-view {
  padding: var(--space-5) var(--space-6);
  max-width: 1600px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

@media (width <= 640px) {
  .unified-mcp-view {
    padding: var(--space-3);
  }
}
</style>
