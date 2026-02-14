<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="mb-6" />
    <div class="max-w-[1200px] mx-auto">
      <!-- Header -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-xl sm:text-2xl font-bold text-text-primary flex items-center">
            <Settings2
              class="w-6 h-6 sm:w-7 sm:h-7 mr-2 text-emerald-500"
              aria-hidden="true"
            />
            {{ $t('codex.settings.title') }}
          </h2>
        </div>
        <div class="flex gap-3">
          <RouterLink to="/codex">
            <button class="px-4 py-2 rounded-lg font-medium transition-all bg-bg-elevated text-text-secondary border border-border-default hover:bg-bg-surface min-h-[44px] flex items-center">
              <ArrowLeft
                class="w-4 h-4 mr-2"
                aria-hidden="true"
              />
              {{ $t('common.back') }}
            </button>
          </RouterLink>
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-emerald-500 text-white shadow-md hover:shadow-lg flex items-center min-h-[44px]"
            :disabled="saving"
            @click="handleSave"
          >
            <Save
              class="w-4 h-4 mr-2"
              aria-hidden="true"
            />
            {{ saving ? $t('codex.settings.saving') : $t('common.save') }}
          </button>
        </div>
      </div>

      <!-- Loading -->
      <div
        v-if="loading"
        class="text-center py-20 text-text-muted"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-emerald-500/30 border-t-emerald-500" />
        <span>{{ $t('common.loading') }}</span>
      </div>

      <template v-else>
        <!-- Tab Navigation -->
        <div
          class="mb-6 flex gap-2 overflow-x-auto pb-2 scrollbar-thin md:flex-wrap md:overflow-x-visible md:pb-0"
          role="tablist"
        >
          <button
            v-for="tab in tabs"
            :key="tab.key"
            role="tab"
            :aria-selected="activeTab === tab.key"
            class="px-4 py-2 rounded-lg font-medium text-sm transition-all min-h-[44px] whitespace-nowrap flex-shrink-0 flex items-center gap-2"
            :class="activeTab === tab.key ? 'bg-emerald-500 text-white shadow-md' : 'bg-bg-elevated text-text-secondary border border-border-default hover:bg-bg-surface'"
            @click="activeTab = tab.key"
          >
            <component
              :is="tab.icon"
              class="w-4 h-4"
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.model') }}</label>
              <input
                v-model="form.model"
                type="text"
                :placeholder="$t('codex.settings.model.modelPlaceholder')"
                class="settings-input"
              >
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.modelProvider') }}</label>
              <input
                v-model="form.model_provider"
                type="text"
                placeholder="openai / ollama"
                class="settings-input"
              >
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.reasoningEffort') }}</label>
              <select
                v-model="form.model_reasoning_effort"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['low','medium','high']"
                  :key="o"
                  :value="o"
                >
                  {{ o }}
                </option>
              </select>
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.reasoningSummary') }}</label>
              <select
                v-model="form.model_reasoning_summary"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['auto','concise','detailed','none']"
                  :key="o"
                  :value="o"
                >
                  {{ o }}
                </option>
              </select>
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.verbosity') }}</label>
              <select
                v-model="form.model_verbosity"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['low','medium','high']"
                  :key="o"
                  :value="o"
                >
                  {{ o }}
                </option>
              </select>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.contextWindow') }}</label>
                <input
                  v-model.number="form.model_context_window"
                  type="number"
                  placeholder="128000"
                  class="settings-input"
                >
              </div>
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.autoCompactLimit') }}</label>
                <input
                  v-model.number="form.model_auto_compact_token_limit"
                  type="number"
                  placeholder="80000"
                  class="settings-input"
                >
              </div>
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.model.personality') }}</label>
              <select
                v-model="form.personality"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['none','friendly','pragmatic']"
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.security.approvalPolicy') }}</label>
              <select
                v-model="form.approval_policy"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['auto','on-request','read-only','full-access']"
                  :key="o"
                  :value="o"
                >
                  {{ o }}
                </option>
              </select>
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.security.sandboxMode') }}</label>
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.security.writableRoots') }}</label>
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.security.shellIncludeOnly') }}</label>
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.tools.webSearch') }}</label>
              <select
                v-model="form.web_search"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['disabled','cached','live']"
                  :key="o"
                  :value="o"
                >
                  {{ o }}
                </option>
              </select>
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.tools.fileOpener') }}</label>
              <select
                v-model="form.file_opener"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['vscode','cursor','windsurf','none']"
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.tools.developerInstructions') }}</label>
              <textarea
                v-model="form.developer_instructions"
                rows="3"
                class="settings-input"
                :placeholder="$t('codex.settings.tools.developerInstructionsPlaceholder')"
              />
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.tools.instructions') }}</label>
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.ui.alternateScreen') }}</label>
              <select
                v-model="tuiAlternateScreen"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['auto','always','never']"
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
              v-model="tuiNotifications"
              :label="$t('codex.settings.ui.notifications')"
            />
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
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.features.historyPersistence') }}</label>
              <select
                v-model="historyPersistence"
                class="settings-input"
              >
                <option value="">
                  --
                </option>
                <option
                  v-for="o in ['save-all','none']"
                  :key="o"
                  :value="o"
                >
                  {{ o }}
                </option>
              </select>
            </div>

            <div>
              <label class="block mb-1.5 text-sm font-semibold text-text-primary">{{ $t('codex.settings.features.historyMaxBytes') }}</label>
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
              <label class="block mb-2 text-sm font-semibold text-text-primary">{{ $t('codex.settings.features.featureFlags') }}</label>
              <div class="space-y-2">
                <ToggleField
                  v-for="(val, key) in form.features"
                  :key="key"
                  :model-value="val"
                  :label="String(key)"
                  @update:model-value="(v: boolean) => { if (form.features) form.features[key as string] = v }"
                />
              </div>
            </div>
          </Card>
        </div>

        <!-- Toast -->
        <Transition name="fade">
          <div
            v-if="toast"
            class="fixed bottom-6 right-6 z-50 px-5 py-3 rounded-xl shadow-lg text-sm font-medium"
            :class="toast.type === 'success' ? 'bg-emerald-500 text-white' : 'bg-red-500 text-white'"
          >
            {{ toast.message }}
          </div>
        </Transition>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Settings2, ArrowLeft, Save, Brain, Shield, Wrench, Monitor, Zap } from 'lucide-vue-next'
