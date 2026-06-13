<!--
  Codex Profiles 视图 — 单列模式重构
  - 视觉：Anthropic Design「ccr-codex-profile」暖中性表面 + 琥珀点缀
  - 编辑流程不动：icon 按钮触发现有 CodexProfileEditorModal
  - 键盘：/ 聚焦搜索；⌘K 命令面板；⌘1-9 切换启用 profile；Esc 关闭面板
-->
<template>
  <div class="codex-profiles-view">
    <ModuleSubnav module="codex" />

    <main class="cp-shell">
      <div class="cp-main">
        <ProfilesHeader
          icon="Folder"
          back-to="/codex"
          :labels="{
            title: $t('codex.profiles.title'),
            subtitle: $t('codex.profiles.subtitle'),
            back: $t('codex.profiles.backToCodex'),
            reload: $t('codex.profiles.reloadAction'),
            export: $t('common.export'),
            add: $t('codex.profiles.addProfile'),
          }"
          :palette="{
            label: $t('codex.profiles.commandPaletteButton'),
            shortcut: '⌘K',
            title: $t('codex.profiles.commandPaletteShortcut'),
          }"
          :loading="loading"
          :exporting="exporting"
          :palette-open="paletteOpen"
          @add="handleAdd"
          @export="handleExportProfiles"
          @reload="reloadProfiles"
          @open-palette="paletteOpen = true"
        />

        <ProfilesStatStrip
          :current="currentProfile"
          :total="profiles.length"
          :labels="{
            current: $t('codex.status.currentConfig'),
            notSet: $t('codex.status.notSet'),
            currentHint: $t('codex.profiles.statStrip.profileSubtitle'),
            total: $t('codex.status.totalProfiles'),
            totalHint: $t('codex.profiles.statStrip.totalHint', { enabled: enabledCount, disabled: profiles.length - enabledCount }),
            lastWrite: $t('codex.profiles.statStrip.lastWrite'),
            lastWriteHint: $t('codex.profiles.statStrip.lastWriteHint'),
          }"
          :secondary="{
            icon: currentConfigMode === 'official' ? 'Globe' : 'Server',
            title: $t('codex.status.configMode'),
            value: currentConfigMode === 'official' ? $t('codex.profiles.officialConfig') : $t('codex.profiles.customRelay'),
            hint: 'profiles.toml',
            mono: false,
          }"
          :last-write="lastWriteHint"
          :total-spark="[3, 5, 4, 6, 7, 8, 7]"
          :recent-spark="[2, 4, 3, 5, 4, 6, 5]"
        />

        <ProfilesQuickRail
          :profiles="profiles"
          :current-name="currentProfile"
          :disabled="actionLoading"
          :busy-name="busyAction === 'apply' ? busyProfileName : null"
          @apply="handleApply"
        />

        <ProfilesToolbar
          ref="toolbarRef"
          i18n-prefix="codex.profiles.toolbar"
          :query="query"
          :status-filter="statusFilter"
          :tag-filter="tagFilter"
          :sort-by="sortBy"
          :view-mode="viewMode"
          :result-count="filtered.length"
          :total="profiles.length"
          :all-tags="allTags"
          @update:query="query = $event"
          @update:status-filter="statusFilter = $event"
          @update:tag-filter="tagFilter = $event"
          @update:sort-by="sortBy = $event"
          @update:view-mode="viewMode = $event"
        />

        <div
          v-if="loading"
          class="cp-loading"
        >
          <div class="cp-loading__spinner" />
        </div>

        <div
          v-else-if="profiles.length === 0"
          class="cp-empty"
        >
          <SIcon
            name="Boxes"
            size="w-7 h-7"
          />
          <div class="cp-empty__title">
            {{ $t('codex.profiles.emptyState') }}
          </div>
          <div class="cp-empty__hint">
            {{ $t('codex.profiles.emptyHint') }}
          </div>
          <button
            type="button"
            class="cp-btn cp-btn--primary"
            @click="handleAdd"
          >
            <SIcon
              name="Plus"
              size="w-3.5 h-3.5"
            />
            {{ $t('codex.profiles.addProfile') }}
          </button>
        </div>

        <div
          v-else-if="filtered.length === 0"
          class="cp-empty"
        >
          <SIcon
            name="Search"
            size="w-7 h-7"
          />
          <div class="cp-empty__title">
            {{ $t('codex.profiles.empty.noResults', { query }) }}
          </div>
          <button
            type="button"
            class="cp-btn cp-btn--ghost"
            @click="resetFilters"
          >
            {{ $t('codex.profiles.empty.clearFilters') }}
          </button>
        </div>

        <template v-else-if="viewMode === 'list'">
          <ProfilesSection
            v-if="enabledList.length > 0"
            :title="$t('codex.profiles.groups.enabled')"
            :count="enabledList.length"
          >
            <div class="cp-list-head">
              <span />
              <span>{{ $t('codex.profiles.fields.name') }}</span>
              <span>{{ $t('codex.profiles.description') }}</span>
              <span>{{ $t('codex.profiles.fields.baseUrl') }}</span>
              <span>{{ $t('codex.profiles.fields.model') }}</span>
              <span>{{ $t('codex.profiles.fields.authMode') }}</span>
              <span>{{ $t('codex.profiles.fields.tags') }}</span>
              <span class="cp-list-head__right">{{ $t('codex.profiles.toolbar.actionsLabel') }}</span>
            </div>
            <ProfileListRow
              v-for="profile in enabledList"
              :key="profile.name"
              :profile="profile"
              :descriptor="rowDescriptor"
              :is-current="profile.name === currentProfile"
              :busy-action="busyProfileName === profile.name ? busyAction : null"
              :disabled="actionLoading"
              @apply="handleApply"
              @edit="handleEdit"
              @delete="handleDelete"
            />
          </ProfilesSection>

          <ProfilesSection
            v-if="disabledList.length > 0"
            :title="$t('codex.profiles.groups.disabled')"
            :count="disabledList.length"
          >
            <ProfileListRow
              v-for="profile in disabledList"
              :key="profile.name"
              :profile="profile"
              :descriptor="rowDescriptor"
              :is-current="profile.name === currentProfile"
              :busy-action="busyProfileName === profile.name ? busyAction : null"
              :disabled="actionLoading"
              @apply="handleApply"
              @edit="handleEdit"
              @delete="handleDelete"
            />
          </ProfilesSection>
        </template>

        <template v-else>
          <ProfilesSection
            v-if="enabledList.length > 0"
            :title="$t('codex.profiles.groups.enabled')"
            :count="enabledList.length"
          >
            <div class="cp-grid">
              <ProfileCard
                v-for="profile in enabledList"
                :key="profile.name"
                :profile="profile"
                :is-current="profile.name === currentProfile"
                :busy-action="busyProfileName === profile.name ? busyAction : null"
                :disabled="actionLoading"
                @apply="handleApply"
                @edit="handleEdit"
                @delete="handleDelete"
                @copy-env="copyProfileEnv"
              />
            </div>
          </ProfilesSection>

          <ProfilesSection
            v-if="disabledList.length > 0"
            :title="$t('codex.profiles.groups.disabled')"
            :count="disabledList.length"
          >
            <div class="cp-grid">
              <ProfileCard
                v-for="profile in disabledList"
                :key="profile.name"
                :profile="profile"
                :is-current="profile.name === currentProfile"
                :busy-action="busyProfileName === profile.name ? busyAction : null"
                :disabled="actionLoading"
                @apply="handleApply"
                @edit="handleEdit"
                @delete="handleDelete"
                @copy-env="copyProfileEnv"
              />
            </div>
          </ProfilesSection>
        </template>
      </div>

      <ProfilesContextRail
        :profiles="profiles"
        :current="currentProfile"
        :active-profile="activeProfile"
        i18n-prefix="codex.profiles.contextRail"
        :descriptor="railDescriptor"
        @edit="handleEdit"
      />
    </main>

    <CommandPalette
      :open="paletteOpen"
      :profiles="profiles"
      @update:open="paletteOpen = $event"
      @apply="handleApply"
      @add="handleAdd"
      @export="handleExportProfiles"
      @reload="reloadProfiles"
      @import="handleAdd"
    />

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
      :selected-provider-template="selectedProviderTemplate"
      :selected-provider-endpoint="selectedProviderEndpoint"
      :provider-template-draft="codexProfileTemplateDraft"
      @update:model-value="handleFormModelValue"
      @update:selected-model-option="selectedModelOption = $event"
      @update:custom-model-input="customModelInput = $event"
      @select-template="applyCodexProfileTemplate"
      @manual-template="useManualProviderTemplate"
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
  </div>
