<!-- -->
<template>
  <div class="codex-profiles-view">
    <div class="codex-profiles-shell">
      <div class="codex-profiles-stack">
        <ModuleSubnav module="codex" />

        <main class="codex-profiles-main">
          <!-- Header Section -->
          <div class="codex-profiles-header">
            <div class="codex-profiles-header__intro">
              <div class="codex-profiles-header__icon">
                <SIcon
                  name="Settings"
                  size="w-6 h-6"
                  class="text-platform-codex"
                />
              </div>
              <div>
                <h1 class="text-2xl font-bold text-white">
                  {{ $t('codex.profiles.title') }}
                </h1>
                <p class="text-sm text-white/80 mt-1">
                  {{ $t('codex.profiles.subtitle') }}
                </p>
              </div>
            </div>

            <div class="codex-profiles-header__actions">
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
                  {{ $t('codex.profiles.backToCodex') }}
                </Button>
              </RouterLink>

              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                @click="handleAdd"
              >
                <template #leading>
                  <SIcon
                    name="Plus"
                    size="w-4 h-4"
                  />
                </template>
                {{ $t('codex.profiles.addProfile') }}
              </Button>
            </div>
          </div>

          <!-- Status Cards -->
          <div class="codex-profiles-status-grid">
            <!-- Current Config -->
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :gradient-border="true"
              glow-color="warning"
              class="group codex-profiles-status-card"
            >
              <div class="flex items-center gap-4">
                <div class="codex-profiles-status-icon codex-profiles-status-icon--warning">
                  <SIcon
                    name="Zap"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-white/50 uppercase tracking-wider mb-1">
                    {{ $t('codex.status.currentConfig') }}
                  </p>
                  <p class="text-xl font-bold text-white truncate">
                    {{ currentProfile || $t('codex.status.notSet') }}
                  </p>
                </div>
              </div>
            </Card>

            <!-- Total Profiles -->
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :interactive="true"
              glow-color="primary"
              class="group codex-profiles-status-card"
            >
              <div class="flex items-center gap-4">
                <div class="codex-profiles-status-icon codex-profiles-status-icon--primary">
                  <SIcon
                    name="Layers"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-white/50 uppercase tracking-wider mb-1">
                    {{ $t('codex.status.totalProfiles') }}
                  </p>
                  <p class="text-xl font-bold text-white">
                    {{ profiles.length }}
                  </p>
                </div>
              </div>
            </Card>

            <!-- Config Mode -->
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :interactive="true"
              :glow-color="currentConfigMode === 'official' ? 'success' : 'secondary'"
              class="group codex-profiles-status-card"
            >
              <div class="flex items-center gap-4">
                <div 
                  class="codex-profiles-status-icon"
                  :class="currentConfigMode === 'official' ? 'codex-profiles-status-icon--official' : 'codex-profiles-status-icon--relay'"
                >
                  <SIcon
                    :name="currentConfigMode === 'official' ? 'Globe' : 'Server'"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-white/50 uppercase tracking-wider mb-1">
                    {{ $t('codex.status.configMode') }}
                  </p>
                  <p class="text-xl font-bold text-white">
                    {{ currentConfigMode === 'official' ? $t('codex.profiles.officialConfig') : $t('codex.profiles.customRelay') }}
                  </p>
                </div>
              </div>
            </Card>
          </div>

          <!-- Quick Switch -->
          <Card
            v-if="profiles.length > 0"
            surface="workspace"
            :elevation="2"
            motion="subtle"
            padding="lg"
          >
            <div class="flex items-center gap-2 mb-4">
              <SIcon
                name="Shuffle"
                size="w-5 h-5"
                class="text-platform-codex"
              />
              <h3 class="text-base font-semibold text-white">
                {{ $t('codex.profiles.quickSwitch') }}
              </h3>
            </div>
            <div class="codex-profiles-switches">
              <button
                v-for="profile in profiles"
                :key="profile.name"
                class="codex-profiles-switch"
                :class="[
                  profile.name === currentProfile ? 'codex-profiles-switch--active' : 'codex-profiles-switch--idle',
                  actionLoading ? 'codex-profiles-switch--busy' : '',
                ]"
                :disabled="actionLoading"
                @click="handleApply(profile.name)"
              >
                <SIcon
                  v-if="isOfficialConfig(profile)"
                  name="Star"
                  size="w-3.5 h-3.5"
                  :class="profile.name === currentProfile ? 'text-platform-codex' : 'text-yellow-500'"
                />
                <SIcon
                  v-if="busyProfileName === profile.name && busyAction === 'apply'"
                  name="RefreshCw"
                  size="w-3.5 h-3.5"
                  class="animate-spin"
                />
                <span>{{ profile.name }}</span>
                <div 
                  v-if="profile.name === currentProfile" 
                  class="codex-profiles-switch__active-indicator"
                >
                  <SIcon
                    name="Check"
                    size="w-2.5 h-2.5"
                  />
                </div>
              </button>
            </div>
          </Card>

          <!-- Profile List Title -->
          <div class="codex-profiles-section-heading">
            <h2 class="codex-profiles-section-heading__title">
              <SIcon
                name="ListFilter"
                size="w-5 h-5"
                class="text-platform-codex"
              />
              {{ $t('codex.profiles.listTitle') }}
            </h2>
          </div>
            
          <!-- Loading State -->
          <div
            v-if="loading"
            class="codex-profiles-loading"
          >
            <div class="codex-profiles-loading__spinner" />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="profiles.length === 0"
            class="empty-state glass-effect rounded-2xl border border-white/5"
          >
            <div class="p-4 rounded-full glass-surface mb-4">
              <SIcon
                name="Boxes"
                size="w-8 h-8"
                class="text-white/50"
              />
            </div>
            <p class="text-white/80">
              {{ $t('codex.profiles.emptyState') }}
            </p>
          </div>

          <!-- Profile Grid -->
          <div
            v-else
            class="codex-profiles-grid"
          >
            <Card 
              v-for="profile in profiles" 
              :key="profile.name"
              surface="card"
              :elevation="2"
              motion="subtle"
              class="group codex-profiles-card"
              :class="[currentProfile && profile.name === currentProfile ? 'config-card-active' : '']"
              :glow-color="currentProfile && profile.name === currentProfile ? 'warning' : 'primary'"
              padding="lg"
            >
              <!-- Active Indicator Background -->
              <div 
                v-if="currentProfile && profile.name === currentProfile"
                class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl from-platform-codex/10 to-transparent -mr-8 -mt-8 rounded-bl-full pointer-events-none"
              />

              <div class="relative z-10">
                <div class="flex items-start justify-between gap-4 mb-4">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <h3 class="text-lg font-bold font-mono text-white truncate">
                        {{ profile.name }}
                      </h3>
                      <span 
                        v-if="currentProfile && profile.name === currentProfile"
                        class="badge badge-primary"
                      >
                        {{ $t('codex.profiles.currentBadge') }}
                      </span>
                      <span 
                        v-else-if="profile.enabled === false"
                        class="badge badge-danger"
                      >
                        {{ $t('codex.states.disabled') }}
                      </span>
                      <span 
                        v-else
                        class="badge badge-success"
                      >
                        {{ $t('codex.states.enabled') }}
                      </span>
                    </div>
                    <p
                      v-if="profile.description"
                      class="text-sm text-white/80 line-clamp-1"
                    >
                      {{ profile.description }}
                    </p>
                  </div>
                   
                  <!-- Actions -->
                  <div class="codex-profiles-card__actions">
                    <button 
                      class="codex-profiles-action-button codex-profiles-action-button--success"
                      :title="$t('codex.profiles.apply')"
                      :disabled="actionLoading"
                      @click.stop="handleApply(profile.name)"
                    >
                      <SIcon
                        :name="busyProfileName === profile.name && busyAction === 'apply' ? 'RefreshCw' : 'Check'"
                        size="w-4 h-4"
                        :class="{ 'animate-spin': busyProfileName === profile.name && busyAction === 'apply' }"
                      />
                    </button>
                    <button 
                      class="codex-profiles-action-button codex-profiles-action-button--primary"
                      :title="$t('codex.actions.edit')"
                      @click.stop="handleEdit(profile.name)"
                    >
                      <SIcon
                        name="Edit2"
                        size="w-4 h-4"
                      />
                    </button>
                    <button 
                      class="codex-profiles-action-button codex-profiles-action-button--danger"
                      :title="$t('codex.actions.delete')"
                      :disabled="actionLoading"
                      @click.stop="handleDelete(profile.name)"
                    >
                      <SIcon
                        :name="busyProfileName === profile.name && busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
                        size="w-4 h-4"
                        :class="{ 'animate-spin': busyProfileName === profile.name && busyAction === 'delete' }"
                      />
                    </button>
                  </div>
                </div>

                <!-- Info Grid -->
                <div class="codex-profiles-card__info-grid">
                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.baseUrl') }}
                    </span>
                    <code class="codex-profiles-card__code">
                      {{ profileBaseUrl(profile) }}
                    </code>
                  </div>

                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.model') }}
                    </span>
                    <div class="flex items-center gap-2">
                      <span class="codex-profiles-card__model-pill">
                        {{ profile.model }}
                      </span>
                    </div>
                  </div>

                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.authMode') }}
                    </span>
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="codex-profiles-card__meta-pill">
                        {{ authModeLabel(profile.auth_mode) }}
                      </span>
                      <span
                        v-if="profile.openai_login_method"
                        class="px-2 py-0.5 rounded-md text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                      >
                        {{ profile.openai_login_method }}
                      </span>
                    </div>
                  </div>

                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.authSource') }}
                    </span>
                    <div class="flex items-center gap-2 flex-wrap">
                      <code class="codex-profiles-card__code">
                        {{ profile.auth_source || $t('codex.profiles.notAvailable') }}
                      </code>
                      <span
                        v-if="profile.credential_store"
                        class="px-2 py-0.5 rounded-md text-xs font-medium bg-sky-500/10 text-sky-300 border border-sky-500/20"
                      >
                        {{ profile.credential_store }}
                      </span>
                    </div>
                  </div>

                  <div
                    v-if="profile.env_key"
                    class="codex-profiles-card__info-item"
                  >
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.envKey') }}
                    </span>
                    <code class="codex-profiles-card__code">
                      {{ profile.env_key }}
                    </code>
                  </div>
                </div>

                <div
                  v-if="profile.shell_export_script"
                  class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-3"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="text-xs font-medium uppercase tracking-wider text-emerald-300">
                        {{ $t('codex.profiles.envExportTitle') }}
                      </p>
                      <p class="mt-1 text-xs text-white/60">
                        {{ $t('codex.profiles.envExportHint') }}
                      </p>
                    </div>
                    <button
                      class="btn btn-secondary btn-sm"
                      @click.stop="copyProfileEnv(profile)"
                    >
                      {{ $t('codex.profiles.copyEnvExport') }}
                    </button>
                  </div>
                  <pre class="mt-3 overflow-x-auto rounded-lg glass-surface p-3 text-xs text-white/80"><code>{{ profile.shell_export_script }}</code></pre>
                </div>
                 
                <div
                  v-if="profile.tags?.length || profile.provider || (profile.extra && Object.keys(profile.extra).length > 0)"
                  class="mt-4 flex items-center justify-between border-t border-white/5 pt-3"
                >
                  <div class="flex flex-wrap gap-1.5">
                    <span 
                      v-if="profile.provider"
                      class="px-2 py-0.5 rounded-md text-xs font-medium glass-surface text-white/80"
                    >
                      {{ profile.provider }}
                    </span>
                    <span 
                      v-for="tag in profile.tags" 
                      :key="tag"
                      class="px-2 py-0.5 rounded-md text-xs font-medium glass-surface text-white/50"
                    >
                      #{{ tag }}
                    </span>
                  </div>
                   
                  <div
                    v-if="profile.extra && Object.keys(profile.extra).length > 0"
                    class="text-xs text-white/50 font-mono glass-surface px-2 py-1 rounded"
                  >
                    +{{ Object.keys(profile.extra).length }} extras
                  </div>
                </div>
              </div>
            </Card>
          </div>
            
          <CodexProfileEditorModal
            :model-value="showForm"
            :editing-name="editingName"
            :saving="saving"
            :form="form"
            :update-field="updateFormField"
            :available-auth-mode-options="availableAuthModeOptions"
            :model-catalog="modelCatalog"
            :selected-model-option="selectedModelOption"
            :custom-model-input="customModelInput"
            :requires-base-url="requiresBaseUrl"
            :requires-secret="requiresSecret"
            :requires-env-key="requiresEnvKey"
            :auth-token-hint="authTokenHint"
            :is-deprecated-auth-mode="isDeprecatedAuthMode(form.auth_mode)"
            :display-open-ai-login-method="displayOpenAiLoginMethod"
            :auth-mode-label="authModeLabel"
            @update:model-value="handleFormModelValue"
            @update:selected-model-option="selectedModelOption = $event"
            @update:custom-model-input="customModelInput = $event"
            @save="handleSave"
          />

          <ConfirmModal
            v-model:is-open="showConfirmModal"
            :type="confirmDialog.type"
            :title="confirmDialog.title"
            :message="confirmDialog.message"
            :confirm-text="confirmDialog.confirmText"
            :cancel-text="$t('common.cancel')"
            @confirm="executeConfirmedAction"
          />
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import CodexProfileEditorModal from '@/components/codex/CodexProfileEditorModal.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onActivated, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { addCodexCustomModel, addCodexProfile, applyCodexProfile, deleteCodexProfile, getCodexProfile, listCodexModels, listCodexProfiles, updateCodexProfile } from '@/api'
import type {
  CodexAddCustomModelResponse,
  CodexModelsResponse,
  CodexProfile,
  CodexProfileAuthMode,
  CodexProfilesResponse,
} from '@/types'
import { copyToClipboard } from '@/utils/codexHelpers'
import {
  AVAILABLE_AUTH_MODES,
  CUSTOM_MODEL_OPTION,
  type CodexProfileEditorForm,
  authModeToLoginMethod,
  buildCodexProfileRequest,
  codexProfileToEditorForm,
  createCodexProfileEditorForm,
  isDeprecatedAuthMode,
  normalizeModelName,
  resolveModelSelection,
  usesOpenAiAuthMode,
} from '@/utils/codexProfileEditor'
import { logger } from '@/utils/logger'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'CodexProfilesView' })

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(false)
const saving = ref(false)
const actionLoading = ref(false)