import Card from '@/components/ui/Card.vue'
import { getCodexConfig, updateCodexConfig } from '@/api'
import type { CodexConfig } from '@/types'

const { t } = useI18n()

// ============ State ============
const loading = ref(true)
const saving = ref(false)
const activeTab = ref('model')
const toast = ref<{ message: string; type: 'success' | 'error' } | null>(null)

const form = reactive<CodexConfig>({})

// ============ Tabs ============
const tabs = computed(() => [
  { key: 'model', label: t('codex.settings.tabs.model'), icon: Brain },
  { key: 'security', label: t('codex.settings.tabs.security'), icon: Shield },
  { key: 'tools', label: t('codex.settings.tabs.tools'), icon: Wrench },
  { key: 'ui', label: t('codex.settings.tabs.ui'), icon: Monitor },
  { key: 'features', label: t('codex.settings.tabs.features'), icon: Zap },
])

// ============ Computed proxies for nested fields ============

// sandbox_workspace_write
const writableRootsStr = computed({
  get: () => form.sandbox_workspace_write?.writable_roots?.join(', ') ?? '',
  set: (v: string) => {
    if (!form.sandbox_workspace_write) form.sandbox_workspace_write = {}
    form.sandbox_workspace_write.writable_roots = v ? v.split(',').map(s => s.trim()).filter(Boolean) : undefined
  }
})
const sandboxNetworkAccess = computed({
  get: () => form.sandbox_workspace_write?.network_access,
  set: (v: boolean | undefined) => {
    if (!form.sandbox_workspace_write) form.sandbox_workspace_write = {}
    form.sandbox_workspace_write.network_access = v
  }
})

