<template>
  <div class="codex-settings-view">
    <div class="codex-settings-shell">
      <ModuleSubnav module="codex" />

      <div class="codex-settings-stack">
        <!-- Header -->
        <div class="codex-settings-header">
          <div class="codex-settings-header__intro">
            <div class="codex-settings-header__icon">
              <SIcon
                name="Settings2"
                size="w-6 h-6"
                class="text-platform-codex"
              />
            </div>
            <div>
              <h1 class="codex-settings-title">
                {{ $t('codex.settings.title') }}
              </h1>
              <p class="codex-settings-subtitle">
                {{ t('codex.settings.subtitle') }}
              </p>
            </div>
          </div>

          <div class="codex-settings-header__actions">
            <RouterLink
              to="/codex"
              class="inline-flex"
            >
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
              >
                <template #leading>
                  <SIcon
                    name="ArrowLeft"
                    size="w-4 h-4"
                  />
                </template>
                {{ $t('common.back') }}
              </Button>
            </RouterLink>
            <Button
              v-if="activeTab !== 'source'"
              variant="primary"
              surface="card"
              density="compact"
              motion="standard"
              :disabled="saving"
              @click="handleSave"
            >
              <template #leading>
                <SIcon
                  name="Save"
                  size="w-4 h-4"
                />
              </template>
              {{ saving ? $t('codex.settings.saving') : $t('common.save') }}
            </Button>
          </div>
        </div>

        <!-- Loading -->
        <div
          v-if="loading"
          class="codex-settings-loading"
        >
          <div class="codex-settings-spinner" />
          <span>{{ $t('common.loading') }}</span>
        </div>

        <template v-else>
          <!-- Tab Navigation -->
          <div
            class="codex-settings-tabs"
            role="tablist"
          >
            <button
              v-for="tab in tabs"
              :key="tab.key"
              role="tab"
              :aria-selected="activeTab === tab.key"
              :disabled="tab.disabled"
              :title="tab.disabled ? $t('settingsRaw.unsupportedEnvironment') : undefined"
              class="codex-settings-tab"
              :class="{ 'codex-settings-tab--active': activeTab === tab.key }"
              @click="changeTab(tab.key)"
            >
              <SIcon
                :name="tab.icon"
                size="w-4 h-4"
              />
              {{ tab.label }}
            </button>
          </div>

          <!-- Tab: 模型与推理 -->
          <div
            v-show="activeTab === 'model'"
            class="space-y-6"
          >
            <Card
              variant="glass"
              class="p-5 space-y-5"
            >
              <h3 class="text-lg font-bold text-text-primary">
                {{ $t('codex.settings.tabs.model') }}
              </h3>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.model.model')
                }}</label>
                <input
                  v-model="form.model"
                  type="text"
                  :placeholder="$t('codex.settings.model.modelPlaceholder')"
                  class="settings-input"
                >
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.model.modelProvider')
                }}</label>
                <input
                  v-model="form.model_provider"
                  type="text"
                  :placeholder="$t('codex.settings.model.modelProviderPlaceholder')"
                  class="settings-input"
                >
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.model.reasoningEffort')
                }}</label>
                <select
                  v-model="form.model_reasoning_effort"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['low', 'medium', 'high']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.model.reasoningSummary')
                }}</label>
                <select
                  v-model="form.model_reasoning_summary"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['auto', 'concise', 'detailed', 'none']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.model.verbosity')
                }}</label>
                <select
                  v-model="form.model_verbosity"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['low', 'medium', 'high']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                    $t('codex.settings.model.contextWindow')
                  }}</label>
                  <input
                    v-model.number="form.model_context_window"
                    type="number"
                    placeholder="128000"
                    class="settings-input"
                  >
                </div>
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                    $t('codex.settings.model.autoCompactLimit')
                  }}</label>
                  <input
                    v-model.number="form.model_auto_compact_token_limit"
                    type="number"
                    placeholder="80000"
                    class="settings-input"
                  >
                </div>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.model.personality')
                }}</label>
                <select
                  v-model="form.personality"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['none', 'friendly', 'pragmatic']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>
            </Card>
          </div>

          <!-- Tab: 安全与权限 -->
          <div
            v-show="activeTab === 'security'"
            class="space-y-6"
          >
            <Card
              variant="glass"
              class="p-5 space-y-5"
            >
              <h3 class="text-lg font-bold text-text-primary">
                {{ $t('codex.settings.tabs.security') }}
              </h3>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.security.approvalPolicy')
                }}</label>
                <select
                  v-model="form.approval_policy"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['auto', 'on-request', 'read-only', 'full-access']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.security.sandboxMode')
                }}</label>
                <input
                  v-model="form.sandbox_mode"
                  type="text"
                  placeholder="workspace-write"
                  class="settings-input"
                >
              </div>

              <ToggleField
                v-model="form.disable_response_storage"
                :label="$t('codex.settings.security.disableResponseStorage')"
              />

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.security.writableRoots')
                }}</label>
                <input
                  v-model="writableRootsStr"
                  type="text"
                  :placeholder="$t('codex.settings.security.writableRootsPlaceholder')"
                  class="settings-input"
                >
                <p class="text-xs text-text-muted mt-1">
                  {{ $t('codex.settings.security.writableRootsHint') }}
                </p>
              </div>

              <ToggleField
                v-model="sandboxNetworkAccess"
                :label="$t('codex.settings.security.networkAccess')"
              />

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.security.shellIncludeOnly')
                }}</label>
                <input
                  v-model="shellIncludeOnlyStr"
                  type="text"
                  :placeholder="$t('codex.settings.security.shellIncludeOnlyPlaceholder')"
                  class="settings-input"
                >
                <p class="text-xs text-text-muted mt-1">
                  {{ $t('codex.settings.security.shellIncludeOnlyHint') }}
                </p>
              </div>
            </Card>
          </div>

          <!-- Tab: 工具与搜索 -->
          <div
            v-show="activeTab === 'tools'"
            class="space-y-6"
          >
            <Card
              variant="glass"
              class="p-5 space-y-5"
            >
              <h3 class="text-lg font-bold text-text-primary">
                {{ $t('codex.settings.tabs.tools') }}
              </h3>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.tools.webSearch')
                }}</label>
                <select
                  v-model="form.web_search"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['disabled', 'cached', 'live']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.tools.fileOpener')
                }}</label>
                <select
                  v-model="form.file_opener"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['vscode', 'cursor', 'windsurf', 'none']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <ToggleField
                v-model="toolsViewImage"
                :label="$t('codex.settings.tools.viewImage')"
              />
              <ToggleField
                v-model="toolsWebSearch"
                :label="$t('codex.settings.tools.toolWebSearch')"
              />

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.tools.developerInstructions')
                }}</label>
                <textarea
                  v-model="form.developer_instructions"
                  rows="3"
                  class="settings-input"
                  :placeholder="$t('codex.settings.tools.developerInstructionsPlaceholder')"
                />
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.tools.instructions')
                }}</label>
                <textarea
                  v-model="form.instructions"
                  rows="3"
                  class="settings-input"
                  :placeholder="$t('codex.settings.tools.instructionsPlaceholder')"
                />
              </div>
            </Card>
          </div>

          <!-- Tab: 界面设置 -->
          <div
            v-show="activeTab === 'ui'"
            class="space-y-6"
          >
            <Card
              variant="glass"
              class="p-5 space-y-5"
            >
              <h3 class="text-lg font-bold text-text-primary">
                {{ $t('codex.settings.tabs.ui') }}
              </h3>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.ui.alternateScreen')
                }}</label>
                <select
                  v-model="tuiAlternateScreen"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['auto', 'always', 'never']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <ToggleField
                v-model="tuiAnimations"
                :label="$t('codex.settings.ui.animations')"
              />
              <ToggleField
                v-if="!isTuiNotificationEventsConfig"
                v-model="tuiNotifications"
                :label="$t('codex.settings.ui.notifications')"
              />
              <div
                v-else
                class="space-y-2"
              >
                <label class="block text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.ui.notifications')
                }}</label>
                <div class="flex flex-wrap gap-2">
                  <span
                    v-for="event in tuiNotificationEvents"
                    :key="event"
                    class="codex-settings-chip"
                  >
                    {{ event }}
                  </span>
                </div>
              </div>
              <ToggleField
                v-model="tuiShowTooltips"
                :label="$t('codex.settings.ui.showTooltips')"
              />
              <ToggleField
                v-model="form.hide_agent_reasoning"
                :label="$t('codex.settings.ui.hideAgentReasoning')"
              />
              <ToggleField
                v-model="form.show_raw_agent_reasoning"
                :label="$t('codex.settings.ui.showRawAgentReasoning')"
              />
              <ToggleField
                v-model="form.check_for_update_on_startup"
                :label="$t('codex.settings.ui.checkForUpdate')"
              />
              <ToggleField
                v-model="form.suppress_unstable_features_warning"
                :label="$t('codex.settings.ui.suppressUnstableWarning')"
              />
            </Card>
          </div>

          <!-- Tab: 功能开关 -->
          <div
            v-show="activeTab === 'features'"
            class="space-y-6"
          >
            <Card
              variant="glass"
              class="p-5 space-y-5"
            >
              <h3 class="text-lg font-bold text-text-primary">
                {{ $t('codex.settings.tabs.features') }}
              </h3>

              <ToggleField
                v-model="form.experimental_use_rmcp_client"
                :label="$t('codex.settings.features.experimentalRmcp')"
              />

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.features.historyPersistence')
                }}</label>
                <select
                  v-model="historyPersistence"
                  class="settings-input"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="o in ['save-all', 'none']"
                    :key="o"
                    :value="o"
                  >
                    {{ o }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.features.historyMaxBytes')
                }}</label>
                <input
                  v-model.number="historyMaxBytes"
                  type="number"
                  placeholder="10485760"
                  class="settings-input"
                >
              </div>

              <ToggleField
                v-model="analyticsEnabled"
                :label="$t('codex.settings.features.analytics')"
              />
              <ToggleField
                v-model="feedbackEnabled"
                :label="$t('codex.settings.features.feedback')"
              />

              <!-- Dynamic features map -->
              <div v-if="form.features && Object.keys(form.features).length > 0">
                <label class="block mb-2 text-sm font-semibold text-text-primary">{{
                  $t('codex.settings.features.featureFlags')
                }}</label>
                <div class="space-y-2">
                  <ToggleField
                    v-for="(val, key) in form.features"
                    :key="key"
                    :model-value="val"
                    :label="String(key)"
                    @update:model-value="
                      (v: boolean) => {
                        if (form.features) form.features[key as string] = v
                      }
                    "
                  />
                </div>
              </div>
            </Card>
          </div>

          <ConfigSourcePanel
            v-if="activeTab === 'source'"
            language="toml"
            :get-raw="getCodexConfigRaw"
            :save-raw="saveCodexConfigRaw"
            :list-layers="listCodexConfigLayers"
            @saved="handleRawSaved"
            @close="activeTab = 'model'"
            @dirty-change="sourceDirty = $event"
          />

          <!-- Toast -->
          <Transition name="fade">
            <div
              v-if="toast"
              class="codex-settings-toast"
              :class="toast.type === 'success' ? 'codex-settings-toast--success' : 'codex-settings-toast--error'"
            >
              {{ toast.message }}
            </div>
          </Transition>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import ConfigSourcePanel from '@/components/editor/ConfigSourcePanel.vue'
