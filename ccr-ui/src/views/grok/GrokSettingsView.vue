<template>
  <PageShell class="grok-settings stage-page">
    <template #header>
      <PageHeader
        :title="t('grok.settings.title')"
        :eyebrow="t('grok.settings.eyebrow')"
        :description="t('grok.settings.subtitle')"
      >
        <template #leading>
          <div class="grok-settings__mark">
            <SIcon
              name="Settings2"
              size="w-6 h-6"
            />
          </div>
        </template>
        <template #actions>
          <RouterLink to="/grok">
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
              {{ t('grok.settings.back') }}
            </Button>
          </RouterLink>
          <Button
            v-if="activeTab !== 'source' && !localOnly"
            variant="primary"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="loading || saving || !isDirty || Boolean(validationErrorKey)"
            :loading="saving"
            @click="handleSave"
          >
            <template #leading>
              <SIcon
                name="Save"
                size="w-4 h-4"
              />
            </template>
            {{ saving ? t('grok.settings.saving') : t('grok.settings.save') }}
          </Button>
        </template>
      </PageHeader>
    </template>

    <template #subnav>
      <ModuleSubnav module="grok" />
    </template>

    <section
      v-if="localOnly"
      class="grok-settings__local-only"
      data-testid="grok-settings-local-only"
    >
      <SIcon
        name="Monitor"
        size="w-6 h-6"
      />
      <div>
        <h2>{{ t('grok.settings.localOnly.title') }}</h2>
        <p>{{ t('grok.settings.localOnly.description') }}</p>
        <span>{{ t('grok.settings.localOnly.environment', { env: localOnlyEnvType ?? t('grok.states.unknown') }) }}</span>
      </div>
    </section>

    <section
      v-else-if="loading"
      class="grok-settings__loading"
    >
      <div class="grok-settings__spinner" />
      <span>{{ t('grok.settings.loading') }}</span>
    </section>

    <section
      v-else-if="loadError"
      class="grok-settings__error"
      role="alert"
    >
      <SIcon
        name="AlertCircle"
        size="w-5 h-5"
      />
      <div>
        <h2>{{ t('grok.settings.messages.loadFailed') }}</h2>
        <p>{{ loadError }}</p>
      </div>
      <Button
        variant="secondary"
        density="compact"
        @click="loadSettings"
      >
        {{ t('grok.settings.reload') }}
      </Button>
    </section>

    <template v-else>
      <div class="grok-settings__status-strip">
        <div>
          <span>{{ t('grok.settings.status.file') }}</span>
          <strong>{{ settings?.exists ? t('grok.settings.status.exists') : t('grok.settings.status.missing') }}</strong>
        </div>
        <div>
          <span>{{ t('grok.settings.status.activation') }}</span>
          <strong>{{ activationLabel }}</strong>
        </div>
        <div>
          <span>{{ t('grok.settings.status.pending') }}</span>
          <strong>{{ t('grok.settings.status.pendingCount', { count: dirtyKeys.size }) }}</strong>
        </div>
      </div>

      <nav
        class="grok-settings__tabs"
        role="tablist"
        :aria-label="t('grok.settings.tabs.label')"
      >
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          role="tab"
          :aria-selected="activeTab === tab.key"
          :class="{ 'grok-settings__tab--active': activeTab === tab.key }"
          @click="changeTab(tab.key)"
        >
          <SIcon
            :name="tab.icon"
            size="w-4 h-4"
          />
          {{ tab.label }}
        </button>
      </nav>

      <section
        v-if="saveState === 'conflict'"
        class="grok-settings__banner grok-settings__banner--warning"
        data-testid="grok-settings-conflict"
        role="alert"
      >
        <SIcon
          name="RefreshCw"
          size="w-5 h-5"
        />
        <div>
          <strong>{{ t('grok.settings.conflict.title') }}</strong>
          <p>{{ t('grok.settings.conflict.description') }}</p>
        </div>
        <button
          type="button"
          @click="reloadLatest"
        >
          {{ t('grok.settings.conflict.reload') }}
        </button>
      </section>

      <section
        v-if="saveState === 'managed_locked'"
        class="grok-settings__banner grok-settings__banner--warning"
        data-testid="grok-settings-managed-error"
        role="alert"
      >
        <SIcon
          name="Lock"
          size="w-5 h-5"
        />
        <div>
          <strong>{{ t('grok.settings.managed.rejectedTitle') }}</strong>
          <p>{{ managedError || t('grok.settings.managed.description') }}</p>
        </div>
        <RouterLink to="/grok/profiles">
          {{ t('grok.settings.managed.action') }}
        </RouterLink>
      </section>

      <main class="grok-settings__content">
        <div
          v-show="activeTab === 'model'"
          class="grok-settings__tab-panel"
        >
          <section
            v-if="settings?.managed_keys_locked"
            class="grok-settings__managed"
            data-testid="grok-settings-managed-banner"
          >
            <SIcon
              name="Lock"
              size="w-5 h-5"
            />
            <div>
              <strong>{{ t('grok.settings.managed.title') }}</strong>
              <p>{{ t('grok.settings.managed.description') }}</p>
            </div>
            <RouterLink to="/grok/profiles">
              {{ t('grok.settings.managed.action') }}
            </RouterLink>
          </section>

          <section class="grok-settings__section">
            <div class="grok-settings__section-heading">
              <div>
                <p>{{ t('grok.settings.model.eyebrow') }}</p>
                <h2>{{ t('grok.settings.model.title') }}</h2>
              </div>
              <span>{{ t('grok.settings.model.description') }}</span>
            </div>

            <div class="grok-settings__fields">
              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.defaultModel') }}</span>
                <input
                  data-testid="grok-settings-model"
                  type="text"
                  :value="form['models.default'] ?? ''"
                  :disabled="settings?.managed_keys_locked"
                  :placeholder="t('grok.settings.placeholders.defaultModel')"
                  @input="setInputValue('models.default', $event)"
                >
                <small>{{ t('grok.settings.helpers.defaultModel') }}</small>
              </label>

              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.reasoningEffort') }}</span>
                <select
                  data-testid="grok-settings-reasoning"
                  :value="form['models.default_reasoning_effort'] ?? ''"
                  :disabled="settings?.managed_keys_locked"
                  @change="setSelectValue('models.default_reasoning_effort', $event)"
                >
                  <option value="">
                    {{ t('grok.settings.options.unset') }}
                  </option>
                  <option
                    v-if="hasUnknownOption('models.default_reasoning_effort', reasoningEfforts)"
                    :value="form['models.default_reasoning_effort'] as string"
                  >
                    {{ t('grok.settings.options.currentValue', { value: form['models.default_reasoning_effort'] }) }}
                  </option>
                  <option
                    v-for="option in reasoningEfforts"
                    :key="option"
                    :value="option"
                  >
                    {{ option }}
                  </option>
                </select>
                <small>{{ t('grok.settings.helpers.reasoningEffort') }}</small>
              </label>
            </div>
          </section>

          <section class="grok-settings__section">
            <div class="grok-settings__section-heading">
              <div>
                <p>{{ t('grok.settings.customModels.eyebrow') }}</p>
                <h2>{{ t('grok.settings.customModels.title') }}</h2>
              </div>
              <button
                type="button"
                @click="changeTab('source')"
              >
                {{ t('grok.settings.customModels.sourceAction') }}
              </button>
            </div>

            <div
              v-if="settings?.custom_models.length"
              class="grok-settings__model-list"
            >
              <article
                v-for="model in settings.custom_models"
                :key="model.id"
                class="grok-settings__model-row"
              >
                <div class="grok-settings__model-id">
                  <span>{{ model.name || model.id }}</span>
                  <code>{{ model.id }}</code>
                </div>
                <div>
                  <span>{{ t('grok.settings.customModels.model') }}</span>
                  <strong>{{ model.model || t('grok.settings.options.unset') }}</strong>
                </div>
                <div>
                  <span>{{ t('grok.settings.customModels.baseUrl') }}</span>
                  <strong>{{ model.base_url_display || t('grok.settings.options.unset') }}</strong>
                </div>
              </article>
            </div>
            <p
              v-else
              class="grok-settings__empty"
            >
              {{ t('grok.settings.customModels.empty') }}
            </p>
          </section>
        </div>

        <div
          v-show="activeTab === 'sessionUi'"
          class="grok-settings__tab-panel"
        >
          <section class="grok-settings__section">
            <div class="grok-settings__section-heading">
              <div>
                <p>{{ t('grok.settings.sessionUi.uiEyebrow') }}</p>
                <h2>{{ t('grok.settings.sessionUi.uiTitle') }}</h2>
              </div>
              <span>{{ t('grok.settings.sessionUi.uiDescription') }}</span>
            </div>

            <div class="grok-settings__fields grok-settings__fields--single">
              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.theme') }}</span>
                <select
                  data-testid="grok-settings-theme"
                  :value="form['ui.theme'] ?? ''"
                  @change="setSelectValue('ui.theme', $event)"
                >
                  <option value="">
                    {{ t('grok.settings.options.unset') }}
                  </option>
                  <option
                    v-if="hasUnknownOption('ui.theme', themes)"
                    :value="form['ui.theme'] as string"
                  >
                    {{ t('grok.settings.options.currentValue', { value: form['ui.theme'] }) }}
                  </option>
                  <option
                    v-for="option in themes"
                    :key="option"
                    :value="option"
                  >
                    {{ t(`grok.settings.themeOptions.${option}`) }}
                  </option>
                </select>
                <small>{{ t('grok.settings.helpers.theme') }}</small>
              </label>
            </div>
          </section>

          <section class="grok-settings__section">
            <div class="grok-settings__section-heading">
              <div>
                <p>{{ t('grok.settings.sessionUi.sessionEyebrow') }}</p>
                <h2>{{ t('grok.settings.sessionUi.sessionTitle') }}</h2>
              </div>
              <span>{{ t('grok.settings.sessionUi.sessionDescription') }}</span>
            </div>

            <div class="grok-settings__fields">
              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.autoCompact') }}</span>
                <input
                  data-testid="grok-settings-auto-compact"
                  type="number"
                  min="0"
                  max="100"
                  step="1"
                  :value="form['session.auto_compact_threshold_percent'] ?? ''"
                  :aria-invalid="validationErrorKey === 'session.auto_compact_threshold_percent'"
                  @input="setInputValue('session.auto_compact_threshold_percent', $event)"
                >
                <small :class="{ 'grok-settings__field-error': validationErrorKey === 'session.auto_compact_threshold_percent' }">
                  {{ validationErrorKey === 'session.auto_compact_threshold_percent'
                    ? t('grok.settings.validation.autoCompact')
                    : t('grok.settings.helpers.autoCompact') }}
                </small>
              </label>

              <div class="grok-settings__field">
                <span>{{ t('grok.settings.fields.loadEnvrc') }}</span>
                <div
                  class="grok-settings__segmented"
                  role="group"
                  :aria-label="t('grok.settings.fields.loadEnvrc')"
                >
                  <button
                    v-for="option in booleanOptions"
                    :key="String(option.value)"
                    type="button"
                    :class="{ 'grok-settings__segmented--active': form['session.load_envrc'] === option.value }"
                    @click="setBooleanValue('session.load_envrc', option.value)"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <small>{{ t('grok.settings.helpers.loadEnvrc') }}</small>
              </div>
            </div>
          </section>
        </div>

        <div
          v-show="activeTab === 'cli'"
          class="grok-settings__tab-panel"
        >
          <section class="grok-settings__section">
            <div class="grok-settings__section-heading">
              <div>
                <p>{{ t('grok.settings.cli.eyebrow') }}</p>
                <h2>{{ t('grok.settings.cli.title') }}</h2>
              </div>
              <span>{{ t('grok.settings.cli.description') }}</span>
            </div>

            <div class="grok-settings__fields">
              <div class="grok-settings__field">
                <span>{{ t('grok.settings.fields.autoUpdate') }}</span>
                <div
                  class="grok-settings__segmented"
                  role="group"
                  :aria-label="t('grok.settings.fields.autoUpdate')"
                >
                  <button
                    v-for="option in booleanOptions"
                    :key="String(option.value)"
                    type="button"
                    :class="{ 'grok-settings__segmented--active': form['cli.auto_update'] === option.value }"
                    @click="setBooleanValue('cli.auto_update', option.value)"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <small>{{ t('grok.settings.helpers.autoUpdate') }}</small>
              </div>

              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.channel') }}</span>
                <select
                  data-testid="grok-settings-channel"
                  :value="form['cli.channel'] ?? ''"
                  @change="setSelectValue('cli.channel', $event)"
                >
                  <option value="">
                    {{ t('grok.settings.options.unset') }}
                  </option>
                  <option
                    v-if="hasUnknownOption('cli.channel', channels)"
                    :value="form['cli.channel'] as string"
                  >
                    {{ t('grok.settings.options.currentValue', { value: form['cli.channel'] }) }}
                  </option>
                  <option
                    v-for="option in channels"
                    :key="option"
                    :value="option"
                  >
                    {{ option }}
                  </option>
                </select>
                <small>{{ t('grok.settings.helpers.channel') }}</small>
              </label>

              <div class="grok-settings__field">
                <span>{{ t('grok.settings.fields.showTips') }}</span>
                <div
                  class="grok-settings__segmented"
                  role="group"
                  :aria-label="t('grok.settings.fields.showTips')"
                >
                  <button
                    v-for="option in booleanOptions"
                    :key="String(option.value)"
                    type="button"
                    :class="{ 'grok-settings__segmented--active': form['cli.show_tips'] === option.value }"
                    @click="setBooleanValue('cli.show_tips', option.value)"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <small>{{ t('grok.settings.helpers.showTips') }}</small>
              </div>
            </div>
          </section>

          <section class="grok-settings__section">
            <div class="grok-settings__section-heading">
              <div>
                <p>{{ t('grok.settings.worktrees.eyebrow') }}</p>
                <h2>{{ t('grok.settings.worktrees.title') }}</h2>
              </div>
              <span>{{ t('grok.settings.worktrees.description') }}</span>
            </div>

            <div class="grok-settings__fields">
              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.newSessionWorktree') }}</span>
                <select
                  :value="form['hints.new_session_worktree_mode'] ?? ''"
                  @change="setSelectValue('hints.new_session_worktree_mode', $event)"
                >
                  <option value="">
                    {{ t('grok.settings.options.unset') }}
                  </option>
                  <option
                    v-if="hasUnknownOption('hints.new_session_worktree_mode', worktreeModes)"
                    :value="form['hints.new_session_worktree_mode'] as string"
                  >
                    {{ t('grok.settings.options.currentValue', { value: form['hints.new_session_worktree_mode'] }) }}
                  </option>
                  <option
                    v-for="option in worktreeModes"
                    :key="option"
                    :value="option"
                  >
                    {{ t(`grok.settings.worktreeOptions.${option}`) }}
                  </option>
                </select>
                <small>{{ t('grok.settings.helpers.newSessionWorktree') }}</small>
              </label>

              <label class="grok-settings__field">
                <span>{{ t('grok.settings.fields.forkWorktree') }}</span>
                <select
                  :value="form['hints.fork_worktree_mode'] ?? ''"
                  @change="setSelectValue('hints.fork_worktree_mode', $event)"
                >
                  <option value="">
                    {{ t('grok.settings.options.unset') }}
                  </option>
                  <option
                    v-if="hasUnknownOption('hints.fork_worktree_mode', worktreeModes)"
                    :value="form['hints.fork_worktree_mode'] as string"
                  >
                    {{ t('grok.settings.options.currentValue', { value: form['hints.fork_worktree_mode'] }) }}
                  </option>
                  <option
                    v-for="option in worktreeModes"
                    :key="option"
                    :value="option"
                  >
                    {{ t(`grok.settings.worktreeOptions.${option}`) }}
                  </option>
                </select>
                <small>{{ t('grok.settings.helpers.forkWorktree') }}</small>
              </label>
            </div>
          </section>
        </div>

        <ConfigSourcePanel
          v-if="activeTab === 'source'"
          language="toml"
          :get-raw="grokApi.getGrokConfigRaw"
          :save-raw="grokApi.saveGrokConfigRaw"
          :list-layers="grokApi.listGrokConfigLayers"
          :backup-notice="t('grok.settings.source.noBackup')"
          :policy-notice="t('grok.settings.source.policyNotice')"
          :policy-layer-ids="policyLayerIds"
          @saved="handleRawSaved"
          @close="activeTab = 'model'"
          @dirty-change="sourceDirty = $event"
        />
      </main>

      <footer
        v-if="activeTab !== 'source'"
        class="grok-settings__footer"
      >
        <SIcon
          name="Info"
          size="w-4 h-4"
        />
        <p>
          {{ t('grok.settings.footer.moreConfig') }}
          <button
            type="button"
            @click="changeTab('source')"
          >
            {{ t('grok.settings.footer.openSource') }}
          </button>
          {{ t('grok.settings.footer.formatting') }}
        </p>
      </footer>
    </template>
  </PageShell>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { grokApi } from '@/api'