</template>

<script setup lang="ts">
import { computed, h, onActivated, onBeforeUnmount, onMounted, reactive, ref, type FunctionalComponent } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  addCodexCustomModel,
  addCodexProfile,
  applyCodexProfile,
  deleteCodexProfile,
  exportCodexProfiles,
  getCodexProfile,
  listCodexModels,
  listCodexProfiles,
  updateCodexProfile,
} from '@/api'
import CodexProfileEditorModal from '@/components/codex/CodexProfileEditorModal.vue'
import CommandPalette from '@/components/codex/CommandPalette.vue'
import ProfileCard from '@/components/codex/ProfileCard.vue'
import ProfileListRow, { type ProfileRowDescriptor } from '@/components/profiles/ProfileListRow.vue'
import ProfilesContextRail, { type ContextRailDescriptor, type ContextRailActiveField } from '@/components/profiles/ProfilesContextRail.vue'
import { useCodexProfilesInsights } from '@/composables/useCodexProfilesInsights'
import ProfilesHeader from '@/components/profiles/ProfilesHeader.vue'
import ProfilesQuickRail from '@/components/codex/ProfilesQuickRail.vue'
import ProfilesStatStrip from '@/components/profiles/ProfilesStatStrip.vue'
import ProfilesToolbar, { type ProfilesViewMode } from '@/components/profiles/ProfilesToolbar.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import SIcon from '@/components/ui/SIcon.vue'
import {
  useCodexProfilesFilter,
  type CodexProfilesSortBy,
  type CodexProfilesStatusFilter,
} from '@/composables/useCodexProfilesFilter'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useUIStore } from '@/stores/ui'
import type {
  CodexAddCustomModelResponse,
  CodexModelsResponse,
  CodexProfile,
  CodexProfileAuthMode,
  CodexProfilesResponse,
} from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { copyText } from '@/utils/clipboard'
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
import { downloadTextFile } from '@/utils/download'
import { logger } from '@/utils/logger'
import { mapTemplateToCodexProfilePatch } from '@/utils/providerTemplates'
import { REFRESH_TTL_MS } from '@/config/constants'

