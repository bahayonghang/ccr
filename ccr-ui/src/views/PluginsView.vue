<template>
  <PageShell>
    <template #header>
      <PageHeader
        :title="$t('plugins.title')"
        :description="$t('plugins.subtitle')"
      >
        <template #status>
          <span>{{ plugins.length }}</span>
        </template>
        <template #actions>
          <Button
            variant="primary"
            @click="handleAdd"
          >
            <SIcon
              name="Plus"
              size="w-5 h-5"
            />
            {{ $t('plugins.addPlugin') }}
          </Button>
        </template>
      </PageHeader>
    </template>
    <template #subnav>
      <ModuleSubnav module="claude-code" />
    </template>

          <!-- Content -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div class="w-10 h-10 rounded-full border-4 border-accent-secondary/30 border-t-accent-secondary animate-spin" />
          </div>

          <div
            v-else-if="!plugins || plugins.length === 0"
            class="plugins-empty-state py-16 text-center"
          >
            <div class="bg-bg-elevated w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <SIcon
                name="Puzzle"
                size="w-10 h-10"
                class="opacity-30 text-text-muted"
              />
            </div>
            <p class="text-lg font-bold text-text-primary">
              {{ $t('plugins.noPlugins') }}
            </p>
          </div>

          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5"
          >
            <div
              v-for="plugin in plugins"
              :key="plugin.id"
              class="plugin-card group flex flex-col rounded-2xl p-5"
            >
              <div class="flex items-start justify-between mb-3">
                <div class="flex-1 min-w-0">
                  <h3 class="text-lg font-bold text-text-primary group-hover:text-accent-secondary transition-colors truncate">
                    {{ plugin.name }}
                  </h3>
                  <p class="text-xs font-mono mt-1 text-text-muted truncate">
                    ID: {{ plugin.id }}
                  </p>
                </div>
                <span
                  v-if="!plugin.enabled"
                  class="ml-2 px-2 py-0.5 rounded text-xs font-semibold uppercase bg-accent-danger/10 text-accent-danger border border-accent-danger/20 flex-shrink-0"
                >
                  {{ $t('plugins.disabled') }}
                </span>
              </div>

              <div class="flex-1 mb-4 space-y-2">
                <p class="text-sm text-text-secondary">
                  <strong class="text-text-primary">{{ $t('plugins.version') }}</strong> {{ plugin.version }}
                </p>
                <div
                  v-if="plugin.config"
                  class="text-xs font-mono p-3 rounded-xl bg-bg-surface border border-border-default/50 overflow-auto max-h-32 text-text-secondary"
                >
                  {{ JSON.stringify(plugin.config, null, 2) }}
                </div>
              </div>

              <div class="flex gap-2 pt-3 border-t border-border-default/30">
                <button
                  class="flex-1 px-3 py-2 rounded-lg transition-[color,background-color,border-color,transform] hover:scale-105 flex items-center justify-center gap-2 text-sm font-medium border"
                  :class="plugin.enabled ? 'bg-accent-danger/10 text-accent-danger border-accent-danger/20 hover:bg-accent-danger/20' : 'bg-accent-success/10 text-accent-success border-accent-success/20 hover:bg-accent-success/20'"
                  :title="plugin.enabled ? $t('plugins.disable') : $t('plugins.enable')"
                  @click="handleToggle(plugin.id)"
                >
                  <SIcon
                    v-if="!plugin.enabled"
                    name="PowerOff"
                    size="w-4 h-4"
                  />
                  <SIcon
                    v-else
                    name="Power"
                    size="w-4 h-4"
                  />
                  <span>{{ plugin.enabled ? $t('plugins.disable') : $t('plugins.enable') }}</span>
                </button>
                <button
                  class="p-2 rounded-lg transition-[color,background-color,border-color,transform] hover:scale-110 text-text-secondary hover:text-accent-secondary hover:bg-accent-secondary/10"
                  :title="$t('plugins.edit')"
                  @click="handleEdit(plugin)"
                >
                  <SIcon
                    name="Edit2"
                    size="w-4 h-4"
                  />
                </button>
                <button
                  class="p-2 rounded-lg transition-[color,background-color,border-color,transform] hover:scale-110 text-text-secondary hover:text-accent-danger hover:bg-accent-danger/10"
                  :title="$t('plugins.delete')"
                  @click="handleDelete(plugin.id)"
                >
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </div>
          </div>

          <!-- Add/Edit Form Modal -->
          <BaseModal
            :model-value="showAddForm"
            :title="editingPlugin ? $t('plugins.editPlugin') : $t('plugins.addPlugin')"
            :description="$t('plugins.subtitle')"
            size="xl"
            surface="solid"
            content-class="plugins-editor-modal"
            @update:model-value="showAddForm = $event"
          >
            <template #header="{ titleId }">
              <h2
                :id="titleId"
                class="mb-0 flex items-center gap-3 text-2xl font-bold text-text-primary"
              >
                <div class="w-10 h-10 rounded-xl bg-accent-secondary/10 flex items-center justify-center text-accent-secondary">
                  <SIcon
                    :name="editingPlugin ? 'Edit2' : 'Plus'"
                    size="w-5 h-5"
                  />
                </div>
                {{ editingPlugin ? $t('plugins.editPlugin') : $t('plugins.addPlugin') }}
              </h2>
            </template>

            <div class="space-y-5">
              <div>
                <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                  {{ $t('plugins.form.id') }} <span class="text-accent-danger">*</span>
                </label>
                <input
                  v-model="formData.id"
                  type="text"
                  class="w-full px-4 py-3 rounded-xl bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors"
                  :placeholder="$t('plugins.form.idPlaceholder')"
                >
              </div>

              <div>
                <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                  {{ $t('plugins.form.name') }} <span class="text-accent-danger">*</span>
                </label>
                <input
                  v-model="formData.name"
                  type="text"
                  class="w-full px-4 py-3 rounded-xl bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors"
                  :placeholder="$t('plugins.form.namePlaceholder')"
                >
              </div>

              <div>
                <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                  {{ $t('plugins.form.version') }} <span class="text-accent-danger">*</span>
                </label>
                <input
                  v-model="formData.version"
                  type="text"
                  class="w-full px-4 py-3 rounded-xl bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors"
                  :placeholder="$t('plugins.form.versionPlaceholder')"
                >
              </div>

              <div>
                <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                  {{ $t('plugins.form.config') }}
                </label>
                <textarea
                  v-model="configJson"
                  rows="8"
                  class="w-full px-4 py-3 rounded-xl font-mono text-sm bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors"
                  :placeholder="$t('plugins.form.configPlaceholder')"
                />
                <div class="text-xs mt-1.5 text-text-muted">
                  {{ $t('plugins.form.configHint') }}
                </div>
              </div>

              <div class="flex items-center gap-3 p-4 rounded-xl bg-bg-elevated border border-border-default/50">
                <input
                  id="enabled"
                  v-model="formData.enabled"
                  type="checkbox"
                  class="w-5 h-5 rounded text-accent-secondary focus:ring-accent-secondary/20 border-border-default"
                >
                <label
                  for="enabled"
                  class="text-sm font-medium text-text-secondary cursor-pointer"
                >
                  {{ $t('plugins.form.enablePlugin') }}
                </label>
              </div>
            </div>

            <template #footer>
              <div class="flex w-full gap-4">
                <Button
                  variant="secondary"
                  surface="status"
                  motion="subtle"
                  class="flex-1"
                  @click="showAddForm = false"
                >
                  {{ $t('plugins.form.cancel') }}
                </Button>
                <Button
                  variant="primary"
                  surface="card"
                  motion="standard"
                  class="flex-1"
                  @click="handleSubmit"
                >
                  {{ editingPlugin ? $t('plugins.form.update') : $t('plugins.form.add') }}
                </Button>
              </div>
            </template>
          </BaseModal>

          <ConfirmModal
            v-model:is-open="showDeleteModal"
            type="danger"
            :title="$t('plugins.delete')"
            :message="deleteConfirmMessage"
            :confirm-text="$t('common.delete')"
            :cancel-text="$t('common.cancel')"
            @confirm="confirmDelete"
          />
  </PageShell>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { listPlugins, addPlugin, updatePlugin, deletePlugin, togglePlugin } from '@/api'