import { getCurrentEnvironment } from '@/api/runtime/environment'
import ConfigSourcePanel from '@/components/editor/ConfigSourcePanel.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUIStore } from '@/stores/ui'
import type { GrokSettingsCommandResponse } from '@/types'
import { getErrorMessage } from '@/types/api'
import {
  buildGrokSettingsPatch,
  createEmptyGrokSettingsForm,
  GROK_CHANNELS,
  GROK_REASONING_EFFORTS,
  GROK_THEMES,
  GROK_WORKTREE_MODES,
  grokSettingsResponseToForm,
  validateGrokSettingsForm,
  type GrokSettingsFormValue,
  type GrokSettingsKey,
  type GrokSettingsOkResponse,
} from '@/utils/grokSettings'

defineOptions({ name: 'GrokSettingsView' })

type SettingsTab = 'model' | 'sessionUi' | 'cli' | 'source'
type SaveState = 'idle' | 'conflict' | 'managed_locked'

const { t } = useI18n()
const uiStore = useUIStore()
const loading = ref(true)
const saving = ref(false)
const loadError = ref<string | null>(null)
const localOnly = ref(false)
const localOnlyEnvType = ref<string | null>(null)
const activeTab = ref<SettingsTab>('model')
const sourceDirty = ref(false)
const saveState = ref<SaveState>('idle')
const managedError = ref<string | null>(null)
const settings = ref<GrokSettingsOkResponse | null>(null)
const form = reactive(createEmptyGrokSettingsForm())
const baseline = ref(createEmptyGrokSettingsForm())
const dirtyKeys = ref(new Set<GrokSettingsKey>())