defineOptions({ name: 'CodexProfilesView' })

interface ProfilesExportResponse { content: string, filename: string }

const { t } = useI18n()
const uiStore = useUIStore()

// ===== 加载与状态 =====
const loading = ref(false)
const saving = ref(false)
const actionLoading = ref(false)
const exporting = ref(false)

const profiles = ref<CodexProfile[]>([])
const currentProfile = ref<string | null>(null)
const codexBuiltinModels = ref<string[]>([])
const codexCustomModels = ref<string[]>([])
const selectedModelOption = ref<string>('')
const customModelInput = ref('')
const selectedProviderTemplate = ref<string | null>(null)
const selectedProviderEndpoint = ref('')

// ===== 编辑表单与确认弹窗 =====
const showForm = ref(false)
const editingName = ref<string | null>(null)
const busyProfileName = ref<string | null>(null)
const busyAction = ref<'apply' | 'delete' | null>(null)
const showConfirmModal = ref(false)
const lastLoadedAt = ref(0)
const lastWriteHint = ref<string | null>(null)
const confirmDialog = reactive<{
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
}>({ title: '', message: '', confirmText: '', type: 'warning' })
let confirmedAction: (() => Promise<void>) | null = null

// ===== 列表筛选状态 =====
const query = ref('')
const statusFilter = ref<CodexProfilesStatusFilter>('all')
const tagFilter = ref<string | null>(null)
const sortBy = ref<CodexProfilesSortBy>('recent')
const viewMode = ref<ProfilesViewMode>('card')
const paletteOpen = ref(false)