import {
  getCodexConfig,
  getCodexConfigRaw,
  getCurrentEnvironment,
  listCodexConfigLayers,
  saveCodexConfigRaw,
  updateCodexConfig,
} from '@/api'
import { useUIStore } from '@/stores/ui'
import type { CodexConfig } from '@/types'
import { logger } from '@/utils/logger'

const { t } = useI18n()
const uiStore = useUIStore()

// ============ State ============
const loading = ref(true)
const saving = ref(false)
const activeTab = ref('model')
const rawLocal = ref(true)
const sourceDirty = ref(false)
const toast = ref<{ message: string; type: 'success' | 'error' } | null>(null)

const form = reactive<CodexConfig>({})

// ============ Tabs ============
const tabs = computed(() => [
  { key: 'model', label: t('codex.settings.tabs.model'), icon: 'Brain' },
  { key: 'security', label: t('codex.settings.tabs.security'), icon: 'Shield' },
  { key: 'tools', label: t('codex.settings.tabs.tools'), icon: 'Wrench' },
  { key: 'ui', label: t('codex.settings.tabs.ui'), icon: 'Monitor' },
  { key: 'features', label: t('codex.settings.tabs.features'), icon: 'Zap' },
  { key: 'source', label: t('settingsRaw.sourceTab'), icon: 'FileCode2', disabled: !rawLocal.value },
])