const reasoningEfforts = GROK_REASONING_EFFORTS
const themes = GROK_THEMES
const channels = GROK_CHANNELS
const worktreeModes = GROK_WORKTREE_MODES
const policyLayerIds = [
  'managed_user',
  'managed_system',
  'requirements_user',
  'requirements_system',
]

const tabs = computed<Array<{ key: SettingsTab, label: string, icon: string }>>(() => [
  { key: 'model', label: t('grok.settings.tabs.model'), icon: 'Brain' },
  { key: 'sessionUi', label: t('grok.settings.tabs.sessionUi'), icon: 'Monitor' },
  { key: 'cli', label: t('grok.settings.tabs.cli'), icon: 'Terminal' },
  { key: 'source', label: t('grok.settings.tabs.source'), icon: 'FileCode' },
])

const booleanOptions = computed(() => [
  { value: null, label: t('grok.settings.options.unset') },
  { value: true, label: t('grok.settings.options.enabled') },
  { value: false, label: t('grok.settings.options.disabled') },
])

const isDirty = computed(() => dirtyKeys.value.size > 0)
const validationErrorKey = computed(() => validateGrokSettingsForm(form, dirtyKeys.value))
const activationLabel = computed(() => {
  const activation = settings.value?.activation ?? 'inactive'
  const key = activation === 'unsafe_missing_entry_state'
    ? 'unsafeMissingEntryState'
    : activation
  const label = t(`grok.states.activation.${key}`)
  return settings.value?.activation_name
    ? `${label} · ${settings.value.activation_name}`
    : label
})