// 列表行平台策略：base_url/model/authMode 解析 + 操作文案 + 编辑图标
const rowDescriptor = computed<ProfileRowDescriptor<CodexProfile>>(() => ({
  baseUrl: (profile) => {
    const raw = profile.base_url?.trim()
    return raw && raw.length > 0 ? raw : t('codex.profiles.officialBaseUrl')
  },
  model: (profile) => profile.model || '—',
  authMode: (profile) => t(`codex.profiles.authModes.${profile.auth_mode || 'no_auth'}`),
  editIcon: 'Edit2',
  labels: {
    apply: t('codex.profiles.apply'),
    edit: t('codex.actions.edit'),
    delete: t('codex.actions.delete'),
  },
}))

const codexAuthModeLabel = (mode: string): string => t(`codex.profiles.authModes.${mode}`)

// 上下文侧栏平台策略：洞察来源 + 当前 profile 字段列表 + 文案/图标
const railDescriptor: ContextRailDescriptor<CodexProfile> = {
  editIcon: 'Edit2',
  useInsights: useCodexProfilesInsights,
  activeFields: (profile) => {
    const baseUrl = profile.base_url?.trim() || t('codex.profiles.officialBaseUrl')
    const fields: ContextRailActiveField[] = [
      { label: t('codex.profiles.fields.baseUrl'), value: baseUrl },
      { label: t('codex.profiles.fields.model'), value: profile.model || '—', variant: 'accent' },
      {
        label: t('codex.profiles.fields.authMode'),
        value: codexAuthModeLabel(profile.auth_mode ?? 'no_auth'),
        variant: 'muted',
      },
    ]
    if (profile.openai_login_method) {
      fields.push({ label: t('codex.profiles.fields.openAiLoginMethod'), value: profile.openai_login_method, variant: 'muted' })
    }
    if (profile.account) {
      fields.push({ label: t('codex.profiles.fields.account'), value: profile.account })
    }
    if (profile.provider) {
      fields.push({ label: t('codex.profiles.fields.provider'), value: profile.provider, variant: 'muted' })
    }
    if (profile.credential_store) {
      fields.push({ label: t('codex.profiles.contextRail.credentialStore'), value: profile.credential_store, variant: 'muted' })
    }
    if (profile.env_key) {
      fields.push({ label: t('codex.profiles.fields.envKey'), value: profile.env_key })
    }
    return fields
  },
  authModeLabel: codexAuthModeLabel,
  isDeprecatedMode: mode => mode === 'openai_chatgpt' || mode === 'provider_env_key',
  missingMessage: missing =>
    missing
      .map(field =>
        field === 'base_url'
          ? t('codex.profiles.contextRail.issues.missingBaseUrl')
          : t('codex.profiles.contextRail.issues.missingModel'),
      )
      .join(' · '),
  runtimeSummary: profile => `${profile.model} · ${profile.base_url}`,
  deprecatedMessage: profile =>
    translateWithFallback(t, 'codex.profiles.contextRail.issues.deprecatedAuth', '使用已弃用模式：{mode}', {
      mode: codexAuthModeLabel(profile.auth_mode ?? 'no_auth'),
    }),
}
const toolbarRef = ref<InstanceType<typeof ProfilesToolbar> | null>(null)

