<template>
  <PageShell class="mcp-manager-view">
    <template #header>
      <PageHeader
        :title="t('mcp.manager.hero.title')"
        :description="t('mcp.manager.hero.subtitle')"
      >
        <template #actions>
          <button
            type="button"
            class="mcp-action mcp-action--ghost"
            @click="refresh"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': loading }"
            />
            {{ t('common.refresh') }}
          </button>
          <button
            type="button"
            class="mcp-action mcp-action--ghost"
            @click="openImport"
          >
            <SIcon
              name="Download"
              size="w-4 h-4"
            />
            {{ t('common.import') }}
          </button>
          <button
            type="button"
            class="mcp-action mcp-action--primary"
            @click="openCreate"
          >
            <SIcon
              name="Plus"
              size="w-4 h-4"
            />
            {{ t('mcp.manager.actions.addServer') }}
          </button>
        </template>
      </PageHeader>
    </template>

    <details
      class="mcp-presets-drawer"
      @toggle="showPresetsDrawer = ($event.target as HTMLDetailsElement).open"
    >
      <summary>
        <span>{{ t('mcp.manager.presetsDrawer.title') }}</span>
        <em>{{ t('mcp.manager.presetsDrawer.subtitle') }}</em>
      </summary>
      <div
        v-if="showPresetsDrawer"
        class="mcp-presets-drawer__content"
      >
        <McpPresetsPanel @installed="refresh" />
        <McpSyncPanel @synced="refresh" />
      </div>
    </details>

    <PillToggleGroup
      class="mcp-scope-rail"
      :options="scopeToggleOptions"
      :model-value="filterScope"
      @update:model-value="filterScope = $event"
    />

    <div
      v-if="error"
      class="mcp-alert mcp-alert--error"
    >
      <SIcon
        name="AlertTriangle"
        size="w-4 h-4"
      />
      {{ error }}
    </div>

    <MasterDetailLayout list-width="23rem">
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

        <McpImportPanel
          v-else-if="panelMode.type === 'import'"
          :platforms="ALL_PLATFORMS"
          :platform-meta="PLATFORM_META"
          @cancel="closePanel"
          @import="handleImportServers"
        />

        <McpDetailPanel
          v-else
          :group="activeGroup"
          :diagnostics="sourceDiagnostics"
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
      :title="t('mcp.manager.bulkDelete.title')"
      :message="bulkDeleteMessage"
      :cancel-label="t('common.cancel')"
      :confirm-label="t('mcp.manager.bulkDelete.confirm', { count: bulkDeleteItems.length })"
      :overflow-message="bulkDeleteOverflowMessage"
      :loading="bulkDeleting"
      @confirm="confirmBulkDelete"
      @cancel="showBulkDeleteDialog = false"
    />
  </PageShell>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { getErrorMessage } from '@/utils/errorHandler'
import { useI18n } from 'vue-i18n'
import MasterDetailLayout from '@/components/common/MasterDetailLayout.vue'
import BulkDeleteDialog from '@/components/common/BulkDeleteDialog.vue'
import type { BulkDeleteItem } from '@/components/common/BulkDeleteDialog.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import PillToggleGroup from '@/components/ui/PillToggleGroup.vue'
import SIcon from '@/components/ui/SIcon.vue'
import McpListPanel from '@/components/mcp/McpListPanel.vue'
import McpDetailPanel from '@/components/mcp/McpDetailPanel.vue'
import McpCreatePanel from '@/components/mcp/McpCreatePanel.vue'
import McpImportPanel from '@/components/mcp/McpImportPanel.vue'
import McpPresetsPanel from '@/components/McpPresetsPanel.vue'
import McpSyncPanel from '@/components/McpSyncPanel.vue'
import { importUnifiedMcpServers } from '@/api'
import { useMcpManager } from '@/composables/useMcpManager'
import { useUIStore } from '@/stores/ui'
import type { McpGroup } from '@/types/mcpManager'
import type { McpScopeFilter, UnifiedMcpRequest, UnifiedMcpServer } from '@/types/unifiedMcp'

const uiStore = useUIStore()
const { t } = useI18n({ useScope: 'global' })
const showPresetsDrawer = ref(false)