const setLocalOnly = (envType: string) => {
  localOnly.value = true
  localOnlyEnvType.value = envType
  settings.value = null
  dirtyKeys.value = new Set()
  saveState.value = 'idle'
}

const applySettings = (response: GrokSettingsOkResponse) => {
  const nextForm = grokSettingsResponseToForm(response)
  Object.assign(form, nextForm)
  baseline.value = { ...nextForm }
  settings.value = response
  dirtyKeys.value = new Set()
  saveState.value = 'idle'
  managedError.value = null
}

const loadSettings = async () => {
  loading.value = true
  loadError.value = null
  try {
    const environment = await getCurrentEnvironment()
    if (environment.env_type !== 'local') {
      setLocalOnly(environment.env_type)
      return
    }

    localOnly.value = false
    localOnlyEnvType.value = null
    const response: GrokSettingsCommandResponse = await grokApi.getGrokSettings()
    if (response.status === 'unsupported_environment') {
      setLocalOnly(response.env_type)
      return
    }
    applySettings(response)
  } catch (error) {
    settings.value = null
    dirtyKeys.value = new Set()
    loadError.value = getErrorMessage(error, t('grok.settings.messages.loadFailed'))
  } finally {
    loading.value = false
  }
}

const updateField = (key: GrokSettingsKey, value: GrokSettingsFormValue) => {
  form[key] = value
  const next = new Set(dirtyKeys.value)
  if (value === baseline.value[key]) next.delete(key)
  else next.add(key)
  dirtyKeys.value = next
}