const { allTags, filtered, enabledList, disabledList, activeProfile } = useCodexProfilesFilter({
  profiles,
  currentProfile,
  query,
  statusFilter,
  tagFilter,
  sortBy,
})

const enabledCount = computed(() =>
  profiles.value.filter(p => p.enabled !== false).length,
)

const isOfficialConfig = (profile: CodexProfile) => !profile.base_url?.trim()

const currentConfigMode = computed<'official' | 'custom'>(() => {
  if (!currentProfile.value) return 'official'
  const found = profiles.value.find(p => p.name === currentProfile.value)
  return found && isOfficialConfig(found) ? 'official' : 'custom'
})

// ===== 派生 =====
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

const authModeLabel = (authMode?: CodexProfileAuthMode | null) =>
  t(`codex.profiles.authModes.${authMode || 'no_auth'}`)

const form = reactive(createCodexProfileEditorForm())

const updateFormField = (field: keyof CodexProfileEditorForm, value: string | boolean) => {
  if (field === 'enabled' || field === 'requires_openai_auth') {
    form[field] = Boolean(value) as never
  } else {
    form[field] = String(value) as never
  }
  if (field === 'auth_mode') syncDerivedAuthFields()
}

const requiresBaseUrl = computed(() => !usesOpenAiAuthMode(form.auth_mode))
const requiresSecret = computed(() => form.auth_mode === 'openai_api_key')
const requiresEnvKey = computed(() => form.auth_mode === 'provider_env_key')
const displayOpenAiLoginMethod = computed(
  () => authModeToLoginMethod(form.auth_mode) || t('codex.profiles.notAvailable'),
)
const resolvedModelValue = computed(() =>
  selectedModelOption.value === CUSTOM_MODEL_OPTION
    ? normalizeModelName(customModelInput.value)
    : normalizeModelName(selectedModelOption.value),
)
const codexProfileTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'codex',
  defaultName: form.provider || form.name || 'Codex profile provider',
  name: form.provider || form.name,
  category: 'third_party',
  baseUrls: form.base_url.trim() ? [form.base_url.trim()] : [],
  modelCatalog: resolvedModelValue.value ? [resolvedModelValue.value] : [],
  platformOverride: {
    baseUrl: form.base_url,
    provider: form.provider,
    providerType: form.provider_type,
    description: form.description,
    modelCatalog: resolvedModelValue.value ? [resolvedModelValue.value] : [],
  },
}))
const authTokenHint = computed(() => {
  if (form.auth_mode === 'openai_chatgpt')   return t('codex.profiles.authTokenHints.openai_chatgpt')
  if (form.auth_mode === 'openai_api_key')   return t('codex.profiles.authTokenHints.openai_api_key')
  if (form.auth_mode === 'provider_env_key') return t('codex.profiles.authTokenHints.provider_env_key')
  return t('codex.profiles.authTokenHints.no_auth')
})

const extractErrorMessage = (error: unknown): string | null => {
  if (typeof error === 'string') return error
  if (error && typeof error === 'object') {
    const c = error as { message?: unknown, error?: unknown, cause?: unknown }
    for (const v of [c.message, c.error, c.cause]) {
      if (typeof v === 'string' && v.trim()) return v
    }
  }
  return null
}

const buildShellExportFallback = (profile: CodexProfile) => {
  const envExport = profile.env_export
  if (!envExport || Object.keys(envExport).length === 0) return ''
  return Object.entries(envExport)
    .map(([key, value]) => `export ${key}=${JSON.stringify(value)}`)
    .join('\n')
}

