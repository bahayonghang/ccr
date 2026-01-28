<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
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
          :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
        >
          <!-- Header -->
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Puzzle
                class="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
              <h1
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t(`${i18nPrefix}.title`) }}
              </h1>
              <span
                class="px-3 py-1 rounded-full text-sm font-medium"
                :style="{ background: 'var(--accent-primary)', color: '#fff' }"
              >{{ plugins.length }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                :to="parentPath"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
              >
                <ArrowLeft class="w-4 h-4" /><span>{{ $t('common.back') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
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
              :style="{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }"
            />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="!plugins || plugins.length === 0"
            class="text-center py-10"
            :style="{ color: 'var(--text-muted)' }"
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
              :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(139, 92, 246, 0.12)', outline: 'none', cursor: 'default' }"
              @mouseenter="(e) => onCardHover(e.currentTarget as HTMLElement, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget as HTMLElement, false)"
            >
              <!-- Plugin Header -->
              <div class="flex items-start justify-between mb-3">
                <div class="flex-1">
                  <h3
                    class="text-lg font-bold"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ plugin.id }}
                  </h3>
                </div>
                <button
                  class="p-1 rounded transition-colors"
                  :style="{ background: 'var(--bg-secondary)' }"
                  @click="toggleExpanded(plugin)"
                >
                  <ChevronDown
                    class="w-4 h-4 transition-transform"
                    :style="{ transform: plugin._expanded ? 'rotate(180deg)' : 'rotate(0deg)', color: 'var(--text-secondary)' }"
                  />
                </button>
              </div>

              <!-- Plugin Info (expandable) -->
              <div
                v-if="plugin._expanded"
                class="mb-4"
              >
                <p
                  class="text-xs font-mono p-3 rounded overflow-auto max-h-48"
                  :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                >
                  {{ JSON.stringify(plugin.data, null, 2) }}
                </p>
              </div>

              <!-- Actions -->
              <div class="flex gap-2">
                <button
                  class="flex-1 p-2 rounded-lg transition-all hover:scale-105 flex items-center justify-center gap-1 text-sm font-medium"
                  :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--text-secondary)' }"
                  :title="$t('common.expand')"
                  @click="toggleExpanded(plugin)"
                >
                  <Eye class="w-4 h-4" />
                </button>
                <button
                  class="p-2 rounded-lg transition-all hover:scale-110"
                  :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }"
                  :title="$t('common.edit')"
                  @click="openEditForm(plugin)"
                >
                  <Edit2 class="w-4 h-4" />
                </button>
                <button
                  class="p-2 rounded-lg transition-all hover:scale-110"
                  :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }"
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
              :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }"
            >
              <h2
                class="text-xl font-bold mb-4"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ editingPlugin ? $t(`${i18nPrefix}.editPlugin`) : $t(`${i18nPrefix}.addPlugin`) }}
              </h2>

              <div class="space-y-4">
                <!-- Plugin ID -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.idLabel`) }} *</label>
                  <input
                    v-model="formData.id"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t(`${i18nPrefix}.idPlaceholder`)"
                    :disabled="!!editingPlugin"
                  >
                </div>

                <!-- Plugin Data (JSON) -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.dataLabel`) }} *</label>
                  <textarea
                    v-model="formData.data"
                    class="w-full px-3 py-2 rounded-lg font-mono text-sm"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)', minHeight: '200px' }"
                    :placeholder="$t(`${i18nPrefix}.dataPlaceholder`)"
                  />
                  <div
                    class="text-xs mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t(`${i18nPrefix}.dataHint`) }}
                  </div>
                </div>
              </div>

              <!-- Form Actions -->
              <div class="flex gap-3 mt-6">
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold text-white"
                  :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }"
                  @click="submitForm"
                >
                  {{ editingPlugin ? $t('common.save') : $t('common.add') }}
                </button>
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold"
                  :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }"
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
import { Puzzle, Plus, Edit2, Trash2, ArrowLeft, Home, Eye, ChevronDown } from 'lucide-vue-next'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import { Breadcrumb } from '@/components/ui'
import { useDroidPlugins } from '@/composables/useDroidPlugins'

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
  loadPlugins,
  deletePlugin,
  toggleExpanded,
  openAddForm,
  openEditForm,
  closeForm,
  submitForm,
} = useDroidPlugins()

// ============ Computed ============

/** 平台图标 */
const platformIcon = computed(() => Puzzle)

/** 平台显示名称 */
const platformName = computed(() => 'Droid')

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
    el.style.borderColor = 'rgba(139, 92, 246, 0.24)'
    el.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.08)'
    el.style.transform = 'translateY(-2px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(139, 92, 246, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