const setInputValue = (key: GrokSettingsKey, event: Event) => {
  updateField(key, (event.target as HTMLInputElement).value)
}

const setSelectValue = (key: GrokSettingsKey, event: Event) => {
  updateField(key, (event.target as HTMLSelectElement).value)
}

const setBooleanValue = (key: GrokSettingsKey, value: boolean | null) => {
  updateField(key, value)
}

const hasUnknownOption = (
  key: GrokSettingsKey,
  options: readonly string[],
): boolean => {
  const value = form[key]
  return typeof value === 'string' && value !== '' && !options.includes(value)
}

const handleSave = async () => {
  if (!isDirty.value || saving.value) return
  if (validationErrorKey.value) {
    uiStore.showError(t('grok.settings.validation.autoCompact'))
    return
  }

  saving.value = true
  managedError.value = null
  try {
    const response = await grokApi.updateGrokSettings(
      buildGrokSettingsPatch(form, dirtyKeys.value),
    )
    if (response.status === 'saved') {
      uiStore.showSuccess(t('grok.settings.messages.saveSuccess'))
      await loadSettings()
    } else if (response.status === 'conflict') {
      saveState.value = 'conflict'
    } else if (response.status === 'managed_locked') {
      saveState.value = 'managed_locked'
      managedError.value = response.message
    } else {
      setLocalOnly(response.env_type)
    }
  } catch (error) {
    uiStore.showError(getErrorMessage(error, t('grok.settings.messages.saveFailed')))
  } finally {
    saving.value = false
  }
}