const profiles = ref<CodexProfile[]>([])
const currentProfile = ref<string | null>(null)
const codexBuiltinModels = ref<string[]>([])
const codexCustomModels = ref<string[]>([])
const selectedModelOption = ref<string>('')
const customModelInput = ref('')

const showForm = ref(false)
const editingName = ref<string | null>(null)
const busyProfileName = ref<string | null>(null)
const busyAction = ref<'apply' | 'delete' | null>(null)
const showConfirmModal = ref(false)
const lastLoadedAt = ref(0)
const confirmDialog = reactive<{
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
}>({
  title: '',
  message: '',
  confirmText: '',
  type: 'warning',
})
let confirmedAction: (() => Promise<void>) | null = null

const REFRESH_TTL_MS = 30_000

const extractErrorMessage = (error: unknown) => {
  if (typeof error === 'string') {
    return error
  }
  if (error && typeof error === 'object') {
    const candidate = error as { message?: unknown, error?: unknown, cause?: unknown }
    for (const value of [candidate.message, candidate.error, candidate.cause]) {
      if (typeof value === 'string' && value.trim()) {
        return value
      }
    }
  }
  return null
}

const modelCatalog = computed(() => {
  const merged = [...codexBuiltinModels.value, ...codexCustomModels.value]
  return merged.filter((model, index) => merged.indexOf(model) === index)
})