const copyProfileEnv = async (profile: CodexProfile) => {
  const script = profile.shell_export_script || buildShellExportFallback(profile)
  if (!script) return
  try {
    const ok = await copyText(script)
    if (!ok) throw new Error('copy failed')
    uiStore.showSuccess(t('codex.profiles.messages.envExportCopied'))
  } catch (error) {
    logger.error('Failed to copy profile env export:', error)
    uiStore.showError(t('codex.profiles.messages.envExportCopyFailed'))
  }
}

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
    lastWriteHint.value = new Date().toLocaleTimeString()
  } catch (error) {
    logger.error('Failed to load codex profiles:', error)
    uiStore.showError(t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const reloadProfiles = () => loadProfiles()

const ensureLoaded = async (force = false) => {
  if (loading.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) return
  await loadProfiles()
}

const handleExportProfiles = async () => {
  exporting.value = true
  try {
    const payload = await exportCodexProfiles<ProfilesExportResponse>(true)
    downloadTextFile(payload.filename, payload.content, 'application/toml;charset=utf-8')
    uiStore.showSuccess(t('codex.profiles.exportSuccess'))
  } catch (error) {
    logger.error('Failed to export codex profiles:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.profiles.exportFailed'))
  } finally {
    exporting.value = false
  }
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
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyProfileToForm = (profile: CodexProfile) => {
  Object.assign(form, codexProfileToEditorForm(profile))
  const selection = resolveModelSelection(profile.model, modelCatalog.value)
  selectedModelOption.value = selection.selectedModelOption
  customModelInput.value = selection.customModelInput
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const openFormModal = async (name?: string) => {
  editingName.value = name ?? null
  await loadModels()
  resetForm()
  showForm.value = true
  if (!name) return
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

const handleAdd = async () => { await openFormModal() }

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
  if (!value) editingName.value = null
}

const useManualProviderTemplate = () => {
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyCodexProfileTemplate = (selection: ProviderTemplateSelection) => {
  const patch = mapTemplateToCodexProfilePatch(selection.template, selection.endpoint)

  selectedProviderTemplate.value = selection.template.id
  selectedProviderEndpoint.value = selection.endpoint || ''
  form.base_url = patch.base_url || ''
  form.provider = patch.provider || selection.template.name
  form.provider_type = patch.provider_type || ''

  if (!form.name.trim()) {
    form.name = patch.suggestedName || selection.template.id
  }
  if (!form.description.trim() && patch.description) {
    form.description = patch.description
  }
  if (patch.model) {
    const modelSelection = resolveModelSelection(patch.model, modelCatalog.value)
    selectedModelOption.value = modelSelection.selectedModelOption
    customModelInput.value = modelSelection.customModelInput
  }
}

const syncDerivedAuthFields = () => {
  form.openai_login_method = authModeToLoginMethod(form.auth_mode) ?? null
  form.requires_openai_auth = usesOpenAiAuthMode(form.auth_mode)
  if (!requiresEnvKey.value) form.env_key = ''
}

const handleSave = async () => {
  syncDerivedAuthFields()

  if (!form.name.trim()) {
    uiStore.showError(t('codex.profiles.validation.nameRequired')); return
  }
  if (requiresBaseUrl.value && !form.base_url?.trim()) {
    uiStore.showError(t('codex.profiles.validation.baseUrlRequired')); return
  }
  if (requiresSecret.value && !form.auth_token?.trim()) {
    uiStore.showError(t('codex.profiles.validation.authTokenRequired')); return
  }
  if (requiresEnvKey.value && !form.env_key?.trim()) {
    uiStore.showError(t('codex.profiles.validation.envKeyRequired')); return
  }
  if (!resolvedModelValue.value) {
    uiStore.showError(t('codex.profiles.validation.modelRequired')); return
  }

  const request = buildCodexProfileRequest(form, resolvedModelValue.value)

  try {
    saving.value = true
    const isEditing = Boolean(editingName.value)
    if (selectedModelOption.value === CUSTOM_MODEL_OPTION) {
      const response = await addCodexCustomModel<CodexAddCustomModelResponse>(resolvedModelValue.value)
      const models = response.models || []
      codexCustomModels.value = models.filter(m => !codexBuiltinModels.value.includes(m))
    }
    if (editingName.value) {
      await updateCodexProfile(editingName.value, request)
    } else {
      await addCodexProfile(request)
    }
    handleCloseForm()
    await loadProfiles()
    uiStore.showSuccess(isEditing ? t('codex.profiles.updateProfile') : t('codex.profiles.addProfile'))
  } catch (error) {
    logger.error('Failed to save codex profile:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.saveFailed'))
  } finally {
    saving.value = false
  }
}

const formatProfileConfirmMessage = (key: string, name: string, fallback: string) =>
  translateWithFallback(t, key, fallback, { name })

const handleDelete = (name: string) => {
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
        name, 'delete',
        () => deleteCodexProfile(name),
        t('codex.actions.delete'),
        t('codex.states.deleteFailed'),
      )
    },
  })
}