async function changeTab(nextTab: string) {
  if (nextTab === 'source' && !rawLocal.value) {
    uiStore.showWarning(t('settingsRaw.unsupportedEnvironment'))
    return
  }
  if (activeTab.value === 'source' && sourceDirty.value) {
    const discard = await uiStore.requestConfirm({
      title: t('settingsRaw.discardTitle'),
      message: t('settingsRaw.discardMessage'),
      confirmText: t('settingsRaw.discard'),
      cancelText: t('common.cancel'),
      type: 'warning',
      surface: 'solid',
    })
    if (!discard) return
  }
  activeTab.value = nextTab
}

async function handleRawSaved() {
  sourceDirty.value = false
  activeTab.value = 'model'
  await loadConfig()
}

async function loadActiveEnvironment() {
  try {
    const environment = await getCurrentEnvironment<{ env_type?: string }>()
    rawLocal.value = !environment || environment.env_type === 'local'
  } catch {
    rawLocal.value = true
  }
}

// ============ Computed proxies for nested fields ============

// sandbox_workspace_write
const writableRootsStr = computed({
  get: () => form.sandbox_workspace_write?.writable_roots?.join(', ') ?? '',
  set: (v: string) => {
    if (!form.sandbox_workspace_write) form.sandbox_workspace_write = {}
    form.sandbox_workspace_write.writable_roots = v
      ? v
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : undefined
  },
})
const sandboxNetworkAccess = computed({
  get: () => form.sandbox_workspace_write?.network_access,
  set: (v: boolean | undefined) => {
    if (!form.sandbox_workspace_write) form.sandbox_workspace_write = {}
    form.sandbox_workspace_write.network_access = v
  },
})