const availableAuthModeOptions = computed(() => {
  const options = [...AVAILABLE_AUTH_MODES]
  if (isDeprecatedAuthMode(form.auth_mode) && !options.includes(form.auth_mode)) {
    options.push(form.auth_mode)
  }
  return options
})

const isOfficialConfig = (profile: CodexProfile) => {
  return !profile.base_url?.trim()
}

const authModeLabel = (authMode?: CodexProfileAuthMode | null) => {
  return t(`codex.profiles.authModes.${authMode || 'no_auth'}`)
}

const profileBaseUrl = (profile: CodexProfile) => {
  return profile.base_url?.trim() || t('codex.profiles.officialBaseUrl')
}

const buildShellExportFallback = (profile: CodexProfile) => {
  const envExport = profile.env_export
  if (!envExport || Object.keys(envExport).length === 0) {
    return ''
  }
  return Object.entries(envExport)
    .map(([key, value]) => `export ${key}=${JSON.stringify(value)}`)
    .join('\n')
}

const copyProfileEnv = async (profile: CodexProfile) => {
  const script = profile.shell_export_script || buildShellExportFallback(profile)
  if (!script) {
    return
  }

  try {
    const copied = await copyToClipboard(script)
    if (!copied) {
      throw new Error('copy failed')
    }
    uiStore.showSuccess(t('codex.profiles.messages.envExportCopied'))
  } catch (error) {
    logger.error('Failed to copy profile env export:', error)
    uiStore.showError(t('codex.profiles.messages.envExportCopyFailed'))
  }
}

