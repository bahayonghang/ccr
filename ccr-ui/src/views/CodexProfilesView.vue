<!--
  Codex Profiles 视图 — 与 Claude Code Profiles 同骨架
  - Header 收敛 / 四槽 StatStrip（特色槽 = Config mode）/ 瘦身 QuickRail / Filters 弹层 / 预览检查器
  - 编辑流程走 CodexProfileEditorModal；键盘：/ 聚焦搜索；⌘K 命令面板；⌘/Ctrl+1-9 切换钉选 profile
-->
<template>
  <div class="codex-profiles-view">
    <ModuleSubnav module="codex" />

    <main class="cp-shell">
      <div class="cp-main">
        <ProfilesHeader
          icon="Folder"
          back-to="/codex"
          actions-menu
          :labels="{
            title: $t('codex.profiles.title'),
            subtitle: $t('codex.profiles.subtitle'),
            back: $t('codex.profiles.backToCodex'),
            reload: $t('codex.profiles.reloadAction'),
            export: $t('common.export'),
            add: $t('codex.profiles.addProfile'),
            source: $t('profilesRaw.edit'),
            overflow: $t('codex.profiles.overflowMenu'),
          }"
          :palette="{
            label: $t('codex.profiles.commandPaletteButton'),
            shortcut: `${quickSwitch.modifier.value}K`,
            title: $t('codex.profiles.commandPaletteShortcut'),
          }"
          :loading="loading || isRefreshing"
          :exporting="exporting"
          :palette-open="paletteOpen"
          :source-disabled="!rawLocal"
          :source-title="rawLocal ? undefined : $t('settingsRaw.unsupportedEnvironment')"
          @add="handleAdd"
          @export="handleExportProfiles"
          @reload="refreshProfiles"
          @open-palette="paletteOpen = true"
          @edit-source="openRawEditor"
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
          }"
          :secondary="{
            icon: currentConfigMode === 'official' ? 'Globe' : 'Server',
            title: $t('codex.status.configMode'),
            value: currentConfigMode === 'official' ? $t('codex.profiles.officialConfig') : $t('codex.profiles.customRelay'),
            hint: $t('codex.profiles.statStrip.configModeHint', { official: officialCount, custom: profiles.length - officialCount }),
            mono: false,
          }"
          :health="healthSlot"
          @health-click="focusInspector"
        />

        <ProfilesQuickRail
          :profiles="profiles"
          :current-name="currentProfile"
          i18n-prefix="codex.profiles"
          :disabled="rowsDisabled"
          :busy-name="pendingAction?.kind === 'apply' ? pendingAction.name : null"
          :quick-switch="quickSwitch"
          :more-count="quickRailMoreCount"
          @apply="handleApply"
          @more="paletteOpen = true"
        />

        <ProfilesToolbar
          ref="toolbarRef"
          i18n-prefix="codex.profiles.toolbar"
          compact-filters
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
          class="cp-state"
        >
          <div class="cp-state__spinner" />
        </div>

        <div
          v-else-if="loadError"
          class="cp-state cp-state--error"
        >
          <SIcon
            name="AlertTriangle"
            size="w-6 h-6"
          />
          <div class="cp-state__title">
            {{ $t('codex.profiles.loadFailedTitle') }}
          </div>
          <div class="cp-state__hint">
            {{ loadError }}
          </div>
          <button
            type="button"
            class="cp-state__btn"
            @click="refreshProfiles()"
          >
            {{ $t('codex.profiles.retry') }}
          </button>
        </div>

        <div
          v-else-if="refreshError"
          class="cp-state cp-state--warn"
        >
          <SIcon
            name="AlertCircle"
            size="w-5 h-5"
          />
          <div class="cp-state__title">
            {{ $t('codex.profiles.refreshFailedTitle') }}
          </div>
          <div class="cp-state__hint">
            {{ refreshError }} · {{ $t('codex.profiles.refreshFailedHint') }}
          </div>
          <button
            type="button"
            class="cp-state__btn"
            :disabled="isRefreshing"
            @click="refreshProfiles()"
          >
            {{ $t('codex.profiles.retry') }}
          </button>
        </div>

        <div
          v-else-if="profiles.length === 0"
          class="cp-state"
        >
          <SIcon
            name="Boxes"
            size="w-7 h-7"
          />
          <div class="cp-state__title">
            {{ $t('codex.profiles.emptyState') }}
          </div>
          <div class="cp-state__hint">
            {{ $t('codex.profiles.emptyHint') }}
          </div>
          <button
            type="button"
            class="cp-state__btn cp-state__btn--primary"
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
          class="cp-state"
        >
          <SIcon
            name="SearchX"
            size="w-7 h-7"
          />
          <div class="cp-state__title">
            {{ $t('codex.profiles.empty.noResults', { query }) }}
          </div>
          <button
            type="button"
            class="cp-state__btn"
            @click="resetFilters"
          >
            {{ $t('codex.profiles.empty.clearFilters') }}
          </button>
        </div>

        <div
          v-else
          ref="listRef"
        >
          <ProfilesSection
            v-for="section in listSections"
            :key="section.id"
            :title="section.title"
            :count="section.profiles.length"
          >
            <div
              v-if="viewMode === 'list'"
              class="cp-list"
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
                v-for="profile in section.profiles"
                :key="profile.name"
                v-bind="rowInteraction(profile.name)"
                :profile="profile"
                :descriptor="rowDescriptor"
                :is-current="profile.name === currentProfile"
                :disabled="rowsDisabled"
                :busy-action="busyActionFor(profile.name)"
                @apply="handleApply"
                @edit="handleEdit"
                @delete="handleDelete"
              />
            </div>
            <div
              v-else
              class="cp-grid"
            >
              <ProfileCard
                v-for="profile in section.profiles"
                :key="profile.name"
                v-bind="rowInteraction(profile.name)"
                :profile="profile"
                :is-current="profile.name === currentProfile"
                :disabled="rowsDisabled"
                :busy-action="busyActionFor(profile.name)"
                @apply="handleApply"
                @edit="handleEdit"
                @delete="handleDelete"
                @copy-env="copyProfileEnv"
              />
            </div>
          </ProfilesSection>
        </div>
      </div>

      <ProfilesInspector
        ref="inspectorRef"
        :profiles="profiles"
        :preview-profile="previewProfile"
        :current-profile="currentProfileRecord"
        i18n-prefix="codex.profiles.inspector"
        :descriptor="inspectorDescriptor"
        :session-write-at="isPreviewingCurrent ? lastWriteHint : null"
        :selected-tag="tagFilter"
        @edit="handleEdit"
        @locate="locateProfile"
        @tag-select="onInspectorTagSelect"
      />
    </main>

    <ProfilesCommandPalette
      :open="paletteOpen"
      :profiles="profiles"
      :descriptor="paletteDescriptor"
      :actions="paletteActions"
      i18n-prefix="codex.profiles.commandPalette"
      @update:open="paletteOpen = $event"
      @apply="handleApply"
    />

    <CodexProfileEditorModal
      ref="editorModalRef"
      :model-value="showForm"
      :editing-name="editingName"
      :saving="saving"
      :form="form"
      :update-field="updateFormField"
      :available-auth-mode-options="availableAuthModeOptions"
      :model-catalog="modelCatalog"
      :current-model-option="currentModelOption"
      :selected-model-option="selectedModelOption"
      :custom-model-input="customModelInput"
      :resolved-model="resolvedModelValue"
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

    <ProfilesRawEditorPanel
      v-if="showRawEditor"
      :get-raw="getCodexProfilesRaw"
      :save-raw="saveCodexProfilesRaw"
      @saved="handleRawSaved"
      @close="showRawEditor = false"
    />

    <ConfirmModal
      v-model:is-open="showConfirmModal"
      :type="confirmDialog.type"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :confirm-text="confirmDialog.confirmText"
      :cancel-text="$t('common.cancel')"
      :footnote="confirmDialog.footnote"
      @confirm="executeConfirmedAction"
    >
      <template
        v-if="confirmDiffRows.length > 0"
        #details
      >
        <ProfileDiffRows :rows="confirmDiffRows" />
      </template>
    </ConfirmModal>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onActivated, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  addCodexProfile,
  applyCodexProfile,
  deleteCodexProfile,
  exportCodexProfiles,
  getCurrentEnvironment,
  getCodexProfile,
  listCodexModels,
  listCodexProfiles,
  updateCodexProfile,
} from '@/api'
import { getCodexProfilesRaw, saveCodexProfilesRaw } from '@/api/domains/codex'
import CodexProfileEditorModal from '@/components/codex/CodexProfileEditorModal.vue'
import ProfileCard from '@/components/codex/ProfileCard.vue'
import ProfileDiffRows from '@/components/profiles/ProfileDiffRows.vue'
import ProfileListRow from '@/components/profiles/ProfileListRow.vue'
import ProfilesCommandPalette, { type ProfilesCommandPaletteAction, type ProfilesCommandPaletteDescriptor } from '@/components/profiles/ProfilesCommandPalette.vue'
import ProfilesInspector from '@/components/profiles/ProfilesInspector.vue'
import ProfilesSection from '@/components/profiles/ProfilesSection.vue'
import { useConfirmAction } from '@/composables/useConfirmAction'
import { useProfilesHotkeys } from '@/composables/useProfilesHotkeys'
import { useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import ProfilesHeader from '@/components/profiles/ProfilesHeader.vue'
import ProfilesRawEditorPanel from '@/components/profiles/ProfilesRawEditorPanel.vue'
import ProfilesQuickRail from '@/components/profiles/ProfilesQuickRail.vue'
import ProfilesStatStrip, { type ProfilesStatStripHealth } from '@/components/profiles/ProfilesStatStrip.vue'
import ProfilesToolbar, { type ProfilesViewMode } from '@/components/profiles/ProfilesToolbar.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import SIcon from '@/components/ui/SIcon.vue'
import {
  useCodexProfilesFilter,
  type CodexProfilesSortBy,
  type CodexProfilesStatusFilter,
} from '@/composables/useCodexProfilesFilter'
import { useUIStore } from '@/stores/ui'
import type {
  CodexProfile,
  CodexProfileAuthMode,
} from '@/types'
import { getErrorMessage } from '@/types/api'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { copyText } from '@/utils/clipboard'
import {
  AVAILABLE_AUTH_MODES,
  CUSTOM_MODEL_OPTION,
  type CodexProfileEditorForm,
  authModeToLoginMethod,
  buildCodexProfileModelCatalog,
  buildCodexProfileRequest,
  codexProfileToEditorForm,
  createCodexProfileEditorForm,
  isDeprecatedAuthMode,
  normalizeModelName,
  resolveModelSelection,
  usesOpenAiAuthMode,
} from '@/utils/codexProfileEditor'
import {
  createCodexDiffFields,
  createCodexInspectorDescriptor,
  createCodexRowDescriptor,
} from '@/utils/codexProfiles'
import { buildProfileDiff, type ProfileDiffRow } from '@/utils/profileDiff'
import { downloadTextFile } from '@/utils/download'
import { logger } from '@/utils/logger'
import { mapTemplateToCodexProfilePatch } from '@/utils/providerTemplates'
import { REFRESH_TTL_MS } from '@/config/constants'

defineOptions({ name: 'CodexProfilesView' })

const { t } = useI18n()
const uiStore = useUIStore()

// ===== 加载与状态 =====
const loading = ref(false)
const isRefreshing = ref(false)
const saving = ref(false)
const exporting = ref(false)
const loadError = ref<string | null>(null)
const refreshError = ref<string | null>(null)

const profiles = ref<CodexProfile[]>([])
const currentProfile = ref<string | null>(null)
const codexBuiltinModels = ref<string[]>([])
const currentModelOption = ref('')
const selectedModelOption = ref<string>('')
const customModelInput = ref('')
const selectedProviderTemplate = ref<string | null>(null)
const selectedProviderEndpoint = ref('')

// ===== 编辑表单与确认弹窗 =====
const showForm = ref(false)
const editingName = ref<string | null>(null)
const lastLoadedAt = ref(0)
const lastWriteHint = ref<string | null>(null)
const showRawEditor = ref(false)
const rawLocal = ref(false)
const paletteOpen = ref(false)
const {
  isOpen: showConfirmModal,
  dialog: confirmDialog,
  busy: confirmActionBusy,
  openConfirmDialog,
  executeConfirmedAction,
} = useConfirmAction()

// 确认框附加的「当前 → 目标」diff（仅 apply 场景填充）
const confirmDiffRows = ref<ProfileDiffRow[]>([])
// 待确认/执行中的行级操作，驱动行内 busy 反馈
const pendingAction = ref<{ name: string, kind: 'apply' | 'delete' } | null>(null)

// ===== 列表筛选状态 =====
const query = ref('')
const statusFilter = ref<CodexProfilesStatusFilter>('all')
const tagFilter = ref<string | null>(null)
const sortBy = ref<CodexProfilesSortBy>('recent')
const viewMode = ref<ProfilesViewMode>('card')

// 平台策略：列表行与检查器的字段解析/文案/图标统一由 utils 组装
const rowDescriptor = computed(() => createCodexRowDescriptor(t))
const inspectorDescriptor = createCodexInspectorDescriptor(t)

// 快速切换：钉选（数字编号唯一来源）+ 最近使用，按平台键持久化
const quickSwitch = useProfilesQuickSwitch({
  platform: 'codex',
  getProfileNames: () => profiles.value.map(profile => profile.name),
  onPinLimit: () => uiStore.showWarning(t('codex.profiles.pinLimitReached')),
})

const toolbarRef = ref<InstanceType<typeof ProfilesToolbar> | null>(null)
// Inspector 是泛型组件，InstanceType 无法解析，只取滚动定位需要的 $el
const inspectorRef = ref<{ $el?: unknown } | null>(null)
const editorModalRef = ref<InstanceType<typeof CodexProfileEditorModal> | null>(null)
const listRef = ref<HTMLElement | null>(null)

const { allTags, filtered, enabledList, disabledList } = useCodexProfilesFilter({
  profiles,
  currentProfile,
  query,
  statusFilter,
  tagFilter,
  sortBy,
})

const currentProfileRecord = computed(
  () => profiles.value.find(profile => profile.name === currentProfile.value) ?? null,
)

const enabledCount = computed(() =>
  profiles.value.filter(p => p.enabled !== false).length,
)

const isOfficialConfig = (profile: CodexProfile) => !profile.base_url?.trim()

const officialCount = computed(() => profiles.value.filter(isOfficialConfig).length)

const currentConfigMode = computed<'official' | 'custom'>(() => {
  const found = currentProfileRecord.value
  return !found || isOfficialConfig(found) ? 'official' : 'custom'
})

// 启用/停用两个分组共用同一套行渲染，空分组不出现
const listSections = computed(() =>
  [
    { id: 'enabled', title: t('codex.profiles.groups.enabled'), profiles: enabledList.value },
    { id: 'disabled', title: t('codex.profiles.groups.disabled'), profiles: disabledList.value },
  ].filter(section => section.profiles.length > 0),
)

const rowsDisabled = computed(
  () => loading.value || isRefreshing.value || saving.value || confirmActionBusy.value,
)

const busyActionFor = (name: string): 'apply' | 'delete' | null => {
  if (!confirmActionBusy.value || pendingAction.value?.name !== name) return null
  return pendingAction.value.kind
}

/* ========================================================================
 * 统计条 Health 槽 + 检查器预览目标
 * ======================================================================== */

const insights = inspectorDescriptor.useInsights(profiles)

const healthSlot = computed<ProfilesStatStripHealth>(() => {
  const count = insights.totalIssueCount.value
  return {
    title: t('codex.profiles.statStrip.healthTitle'),
    value: String(count),
    hint: count === 0
      ? t('codex.profiles.statStrip.healthHintOk')
      : t('codex.profiles.statStrip.healthHintIssues', { count }),
    warn: count > 0,
  }
})

// 预览目标：hover 优先于 focus，两者皆空时回落当前 profile
const hoveredName = ref<string | null>(null)
const focusedName = ref<string | null>(null)

const previewProfile = computed<CodexProfile | null>(() => {
  const name = hoveredName.value ?? focusedName.value
  if (name) {
    const match = profiles.value.find(profile => profile.name === name)
    if (match) return match
  }
  return currentProfileRecord.value
})

const isPreviewingCurrent = computed(
  () => !!previewProfile.value && previewProfile.value.name === currentProfile.value,
)

const clearHovered = (name: string) => {
  if (hoveredName.value === name) hoveredName.value = null
}

const onRowFocusOut = (name: string, event: FocusEvent) => {
  const container = event.currentTarget as HTMLElement | null
  const next = event.relatedTarget as Node | null
  // 焦点仍留在同一行/卡片内部（例如角落菜单按钮）时保持预览
  if (next && container?.contains(next)) return
  if (focusedName.value === name) focusedName.value = null
}

/**
 * 行/卡片的预览联动与定位标记：以 v-bind 透传原生监听与 data 属性，
 * 避免给两套行组件的 props 契约塞入平台特有的交互字段。
 */
const rowInteraction = (name: string): Record<string, unknown> => ({
  'data-profile-name': name,
  onMouseenter: () => { hoveredName.value = name },
  onMouseleave: () => clearHovered(name),
  onFocusin: () => { focusedName.value = name },
  onFocusout: (event: FocusEvent) => onRowFocusOut(name, event),
})

const focusInspector = () => {
  const element = inspectorRef.value?.$el
  if (element instanceof HTMLElement) {
    element.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

// Health 条目 → 滚动定位到对应卡片并短暂高亮
const locateProfile = (name: string) => {
  void nextTick(() => {
    const escaped = typeof CSS !== 'undefined' && CSS.escape
      ? CSS.escape(name)
      : name.replace(/["\\]/g, '\\$&')
    const target = listRef.value?.querySelector<HTMLElement>(`[data-profile-name="${escaped}"]`)
    if (!target) return
    target.scrollIntoView({ behavior: 'smooth', block: 'center' })
    target.classList.add('cp-locate-flash')
    window.setTimeout(() => target.classList.remove('cp-locate-flash'), 1600)
  })
}

const onInspectorTagSelect = (tag: string) => {
  tagFilter.value = tagFilter.value === tag ? null : tag
}

// 栏内钉选/最近 chip 之外的可用 profile 数走「+N more → ⌘K」
const quickRailMoreCount = computed(() => {
  const shown = Math.min(
    quickSwitch.pinned.value.length + quickSwitch.recentNotPinned.value.length,
    8,
  )
  return Math.max(0, enabledCount.value - shown)
})

/* ========================================================================
 * 编辑表单派生态
 * ======================================================================== */

const modelCatalog = computed(() =>
  buildCodexProfileModelCatalog(codexBuiltinModels.value, currentModelOption.value),
)

const form = reactive(createCodexProfileEditorForm())

const availableAuthModeOptions = computed(() => {
  const options = [...AVAILABLE_AUTH_MODES]
  // 遗留 profile 仍可能停在弃用模式，编辑时把当前值追加进选项避免静默改写
  if (isDeprecatedAuthMode(form.auth_mode) && !options.includes(form.auth_mode)) {
    options.push(form.auth_mode)
  }
  return options
})

const authModeLabel = (authMode?: CodexProfileAuthMode | null) =>
  t(`codex.profiles.authModes.${authMode || 'no_auth'}`)

const updateFormField = (field: keyof CodexProfileEditorForm, value: string | boolean) => {
  if (field === 'enabled') {
    form.enabled = Boolean(value)
    return
  }
  form[field] = String(value) as never
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
const authTokenHint = computed(() => t(`codex.profiles.authTokenHints.${form.auth_mode}`))

/* ========================================================================
 * 数据加载
 * ======================================================================== */

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
    const data = await listCodexModels()
    codexBuiltinModels.value = data.builtin_models || []
  } catch (error) {
    logger.error('Failed to load codex models:', error)
  }
}

const markWrite = () => {
  lastWriteHint.value = new Date().toLocaleTimeString()
}

const loadProfiles = async (options: { preserveData?: boolean } = {}) => {
  const preserveData = options.preserveData === true

  if (preserveData) {
    isRefreshing.value = true
    refreshError.value = null
  } else {
    loading.value = true
    loadError.value = null
  }

  try {
    const [profilesData] = await Promise.all([
      listCodexProfiles(),
      loadModels(),
    ])
    profiles.value = profilesData.profiles || []
    currentProfile.value = profilesData.current_profile ?? null
    lastLoadedAt.value = Date.now()
    loadError.value = null
    refreshError.value = null
  } catch (error) {
    logger.error('Failed to load codex profiles:', error)
    const message = getErrorMessage(error, t('codex.states.loadFailed'))

    if (preserveData) {
      refreshError.value = message
    } else {
      profiles.value = []
      loadError.value = message
      uiStore.showError(message)
    }
  } finally {
    if (preserveData) {
      isRefreshing.value = false
    } else {
      loading.value = false
    }
  }
}

const refreshProfiles = async () => {
  await loadProfiles({ preserveData: profiles.value.length > 0 })
}

const ensureLoaded = async (force = false) => {
  if (loading.value || isRefreshing.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) return
  await refreshProfiles()
}

const handleExportProfiles = async () => {
  exporting.value = true
  try {
    const payload = await exportCodexProfiles(true)
    downloadTextFile(payload.filename, payload.content, 'application/toml;charset=utf-8')
    uiStore.showSuccess(t('codex.profiles.exportSuccess'))
  } catch (error) {
    logger.error('Failed to export codex profiles:', error)
    uiStore.showError(getErrorMessage(error, t('codex.profiles.exportFailed')))
  } finally {
    exporting.value = false
  }
}

const loadActiveEnvironment = async () => {
  try {
    const environment = await getCurrentEnvironment()
    rawLocal.value = !environment || environment.env_type === 'local'
  } catch {
    rawLocal.value = false
  }
}

const openRawEditor = async () => {
  if (!rawLocal.value) {
    uiStore.showWarning(t('settingsRaw.unsupportedEnvironment'))
    return
  }
  const confirmed = await uiStore.requestConfirm({
    title: t('profilesRaw.openWarningTitle'),
    message: t('profilesRaw.openWarningMessage'),
    confirmText: t('profilesRaw.continue'),
    cancelText: t('common.cancel'),
    type: 'warning',
    surface: 'solid',
  })
  if (confirmed) showRawEditor.value = true
}

const handleRawSaved = async () => {
  showRawEditor.value = false
  markWrite()
  await loadProfiles()
}

/* ========================================================================
 * 编辑流程
 * ======================================================================== */

const resetForm = () => {
  Object.assign(form, createCodexProfileEditorForm())
  currentModelOption.value = ''
  selectedModelOption.value = modelCatalog.value[0] || CUSTOM_MODEL_OPTION
  customModelInput.value = ''
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyProfileToForm = (profile: CodexProfile) => {
  Object.assign(form, codexProfileToEditorForm(profile))
  const normalizedModel = normalizeModelName(profile.model)
  currentModelOption.value = codexBuiltinModels.value.includes(normalizedModel)
    ? ''
    : normalizedModel
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
  const profile = await getCodexProfile(name)
  if (!profile) throw new Error(`Codex profile '${name}' not found`)
  applyProfileToForm(profile)
}

const handleAdd = async () => { await openFormModal() }

const handleEdit = async (name: string) => {
  try {
    await openFormModal(name)
  } catch (error) {
    logger.error('Failed to load codex profile:', error)
    uiStore.showError(getErrorMessage(error, t('codex.states.loadFailed')))
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
    const modelSelection = resolveModelSelection(patch.model, codexBuiltinModels.value)
    selectedModelOption.value = modelSelection.selectedModelOption
    customModelInput.value = modelSelection.customModelInput
  }

  void editorModalRef.value?.scrollToSection('auth')
}

// 校验在模态内完成（失败时汇总条 + 跳转错误分段），这里只负责序列化与写入
const handleSave = async () => {
  const request = buildCodexProfileRequest(form, resolvedModelValue.value)
  const previousName = editingName.value

  try {
    saving.value = true
    if (previousName) {
      await updateCodexProfile(previousName, request)
      // 重命名跟随：钉选/最近列表里的旧名替换为新名，保持数字编号稳定
      if (request.name !== previousName) quickSwitch.renamePinned(previousName, request.name)
    } else {
      await addCodexProfile(request)
    }
    handleCloseForm()
    markWrite()
    await loadProfiles({ preserveData: profiles.value.length > 0 })
    uiStore.showSuccess(previousName ? t('codex.profiles.updateProfile') : t('codex.profiles.addProfile'))
  } catch (error) {
    logger.error('Failed to save codex profile:', error)
    uiStore.showError(getErrorMessage(error, t('codex.states.saveFailed')))
  } finally {
    saving.value = false
  }
}

/* ========================================================================
 * 应用 / 删除
 * ======================================================================== */

const handleDelete = (name: string) => {
  confirmDiffRows.value = []
  pendingAction.value = { name, kind: 'delete' }
  openConfirmDialog({
    title: t('codex.actions.delete'),
    message: t('codex.profiles.deleteConfirm', { name }),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    footnote: t('codex.profiles.confirmDeleteBackupFootnote'),
    action: async () => {
      try {
        await deleteCodexProfile(name)
        markWrite()
        await loadProfiles({ preserveData: profiles.value.length > 0 })
        uiStore.showSuccess(t('codex.profiles.messages.deleteSuccess'))
      } catch (error) {
        logger.error('Failed to delete codex profile:', error)
        uiStore.showError(getErrorMessage(error, t('codex.states.deleteFailed')))
      } finally {
        pendingAction.value = null
      }
    },
  })
}

const handleApply = (name: string) => {
  const targetProfile = profiles.value.find(profile => profile.name === name)
  if (!targetProfile || targetProfile.name === currentProfile.value || targetProfile.enabled === false) return

  // 确认框内展示 base_url / model / auth_mode 三行「当前 → 目标」对比
  confirmDiffRows.value = buildProfileDiff(
    currentProfileRecord.value,
    targetProfile,
    createCodexDiffFields(t),
  )
  pendingAction.value = { name, kind: 'apply' }

  openConfirmDialog({
    title: t('codex.profiles.apply'),
    message: t('codex.profiles.confirmApply', { name }),
    confirmText: t('codex.profiles.apply'),
    type: 'warning',
    action: async () => {
      try {
        await applyCodexProfile(name)
        quickSwitch.recordUse(name)
        markWrite()
        await loadProfiles({ preserveData: profiles.value.length > 0 })
        uiStore.showSuccess(t('codex.profiles.apply'))
      } catch (error) {
        logger.error('Failed to apply codex profile:', error)
        uiStore.showError(getErrorMessage(error, t('codex.states.saveFailed')))
      } finally {
        pendingAction.value = null
        confirmDiffRows.value = []
      }
    },
  })
}

const resetFilters = () => {
  query.value = ''
  statusFilter.value = 'all'
  tagFilter.value = null
}

// ===== 命令面板：策略注入(profile 判定/副标题) + 常用命令 =====
const paletteDescriptor: ProfilesCommandPaletteDescriptor<CodexProfile> = {
  isEnabled: profile => profile.enabled !== false,
  hint: profile => profile.description || profile.base_url || undefined,
}

const paletteActions = computed<ProfilesCommandPaletteAction[]>(() => [
  { id: '__add', icon: 'Plus', labelKey: 'codex.profiles.commandPalette.actionAdd', handler: () => { void handleAdd() } },
  { id: '__reload', icon: 'RefreshCw', labelKey: 'codex.profiles.commandPalette.actionReload', handler: () => { void refreshProfiles() } },
  { id: '__export', icon: 'Download', labelKey: 'codex.profiles.commandPalette.actionExport', handler: () => { void handleExportProfiles() } },
])

// 当前激活的标签若因数据变化而失效，自动回退到"全部"
watch([allTags, tagFilter], ([tags, tag]) => {
  if (tag && !tags.includes(tag)) tagFilter.value = null
})

// 预览目标被删除/重命名后立即回落到当前 profile
watch(profiles, (list) => {
  const names = new Set(list.map(profile => profile.name))
  if (hoveredName.value && !names.has(hoveredName.value)) hoveredName.value = null
  if (focusedName.value && !names.has(focusedName.value)) focusedName.value = null
})

// 取消确认框时清掉行内 busy 标记与 diff 数据，避免残留到下一次确认
watch(showConfirmModal, (isOpen) => {
  if (isOpen || confirmActionBusy.value) return
  pendingAction.value = null
  confirmDiffRows.value = []
})

// ===== 键盘快捷键：/ ⌘K ⌘1-9 Esc（两页共用实现） =====
useProfilesHotkeys({
  paletteOpen,
  focusSearch: () => toolbarRef.value?.focusSearch(),
  getApplicableProfiles: () => profiles.value.filter(p => p.enabled !== false),
  getStableTargets: () => quickSwitch.stableTargets.value,
  onApply: handleApply,
})

onMounted(async () => {
  await Promise.all([loadProfiles(), loadActiveEnvironment()])
})

onActivated(() => {
  void ensureLoaded(false)
  void loadActiveEnvironment()
})
</script>

<style scoped>
/* ===========================================================
   作用域设计令牌：仅在本视图内生效，子组件靠继承解析 --cp-*
   主色跟随共享 accent-primary（与 Claude Profiles 页一致）
   =========================================================== */
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

  /* 主色 → 共享 accent-primary（跟随用户 data-accent 选择，两平台一致） */
  --cp-accent: var(--color-accent-primary);
  --cp-accent-soft: rgb(var(--color-accent-primary-rgb) / 14%);
  --cp-accent-line: rgb(var(--color-accent-primary-rgb) / 35%);
  --cp-accent-hover: var(--color-accent-primary-hover);
  --cp-on-accent: var(--color-text-inverted);

  /* 平台识别色：仅用于页头图标徽章，不跟随用户 accent 选择 */
  --cp-icon-color: var(--color-platform-codex);
  --cp-icon-soft: rgb(var(--color-platform-codex-rgb) / 14%);
  --cp-icon-line: rgb(var(--color-platform-codex-rgb) / 35%);

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
  font-size: 0.8125rem;
  line-height: 1.5;
}

.cp-shell {
  max-width: 1680px;
  margin: 16px auto 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}

@media (width >= 1280px) {
  .cp-shell {
    grid-template-columns: minmax(0, 1fr) 340px;
  }
}

.cp-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
}

/* 卡片视图栅格：≥1280px 双列，≥1680px 视口宽度可到三列 */
.cp-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(420px, 1fr));
  gap: 10px;
}

@media (width <= 1279px) {
  .cp-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

/* 列表视图 */
.cp-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-list-head {
  display: grid;
  grid-template-columns: 12px minmax(120px, 160px) minmax(0, 1.2fr) minmax(0, 1.5fr) minmax(
      80px,
      110px
    ) minmax(80px, 120px) minmax(60px, 1fr) auto;
  gap: 12px;
  padding: 2px 14px 4px;
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  letter-spacing: 0.05rem;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-list-head__right {
  text-align: right;
}

@media (width <= 1024px) {
  .cp-list-head {
    display: none;
  }
}

/* Health 审计条目定位后的短暂高亮 */
.cp-locate-flash {
  outline: 2px solid var(--cp-accent);
  outline-offset: 2px;
  transition: outline-color 200ms ease;
}

/* 加载/空/错误三态 */
.cp-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 8px;
  padding: 48px 16px;
  border-radius: 12px;
  border: 1px dashed var(--cp-line-2);
  background: var(--cp-bg-2);
  color: var(--cp-ink-3);
}

.cp-state--error {
  border-style: solid;
  border-color: rgb(var(--color-danger-rgb) / 30%);
  color: var(--cp-danger);
}

.cp-state--warn {
  border-style: solid;
  border-color: rgb(var(--color-warning-rgb) / 30%);
  color: var(--cp-warn);
}

.cp-state__title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--cp-ink-0);
}

.cp-state__hint {
  font-size: 0.8125rem;
  color: var(--cp-ink-2);
  max-width: 420px;
}

.cp-state__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
  padding: 7px 14px;
  border-radius: 8px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-3);
  color: var(--cp-ink-1);
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease;
}

.cp-state__btn:hover:not(:disabled) {
  background: var(--cp-accent-soft);
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
}

.cp-state__btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.cp-state__btn--primary {
  background: var(--cp-accent);
  border-color: var(--cp-accent);
  color: var(--cp-on-accent);
}

.cp-state__spinner {
  width: 32px;
  height: 32px;
  border-radius: 999px;
  border: 2px solid var(--cp-line-2);
  border-top-color: var(--cp-accent);
  animation: cp-state-spin 1s linear infinite;
}

@keyframes cp-state-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .cp-state__spinner {
    animation: none;
  }

  .cp-state__btn {
    transition: none;
  }
}

@media (width <= 720px) {
  .codex-profiles-view {
    padding: 16px;
  }
}
</style>
