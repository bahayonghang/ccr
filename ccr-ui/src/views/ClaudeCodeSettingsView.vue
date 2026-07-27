<template>
  <div class="claude-settings-page">
    <div class="claude-settings-spacer" />

    <div class="claude-settings-shell">
      <!-- Header -->
      <div class="claude-settings-header">
        <div class="claude-settings-title-row">
          <h2 class="claude-settings-title">
            <SIcon
              name="Settings2"
              size="w-6 h-6"
              class="claude-settings-title-icon"
            />
            {{ $t('claudeSettings.title') }}
          </h2>
        </div>
        <div class="claude-settings-actions">
          <RouterLink to="/claude-code">
            <button class="claude-settings-button claude-settings-button--secondary">
              <SIcon
                name="ArrowLeft"
                size="w-4 h-4"
                class="claude-settings-button__icon"
              />
              {{ $t('claudeSettings.back') }}
            </button>
          </RouterLink>
          <button
            v-if="activeTab !== 'source'"
            class="claude-settings-button claude-settings-button--primary"
            :disabled="saving"
            @click="handleSave"
          >
            <SIcon
              name="Save"
              size="w-4 h-4"
              class="claude-settings-button__icon"
            />
            {{ saving ? $t('claudeSettings.saving') : $t('claudeSettings.save') }}
          </button>
        </div>
      </div>

      <!-- Loading -->
      <div
        v-if="loading"
        class="claude-settings-loading"
      >
        <div class="loading-spinner claude-settings-loading__spinner" />
        <span>{{ loadingLabel }}</span>
      </div>

      <template v-else>
        <!-- Tab Navigation -->
        <div
          class="claude-settings-tabs"
          role="tablist"
        >
          <button
            v-for="tab in tabs"
            :key="tab.key"
            role="tab"
            :aria-selected="activeTab === tab.key"
            :disabled="tab.disabled"
            :title="tab.disabled ? $t('settingsRaw.unsupportedEnvironment') : undefined"
            class="claude-settings-tab"
            :class="activeTab === tab.key ? 'claude-settings-tab--active' : 'claude-settings-tab--inactive'"
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
          class="claude-settings-panel"
        >
          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <h3 class="claude-settings-section-title">
                {{ $t('claudeSettings.tabs.model') }}
              </h3>

              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.model.defaultModel') }}</label>
                <select
                  v-model="form.model"
                  class="claude-settings-control"
                >
                  <option value="">
                    {{ $t('claudeSettings.model.noOverride') }}
                  </option>
                  <option
                    v-for="m in modelOptions"
                    :key="m"
                    :value="m"
                  >
                    {{ m }}
                  </option>
                </select>
              </div>

              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.model.effortLevel') }}</label>
                <select
                  v-model="form.effortLevel"
                  class="claude-settings-control"
                >
                  <option value="">
                    {{ $t('claudeSettings.model.noOverride') }}
                  </option>
                  <option value="low">
                    low
                  </option>
                  <option value="medium">
                    medium
                  </option>
                  <option value="high">
                    high
                  </option>
                </select>
              </div>

              <!-- Toggle: alwaysThinkingEnabled -->
              <label class="claude-settings-checkbox">
                <input
                  v-model="form.alwaysThinkingEnabled"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.model.alwaysThinking') }}</span>
              </label>

              <div class="claude-settings-grid">
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.model.maxThinkingTokens') }}</label>
                  <input
                    v-model="form.maxThinkingTokens"
                    type="text"
                    placeholder="31999"
                    class="claude-settings-control"
                  >
                </div>
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.model.maxOutputTokens') }}</label>
                  <input
                    v-model="form.maxOutputTokens"
                    type="text"
                    placeholder="64000"
                    class="claude-settings-control"
                  >
                </div>
              </div>

              <!-- TagList: availableModels -->
              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.model.availableModels') }}</label>
                <div
                  v-if="form.availableModels.length > 0"
                  class="claude-settings-chip-list"
                >
                  <span
                    v-for="(item, i) in form.availableModels"
                    :key="i"
                    class="claude-settings-chip"
                  >
                    {{ item }}
                    <button
                      class="claude-settings-chip-remove"
                      @click="form.availableModels.splice(i, 1)"
                    ><SIcon
                      name="X"
                      size="w-3 h-3"
                    /></button>
                  </span>
                </div>
                <div class="claude-settings-chip-entry">
                  <input
                    v-model="tagInputs.availableModels"
                    :placeholder="$t('claudeSettings.model.addModel')"
                    class="claude-settings-chip-input"
                    @keydown.enter.prevent="addTag('availableModels', form.availableModels)"
                  >
                  <button
                    class="claude-settings-chip-button"
                    @click="addTag('availableModels', form.availableModels)"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>
            </div>
          </Card>
        </div>

        <!-- Tab: 权限管理 -->
        <div
          v-show="activeTab === 'permissions'"
          class="claude-settings-panel"
        >
          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <h3 class="claude-settings-section-title">
                {{ $t('claudeSettings.tabs.permissions') }}
              </h3>

              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.permissions.defaultMode') }}</label>
                <select
                  v-model="permDefaultMode"
                  class="claude-settings-control"
                >
                  <option value="">
                    {{ $t('claudeSettings.model.noOverride') }}
                  </option>
                  <option
                    v-for="mode in permissionModeOptions"
                    :key="mode"
                    :value="mode"
                  >
                    {{ mode }}
                  </option>
                </select>
              </div>

              <!-- Toggle: skipDangerousModePermissionPrompt -->
              <label class="claude-settings-checkbox">
                <input
                  v-model="form.skipDangerousModePermissionPrompt"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.permissions.skipDangerous') }}</span>
              </label>

              <!-- TagList: permAllow -->
              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.permissions.allow') }}</label>
                <div
                  v-if="permAllow.length > 0"
                  class="claude-settings-chip-list"
                >
                  <span
                    v-for="(item, i) in permAllow"
                    :key="i"
                    class="claude-settings-chip"
                  >
                    {{ item }}
                    <button
                      class="claude-settings-chip-remove"
                      @click="permAllow.splice(i, 1)"
                    ><SIcon
                      name="X"
                      size="w-3 h-3"
                    /></button>
                  </span>
                </div>
                <div class="claude-settings-chip-entry">
                  <input
                    v-model="tagInputs.permAllow"
                    placeholder="Bash, Read, Write..."
                    class="claude-settings-chip-input"
                    @keydown.enter.prevent="addTag('permAllow', permAllow)"
                  >
                  <button
                    class="claude-settings-chip-button"
                    @click="addTag('permAllow', permAllow)"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>

              <!-- TagList: permDeny -->
              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.permissions.deny') }}</label>
                <div
                  v-if="permDeny.length > 0"
                  class="claude-settings-chip-list"
                >
                  <span
                    v-for="(item, i) in permDeny"
                    :key="i"
                    class="claude-settings-chip"
                  >
                    {{ item }}
                    <button
                      class="claude-settings-chip-remove"
                      @click="permDeny.splice(i, 1)"
                    ><SIcon
                      name="X"
                      size="w-3 h-3"
                    /></button>
                  </span>
                </div>
                <div class="claude-settings-chip-entry">
                  <input
                    v-model="tagInputs.permDeny"
                    placeholder="mcp__dangerous..."
                    class="claude-settings-chip-input"
                    @keydown.enter.prevent="addTag('permDeny', permDeny)"
                  >
                  <button
                    class="claude-settings-chip-button"
                    @click="addTag('permDeny', permDeny)"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>

              <!-- TagList: permAdditionalDirs -->
              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.permissions.additionalDirs') }}</label>
                <div
                  v-if="permAdditionalDirs.length > 0"
                  class="claude-settings-chip-list"
                >
                  <span
                    v-for="(item, i) in permAdditionalDirs"
                    :key="i"
                    class="claude-settings-chip"
                  >
                    {{ item }}
                    <button
                      class="claude-settings-chip-remove"
                      @click="permAdditionalDirs.splice(i, 1)"
                    ><SIcon
                      name="X"
                      size="w-3 h-3"
                    /></button>
                  </span>
                </div>
                <div class="claude-settings-chip-entry">
                  <input
                    v-model="tagInputs.permAdditionalDirs"
                    placeholder="/path/to/dir"
                    class="claude-settings-chip-input"
                    @keydown.enter.prevent="addTag('permAdditionalDirs', permAdditionalDirs)"
                  >
                  <button
                    class="claude-settings-chip-button"
                    @click="addTag('permAdditionalDirs', permAdditionalDirs)"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>
            </div>
          </Card>
        </div>

        <!-- Tab: 环境变量 -->
        <div
          v-show="activeTab === 'env'"
          class="claude-settings-panel"
        >
          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <div class="claude-settings-panel-header">
                <h3 class="claude-settings-section-title">
                  {{ $t('claudeSettings.tabs.env') }}
                </h3>
                <button
                  class="claude-settings-chip-button claude-settings-chip-button--wide"
                  @click="addEnvVar"
                >
                  <SIcon
                    name="Plus"
                    size="w-4 h-4"
                  /> {{ $t('claudeSettings.env.add') }}
                </button>
              </div>

              <div
                v-if="envEntries.length === 0"
                class="claude-settings-empty"
              >
                {{ $t('claudeSettings.env.empty') }}
              </div>

              <div
                v-for="(entry, idx) in envEntries"
                :key="idx"
                class="claude-settings-env-row"
              >
                <input
                  v-model="entry.key"
                  placeholder="KEY"
                  class="claude-settings-chip-input claude-settings-control--mono"
                >
                <input
                  v-model="entry.value"
                  placeholder="value"
                  :type="entry.key.includes('TOKEN') || entry.key.includes('KEY') || entry.key.includes('SECRET') ? 'password' : 'text'"
                  class="claude-settings-chip-input claude-settings-control--mono claude-settings-control--value"
                >
                <button
                  class="claude-settings-delete-button"
                  @click="envEntries.splice(idx, 1)"
                >
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </div>
          </Card>
        </div>

        <!-- Tab: UI 体验 -->
        <div
          v-show="activeTab === 'ui'"
          class="claude-settings-panel"
        >
          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <h3 class="claude-settings-section-title">
                {{ $t('claudeSettings.tabs.ui') }}
              </h3>

              <div class="claude-settings-grid">
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.ui.theme') }}</label>
                  <input
                    v-model="form.theme"
                    type="text"
                    placeholder="dark, light, dark-daltonized..."
                    class="claude-settings-control"
                  >
                </div>
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.ui.language') }}</label>
                  <input
                    v-model="form.language"
                    type="text"
                    placeholder="zh-CN, en, ja..."
                    class="claude-settings-control"
                  >
                </div>
              </div>

              <div class="claude-settings-checkbox-group">
                <label class="claude-settings-checkbox">
                  <input
                    v-model="form.showTurnDuration"
                    type="checkbox"
                    class="claude-settings-checkbox__input"
                  >
                  <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.showTurnDuration') }}</span>
                </label>
                <label class="claude-settings-checkbox">
                  <input
                    v-model="form.spinnerTipsEnabled"
                    type="checkbox"
                    class="claude-settings-checkbox__input"
                  >
                  <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.spinnerTips') }}</span>
                </label>
                <label class="claude-settings-checkbox">
                  <input
                    v-model="form.terminalProgressBarEnabled"
                    type="checkbox"
                    class="claude-settings-checkbox__input"
                  >
                  <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.progressBar') }}</span>
                </label>
                <label class="claude-settings-checkbox">
                  <input
                    v-model="form.showSpinnerTree"
                    type="checkbox"
                    class="claude-settings-checkbox__input"
                  >
                  <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.spinnerTree') }}</span>
                </label>
                <label class="claude-settings-checkbox">
                  <input
                    v-model="form.prefersReducedMotion"
                    type="checkbox"
                    class="claude-settings-checkbox__input"
                  >
                  <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.reducedMotion') }}</span>
                </label>
              </div>
            </div>
          </Card>

          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <h3 class="claude-settings-section-title">
                {{ $t('claudeSettings.ui.misc') }}
              </h3>
              <div class="claude-settings-grid">
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.ui.updateChannel') }}</label>
                  <select
                    v-model="form.autoUpdatesChannel"
                    class="claude-settings-control"
                  >
                    <option value="">
                      {{ $t('claudeSettings.model.noOverride') }}
                    </option>
                    <option
                      v-for="channel in updateChannelOptions"
                      :key="channel"
                      :value="channel"
                    >
                      {{ channel }}
                    </option>
                  </select>
                </div>
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.ui.cleanupDays') }}</label>
                  <input
                    v-model.number="form.cleanupPeriodDays"
                    type="number"
                    placeholder="30"
                    class="claude-settings-control"
                  >
                </div>
              </div>
              <label class="claude-settings-checkbox">
                <input
                  v-model="form.autoUpdates"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.autoUpdates') }}</span>
              </label>
              <label class="claude-settings-checkbox">
                <input
                  v-model="form.respectGitignore"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.ui.respectGitignore') }}</span>
              </label>
            </div>
          </Card>
        </div>

        <!-- Tab: 沙箱安全 -->
        <div
          v-show="activeTab === 'sandbox'"
          class="claude-settings-panel"
        >
          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <h3 class="claude-settings-section-title">
                {{ $t('claudeSettings.tabs.sandbox') }}
              </h3>

              <label class="claude-settings-checkbox">
                <input
                  v-model="sandboxEnabled"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.sandbox.enabled') }}</span>
              </label>
              <label class="claude-settings-checkbox">
                <input
                  v-model="sandboxAutoAllow"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.sandbox.autoAllowBash') }}</span>
              </label>
              <label class="claude-settings-checkbox">
                <input
                  v-model="sandboxAllowLocal"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.sandbox.allowLocalBinding') }}</span>
              </label>

              <!-- TagList: sandboxAllowedDomains -->
              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.sandbox.allowedDomains') }}</label>
                <div
                  v-if="sandboxAllowedDomains.length > 0"
                  class="claude-settings-chip-list"
                >
                  <span
                    v-for="(item, i) in sandboxAllowedDomains"
                    :key="i"
                    class="claude-settings-chip"
                  >
                    {{ item }}
                    <button
                      class="claude-settings-chip-remove"
                      @click="sandboxAllowedDomains.splice(i, 1)"
                    ><SIcon
                      name="X"
                      size="w-3 h-3"
                    /></button>
                  </span>
                </div>
                <div class="claude-settings-chip-entry">
                  <input
                    v-model="tagInputs.sandboxAllowedDomains"
                    placeholder="api.anthropic.com"
                    class="claude-settings-chip-input"
                    @keydown.enter.prevent="addTag('sandboxAllowedDomains', sandboxAllowedDomains)"
                  >
                  <button
                    class="claude-settings-chip-button"
                    @click="addTag('sandboxAllowedDomains', sandboxAllowedDomains)"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>

              <!-- TagList: sandboxExcludedCmds -->
              <div>
                <label class="claude-settings-field-label">{{ $t('claudeSettings.sandbox.excludedCommands') }}</label>
                <div
                  v-if="sandboxExcludedCmds.length > 0"
                  class="claude-settings-chip-list"
                >
                  <span
                    v-for="(item, i) in sandboxExcludedCmds"
                    :key="i"
                    class="claude-settings-chip"
                  >
                    {{ item }}
                    <button
                      class="claude-settings-chip-remove"
                      @click="sandboxExcludedCmds.splice(i, 1)"
                    ><SIcon
                      name="X"
                      size="w-3 h-3"
                    /></button>
                  </span>
                </div>
                <div class="claude-settings-chip-entry">
                  <input
                    v-model="tagInputs.sandboxExcludedCmds"
                    placeholder="docker, npm..."
                    class="claude-settings-chip-input"
                    @keydown.enter.prevent="addTag('sandboxExcludedCmds', sandboxExcludedCmds)"
                  >
                  <button
                    class="claude-settings-chip-button"
                    @click="addTag('sandboxExcludedCmds', sandboxExcludedCmds)"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>
            </div>
          </Card>
        </div>

        <!-- Tab: Git 归属 -->
        <div
          v-show="activeTab === 'git'"
          class="claude-settings-panel"
        >
          <Card
            variant="glass"
            pattern
          >
            <div class="claude-settings-panel-body">
              <h3 class="claude-settings-section-title">
                {{ $t('claudeSettings.tabs.git') }}
              </h3>

              <div class="claude-settings-grid">
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.git.commitAttribution') }}</label>
                  <select
                    v-model="attrCommit"
                    class="claude-settings-control"
                  >
                    <option value="">
                      {{ $t('claudeSettings.model.noOverride') }}
                    </option>
                    <option
                      v-for="option in attributionOptions"
                      :key="option"
                      :value="option"
                    >
                      {{ option }}
                    </option>
                  </select>
                </div>
                <div>
                  <label class="claude-settings-field-label">{{ $t('claudeSettings.git.prAttribution') }}</label>
                  <select
                    v-model="attrPr"
                    class="claude-settings-control"
                  >
                    <option value="">
                      {{ $t('claudeSettings.model.noOverride') }}
                    </option>
                    <option
                      v-for="option in attributionOptions"
                      :key="option"
                      :value="option"
                    >
                      {{ option }}
                    </option>
                  </select>
                </div>
              </div>

              <label class="claude-settings-checkbox">
                <input
                  v-model="form.includeCoAuthoredBy"
                  type="checkbox"
                  class="claude-settings-checkbox__input"
                >
                <span class="claude-settings-checkbox__label">{{ $t('claudeSettings.git.includeCoAuthored') }}</span>
              </label>
            </div>
          </Card>
        </div>

        <ConfigSourcePanel
          v-if="activeTab === 'source'"
          language="json"
          :get-raw="getClaudeSettingsRaw"
          :save-raw="saveClaudeSettingsRaw"
          :list-layers="listClaudeSettingsLayers"
          @saved="handleRawSaved"
          @close="activeTab = 'model'"
          @dirty-change="sourceDirty = $event"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/Card.vue'
