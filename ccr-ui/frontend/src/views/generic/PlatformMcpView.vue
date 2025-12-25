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
              <Server
                class="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
              <h1
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t(`${i18nPrefix}.pageTitle`) }}
              </h1>
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
                <Plus class="w-4 h-4" />{{ $t(`${i18nPrefix}.addServer`) }}
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

          <!-- Server List -->
          <div
            v-else
            class="space-y-3"
          >
            <!-- Empty State -->
            <div
              v-if="!servers || servers.length === 0"
              class="text-center py-10"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t(`${i18nPrefix}.emptyState`) }}
            </div>

            <!-- Server Cards -->
            <div
              v-for="server in servers"
              :key="getServerIdentifier(server)"
              class="group rounded-lg p-4 transition-all duration-300"
              :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(99, 102, 241, 0.12)', outline: 'none', cursor: 'default' }"
              @mouseenter="(e) => onCardHover(e.currentTarget as HTMLElement, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget as HTMLElement, false)"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-2">
                    <h3
                      class="text-lg font-bold font-mono"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ server.name || server.command || server.url }}
                    </h3>
                    <span
                      v-if="server.url"
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-secondary)', color: 'white' }"
                    >HTTP</span>
                    <span
                      v-else
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-primary)', color: 'white' }"
                    >STDIO</span>
                  </div>
                  <div class="space-y-2 text-sm">
                    <div v-if="server.command">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('common.command') }}:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >{{ server.command }}</code>
                    </div>
                    <div v-if="server.url">
                      <span :style="{ color: 'var(--text-muted)' }">URL:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >{{ server.url }}</code>
                    </div>
                    <div v-if="server.args && server.args.length > 0">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('common.args') }}:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                      >{{ server.args.join(' ') }}</code>
                    </div>
                    <div v-if="server.env && Object.keys(server.env).length > 0">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('common.envVars') }}:</span>
                      <div class="ml-2 mt-1 space-y-1">
                        <div
                          v-for="[key, value] in Object.entries(server.env)"
                          :key="key"
                          class="text-xs font-mono px-2 py-1 rounded"
                          :style="{ background: 'var(--bg-secondary)' }"
                        >
                          <span :style="{ color: 'var(--accent-secondary)' }">{{ key }}</span>=<span :style="{ color: 'var(--text-primary)' }">{{ value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }"
                    :title="$t('common.edit')"
                    @click="openEditForm(server)"
                  >
                    <Edit2 class="w-4 h-4" />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }"
                    :title="$t('common.delete')"
                    @click="deleteServer(server)"
                  >
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
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
                {{ editingServer ? $t(`${i18nPrefix}.editServer`) : $t(`${i18nPrefix}.addServer`) }}
              </h2>

              <!-- Server Type Toggle -->
              <div class="mb-4">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    v-model="isHttpServer"
                    type="checkbox"
                    class="w-4 h-4"
                  >
                  <span
                    class="text-sm font-semibold"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.httpServerHint`) }}</span>
                </label>
              </div>

              <div class="space-y-4">
                <!-- URL (HTTP Server) -->
                <div v-if="isHttpServer">
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.urlLabel`) }} *</label>
                  <input
                    v-model="formData.url"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t(`${i18nPrefix}.urlPlaceholder`)"
                  >
                </div>

                <!-- Command (STDIO Server) -->
                <div v-else>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.commandLabel`) }} *</label>
                  <input
                    v-model="formData.command"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t(`${i18nPrefix}.commandPlaceholder`)"
                  >
                </div>

                <!-- Args (STDIO Server) -->
                <div v-if="!isHttpServer">
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.argsLabel`) }}</label>
                  <input
                    v-model="argInput"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t(`${i18nPrefix}.argsPlaceholder`)"
                  >
                  <div
                    class="text-xs mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t(`${i18nPrefix}.argsHint`) }}
                  </div>
                </div>

                <!-- Environment Variables -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t(`${i18nPrefix}.envLabel`) }}</label>
                  <div class="flex gap-2 mb-2">
                    <input
                      v-model="envKey"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      placeholder="KEY"
                    >
                    <input
                      v-model="envValue"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      placeholder="VALUE"
                    >
                    <button
                      class="px-4 py-2 rounded-lg font-semibold text-sm text-white"
                      :style="{ background: 'var(--accent-primary)' }"
                      @click="addEnvVar"
                    >
                      {{ $t('common.add') }}
                    </button>
                  </div>
                  <div class="space-y-1">
                    <div
                      v-for="[key, value] in Object.entries(formData.env || {})"
                      :key="key"
                      class="flex items-center justify-between px-3 py-2 rounded"
                      :style="{ background: 'var(--bg-secondary)' }"
                    >
                      <code
                        class="text-sm font-mono"
                        :style="{ color: 'var(--text-primary)' }"
                      >{{ key }}={{ value }}</code>
                      <button
                        class="text-xs"
                        :style="{ color: 'var(--accent-danger)' }"
                        @click="removeEnvVar(key)"
                      >
                        {{ $t('common.delete') }}
                      </button>
                    </div>
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
                  {{ editingServer ? $t('common.save') : $t('common.add') }}
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
import { Server, Plus, Edit2, Trash2, ArrowLeft, Home, Sparkles, Zap, Flame } from 'lucide-vue-next'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'
import { usePlatformMcp, type PlatformType, getServerIdentifier } from '@/composables/usePlatformMcp'

// ============ Props ============

interface Props {
  platform: PlatformType
}

const props = defineProps<Props>()

// ============ Composable ============

const { t } = useI18n()

const {
  moduleColor,
  i18nPrefix,
  parentPath,
  servers,
  loading,
  showForm,
  editingServer,
  isHttpServer,
  formData,
  argInput,
  envKey,
  envValue,
  loadServers,
  deleteServer,
  openAddForm,
  openEditForm,
  closeForm,
  submitForm,
  addEnvVar,
  removeEnvVar,
} = usePlatformMcp(props.platform)

// ============ Computed ============

/** 侧边栏模块名称 */
const sidebarModule = computed(() => {
  const moduleMap: Record<PlatformType, string> = {
    gemini: 'gemini-cli',
    qwen: 'qwen',
    iflow: 'iflow',
  }
  return moduleMap[props.platform]
})

/** 平台图标 */
const platformIcon = computed(() => {
  const iconMap: Record<PlatformType, typeof Sparkles> = {
    gemini: Sparkles,
    qwen: Zap,
    iflow: Flame,
  }
  return iconMap[props.platform]
})

/** 平台显示名称 */
const platformName = computed(() => {
  const nameMap: Record<PlatformType, string> = {
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
  { label: t(`${i18nPrefix.value}.title`), path: `${parentPath.value}/mcp`, icon: Server },
])

// ============ Lifecycle ============

onMounted(() => {
  loadServers()
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