// 当前配置模式
const currentConfigMode = computed(() => {
  if (!currentProfile.value) return 'official'
  const profile = profiles.value.find(p => p.name === currentProfile.value)
  return profile && isOfficialConfig(profile) ? 'official' : 'custom'
})

const form = reactive(createCodexProfileEditorForm())

const updateFormField = (field: keyof CodexProfileEditorForm, value: string | boolean) => {
  if (field === 'enabled' || field === 'requires_openai_auth') {
    form[field] = Boolean(value) as never
  } else {
    form[field] = String(value) as never
  }

  if (field === 'auth_mode') {
    syncDerivedAuthFields()
  }
}

const requiresBaseUrl = computed(() => !usesOpenAiAuthMode(form.auth_mode))
const requiresSecret = computed(() => form.auth_mode === 'openai_api_key')
const requiresEnvKey = computed(() => form.auth_mode === 'provider_env_key')
const displayOpenAiLoginMethod = computed(() => authModeToLoginMethod(form.auth_mode) || t('codex.profiles.notAvailable'))
const resolvedModelValue = computed(() => {
  return selectedModelOption.value === CUSTOM_MODEL_OPTION
    ? normalizeModelName(customModelInput.value)
    : normalizeModelName(selectedModelOption.value)
})
const authTokenHint = computed(() => {
  if (form.auth_mode === 'openai_chatgpt') {
    return t('codex.profiles.authTokenHints.openai_chatgpt')
  }
  if (form.auth_mode === 'openai_api_key') {
    return t('codex.profiles.authTokenHints.openai_api_key')
  }
  if (form.auth_mode === 'provider_env_key') {
    return t('codex.profiles.authTokenHints.provider_env_key')
  }
  return t('codex.profiles.authTokenHints.no_auth')
})