import ConfigSourcePanel from '@/components/editor/ConfigSourcePanel.vue'
import {
  getClaudeSettings,
  getClaudeSettingsRaw,
  getCurrentEnvironment,
  listClaudeSettingsLayers,
  saveClaudeSettingsRaw,
  updateClaudeSettings,
} from '@/api'
import type { ClaudeSettingsData } from '@/api'
import { useUIStore } from '@/stores/ui'

const { t, locale } = useI18n()
const uiStore = useUIStore()
const loadingLabel = computed(() => locale.value.startsWith('zh') ? '加载中...' : 'Loading...')

const loading = ref(true)
const saving = ref(false)
const activeTab = ref('model')
const rawLocal = ref(true)
const sourceDirty = ref(false)

const tabs = computed(() => [
  { key: 'model', label: t('claudeSettings.tabs.model'), icon: 'Brain' },
  { key: 'permissions', label: t('claudeSettings.tabs.permissions'), icon: 'Shield' },
  { key: 'env', label: t('claudeSettings.tabs.env'), icon: 'Terminal' },
  { key: 'ui', label: t('claudeSettings.tabs.ui'), icon: 'Palette' },
  { key: 'sandbox', label: t('claudeSettings.tabs.sandbox'), icon: 'Lock' },
  { key: 'git', label: t('claudeSettings.tabs.git'), icon: 'GitBranch' },
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
  await loadSettings()
}

async function loadActiveEnvironment() {
  try {
    const environment = await getCurrentEnvironment()
    rawLocal.value = !environment || environment.env_type === 'local'
  } catch {
    rawLocal.value = true
  }
}

const modelOptions = ['opus', 'sonnet', 'haiku', 'claude-opus-4-6', 'claude-sonnet-4-5-20250929', 'claude-haiku-4-5-20251001']
const permissionModeOptions = ['default', 'plan', 'bypassPermissions']
const updateChannelOptions = ['stable', 'latest']
const attributionOptions = ['none', 'co-authored-by', 'authored-by']

// --- Form state ---
const form = reactive<{
  model?: string
  availableModels: string[]
  alwaysThinkingEnabled?: boolean
  maxThinkingTokens?: string
  maxOutputTokens?: string
  effortLevel?: string
  skipDangerousModePermissionPrompt?: boolean
  theme?: string
  language?: string
  showTurnDuration?: boolean
  prefersReducedMotion?: boolean
  spinnerTipsEnabled?: boolean
  terminalProgressBarEnabled?: boolean
  showSpinnerTree?: boolean
  includeCoAuthoredBy?: boolean
  autoUpdates?: boolean
  autoUpdatesChannel?: string
  cleanupPeriodDays?: number
  respectGitignore?: boolean
}>({
  availableModels: [],
})

// Permissions (flat refs for reactivity)
const permAllow = ref<string[]>([])
const permDeny = ref<string[]>([])
const permDefaultMode = ref('')
const permAdditionalDirs = ref<string[]>([])

// Env entries (key-value pairs)
const envEntries = ref<{ key: string; value: string }[]>([])

// Sandbox
const sandboxEnabled = ref<boolean | undefined>()
const sandboxAutoAllow = ref<boolean | undefined>()
const sandboxAllowLocal = ref<boolean | undefined>()
const sandboxAllowedDomains = ref<string[]>([])
const sandboxExcludedCmds = ref<string[]>([])

// Attribution
const attrCommit = ref('')
const attrPr = ref('')

// Tag list input state
const tagInputs = reactive<Record<string, string>>({
  availableModels: '',
  permAllow: '',
  permDeny: '',
  permAdditionalDirs: '',
  sandboxAllowedDomains: '',
  sandboxExcludedCmds: '',
})

// --- Tag helpers ---
function addTag(field: string, targetArray: string[]) {
  const val = tagInputs[field]?.trim()
  if (val && !targetArray.includes(val)) {
    targetArray.push(val)
    tagInputs[field] = ''
  }
}

function parseOptionalInteger(value?: string): number | undefined {
  const trimmed = value?.trim()
  if (!trimmed) {
    return undefined
  }
  const parsed = Number.parseInt(trimmed, 10)
  return Number.isFinite(parsed) ? parsed : undefined
}

// --- Load ---
async function loadSettings() {
  loading.value = true
  try {
    const data = await getClaudeSettings()

    form.model = data.model || ''
    form.availableModels = data.availableModels || []
    form.alwaysThinkingEnabled = data.alwaysThinkingEnabled
    form.maxThinkingTokens = data.maxThinkingTokens != null ? String(data.maxThinkingTokens) : ''
    form.maxOutputTokens = data.maxOutputTokens != null ? String(data.maxOutputTokens) : ''
    form.effortLevel = data.effortLevel || ''
    form.skipDangerousModePermissionPrompt = data.skipDangerousModePermissionPrompt
    form.theme = data.theme
    form.language = data.language
    form.showTurnDuration = data.showTurnDuration
    form.prefersReducedMotion = data.prefersReducedMotion
    form.spinnerTipsEnabled = data.spinnerTipsEnabled
    form.terminalProgressBarEnabled = data.terminalProgressBarEnabled
    form.showSpinnerTree = data.showSpinnerTree
    form.includeCoAuthoredBy = data.includeCoAuthoredBy
    form.autoUpdates = data.autoUpdates
    form.autoUpdatesChannel = data.autoUpdatesChannel || ''
    form.cleanupPeriodDays = data.cleanupPeriodDays
    form.respectGitignore = data.respectGitignore

    permAllow.value = data.permissions?.allow || []
    permDeny.value = data.permissions?.deny || []
    permDefaultMode.value = data.permissions?.defaultMode || ''
    permAdditionalDirs.value = data.permissions?.additionalDirectories || []

    envEntries.value = Object.entries(data.env || {}).map(([key, value]) => ({
      key,
      value: typeof value === 'string' ? value : String(value ?? ''),
    }))

    sandboxEnabled.value = data.sandbox?.enabled
    sandboxAutoAllow.value = data.sandbox?.autoAllowBashIfSandboxed
    sandboxAllowLocal.value = data.sandbox?.network?.allowLocalBinding
    sandboxAllowedDomains.value = data.sandbox?.network?.allowedDomains || []
    sandboxExcludedCmds.value = data.sandbox?.excludedCommands || []

    attrCommit.value = data.attribution?.commit || ''
    attrPr.value = data.attribution?.pr || ''
  } catch (e: unknown) {
    uiStore.showError(`Failed to load settings: ${e instanceof Error ? e.message : e}`)
  } finally {
    loading.value = false
  }
}

// --- Save ---
async function handleSave() {
  saving.value = true
  try {
    const env: Record<string, string> = {}
    for (const entry of envEntries.value) {
      if (entry.key.trim()) {
        env[entry.key.trim()] = entry.value
      }
    }

    const data: ClaudeSettingsData = {
      model: form.model || undefined,
      availableModels: form.availableModels.length > 0 ? form.availableModels : undefined,
      alwaysThinkingEnabled: form.alwaysThinkingEnabled,
      maxThinkingTokens: parseOptionalInteger(form.maxThinkingTokens),
      maxOutputTokens: parseOptionalInteger(form.maxOutputTokens),
      effortLevel: form.effortLevel || undefined,
      permissions: {
        allow: permAllow.value,
        deny: permDeny.value,
        defaultMode: permDefaultMode.value || undefined,
        additionalDirectories: permAdditionalDirs.value.length > 0 ? permAdditionalDirs.value : undefined,
      },
      skipDangerousModePermissionPrompt: form.skipDangerousModePermissionPrompt,
      env,
      theme: form.theme || undefined,
      language: form.language || undefined,
      showTurnDuration: form.showTurnDuration,
      prefersReducedMotion: form.prefersReducedMotion,
      spinnerTipsEnabled: form.spinnerTipsEnabled,
      terminalProgressBarEnabled: form.terminalProgressBarEnabled,
      showSpinnerTree: form.showSpinnerTree,
      sandbox: (sandboxEnabled.value != null || sandboxAutoAllow.value != null || sandboxAllowedDomains.value.length > 0 || sandboxExcludedCmds.value.length > 0) ? {
        enabled: sandboxEnabled.value,
        autoAllowBashIfSandboxed: sandboxAutoAllow.value,
        excludedCommands: sandboxExcludedCmds.value.length > 0 ? sandboxExcludedCmds.value : undefined,
        network: (sandboxAllowLocal.value != null || sandboxAllowedDomains.value.length > 0) ? {
          allowLocalBinding: sandboxAllowLocal.value,
          allowedDomains: sandboxAllowedDomains.value.length > 0 ? sandboxAllowedDomains.value : undefined,
        } : undefined,
      } : undefined,
      attribution: (attrCommit.value || attrPr.value) ? {
        commit: attrCommit.value || undefined,
        pr: attrPr.value || undefined,
      } : undefined,
      includeCoAuthoredBy: form.includeCoAuthoredBy,
      autoUpdates: form.autoUpdates,
      autoUpdatesChannel: form.autoUpdatesChannel || undefined,
      cleanupPeriodDays: form.cleanupPeriodDays,
      respectGitignore: form.respectGitignore,
    }

    await updateClaudeSettings(data)
    uiStore.showSuccess(t('claudeSettings.saveSuccess'))
  } catch (e: unknown) {
    uiStore.showError(`Failed to save: ${e instanceof Error ? e.message : e}`)
  } finally {
    saving.value = false
  }
}

function addEnvVar() {
  envEntries.value.push({ key: '', value: '' })
}

onMounted(() => {
  void loadActiveEnvironment()
  void loadSettings()
})
</script>

<style scoped>
.claude-settings-page {
  min-height: 100%;
  padding: 1.25rem;
  transition: color 0.3s ease, background-color 0.3s ease, border-color 0.3s ease;
}

.claude-settings-spacer {
  margin-bottom: 1.5rem;
}

.claude-settings-shell {
  width: 100%;
  max-width: 75rem;
  margin: 0 auto;
}

.claude-settings-header {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.claude-settings-title-row,
.claude-settings-title,
.claude-settings-actions,
.claude-settings-panel-header,
.claude-settings-chip-entry,
.claude-settings-env-row,
.claude-settings-checkbox,
.claude-settings-tab,
.claude-settings-button {
  display: flex;
  align-items: center;
}

.claude-settings-title-row {
  gap: 1rem;
}

.claude-settings-title {
  gap: 0.5rem;
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.claude-settings-title-icon {
  margin-right: 0.5rem;
  color: var(--color-accent-secondary);
}

.claude-settings-actions {
  gap: 0.75rem;
}

.claude-settings-button {
  min-height: 2.75rem;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  font-weight: 500;
  transition: color 0.2s ease, background-color 0.2s ease, border-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
}

.claude-settings-button:disabled,
.claude-settings-tab:disabled,
.claude-settings-chip-button:disabled,
.claude-settings-delete-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.claude-settings-button__icon {
  margin-right: 0.5rem;
}

.claude-settings-button--secondary {
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
}

.claude-settings-button--secondary:hover {
  background: var(--color-bg-surface);
}

.claude-settings-button--primary,
.claude-settings-chip-button,
.claude-settings-chip-button--wide {
  background: var(--color-accent-secondary);
  color: #fff;
}

.claude-settings-button--primary {
  box-shadow: var(--shadow-md);
}

.claude-settings-button--primary:hover,
.claude-settings-chip-button:hover,
.claude-settings-chip-button--wide:hover {
  transform: scale(1.05);
}

.claude-settings-button--primary:hover {
  box-shadow: var(--shadow-lg);
}

.claude-settings-loading,
.claude-settings-empty {
  color: var(--color-text-muted);
  text-align: center;
}

.claude-settings-loading {
  padding: 5rem 0;
}

.claude-settings-loading__spinner {
  width: 2rem;
  height: 2rem;
  margin: 0 auto 1rem;
  border-color: color-mix(in srgb, var(--color-accent-secondary) 30%, transparent);
  border-top-color: var(--color-accent-secondary);
}

.claude-settings-tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
  padding-bottom: 0.5rem;
  overflow-x: auto;
  scrollbar-width: thin;
}

.claude-settings-tab {
  gap: 0.5rem;
  flex-shrink: 0;
  min-height: 2.75rem;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  white-space: nowrap;
  transition: color 0.2s ease, background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.claude-settings-tab--active {
  background: var(--color-accent-secondary);
  color: #fff;
  box-shadow: var(--shadow-md);
}

.claude-settings-tab--inactive {
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
}

.claude-settings-tab--inactive:hover {
  background: var(--color-bg-surface);
}

.claude-settings-panel {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.claude-settings-panel-body {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  padding: 1.25rem;
}

.claude-settings-panel-header {
  justify-content: space-between;
  gap: 1rem;
}

.claude-settings-section-title {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.claude-settings-field-label {
  display: block;
  margin-bottom: 0.375rem;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.claude-settings-control,
.claude-settings-chip-input {
  width: 100%;
  border: 1px solid var(--color-border-default);
  border-radius: 0.5rem;
  outline: none;
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
  transition: border-color 0.2s ease, box-shadow 0.2s ease, background-color 0.2s ease;
}

.claude-settings-control {
  padding: 0.625rem 1rem;
}

.claude-settings-control:focus,
.claude-settings-chip-input:focus {
  border-color: var(--color-accent-secondary);
  box-shadow: 0 0 0 1px var(--color-accent-secondary);
}

.claude-settings-chip-input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
}

.claude-settings-control--mono,
.claude-settings-control--value {
  font-family: var(--font-family-mono);
  font-size: 0.875rem;
}

.claude-settings-control--value {
  flex: 2;
}

.claude-settings-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
}

.claude-settings-checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.claude-settings-checkbox {
  gap: 0.75rem;
  cursor: pointer;
}

.claude-settings-checkbox__input {
  width: 1rem;
  height: 1rem;
  border-radius: 0.25rem;
  border-color: var(--color-border-default);
  color: var(--color-accent-secondary);
}

.claude-settings-checkbox__input:focus {
  box-shadow: 0 0 0 1px var(--color-accent-secondary);
}

.claude-settings-checkbox__label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.claude-settings-chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.claude-settings-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.625rem;
  border: 1px solid color-mix(in srgb, var(--color-accent-secondary) 20%, transparent);
  border-radius: 0.5rem;
  background: color-mix(in srgb, var(--color-accent-secondary) 10%, transparent);
  color: var(--color-accent-secondary);
  font-size: 0.875rem;
}

.claude-settings-chip-remove {
  transition: color 0.2s ease;
}

.claude-settings-chip-remove:hover {
  color: #f87171;
}

.claude-settings-chip-entry {
  gap: 0.5rem;
}

.claude-settings-chip-button,
.claude-settings-chip-button--wide {
  border-radius: 0.5rem;
  font-size: 0.875rem;
  transition: color 0.2s ease, background-color 0.2s ease, border-color 0.2s ease, transform 0.2s ease;
}

.claude-settings-chip-button {
  padding: 0.5rem 0.75rem;
}

.claude-settings-chip-button--wide {
  gap: 0.25rem;
  padding: 0.375rem 0.75rem;
  font-weight: 500;
}

.claude-settings-empty {
  padding: 2rem 0;
}

.claude-settings-env-row {
  gap: 0.5rem;
  align-items: flex-start;
}

.claude-settings-delete-button {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 2.25rem;
  min-height: 2.25rem;
  padding: 0.5rem;
  border-radius: 0.5rem;
  color: #f87171;
  transition: color 0.2s ease, background-color 0.2s ease;
}

.claude-settings-delete-button:hover {
  background: rgb(239 68 68 / 10%);
}

@media (width >= 640px) {
  .claude-settings-header {
    flex-direction: row;
    align-items: center;
  }

  .claude-settings-title {
    font-size: 1.5rem;
  }

  .claude-settings-title-icon {
    width: 1.75rem;
    height: 1.75rem;
  }

  .claude-settings-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width >= 768px) {
  .claude-settings-tabs {
    flex-wrap: wrap;
    overflow-x: visible;
    padding-bottom: 0;
  }
}
</style>