const handleApply = (name: string) => {
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
        name, 'apply',
        () => applyCodexProfile(name),
        t('codex.profiles.apply'),
        t('codex.states.saveFailed'),
      )
    },
  })
}

const resetFilters = () => {
  query.value = ''
  statusFilter.value = 'all'
  tagFilter.value = null
}

// ===== 简易分组容器：标题 + 计数 + 内容插槽（functional component） =====
const ProfilesSection: FunctionalComponent<{ title: string, count: number }> = (
  props,
  { slots },
) => {
  // children 由父组件 v-slot 传入，运行时已是 VNode[]；TS 推断收紧到 VNodeChild
  // 这里跳过 h() 第三参的严格签名检查，等价于直接挂载子节点
  const children = (slots.default?.() ?? []) as never
  return h('section', { class: 'cp-section' }, [
    h('div', { class: 'cp-section__head' }, [
      h(SIcon, { name: 'Folder', size: 'w-3.5 h-3.5', class: 'cp-section__icon' }),
      h('span', { class: 'cp-section__title' }, props.title),
      h('span', { class: 'cp-section__count' }, String(props.count)),
    ]),
    h('div', { class: 'cp-section__body' }, children),
  ])
}
ProfilesSection.props = ['title', 'count']

// ===== 键盘快捷键：/ ⌘K ⌘1-9 Esc =====
const isEditableTarget = (el: EventTarget | null): boolean => {
  if (!(el instanceof HTMLElement)) return false
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
}

const onWindowKeyDown = (event: KeyboardEvent) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    paletteOpen.value = !paletteOpen.value
    return
  }
  if ((event.metaKey || event.ctrlKey) && /^[1-9]$/.test(event.key)) {
    const idx = Number.parseInt(event.key, 10) - 1
    const enabledOnly = profiles.value.filter(p => p.enabled !== false)
    const target = enabledOnly[idx]
    if (target) {
      event.preventDefault()
      handleApply(target.name)
    }
    return
  }
  if (event.key === '/' && !isEditableTarget(event.target)) {
    event.preventDefault()
    toolbarRef.value?.focusSearch()
    return
  }
  if (event.key === 'Escape' && paletteOpen.value) {
    paletteOpen.value = false
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onWindowKeyDown)
  await ensureLoaded(true)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onWindowKeyDown)
})

onActivated(() => { void ensureLoaded(false) })
</script>

