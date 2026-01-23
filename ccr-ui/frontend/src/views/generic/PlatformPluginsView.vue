<template>
  <div :style="{ background: 'var(--color-bg-base)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="breadcrumbItems"
        :module-color="moduleColor"
      />
      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar :module="sidebarModule" />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--color-border-default)', boxShadow: 'var(--shadow-sm)' }"
        >
          <!-- Header -->
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Puzzle
                class="w-6 h-6"
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
                :style="{ background: 'var(--color-accent-primary)', color: '#fff' }"
              >{{ plugins.length }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                :to="parentPath"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--color-bg-elevated)', color: 'var(--color-text-secondary)', border: '1px solid var(--color-border-default)' }"
              >
                <ArrowLeft class="w-4 h-4" /><span>{{ $t('common.back') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                :style="{ background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
                @click="openAddForm"
              >
                <Plus class="w-4 h-4" />{{ $t(`${i18nPrefix}.addPlugin`) }}
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
              class="group rounded-lg p-5 transition-all duration-300"
              :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', outline: 'none', cursor: 'default' }"
              @mouseenter="(e) => onCardHover(e.currentTarget as HTMLElement, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget as HTMLElement, false)"
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
                  :style="{ background: 'var(--color-danger)', color: 'white' }"
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
                  class="flex-1 p-2 rounded-lg transition-all hover:scale-105 flex items-center justify-center gap-1 text-sm font-medium"
                  :style="{
                    background: plugin.enabled ? 'var(--color-bg-elevated)' : 'var(--color-success)',
                    border: '1px solid var(--color-border-default)',
                    color: plugin.enabled ? 'var(--color-text-secondary)' : 'white'
                  }"
                  :title="plugin.enabled ? $t('common.disable') : $t('common.enable')"
                  @click="togglePlugin(plugin)"
                >
                  <Power
                    v-if="plugin.enabled"
                    class="w-4 h-4"
                  />
                  <PowerOff
                    v-else
                    class="w-4 h-4"
                  />
                </button>
                <button
                  class="p-2 rounded-lg transition-all hover:scale-110"
                  :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)', color: 'var(--color-accent-primary)' }"
                  :title="$t('common.edit')"
                  @click="openEditForm(plugin)"
                >
                  <Edit2 class="w-4 h-4" />
                </button>
                <button
                  class="p-2 rounded-lg transition-all hover:scale-110"
                  :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)', color: 'var(--color-danger)' }"
                  :title="$t('common.delete')"
                  @click="deletePlugin(plugin)"
                >
                  <Trash2 class="w-4 h-4" />
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
                  class="flex-1 px-4 py-2 rounded-lg font-semibold text-white"
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
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Puzzle, Plus, Edit2, Trash2, Power, PowerOff, ArrowLeft, Home, Sparkles, Zap, Flame } from 'lucide-vue-next'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'
import { usePlatformPlugins, type PluginPlatformType } from '@/composables/usePlatformPlugins'

// ============ Props ============

interface Props {
  platform: PluginPlatformType
}

const props = defineProps<Props>()

// ============ Composable ============

const { t } = useI18n()

const {
  moduleColor,
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

/** 平台图标 */
const platformIcon = computed(() => {
  const iconMap: Record<PluginPlatformType, typeof Sparkles> = {
    gemini: Sparkles,
    qwen: Zap,
    iflow: Flame,
  }
  return iconMap[props.platform]
})

/** 平台显示名称 */
const platformName = computed(() => {
  const nameMap: Record<PluginPlatformType, string> = {
    gemini: 'Gemini CLI',
    qwen: 'Qwen',
    iflow: 'iFlow',
  }
  return nameMap[props.platform]
})

/** 面包屑导航项 */
const breadcrumbItems = computed(() => [
  { label: t('common.home'), path: '/', icon: Home },
  { label: platformName.value, path: parentPath.value, icon: platformIcon.value },
  { label: t(`${i18nPrefix.value}.title`), path: `${parentPath.value}/plugins`, icon: Puzzle },
])

// ============ Lifecycle ============

onMounted(() => {
  loadPlugins()
})

// ============ Event Handlers ============

/** 卡片悬停效果 */
function onCardHover(el: HTMLElement, hover: boolean): void {
  if (hover) {
    el.style.background = 'rgba(255, 255, 255, 0.9)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.24)'
    el.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.08)'
    el.style.transform = 'translateY(-2px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
