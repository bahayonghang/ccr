<template>
  <div class="mcp-manager-view">
    <MasterDetailLayout list-width="20rem">
      <template #list>
        <McpListPanel
          :groups="filteredGroups"
          :search-query="searchQuery"
          :selected-keys="effectiveSelectedKeys"
          :is-multi-select-mode="isMultiSelectMode"
          :loading="loading"
          @update:search-query="searchQuery = $event"
          @select="selectGroup"
          @create="openCreate"
          @import="openImport"
          @refresh="refresh"
          @toggle-multi-select="toggleMultiSelect"
          @bulk-delete="handleBulkDelete"
        />
      </template>

      <template #detail>
        <!-- 创建面板 -->
        <McpCreatePanel
          v-if="panelMode.type === 'create'"
          :is-editing="false"
          :form-data="formData"
          :is-http-mode="isHttpMode"
          :arg-input="argInput"
          :env-key="envKey"
          :env-value="envValue"
          :header-key="headerKey"
          :header-value="headerValue"
          :platforms="ALL_PLATFORMS"
          :platform-meta="PLATFORM_META"
          @submit="handleSubmit"
          @cancel="closePanel"
          @update:is-http-mode="isHttpMode = $event"
          @update:arg-input="argInput = $event"
          @update:env-key="envKey = $event"
          @update:env-value="envValue = $event"
          @update:header-key="headerKey = $event"
          @update:header-value="headerValue = $event"
          @update-field="handleUpdateField"
          @add-env="addEnvVar"
          @remove-env="removeEnvVar"
          @add-header="addHeader"
          @remove-header="removeHeader"
        />

        <!-- 编辑面板 -->
        <McpCreatePanel
          v-else-if="panelMode.type === 'edit'"
          :is-editing="true"
          :form-data="formData"
          :is-http-mode="isHttpMode"
          :arg-input="argInput"
          :env-key="envKey"
          :env-value="envValue"
          :header-key="headerKey"
          :header-value="headerValue"
          :platforms="ALL_PLATFORMS"
          :platform-meta="PLATFORM_META"
          @submit="handleSubmit"
          @cancel="closePanel"
          @update:is-http-mode="isHttpMode = $event"
          @update:arg-input="argInput = $event"
          @update:env-key="envKey = $event"
          @update:env-value="envValue = $event"
          @update:header-key="headerKey = $event"
          @update:header-value="headerValue = $event"
          @update-field="handleUpdateField"
          @add-env="addEnvVar"
          @remove-env="removeEnvVar"
          @add-header="addHeader"
          @remove-header="removeHeader"
        />

        <!-- 导入面板 -->
        <McpImportPanel
          v-else-if="panelMode.type === 'import'"
          :platforms="ALL_PLATFORMS"
          :platform-meta="PLATFORM_META"
          @cancel="closePanel"
          @import="handleImportServers"
        />

        <!-- 详情面板 (默认) -->
        <McpDetailPanel
          v-else
          :group="activeGroup"
          @edit="openEdit"
          @delete="handleDeleteGroup"
          @toggle="handleToggle"
        />
      </template>
    </MasterDetailLayout>

    <!-- 批量删除确认 -->
    <BulkDeleteDialog
      :is-open="showBulkDeleteDialog"
      :items="bulkDeleteItems"
      resource-label="MCP server"
      :loading="bulkDeleting"
      @confirm="confirmBulkDelete"
      @cancel="showBulkDeleteDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import MasterDetailLayout from '@/components/common/MasterDetailLayout.vue'
import BulkDeleteDialog from '@/components/common/BulkDeleteDialog.vue'
import type { BulkDeleteItem } from '@/components/common/BulkDeleteDialog.vue'
import McpListPanel from '@/components/mcp/McpListPanel.vue'
import McpDetailPanel from '@/components/mcp/McpDetailPanel.vue'
import McpCreatePanel from '@/components/mcp/McpCreatePanel.vue'
import McpImportPanel from '@/components/mcp/McpImportPanel.vue'
import { useMcpManager } from '@/composables/useMcpManager'
import { useUIStore } from '@/stores/ui'
import type { McpGroup } from '@/types/mcpManager'
import type { UnifiedMcpRequest, UnifiedMcpServer } from '@/types/unifiedMcp'

const uiStore = useUIStore()

const {
  loading,
  formData,
  isHttpMode,
  argInput,
  envKey,
  envValue,
  headerKey,
  headerValue,
  PLATFORM_META,
  ALL_PLATFORMS,
  panelMode,
  searchQuery,
  filteredGroups,
  activeGroup,
  effectiveSelectedKeys,
  selectedGroups,
  isMultiSelectMode,
  selectGroup,
  openCreate,
  openImport,
  openEdit,
  closePanel,
  toggleMultiSelect,
  bulkDelete,
  deleteGroup,
  refresh,
  submitForm,
  addEnvVar,
  removeEnvVar,
  addHeader,
  removeHeader,
  toggleServer,
} = useMcpManager()

// 批量删除状态
const showBulkDeleteDialog = ref(false)
const bulkDeleting = ref(false)

const bulkDeleteItems = computed<BulkDeleteItem[]>(() =>
  selectedGroups.value.map(g => ({
    key: g.name,
    label: g.name,
    badge: `${g.platforms.length} agent(s)`,
  })),
)

function handleUpdateField(field: keyof UnifiedMcpRequest, value: unknown) {
  ;(formData.value as Record<keyof UnifiedMcpRequest, unknown>)[field] = value
}

async function handleSubmit() {
  const success = await submitForm()
  if (success) closePanel()
}

function handleBulkDelete() {
  showBulkDeleteDialog.value = true
}

async function confirmBulkDelete() {
  bulkDeleting.value = true
  try {
    await bulkDelete()
    showBulkDeleteDialog.value = false
    uiStore.showSuccess('Deleted selected servers')
  } catch (err) {
    uiStore.showError(err instanceof Error ? err.message : String(err))
  } finally {
    bulkDeleting.value = false
  }
}

async function handleDeleteGroup(group: McpGroup) {
  try {
    await deleteGroup(group)
    uiStore.showSuccess(`Deleted ${group.name}`)
  } catch (err) {
    uiStore.showError(err instanceof Error ? err.message : String(err))
  }
}

async function handleToggle(server: UnifiedMcpServer) {
  await toggleServer(server)
}

async function handleImportServers(
  servers: Array<{ name: string; type: string; command?: string; args?: string[]; url?: string; env?: Record<string, string>; headers?: Record<string, string> }>,
  platform: string,
) {
  // TODO: 批量导入 — 逐个调用 addUnifiedMcp
  uiStore.showSuccess(`Imported ${servers.length} server(s) to ${platform}`)
  closePanel()
  await refresh()
}
</script>

<style scoped>
.mcp-manager-view {
  height: 100%;
  overflow: hidden;
}
</style>