const loadModels = async () => {
  try {
    const data = await listCodexModels<CodexModelsResponse>()
    codexBuiltinModels.value = data.builtin_models || []
    codexCustomModels.value = data.custom_models || []
  } catch (error) {
    logger.error('Failed to load codex models:', error)
  }
}

const loadProfiles = async () => {
  try {
    loading.value = true
    const [profilesData] = await Promise.all([
      listCodexProfiles<CodexProfilesResponse>(),
      loadModels(),
    ])
    profiles.value = profilesData.profiles || []
    currentProfile.value = profilesData.current_profile ?? null
    lastLoadedAt.value = Date.now()
  } catch (error) {
    logger.error('Failed to load codex profiles:', error)
    uiStore.showError(t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const ensureLoaded = async (force = false) => {
  if (loading.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) {
    return
  }
  await loadProfiles()
}

const openConfirmDialog = (options: {
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
  action: () => Promise<void>
}) => {
  confirmDialog.title = options.title
  confirmDialog.message = options.message
  confirmDialog.confirmText = options.confirmText
  confirmDialog.type = options.type
  confirmedAction = options.action
  showConfirmModal.value = true
}

const executeConfirmedAction = async () => {
  if (!confirmedAction) return
  actionLoading.value = true
  try {
    await confirmedAction()
  } finally {
    actionLoading.value = false
    confirmedAction = null
  }
}

const resetForm = () => {
  Object.assign(form, createCodexProfileEditorForm())
  selectedModelOption.value = modelCatalog.value[0] || CUSTOM_MODEL_OPTION
  customModelInput.value = ''
}

const applyProfileToForm = (profile: CodexProfile) => {
  Object.assign(form, codexProfileToEditorForm(profile))
  const selection = resolveModelSelection(profile.model, modelCatalog.value)
  selectedModelOption.value = selection.selectedModelOption
  customModelInput.value = selection.customModelInput
}

const openFormModal = async (name?: string) => {
  editingName.value = name ?? null
  await loadModels()
  resetForm()
  showForm.value = true

  if (!name) {
    return
  }

  const profile = await getCodexProfile<CodexProfile>(name)
  applyProfileToForm(profile)
}

const resetBusyState = () => {
  busyProfileName.value = null
  busyAction.value = null
}

const handleProfileAction = async (
  name: string,
  action: 'apply' | 'delete',
  task: () => Promise<void>,
  successMessage: string,
  errorMessage: string,
) => {
  busyProfileName.value = name
  busyAction.value = action
  try {
    await task()
    await loadProfiles()
    uiStore.showSuccess(successMessage)
  } catch (error) {
    logger.error(`Failed to ${action} codex profile:`, error)
    uiStore.showError(extractErrorMessage(error) || errorMessage)
  } finally {
    resetBusyState()
  }
}

const handleAdd = async () => {
  await openFormModal()
}

const handleEdit = async (name: string) => {
  try {
    await openFormModal(name)
  } catch (error) {
    logger.error('Failed to load codex profile:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.loadFailed'))
    showForm.value = false
  }
}

const handleCloseForm = () => {
  showForm.value = false
  editingName.value = null
}

const handleFormModelValue = (value: boolean) => {
  showForm.value = value
  if (!value) {
    editingName.value = null
  }
}

const syncDerivedAuthFields = () => {
  form.openai_login_method = authModeToLoginMethod(form.auth_mode) ?? null
  form.requires_openai_auth = usesOpenAiAuthMode(form.auth_mode)

  if (!requiresEnvKey.value) {
    form.env_key = ''
  }
}

const handleSave = async () => {
  syncDerivedAuthFields()

  if (!form.name.trim()) {
    uiStore.showError(t('codex.profiles.validation.nameRequired'))
    return
  }
  if (requiresBaseUrl.value && !form.base_url?.trim()) {
    uiStore.showError(t('codex.profiles.validation.baseUrlRequired'))
    return
  }
  if (requiresSecret.value && !form.auth_token?.trim()) {
    uiStore.showError(t('codex.profiles.validation.authTokenRequired'))
    return
  }
  if (requiresEnvKey.value && !form.env_key?.trim()) {
    uiStore.showError(t('codex.profiles.validation.envKeyRequired'))
    return
  }
  if (!resolvedModelValue.value) {
    uiStore.showError(t('codex.profiles.validation.modelRequired'))
    return
  }

  const request = buildCodexProfileRequest(form, resolvedModelValue.value)

  try {
    saving.value = true
    const isEditing = Boolean(editingName.value)
    if (selectedModelOption.value === CUSTOM_MODEL_OPTION) {
      const response = await addCodexCustomModel<CodexAddCustomModelResponse>(resolvedModelValue.value)
      const models = response.models || []
      codexCustomModels.value = models.filter(model => !codexBuiltinModels.value.includes(model))
    }
    if (editingName.value) {
      await updateCodexProfile(editingName.value, request)
    } else {
      await addCodexProfile(request)
    }
    handleCloseForm()
    await loadProfiles()
    uiStore.showSuccess(
      isEditing ? t('codex.profiles.updateProfile') : t('codex.profiles.addProfile')
    )
  } catch (error) {
    logger.error('Failed to save codex profile:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.saveFailed'))
  } finally {
    saving.value = false
  }
}

const formatProfileConfirmMessage = (
  key: string,
  name: string,
  fallback: string,
) => {
  const message = t(key, { name })
  if (message !== key && !message.includes('{name}')) {
    return message
  }
  return fallback.replace('{name}', name)
}

const handleDelete = async (name: string) => {
  openConfirmDialog({
    title: t('codex.actions.delete'),
    message: formatProfileConfirmMessage(
      'codex.profiles.deleteConfirm',
      name,
      '确定删除 Profile "{name}" 吗？此操作不可撤销。',
    ),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      await handleProfileAction(
        name,
        'delete',
        () => deleteCodexProfile(name),
        t('codex.actions.delete'),
        t('codex.states.deleteFailed'),
      )
    },
  })
}