const reloadLatest = async () => {
  await loadSettings()
}

const changeTab = async (nextTab: SettingsTab) => {
  if (activeTab.value === nextTab) return
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

const handleRawSaved = async () => {
  sourceDirty.value = false
  activeTab.value = 'model'
  await loadSettings()
}

onMounted(loadSettings)
</script>

<style scoped>
.grok-settings__status-strip,
.grok-settings__tabs,
.grok-settings__banner,
.grok-settings__managed,
.grok-settings__section-heading,
.grok-settings__model-row,
.grok-settings__footer,
.grok-settings__local-only,
.grok-settings__error {
  display: flex;
  align-items: center;
}

.grok-settings__mark {
  display: flex;
  width: 3rem;
  height: 3rem;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  color: var(--color-platform-grok);
  background: rgb(var(--color-platform-grok-rgb) / 12%);
  border: 1px solid rgb(var(--color-platform-grok-rgb) / 24%);
  border-radius: var(--radius-lg);
}

.grok-settings__section-heading p {
  margin: 0;
  color: var(--color-platform-grok);
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0;
}

.grok-settings__status-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  border-bottom: 1px solid var(--stage-border-soft);
}

.grok-settings__status-strip > div {
  min-width: 0;
  padding: 0.875rem 1rem;
  border-right: 1px solid var(--stage-border-soft);
}