// shell_environment_policy
const shellIncludeOnlyStr = computed({
  get: () => form.shell_environment_policy?.include_only?.join(', ') ?? '',
  set: (v: string) => {
    if (!form.shell_environment_policy) form.shell_environment_policy = {}
    form.shell_environment_policy.include_only = v
      ? v
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : undefined
  },
})

// tools
const toolsViewImage = computed({
  get: () => form.tools?.view_image,
  set: (v: boolean | undefined) => {
    if (!form.tools) form.tools = {}
    form.tools.view_image = v
  },
})
const toolsWebSearch = computed({
  get: () => form.tools?.web_search,
  set: (v: boolean | undefined) => {
    if (!form.tools) form.tools = {}
    form.tools.web_search = v
  },
})

// tui
const tuiAlternateScreen = computed({
  get: () => form.tui?.alternate_screen ?? '',
  set: (v: string) => {
    if (!form.tui) form.tui = {}
    form.tui.alternate_screen = v || undefined
  },
})
const tuiAnimations = computed({
  get: () => form.tui?.animations,
  set: (v: boolean | undefined) => {
    if (!form.tui) form.tui = {}
    form.tui.animations = v
  },
})
const tuiNotificationEvents = computed(() =>
  Array.isArray(form.tui?.notifications) ? form.tui.notifications : [],
)
const isTuiNotificationEventsConfig = computed(() => Array.isArray(form.tui?.notifications))
const tuiNotifications = computed({
  get: () => (typeof form.tui?.notifications === 'boolean' ? form.tui.notifications : undefined),
  set: (v: boolean | undefined) => {
    if (!form.tui) form.tui = {}
    form.tui.notifications = v
  },
})
const tuiShowTooltips = computed({
  get: () => form.tui?.show_tooltips,
  set: (v: boolean | undefined) => {
    if (!form.tui) form.tui = {}
    form.tui.show_tooltips = v
  },
})