const handleApply = async (name: string) => {
  openConfirmDialog({
    title: t('codex.profiles.apply'),
    message: formatProfileConfirmMessage(
      'codex.profiles.confirmApply',
      name,
      '确定切换到 Profile "{name}" 吗？',
    ),
    confirmText: t('codex.profiles.apply'),
    type: 'warning',
    action: async () => {
      await handleProfileAction(
        name,
        'apply',
        () => applyCodexProfile(name),
        t('codex.profiles.apply'),
        t('codex.states.saveFailed'),
      )
    },
  })
}

onMounted(async () => {
  await ensureLoaded(true)
})

onActivated(() => {
  void ensureLoaded(false)
})
</script>

<style scoped>
.codex-profiles-view {
  @apply min-h-full p-6;
}

.codex-profiles-shell {
  @apply mx-auto max-w-[1800px];
}

.codex-profiles-stack {
  @apply mt-6 space-y-6;
}

.codex-profiles-main {
  @apply flex w-full min-w-0 flex-col gap-6;
}

.codex-profiles-header {
  @apply flex items-center justify-between;
}

.codex-profiles-header__intro,
.codex-profiles-header__actions,
.codex-profiles-section-heading__title,
.codex-profiles-switches {
  @apply flex items-center gap-3;
}

.codex-profiles-header__icon {
  @apply rounded-xl bg-platform-codex/10 p-2;
}

