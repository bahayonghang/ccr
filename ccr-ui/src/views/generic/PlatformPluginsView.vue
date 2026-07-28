<template>
  <div
    class="min-h-full p-5 transition-colors duration-300"
    :style="{ background: 'var(--color-bg-base)' }"
  >
    <div class="max-w-[1800px] mx-auto">
      <div class="space-y-4">
        <ModuleSubnav :module="sidebarModule" />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--color-border-default)', boxShadow: 'var(--shadow-sm)' }"
        >
          <!-- Header -->
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <SIcon
                name="Puzzle"
                size="w-6 h-6"
                :style="{ color: 'var(--color-accent-primary)' }"
              />
              <h1
                class="text-2xl font-bold"
                :style="{ color: 'var(--color-text-primary)' }"
              >
                {{ $t(`${i18nPrefix}.title`) }}
              </h1>
              <span
                class="px-3 py-1 rounded-full text-sm font-medium"
                :style="{ background: 'var(--color-accent-primary)', color: 'var(--color-accent-primary-contrast)' }"
              >{{ plugins.length }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                :to="parentPath"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--color-bg-elevated)', color: 'var(--color-text-secondary)', border: '1px solid var(--color-border-default)' }"
              >
                <SIcon
                  name="ArrowLeft"
                  size="w-4 h-4"
                /><span>{{ $t('common.back') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-[color:var(--color-accent-primary-contrast)] flex items-center gap-2"
                :style="{ background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
                @click="openAddForm"
              >
                <SIcon
                  name="Plus"
                  size="w-4 h-4"
                />{{ $t(`${i18nPrefix}.addPlugin`) }}
              </button>
            </div>
          </div>

          <!-- Loading State -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div
              class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
              :style="{ borderTopColor: 'var(--color-accent-primary)', borderRightColor: 'var(--color-accent-secondary)' }"
            />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="!plugins || plugins.length === 0"
            class="text-center py-10"
            :style="{ color: 'var(--color-text-muted)' }"
          >
            {{ $t(`${i18nPrefix}.emptyState`) }}
          </div>

          <!-- Plugin Grid -->
          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
          >
            <div
              v-for="plugin in plugins"
              :key="plugin.id"
              class="platform-card group rounded-lg p-5"
            >
              <!-- Plugin Header -->
              <div class="flex items-start justify-between mb-3">
                <div class="flex-1">
                  <h3
                    class="text-lg font-bold"
                    :style="{ color: 'var(--color-text-primary)' }"
                  >
                    {{ plugin.name }}
                  </h3>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--color-text-muted)' }"
                  >
                    ID: {{ plugin.id }}
                  </p>
                </div>
                <span
                  v-if="!plugin.enabled"
                  class="px-2 py-0.5 rounded text-xs font-semibold uppercase"
                  :style="{ background: 'var(--color-danger)', color: 'var(--color-danger-contrast)' }"
                >{{ $t(`${i18nPrefix}.disabledBadge`) }}</span>
              </div>

              <!-- Plugin Info -->
              <div class="mb-4">
                <p
                  class="text-sm"
                  :style="{ color: 'var(--color-text-secondary)' }"
                >
                  <strong>{{ $t('common.version') }}:</strong> {{ plugin.version }}
                </p>
                <p
                  v-if="plugin.config"
                  class="text-xs font-mono mt-2 p-2 rounded overflow-auto max-h-24"
                  :style="{ background: 'var(--color-bg-elevated)', color: 'var(--color-text-primary)' }"
                >
                  {{ JSON.stringify(plugin.config, null, 2) }}
                </p>
              </div>

              <!-- Actions -->
              <div class="flex gap-2">
                <button
                  class="flex-1 p-2 rounded-lg transition-transform hover:scale-105 flex items-center justify-center gap-1 text-sm font-medium"
                  :style="{
                    background: plugin.enabled ? 'var(--color-bg-elevated)' : 'var(--color-success)',
                    border: '1px solid var(--color-border-default)',
                    color: plugin.enabled ? 'var(--color-text-secondary)' : 'white'
                  }"
                  :title="plugin.enabled ? $t('common.disable') : $t('common.enable')"
                  @click="togglePlugin(plugin)"
                >
                  <SIcon
                    v-if="plugin.enabled"
                    name="Power"
                    size="w-4 h-4"
                  />
                  <SIcon
                    v-else
                    name="PowerOff"
                    size="w-4 h-4"
                  />
                </button>
                <button
                  class="p-2 rounded-lg transition-transform hover:scale-110"
                  :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)', color: 'var(--color-accent-primary)' }"
                  :title="$t('common.edit')"
                  @click="openEditForm(plugin)"
                >
                  <SIcon
                    name="Edit2"
                    size="w-4 h-4"
                  />
                </button>
                <button
                  class="p-2 rounded-lg transition-transform hover:scale-110"
                  :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)', color: 'var(--color-danger)' }"
                  :title="$t('common.delete')"
                  @click="handleDeletePlugin(plugin)"
                >
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </div>
          </div>

          <!-- Add/Edit Modal -->
          <div
            v-if="showForm"
            class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
          >
            <div
              class="rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto"
              :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)' }"
            >
              <h2
                class="text-xl font-bold mb-4"
                :style="{ color: 'var(--color-text-primary)' }"
              >
                {{ editingPlugin ? $t(`${i18nPrefix}.editPlugin`) : $t(`${i18nPrefix}.addPlugin`) }}
              </h2>

              <div class="space-y-4">
                <!-- Plugin ID -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--color-text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.idLabel`) }} *</label>
                  <input
                    v-model="formData.id"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
                    :placeholder="$t(`${i18nPrefix}.idPlaceholder`)"
                    :disabled="!!editingPlugin"
                  >
                </div>

                <!-- Plugin Name -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--color-text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.nameLabel`) }} *</label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg"
                    :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
                    :placeholder="$t(`${i18nPrefix}.namePlaceholder`)"
                  >
                </div>

                <!-- Plugin Version -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--color-text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.versionLabel`) }} *</label>
                  <input
                    v-model="formData.version"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
                    placeholder="1.0.0"
                  >
                </div>

                <!-- Plugin Enabled -->
                <div>
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input
                      v-model="formData.enabled"
                      type="checkbox"
                      class="w-4 h-4"
                    >
                    <span
                      class="text-sm font-semibold"
                      :style="{ color: 'var(--color-text-secondary)' }"
                    >{{ $t(`${i18nPrefix}.enabledLabel`) }}</span>
                  </label>
                </div>

                <!-- Plugin Config (JSON) -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--color-text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.configLabel`) }}</label>
                  <textarea
                    v-model="configJson"
                    class="w-full px-3 py-2 rounded-lg font-mono text-sm"
                    :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)', minHeight: '120px' }"
                    :placeholder="$t(`${i18nPrefix}.configPlaceholder`)"
                  />
                  <div
                    class="text-xs mt-1"
                    :style="{ color: 'var(--color-text-muted)' }"
                  >
                    {{ $t(`${i18nPrefix}.configHint`) }}
                  </div>
                </div>
              </div>

              <!-- Form Actions -->
              <div class="flex gap-3 mt-6">
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold text-[color:var(--color-accent-primary-contrast)]"
                  :style="{ background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))' }"
                  @click="submitForm"
                >
                  {{ editingPlugin ? $t('common.save') : $t('common.add') }}
                </button>
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold"
                  :style="{ background: 'var(--color-bg-surface)', color: 'var(--color-text-primary)', border: '1px solid var(--color-border-default)' }"
                  @click="closeForm"
                >
                  {{ $t('common.cancel') }}
                </button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import { useUIStore } from '@/stores/ui'