.grok-settings__status-strip > div:first-child {
  padding-left: 0;
}

.grok-settings__status-strip > div:last-child {
  border-right: 0;
}

.grok-settings__status-strip span,
.grok-settings__model-row span {
  display: block;
  color: var(--stage-text-muted);
  font-size: 0.7rem;
  font-weight: 600;
}

.grok-settings__status-strip strong {
  display: block;
  margin-top: 0.2rem;
  overflow: hidden;
  color: var(--stage-text-primary);
  font-size: 0.8125rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.grok-settings__tabs {
  gap: 0;
  overflow-x: auto;
  border-bottom: 1px solid var(--stage-border-soft);
}

.grok-settings__tabs button {
  position: relative;
  display: inline-flex;
  min-width: 9rem;
  min-height: 3rem;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0 1rem;
  color: var(--stage-text-muted);
  background: transparent;
  border: 0;
  cursor: pointer;
}

.grok-settings__tabs button::after {
  position: absolute;
  right: 1rem;
  bottom: -1px;
  left: 1rem;
  height: 2px;
  background: transparent;
  content: '';
}

.grok-settings__tabs button:hover,
.grok-settings__tab--active {
  color: var(--stage-text-primary) !important;
}

.grok-settings__tabs .grok-settings__tab--active::after {
  background: var(--color-platform-grok);
}

.grok-settings__content {
  min-height: 30rem;
}

.grok-settings__tab-panel {
  display: grid;
}

.grok-settings__section {
  padding: 1.5rem 0;
  border-bottom: 1px solid var(--stage-border-soft);
}

.grok-settings__section-heading {
  align-items: flex-end;
  justify-content: space-between;
  gap: 2rem;
  margin-bottom: 1.25rem;
}

.grok-settings__section-heading h2 {
  margin: 0.25rem 0 0;
  color: var(--stage-text-primary);
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: 0;
}

.grok-settings__section-heading > span {
  max-width: 34rem;
  color: var(--stage-text-secondary);
  font-size: 0.8125rem;
  line-height: 1.5;
  text-align: right;
}

.grok-settings__section-heading > button,
.grok-settings__footer button,
.grok-settings__banner > button,
.grok-settings__banner > a,
.grok-settings__managed > a {
  flex: 0 0 auto;
  padding: 0;
  color: var(--color-platform-grok);
  background: transparent;
  border: 0;
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
}

.grok-settings__fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1.25rem 2rem;
}

.grok-settings__fields--single {
  grid-template-columns: minmax(16rem, 32rem);
}

.grok-settings__field {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.45rem;
}

.grok-settings__field > span {
  color: var(--stage-text-primary);
  font-size: 0.8125rem;
  font-weight: 600;
}

.grok-settings__field input,
.grok-settings__field select {
  width: 100%;
  min-height: 2.625rem;
  padding: 0 0.75rem;
  color: var(--stage-text-primary);
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-medium);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
}

.grok-settings__field input:focus,
.grok-settings__field select:focus {
  border-color: var(--color-platform-grok);
  outline: 2px solid rgb(var(--color-platform-grok-rgb) / 16%);
  outline-offset: 1px;
}

.grok-settings__field input:disabled,
.grok-settings__field select:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.grok-settings__field small {
  color: var(--stage-text-muted);
  font-size: 0.72rem;
  line-height: 1.45;
}

.grok-settings__field-error {
  color: var(--color-danger) !important;
}

.grok-settings__segmented {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  width: min(100%, 22rem);
  padding: 0.2rem;
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-medium);
  border-radius: var(--radius-md);
}

.grok-settings__segmented button {
  min-height: 2.15rem;
  padding: 0 0.625rem;
  color: var(--stage-text-muted);
  background: transparent;
  border: 0;
  border-radius: calc(var(--radius-md) - 2px);
  font-size: 0.75rem;
  cursor: pointer;
}

.grok-settings__segmented button:hover {
  color: var(--stage-text-primary);
}