.codex-profiles-status-grid {
  @apply grid grid-cols-1 gap-4 md:grid-cols-3;
}

.codex-profiles-status-card {
  @apply overflow-hidden;
}

.codex-profiles-status-icon {
  @apply rounded-xl p-3 transition-transform duration-300;
}

.group:hover .codex-profiles-status-icon {
  transform: scale(1.1);
}

.codex-profiles-status-icon--warning {
  @apply bg-yellow-500/10 text-yellow-500;
}

.codex-profiles-status-icon--primary {
  @apply bg-indigo-500/10 text-indigo-500;
}

.codex-profiles-status-icon--official {
  @apply bg-emerald-500/10 text-emerald-500;
}

.codex-profiles-status-icon--relay {
  @apply bg-pink-500/10 text-pink-500;
}

.codex-profiles-switch {
  @apply relative flex items-center gap-2.5 rounded-xl px-4 py-2.5 text-sm font-medium transition-all duration-300;
}

.codex-profiles-switch--active {
  @apply glass-effect-strong border border-platform-codex/50 text-platform-codex;

  box-shadow: 0 0 15px rgb(245 158 11 / 20%);
}

.codex-profiles-switch--idle {
  @apply glass-effect text-white/80 hover:border-platform-codex/30 hover:bg-white/10;
}

.codex-profiles-switch--busy {
  @apply cursor-not-allowed opacity-60;
}

.codex-profiles-switch__active-indicator {
  @apply flex h-4 w-4 items-center justify-center rounded-full bg-platform-codex text-[10px] text-white;
}

.codex-profiles-section-heading {
  @apply flex items-center justify-between;
}

.codex-profiles-section-heading__title {
  @apply text-xl font-bold text-white;
}

.codex-profiles-loading {
  @apply flex justify-center py-20;
}

.codex-profiles-loading__spinner {
  @apply h-12 w-12 animate-spin rounded-full border-4 border-transparent border-r-accent-secondary border-t-accent-primary;
}

.codex-profiles-grid {
  @apply grid grid-cols-1 gap-4 xl:grid-cols-2;
}

.codex-profiles-card {
  @apply relative overflow-hidden transition-[box-shadow,transform] duration-300 hover:-translate-y-1 hover:shadow-xl;
}

.codex-profiles-card__actions {
  @apply flex items-center gap-1 opacity-100 transition-opacity duration-200 xl:opacity-0;
}

.group:hover .codex-profiles-card__actions {
  opacity: 1;
}

.codex-profiles-action-button {
  @apply rounded-lg p-2 transition-colors hover:bg-white/10;
}

.codex-profiles-action-button--success {
  @apply text-accent-success;
}

.codex-profiles-action-button--primary {
  @apply text-accent-primary;
}

.codex-profiles-action-button--danger {
  @apply text-accent-danger;
}

.codex-profiles-card__info-grid {
  @apply grid grid-cols-1 gap-x-6 gap-y-3 text-sm sm:grid-cols-2;
}

.codex-profiles-card__info-item {
  @apply flex flex-col gap-1;
}

.codex-profiles-card__code {
  @apply truncate rounded px-2 py-1 font-mono text-white glass-surface;
}

.codex-profiles-card__model-pill {
  @apply rounded bg-accent-primary/5 px-2 py-0.5 font-mono text-accent-primary;
}

.codex-profiles-card__meta-pill {
  @apply rounded-md px-2 py-0.5 text-xs font-medium text-white/80 glass-surface;
}

</style>