import type { Plugin as PluginType, PluginRequest } from '@/types'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import { logger } from '@/utils/logger'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useUIStore } from '@/stores/ui'

const { t } = useI18n({ useScope: 'global' })
const uiStore = useUIStore()

const plugins = ref<PluginType[]>([])
const loading = ref(true)
const showAddForm = ref(false)
const editingPlugin = ref<PluginType | null>(null)
const formData = ref<PluginRequest>({ id: '', name: '', version: '', enabled: true, config: undefined })
const configJson = ref('')
const showDeleteModal = ref(false)
const pluginToDelete = ref('')
const deleteConfirmMessage = computed(() => translateWithFallback(
  t,
  'plugins.deleteConfirm',
  '确定删除插件 "{id}" 吗？',
  { id: pluginToDelete.value || '' },
))

const loadPlugins = async () => {
  try {
    loading.value = true
    const data = await listPlugins()
    plugins.value = data || []
  } catch (err) {
    logger.error('Failed to load plugins:', err)
    plugins.value = []
    uiStore.showError(t('plugins.loadFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadPlugins()
})

const handleAdd = () => {
  showAddForm.value = true
  editingPlugin.value = null
  formData.value = { id: '', name: '', version: '1.0.0', enabled: true, config: undefined }
  configJson.value = ''
}

const handleEdit = (plugin: PluginType) => {
  editingPlugin.value = plugin
  showAddForm.value = true
  formData.value = { id: plugin.id, name: plugin.name, version: plugin.version, enabled: plugin.enabled, config: plugin.config }
  configJson.value = plugin.config ? JSON.stringify(plugin.config, null, 2) : ''
}

const handleSubmit = async () => {
  if (!formData.value.id || !formData.value.name || !formData.value.version) {
    uiStore.showError(t('plugins.fillRequired'))
    return
  }

  let config = undefined
  if (configJson.value.trim()) {
    try {
      config = JSON.parse(configJson.value)
    } catch (err) {
      uiStore.showError(t('plugins.configJsonError'))
      return
    }
  }

  const request: PluginRequest = { ...formData.value, config }

  try {
    if (editingPlugin.value) {
      await updatePlugin(editingPlugin.value.id, request)
      uiStore.showSuccess(t('plugins.updateSuccess'))
    } else {
      await addPlugin(request)
      uiStore.showSuccess(t('plugins.addSuccess'))
    }
    showAddForm.value = false
    await loadPlugins()
  } catch (err) {
    uiStore.showError(`${t('plugins.operationFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}

const handleDelete = async (id: string) => {
  pluginToDelete.value = id
  showDeleteModal.value = true
}

const confirmDelete = async () => {
  if (!pluginToDelete.value) return
  try {
    await deletePlugin(pluginToDelete.value)
    uiStore.showSuccess(t('plugins.deleteSuccess'))
    await loadPlugins()
  } catch (err) {
    uiStore.showError(`${t('plugins.deleteFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  } finally {
    showDeleteModal.value = false
    pluginToDelete.value = ''
  }
}

const handleToggle = async (id: string) => {
  try {
    await togglePlugin(id)
    await loadPlugins()
  } catch (err) {
    uiStore.showError(`${t('plugins.toggleFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}
</script>

<style scoped>
.plugins-header-shell {
  position: sticky;
  top: 1.5rem;
  z-index: var(--layer-sticky);
  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--elevation-2);
}

.plugins-empty-state {
  border-radius: 1.5rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 45%);
  background: var(--surface-workspace-bg);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--elevation-1);
}

.plugin-card {
  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
  transition:
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.plugin-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--elevation-2);
  border-color: rgb(var(--color-accent-secondary-rgb) / 28%);
}
</style>