// history
const historyPersistence = computed({
  get: () => form.history?.persistence ?? '',
  set: (v: string) => {
    if (!form.history) form.history = {}
    form.history.persistence = v || undefined
  },
})
const historyMaxBytes = computed({
  get: () => form.history?.max_bytes,
  set: (v: number | undefined) => {
    if (!form.history) form.history = {}
    form.history.max_bytes = v
  },
})

// analytics / feedback
const analyticsEnabled = computed({
  get: () => form.analytics?.enabled,
  set: (v: boolean | undefined) => {
    if (!form.analytics) form.analytics = {}
    form.analytics.enabled = v
  },
})
const feedbackEnabled = computed({
  get: () => form.feedback?.enabled,
  set: (v: boolean | undefined) => {
    if (!form.feedback) form.feedback = {}
    form.feedback.enabled = v
  },
})

// ============ Actions ============
function showToast(message: string, type: 'success' | 'error' = 'success') {
  toast.value = { message, type }
  setTimeout(() => {
    toast.value = null
  }, 3000)
}

async function loadConfig() {
  loading.value = true
  try {
    const config = await getCodexConfig()
    Object.assign(form, config)
  } catch (e) {
    logger.error('Failed to load codex config:', e)
    showToast(t('codex.settings.messages.loadFailed'), 'error')
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  saving.value = true
  try {
    const payload: Record<string, unknown> = {
      model: form.model ?? null,
      model_provider: form.model_provider ?? null,
      model_reasoning_effort: form.model_reasoning_effort ?? null,
      model_reasoning_summary: form.model_reasoning_summary ?? null,
      model_verbosity: form.model_verbosity ?? null,
      model_context_window: form.model_context_window ?? null,
      model_auto_compact_token_limit: form.model_auto_compact_token_limit ?? null,
      personality: form.personality ?? null,
      approval_policy: form.approval_policy ?? null,
      sandbox_mode: form.sandbox_mode ?? null,
      disable_response_storage: form.disable_response_storage ?? null,
      sandbox_workspace_write: {
        writable_roots: form.sandbox_workspace_write?.writable_roots ?? null,
        network_access: form.sandbox_workspace_write?.network_access ?? null,
      },
      shell_environment_policy: {
        include_only: form.shell_environment_policy?.include_only ?? null,
      },
      web_search: form.web_search ?? null,
      file_opener: form.file_opener ?? null,
      developer_instructions: form.developer_instructions ?? null,
      instructions: form.instructions ?? null,
      tools: {
        view_image: form.tools?.view_image ?? null,
        web_search: form.tools?.web_search ?? null,
      },
      tui: {
        alternate_screen: form.tui?.alternate_screen ?? null,
        animations: form.tui?.animations ?? null,
        notifications: form.tui?.notifications ?? null,
        show_tooltips: form.tui?.show_tooltips ?? null,
      },
      hide_agent_reasoning: form.hide_agent_reasoning ?? null,
      show_raw_agent_reasoning: form.show_raw_agent_reasoning ?? null,
      check_for_update_on_startup: form.check_for_update_on_startup ?? null,
      suppress_unstable_features_warning: form.suppress_unstable_features_warning ?? null,
      experimental_use_rmcp_client: form.experimental_use_rmcp_client ?? null,
      history: {
        persistence: form.history?.persistence ?? null,
        max_bytes: form.history?.max_bytes ?? null,
      },
      analytics: {
        enabled: form.analytics?.enabled ?? null,
      },
      feedback: {
        enabled: form.feedback?.enabled ?? null,
      },
      features: form.features ?? null,
    }

    await updateCodexConfig(payload as CodexConfig)
    showToast(t('codex.settings.messages.saveSuccess'))
  } catch (e) {
    logger.error('Failed to save codex config:', e)
    showToast(t('codex.settings.messages.saveFailed'), 'error')
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  void loadActiveEnvironment()
  void loadConfig()
})

// ============ ToggleField inline component ============
</script>

<script lang="ts">
import { defineComponent, h } from 'vue'

const ToggleField = defineComponent({
  name: 'ToggleField',
  props: {
    modelValue: { type: Boolean, default: undefined },
    label: { type: String, required: true },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    return () =>
      h('label', { class: 'flex items-center gap-3 cursor-pointer' }, [
        h('input', {
          type: 'checkbox',
          checked: props.modelValue ?? false,
          class: 'w-4 h-4 rounded border-border-default/15 text-accent-primary focus:ring-accent-primary',
          onChange: (e: Event) => emit('update:modelValue', (e.target as HTMLInputElement).checked),
        }),
        h('span', { class: 'text-sm font-semibold text-text-primary' }, props.label),
      ])
  },
})

export default { components: { ToggleField } }
</script>

<style scoped>
.codex-settings-view {
  @apply min-h-full p-6;
}

.codex-settings-shell {
  @apply mx-auto max-w-[1800px];
}

.codex-settings-stack {
  @apply mt-6 space-y-6;
}

.codex-settings-header {
  @apply flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between;
}

.codex-settings-header__intro {
  @apply flex items-center gap-3;
}

.codex-settings-header__actions {
  @apply flex items-center gap-3;
}

.codex-settings-header__icon {
  @apply flex h-12 w-12 items-center justify-center rounded-2xl border shadow-lg backdrop-blur-md;

  border-color: rgb(var(--color-platform-codex-rgb) / 20%);
  background: rgb(var(--color-platform-codex-rgb) / 10%);
}

.codex-settings-title {
  @apply text-2xl font-bold;

  color: var(--stage-text-primary);
}

.codex-settings-subtitle {
  @apply mt-1 text-sm;

  color: var(--stage-text-secondary);
}

.codex-settings-loading {
  @apply flex flex-col items-center justify-center py-20 gap-4;

  color: var(--stage-text-muted);
}

.codex-settings-spinner {
  @apply w-8 h-8 rounded-full border-[3px] animate-spin;

  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  border-top-color: var(--color-accent-primary);
}

.codex-settings-tabs {
  @apply flex gap-2 overflow-x-auto pb-2 md:flex-wrap md:overflow-x-visible md:pb-0;
}

.codex-settings-tab {
  @apply flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-colors min-h-[44px] whitespace-nowrap flex-shrink-0;

  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
  color: var(--stage-text-primary);
}

.codex-settings-tab:hover {
  background: rgb(var(--color-bg-surface-rgb) / 70%);
  border-color: rgb(var(--color-border-default-rgb) / 70%);
}

.codex-settings-tab:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.codex-settings-tab--active {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 25%);
  color: var(--color-accent-primary);
  box-shadow: 0 4px 12px rgb(var(--color-accent-primary-rgb) / 10%);
}

.settings-input {
  @apply w-full px-4 py-2.5 rounded-lg outline-none transition-[border-color,box-shadow];

  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
  color: var(--stage-text-primary);
}

.settings-input:focus {
  border-color: var(--color-accent-primary);
  box-shadow: 0 0 0 1px var(--color-accent-primary);
}

.codex-settings-chip {
  @apply rounded-md px-2 py-1 text-xs font-medium;

  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);
  color: var(--stage-chip-neutral-text);
}

.codex-settings-toast {
  @apply fixed bottom-6 right-6 z-50 px-5 py-3 rounded-xl shadow-lg text-sm font-medium text-white;
}

.codex-settings-toast--success {
  background: var(--color-success);
}

.codex-settings-toast--error {
  background: var(--color-danger);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