<style scoped>
/* 作用域设计令牌：仅在本视图内生效，不污染全局 */
.codex-profiles-view {
  /* 背景层 → 全局 token */
  --cp-bg-0: var(--color-bg-base);
  --cp-bg-1: var(--color-bg-elevated);
  --cp-bg-2: var(--color-bg-surface);
  --cp-bg-3: var(--color-bg-overlay);
  --cp-bg-4: rgb(var(--color-bg-overlay-rgb) / 88%);

  /* 边框 → 全局 token */
  --cp-line: var(--color-border-subtle);
  --cp-line-2: var(--color-border-default);

  /* 文字阶 → 全局 token */
  --cp-ink-0: var(--color-text-primary);
  --cp-ink-1: var(--color-text-secondary);
  --cp-ink-2: var(--color-text-muted);
  --cp-ink-3: var(--color-text-ghost);
  --cp-ink-4: var(--color-text-disabled);

  /* 主色 → Claude Clay（与 Auth 页同源） */
  --cp-accent: var(--color-accent-primary);
  --cp-accent-soft: rgb(var(--color-accent-primary-rgb) / 14%);
  --cp-accent-line: rgb(var(--color-accent-primary-rgb) / 35%);
  --cp-accent-hover: var(--color-accent-primary-hover);
  --cp-on-accent: var(--color-text-inverted);

  /* 状态色 → 全局 token */
  --cp-good: var(--color-success);
  --cp-warn: var(--color-warning);
  --cp-danger: var(--color-danger);
  --cp-info: var(--color-info);
  --cp-mono: var(--font-mono, 'MapleBright', monospace);

  min-height: 100%;
  padding: 24px;
  background: var(--color-bg-base);
  color: var(--cp-ink-1);
  font-size: 13px;
  line-height: 1.5;
}

.cp-shell {
  max-width: 1440px;
  margin: 16px auto 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 18px;
}

.cp-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

@media (width >= 1280px) {
  .cp-shell {
    grid-template-columns: minmax(0, 1fr) 320px;
    align-items: start;
  }
}

.cp-section { margin-bottom: 18px; }

.cp-section__head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 2px 12px;
  color: var(--cp-ink-2);
  font-size: 13px;
  font-weight: 600;
}

.cp-section__icon { color: var(--cp-accent); }
.cp-section__title { color: var(--cp-ink-1); }

.cp-section__count {
  margin-left: 4px;
  color: var(--cp-ink-4);
  font-family: var(--cp-mono);
  font-size: 11px;
}

.cp-section__body { display: contents; }

.cp-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr); /* 单列模式 */
  gap: 14px;
}

.cp-list-head {
  display: grid;
  grid-template-columns: 12px minmax(120px, 160px) minmax(0, 1.2fr) minmax(0, 1.5fr) minmax(80px, 110px) minmax(80px, 120px) minmax(60px, 1fr) auto;
  gap: 12px;
  padding: 8px 14px;
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line);
  border-radius: 8px 8px 0 0;
  color: var(--cp-ink-3);
  font-family: var(--cp-mono);
  font-size: 10.5px;
  letter-spacing: 0.8px;
  text-transform: uppercase;
}

.cp-list-head__right { text-align: right; }

.cp-loading {
  display: flex;
  justify-content: center;
  padding: 80px 0;
}

.cp-loading__spinner {
  width: 36px;
  height: 36px;
  border-radius: 999px;
  border: 3px solid var(--cp-line-2);
  border-top-color: var(--cp-accent);
  animation: cp-spin 1s linear infinite;
}

.cp-empty {
  background: var(--cp-bg-1);
  border: 1px dashed var(--cp-line-2);
  border-radius: 12px;
  padding: 60px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--cp-ink-3);
  font-size: 13px;
  text-align: center;
}

.cp-empty__title {
  color: var(--cp-ink-1);
  font-weight: 500;
}

.cp-empty__hint {
  color: var(--cp-ink-3);
  font-size: 12px;
}

.cp-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 12px;
  border-radius: 7px;
  font-size: 12.5px;
  font-weight: 500;
  font-family: inherit;
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-1);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
}

.cp-btn:hover:not(:disabled) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-btn--primary {
  background: var(--cp-accent);
  border-color: var(--cp-accent);
  color: var(--cp-on-accent);
  font-weight: 600;
}

.cp-btn--ghost {
  background: transparent;
  border-color: var(--cp-line);
  color: var(--cp-ink-2);
}

@keyframes cp-spin { to { transform: rotate(360deg); } }

@media (prefers-reduced-motion: reduce) {
  .cp-btn { transition: none; }
  .cp-loading__spinner { animation: none; }
}

@media (width <= 720px) {
  .codex-profiles-view { padding: 16px; }
}
</style>