// shell_environment_policy
const shellIncludeOnlyStr = computed({
  get: () => form.shell_environment_policy?.include_only?.join(', ') ?? '',
  set: (v: string) => {
    if (!form.shell_environment_policy) form.shell_environment_policy = {}
    form.shell_environment_policy.include_only = v ? v.split(',').map(s => s.trim()).filter(Boolean) : undefined
  }
})

// tools
const toolsViewImage = computed({
  get: () => form.tools?.view_image,
  set: (v: boolean | undefined) => { if (!form.tools) form.tools = {}; form.tools.view_image = v }
})
const toolsWebSearch = computed({
  get: () => form.tools?.web_search,
  set: (v: boolean | undefined) => { if (!form.tools) form.tools = {}; form.tools.web_search = v }
})

// tui
const tuiAlternateScreen = computed({
  get: () => form.tui?.alternate_screen ?? '',
  set: (v: string) => { if (!form.tui) form.tui = {}; form.tui.alternate_screen = v || undefined }
})
const tuiAnimations = computed({
  get: () => form.tui?.animations,
  set: (v: boolean | undefined) => { if (!form.tui) form.tui = {}; form.tui.animations = v }
})
const tuiNotifications = computed({
  get: () => form.tui?.notifications,
  set: (v: boolean | undefined) => { if (!form.tui) form.tui = {}; form.tui.notifications = v }
})
const tuiShowTooltips = computed({
  get: () => form.tui?.show_tooltips,
  set: (v: boolean | undefined) => { if (!form.tui) form.tui = {}; form.tui.show_tooltips = v }
})

// history
const historyPersistence = computed({
  get: () => form.history?.persistence ?? '',
  set: (v: string) => { if (!form.history) form.history = {}; form.history.persistence = v || undefined }
})
const historyMaxBytes = computed({
  get: () => form.history?.max_bytes,
  set: (v: number | undefined) => { if (!form.history) form.history = {}; form.history.max_bytes = v }
})

// analytics / feedback
const analyticsEnabled = computed({
  get: () => form.analytics?.enabled,
  set: (v: boolean | undefined) => { if (!form.analytics) form.analytics = {}; form.analytics.enabled = v }
})
const feedbackEnabled = computed({
  get: () => form.feedback?.enabled,
  set: (v: boolean | undefined) => { if (!form.feedback) form.feedback = {}; form.feedback.enabled = v }
})

// ============ Actions ============
function showToast(message: string, type: 'success' | 'error' = 'success') {
  toast.value = { message, type }
  setTimeout(() => { toast.value = null }, 3000)
}

async function loadConfig() {
  loading.value = true
  try {
    const config = await getCodexConfig()
    Object.assign(form, config)
  } catch (e) {
    console.error('Failed to load codex config:', e)
    showToast(t('codex.settings.messages.loadFailed'), 'error')
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  saving.value = true
  try {
    // 构建提交对象，排除 mcp_servers 和 profiles
    const { mcp_servers: _m, profiles: _p, ...payload } = form
    await updateCodexConfig(payload as CodexConfig)
    showToast(t('codex.settings.messages.saveSuccess'))
  } catch (e) {
    console.error('Failed to save codex config:', e)
    showToast(t('codex.settings.messages.saveFailed'), 'error')
  } finally {
    saving.value = false
  }
}

onMounted(loadConfig)

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
    return () => h('label', { class: 'flex items-center gap-3 cursor-pointer' }, [
      h('input', {
        type: 'checkbox',
        checked: props.modelValue ?? false,
        class: 'w-4 h-4 rounded border-border-default text-emerald-500 focus:ring-emerald-500',
        onChange: (e: Event) => emit('update:modelValue', (e.target as HTMLInputElement).checked),
      }),
      h('span', { class: 'text-sm font-semibold text-text-primary' }, props.label),
    ])
  }
})

export default { components: { ToggleField } }
</script>

<style scoped>
.settings-input {
  @apply w-full px-4 py-2.5 rounded-lg bg-bg-elevated border border-border-default focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 outline-none transition-all text-text-primary;
}
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.loading-spinner {
  border: 3px solid;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
