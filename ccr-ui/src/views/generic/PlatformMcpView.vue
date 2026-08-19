<template>
  <PageShell class="platform-mcp-view">
    <template #header>
      <PageHeader :title="$t(`${i18nPrefix}.pageTitle`)">
        <template #leading>
          <SIcon
            name="Server"
            size="w-6 h-6"
            class="text-accent-primary"
          />
        </template>
        <template #actions>
          <RouterLink
            :to="parentPath"
            class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm bg-bg-elevated text-text-secondary border border-border-default"
          >
            <SIcon
              name="ArrowLeft"
              size="w-4 h-4"
            /><span>{{ $t('common.back') }}</span>
          </RouterLink>
          <Button @click="openAddForm">
            <SIcon
              name="Plus"
              size="w-4 h-4"
              class="mr-2"
            />{{ $t(`${i18nPrefix}.addServer`) }}
          </Button>
        </template>
      </PageHeader>
    </template>

    <template #subnav>
      <ModuleSubnav :module="sidebarModule" />
    </template>

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

    <!-- Server List -->
    <div
      v-else
      class="space-y-3"
    >
      <!-- Empty State -->
      <div
        v-if="!servers || servers.length === 0"
        class="text-center py-10"
        :style="{ color: 'var(--color-text-muted)' }"
      >
        {{ $t(`${i18nPrefix}.emptyState`) }}
      </div>

      <!-- Server Cards -->
      <div
        v-for="server in servers"
        :key="getServerIdentifier(server)"
        class="platform-card group rounded-lg p-4"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-2">
              <h3
                class="text-lg font-bold font-mono"
                :style="{ color: 'var(--color-text-primary)' }"
              >
                {{ server.name || server.command || server.url }}
              </h3>
              <span
                v-if="server.url"
                class="px-2 py-0.5 rounded text-xs font-semibold"
                :style="{ background: 'var(--color-accent-secondary)', color: 'var(--color-accent-primary-contrast)' }"
              >HTTP</span>
              <span
                v-else
                class="px-2 py-0.5 rounded text-xs font-semibold"
                :style="{ background: 'var(--color-accent-primary)', color: 'var(--color-accent-primary-contrast)' }"
              >STDIO</span>
            </div>
            <div class="space-y-2 text-sm">
              <div v-if="server.command">
                <span :style="{ color: 'var(--color-text-muted)' }">{{ $t('common.command') }}:</span>
                <code
                  class="ml-2 px-2 py-1 rounded font-mono"
                  :style="{ background: 'var(--color-bg-elevated)', color: 'var(--color-accent-primary)' }"
                >{{ server.command }}</code>
              </div>
              <div v-if="server.url">
                <span :style="{ color: 'var(--color-text-muted)' }">{{ $t('common.url') }}:</span>
                <code
                  class="ml-2 px-2 py-1 rounded font-mono"
                  :style="{ background: 'var(--color-bg-elevated)', color: 'var(--color-accent-primary)' }"
                >{{ server.url }}</code>
              </div>
              <div v-if="server.args && server.args.length > 0">
                <span :style="{ color: 'var(--color-text-muted)' }">{{ $t('common.args') }}:</span>
                <code
                  class="ml-2 px-2 py-1 rounded font-mono"
                  :style="{ background: 'var(--color-bg-elevated)', color: 'var(--color-text-primary)' }"
                >{{ server.args.join(' ') }}</code>
              </div>
              <div v-if="server.env && Object.keys(server.env).length > 0">
                <span :style="{ color: 'var(--color-text-muted)' }">{{ $t('common.envVars') }}:</span>
                <div class="ml-2 mt-1 space-y-1">
                  <div
                    v-for="[key, value] in Object.entries(server.env)"
                    :key="key"
                    class="text-xs font-mono px-2 py-1 rounded"
                    :style="{ background: 'var(--color-bg-elevated)' }"
                  >
                    <span :style="{ color: 'var(--color-accent-secondary)' }">{{ key }}</span>=<span :style="{ color: 'var(--color-text-primary)' }">{{ value }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="flex gap-2">
            <button
              class="p-2 rounded-lg transition-transform hover:scale-110"
              :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)', color: 'var(--color-accent-primary)' }"
              :title="$t('common.edit')"
              @click="openEditForm(server)"
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
              @click="handleDeleteServer(server)"
            >
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />
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
        :style="{ background: 'var(--color-bg-elevated)', border: '1px solid var(--color-border-default)' }"
      >
        <h2
          class="text-xl font-bold mb-4"
          :style="{ color: 'var(--color-text-primary)' }"
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
              :style="{ color: 'var(--color-text-secondary)' }"
            >{{ $t(`${i18nPrefix}.httpServerHint`) }}</span>
          </label>
        </div>

        <div class="space-y-4">
          <!-- URL (HTTP Server) -->
          <div v-if="isHttpServer">
            <label
              class="block text-sm font-semibold mb-1"
              :style="{ color: 'var(--color-text-secondary)' }"
            >{{ $t(`${i18nPrefix}.urlLabel`) }} *</label>
            <input
              v-model="formData.url"
              type="text"
              class="w-full px-3 py-2 rounded-lg"
              :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
              :placeholder="$t(`${i18nPrefix}.urlPlaceholder`)"
            >
          </div>

          <!-- Command (STDIO Server) -->
          <div v-else>
            <label
              class="block text-sm font-semibold mb-1"
              :style="{ color: 'var(--color-text-secondary)' }"
            >{{ $t(`${i18nPrefix}.commandLabel`) }} *</label>
            <input
              v-model="formData.command"
              type="text"
              class="w-full px-3 py-2 rounded-lg font-mono"
              :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
              :placeholder="$t(`${i18nPrefix}.commandPlaceholder`)"
            >
          </div>

          <!-- Args (STDIO Server) -->
          <div v-if="!isHttpServer">
            <label
              class="block text-sm font-semibold mb-1"
              :style="{ color: 'var(--color-text-secondary)' }"
            >{{ $t(`${i18nPrefix}.argsLabel`) }}</label>
            <input
              v-model="argInput"
              type="text"
              class="w-full px-3 py-2 rounded-lg font-mono"
              :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
              :placeholder="$t(`${i18nPrefix}.argsPlaceholder`)"
            >
            <div
              class="text-xs mt-1"
              :style="{ color: 'var(--color-text-muted)' }"
            >
              {{ $t(`${i18nPrefix}.argsHint`) }}
            </div>
          </div>

          <!-- Environment Variables -->
          <div>
            <label
              class="block text-sm font-semibold mb-1"
              :style="{ color: 'var(--color-text-secondary)' }"
            >{{ $t(`${i18nPrefix}.envLabel`) }}</label>
            <div class="flex gap-2 mb-2">
              <input
                v-model="envKey"
                type="text"
                class="flex-1 px-3 py-2 rounded-lg font-mono"
                :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
                placeholder="KEY"
              >
              <input
                v-model="envValue"
                type="text"
                class="flex-1 px-3 py-2 rounded-lg font-mono"
                :style="{ background: 'var(--color-bg-surface)', border: '1px solid var(--color-border-default)', color: 'var(--color-text-primary)' }"
                placeholder="VALUE"
              >
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-[color:var(--color-accent-primary-contrast)]"
                :style="{ background: 'var(--color-accent-primary)' }"
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
                :style="{ background: 'var(--color-bg-surface)' }"
              >
                <code
                  class="text-sm font-mono"
                  :style="{ color: 'var(--color-text-primary)' }"
                >{{ key }}={{ value }}</code>
                <button
                  class="text-xs"
                  :style="{ color: 'var(--color-danger)' }"
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
            class="flex-1 px-4 py-2 rounded-lg font-semibold text-[color:var(--color-accent-primary-contrast)]"
            :style="{ background: 'var(--color-accent-primary)' }"
            @click="submitForm"
          >
            {{ editingServer ? $t('common.save') : $t('common.add') }}
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
  </PageShell>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import { useUIStore } from '@/stores/ui'
import { usePlatformMcp, type PlatformType, type PlatformMcpServer, getServerIdentifier } from '@/composables/usePlatformMcp'

// ============ Props ============

interface Props {
  platform: PlatformType
}

const props = defineProps<Props>()

const { t } = useI18n()
const uiStore = useUIStore()

// ============ Composable ============

const {
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
    gemini: 'antigravity',
  }
  return moduleMap[props.platform]
})

// ============ Lifecycle ============

onMounted(() => {
  loadServers()
})

// ============ Event Handlers ============

/** 删除服务器：先弹全局确认框，确认后才调用 composable 的纯执行器 */
async function handleDeleteServer(server: PlatformMcpServer): Promise<void> {
  const name = getServerIdentifier(server)
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t(`${i18nPrefix.value}.deleteConfirm`, { name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (confirmed) await deleteServer(server)
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