import { usePlatformPlugins, type PluginPlatformType } from '@/composables/usePlatformPlugins'
import type { Plugin } from '@/types'

// ============ Props ============

interface Props {
  platform: PluginPlatformType
}

const props = defineProps<Props>()

const { t } = useI18n()
const uiStore = useUIStore()

// ============ Composable ============

const {
  i18nPrefix,
  parentPath,
  sidebarModule,
  plugins,
  loading,
  showForm,
  editingPlugin,
  formData,
  configJson,
  loadPlugins,
  deletePlugin,
  togglePlugin,
  openAddForm,
  openEditForm,
  closeForm,
  submitForm,
} = usePlatformPlugins(props.platform)

// ============ Computed ============

// ============ Lifecycle ============

onMounted(() => {
  loadPlugins()
})

// ============ Event Handlers ============

/** 删除插件：先弹全局确认框，确认后才调用 composable 的纯执行器 */
async function handleDeletePlugin(plugin: Plugin): Promise<void> {
  const name = plugin.name || plugin.id
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t(`${i18nPrefix.value}.deleteConfirm`, { name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (confirmed) await deletePlugin(plugin)
}
</script>

<style scoped>
.platform-card {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-default);
  cursor: default;
  transition:
    background-color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

.platform-card:hover {
  background: var(--color-bg-overlay);
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}
</style>