.grok-settings__segmented .grok-settings__segmented--active {
  color: var(--stage-text-primary);
  background: var(--stage-surface-strong);
  box-shadow: 0 0 0 1px var(--stage-border-soft);
}

.grok-settings__managed,
.grok-settings__banner,
.grok-settings__local-only,
.grok-settings__error {
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  color: var(--stage-text-secondary);
  background: var(--stage-surface-medium);
  border-left: 3px solid var(--color-warning);
}

.grok-settings__managed {
  margin-top: 1rem;
}

.grok-settings__managed > div,
.grok-settings__banner > div,
.grok-settings__error > div {
  min-width: 0;
  flex: 1 1 auto;
}

.grok-settings__managed strong,
.grok-settings__banner strong,
.grok-settings__local-only h2,
.grok-settings__error h2 {
  color: var(--stage-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.grok-settings__managed p,
.grok-settings__banner p,
.grok-settings__local-only p,
.grok-settings__error p {
  margin: 0.2rem 0 0;
  font-size: 0.8125rem;
  line-height: 1.5;
}

.grok-settings__banner {
  margin-top: 1rem;
}

.grok-settings__model-list {
  border-top: 1px solid var(--stage-border-soft);
}

.grok-settings__model-row {
  display: grid;
  grid-template-columns: minmax(12rem, 1.1fr) minmax(10rem, 1fr) minmax(14rem, 1.5fr);
  gap: 1rem;
  padding: 0.875rem 0;
  border-bottom: 1px solid var(--stage-border-soft);
}

.grok-settings__model-row strong,
.grok-settings__model-id > span {
  display: block;
  margin-top: 0.2rem;
  overflow: hidden;
  color: var(--stage-text-primary);
  font-size: 0.8125rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.grok-settings__model-id code {
  display: block;
  margin-top: 0.2rem;
  color: var(--stage-text-muted);
  font-size: 0.7rem;
}

.grok-settings__empty {
  margin: 0;
  padding: 1.25rem 0;
  color: var(--stage-text-muted);
  font-size: 0.8125rem;
}

.grok-settings__footer {
  align-items: flex-start;
  gap: 0.5rem;
  padding: 1rem 0;
  color: var(--stage-text-muted);
}

.grok-settings__footer p {
  margin: 0;
  font-size: 0.75rem;
  line-height: 1.55;
}

.grok-settings__footer button {
  margin: 0 0.2rem;
}

.grok-settings__loading {
  display: grid;
  min-height: 24rem;
  place-items: center;
  align-content: center;
  gap: 0.75rem;
  color: var(--stage-text-muted);
}

.grok-settings__spinner {
  width: 1.5rem;
  height: 1.5rem;
  border: 2px solid var(--stage-border-medium);
  border-top-color: var(--color-platform-grok);
  border-radius: 50%;
  animation: grok-settings-spin 700ms linear infinite;
}

.grok-settings__local-only,
.grok-settings__error {
  min-height: 12rem;
  margin-top: 1rem;
}

.grok-settings__local-only span {
  display: block;
  margin-top: 0.5rem;
  color: var(--stage-text-muted);
  font-size: 0.75rem;
}

@keyframes grok-settings-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .grok-settings__spinner {
    animation: none;
  }
}

@media (width <= 760px) {
  .grok-settings__section-heading,
  .grok-settings__local-only,
  .grok-settings__error {
    align-items: flex-start;
    flex-direction: column;
  }

  .grok-settings__status-strip,
  .grok-settings__fields,
  .grok-settings__model-row {
    grid-template-columns: 1fr;
  }

  .grok-settings__status-strip > div,
  .grok-settings__status-strip > div:first-child {
    padding: 0.7rem 0;
    border-right: 0;
    border-bottom: 1px solid var(--stage-border-soft);
  }

  .grok-settings__status-strip > div:last-child {
    border-bottom: 0;
  }

  .grok-settings__tabs button {
    min-width: 8.5rem;
  }

  .grok-settings__section-heading > span {
    text-align: left;
  }

  .grok-settings__managed,
  .grok-settings__banner {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .grok-settings__footer {
    padding-right: 3.5rem;
  }
}
</style>