const {
  loading,
  error,
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
  filterScope,
  scopeCounts,
  sourceDiagnostics,
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

const scopeFilterOptions: Array<{ value: McpScopeFilter; labelKey: string }> = [
  { value: 'effective', labelKey: 'mcp.manager.scopes.effective' },
  { value: 'local', labelKey: 'mcp.manager.scopes.local' },
  { value: 'project', labelKey: 'mcp.manager.scopes.project' },
  { value: 'user', labelKey: 'mcp.manager.scopes.user' },
  { value: 'hidden', labelKey: 'mcp.manager.scopes.hidden' },
]
const scopeToggleOptions = computed(() =>
  scopeFilterOptions.map((scope) => ({
    value: scope.value,
    label: `${t(scope.labelKey)} ${scopeCounts.value[scope.value]}`,
  })),
)

// 批量删除状态
const showBulkDeleteDialog = ref(false)
const bulkDeleting = ref(false)

const bulkDeleteItems = computed<BulkDeleteItem[]>(() =>
  selectedGroups.value.map(g => ({
    key: g.name,
    label: g.name,
    badge: t('mcp.manager.bulkDelete.badge', { count: g.platforms.length }),
  })),
)

const bulkDeleteMessage = computed(() =>
  t('mcp.manager.bulkDelete.message', { count: bulkDeleteItems.value.length }),
)

const bulkDeleteOverflowMessage = computed(() =>
  t('mcp.manager.bulkDelete.overflow', { count: Math.max(bulkDeleteItems.value.length - 10, 0) }),
)

function handleUpdateField(field: keyof UnifiedMcpRequest, value: unknown) {
  ;(formData.value as Record<keyof UnifiedMcpRequest, unknown>)[field] = value
}

async function handleSubmit() {
  if (formData.value.scope === 'project') {
    const confirmed = await uiStore.requestConfirm({
      title: t('common.warning'),
      message: t('mcp.manager.confirm.projectScopeWrite'),
      confirmText: t('common.confirm'),
      cancelText: t('common.cancel'),
      type: 'warning',
    })
    if (!confirmed) return
  }
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
    uiStore.showSuccess(t('mcp.manager.messages.deletedSelected'))
  } catch (err) {
    uiStore.showError(getErrorMessage(err))
  } finally {
    bulkDeleting.value = false
  }
}

async function handleDeleteGroup(group: McpGroup) {
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t('mcp.manager.confirm.deleteGroup', { count: group.items.length, name: group.name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) return

  try {
    await deleteGroup(group)
    uiStore.showSuccess(t('mcp.manager.messages.deletedServer', { name: group.name }))
  } catch (err) {
    uiStore.showError(getErrorMessage(err))
  }
}

async function handleToggle(server: UnifiedMcpServer) {
  await toggleServer(server)
}

async function handleImportServers(
  servers: Array<{ name: string; type: string; command?: string; args?: string[]; url?: string; env?: Record<string, string>; headers?: Record<string, string> }>,
  platform: string,
  scope?: string,
) {
  if (scope === 'project') {
    const confirmed = await uiStore.requestConfirm({
      title: t('common.warning'),
      message: t('mcp.manager.confirm.projectScopeImport'),
      confirmText: t('common.confirm'),
      cancelText: t('common.cancel'),
      type: 'warning',
    })
    if (!confirmed) return
  }

  const requests = servers.map(server => ({
    platform,
    scope,
    name: server.name,
    command: server.command ?? null,
    args: server.args ?? [],
    url: server.url ?? null,
    env: server.env ?? {},
    headers: server.headers ?? {},
    disabled: false,
  }))
  const results = await importUnifiedMcpServers(requests)
  const failed = results.filter(result => !result.ok)
  if (failed.length > 0) {
    uiStore.showError(t('mcp.manager.messages.importPartialFailed', {
      success: results.length - failed.length,
      total: results.length,
      failures: failed.map(item =>
        `${item.name}: ${item.error ?? t('mcp.manager.messages.unknownError')}`,
      ).join('; '),
    }))
  } else {
    uiStore.showSuccess(t('mcp.manager.messages.importSuccess', {
      count: results.length,
      platform: PLATFORM_META[platform as keyof typeof PLATFORM_META]?.label ?? platform,
    }))
  }
  closePanel()
  await refresh()
}
</script>

<style scoped>
.mcp-manager-view {
  min-width: 0;
}

.mcp-presets-drawer {
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 36%);
  padding: 0 1.75rem;
}

.mcp-presets-drawer summary {
  display: flex;
  align-items: baseline;
  gap: 0.65rem;
  padding: 0.85rem 0;
  color: var(--color-text-secondary);
  cursor: pointer;
  list-style: none;
}

.mcp-presets-drawer summary::-webkit-details-marker {
  display: none;
}

.mcp-presets-drawer summary span {
  color: var(--color-text-primary);
  font-size: 0.8rem;
  font-weight: 600;
  letter-spacing: 0;
}

.mcp-presets-drawer summary em {
  color: var(--color-text-muted);
  font-size: 0.76rem;
  font-style: normal;
}

.mcp-presets-drawer__content {
  max-height: 32rem;
  overflow-y: auto;
  padding: 0.4rem 0 1rem;
}

.mcp-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 2.25rem;
  border-radius: 999px;
  padding: 0 0.85rem;
  font-size: 0.8rem;
  font-weight: 650;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.mcp-action:hover {
  transform: translateY(-1px);
}

.mcp-action--ghost {
  border: 1px solid rgb(var(--color-border-default-rgb) / 48%);
  background: rgb(var(--color-bg-surface-rgb) / 56%);
  color: var(--color-text-secondary);
}

.mcp-action--primary {
  border: 1px solid transparent;
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
}

.mcp-scope-rail {
  display: flex;
  gap: 0.5rem;
}

.mcp-scope-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.38rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 650;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.mcp-scope-chip strong {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 0.68rem;
}

.mcp-scope-chip--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 34%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-text-primary);
}

.mcp-alert {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.75rem 1.75rem 0;
  border-radius: 0.85rem;
  padding: 0.7rem 0.85rem;
  font-size: 0.82rem;
}

.mcp-alert--error {
  border: 1px solid rgb(var(--color-danger-rgb) / 26%);
  background: rgb(var(--color-danger-rgb) / 8%);
  color: var(--color-danger);
}

.mcp-manager-view :deep(.master-detail) {
  flex: 1;
  min-height: 0;
}


</style>
