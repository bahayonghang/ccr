<template>
  <div class="profiles-view claude-profiles-view">
    <ModuleSubnav module="claude-code" />

    <main class="cp-shell">
      <div class="cp-main">
        <ProfilesHeader
          icon="Layers"
          back-to="/claude-code"
          :labels="{
            title: $t('claudeProfiles.title'),
            subtitle: $t('claudeProfiles.subtitle'),
            back: $t('claudeProfiles.back'),
            reload: $t('claudeProfiles.reloadAction'),
            export: $t('common.export'),
            add: $t('claudeProfiles.addProfile'),
            source: $t('profilesRaw.edit'),
            overflow: $t('claudeProfiles.overflowMenu'),
          }"
          :palette="{
            label: $t('claudeProfiles.commandPaletteButton'),
            shortcut: `${quickSwitch.modifier.value}K`,
            title: $t('claudeProfiles.commandPaletteShortcut'),
          }"
          :loading="loading || isRefreshing"
          :exporting="isExporting"
          :palette-open="paletteOpen"
          :source-disabled="!rawLocal"
          :source-title="rawLocal ? undefined : $t('settingsRaw.unsupportedEnvironment')"
          @add="openAddForm"
          @export="handleExportProfiles"
          @reload="refreshProfiles"
          @open-palette="paletteOpen = true"
          @edit-source="openRawEditor"
        />

        <section
          v-if="canOff"
          class="claude-profiles-banner"
          data-testid="claude-profile-off-banner"
        >
          <SIcon
            name="Power"
            size="w-5 h-5"
          />
          <div>
            <strong>{{ t('claudeProfiles.runtimeBanner.title') }}</strong>
            <p>{{ t('claudeProfiles.runtimeBanner.description') }}</p>
          </div>
          <button
            type="button"
            class="claude-profiles-banner__action"
            :disabled="loading || isRefreshing || confirmActionBusy"
            @click="handleOff"
          >
            <SIcon
              name="Power"
              size="w-4 h-4"
            />
            {{ t('claudeProfiles.actions.off') }}
          </button>
        </section>

        <ProfilesStatStrip
          :current="currentProfileName"
          :total="profiles.length"
          :labels="{
            current: $t('claudeProfiles.currentProfile'),
            notSet: $t('claudeProfiles.notSet'),
            currentHint: $t('claudeProfiles.statStrip.profileSubtitle'),
            total: $t('claudeProfiles.totalCount'),
            totalHint: $t('claudeProfiles.statStrip.totalHint', { enabled: enabledProfilesCount, disabled: profiles.length - enabledProfilesCount }),
          }"
          :secondary="{
            icon: 'ShieldCheck',
            title: $t('claudeProfiles.statStrip.authTitle'),
            value: `${subscriptionCount} · ${apiKeyCount}`,
            hint: $t('claudeProfiles.statStrip.authSplit', { subscription: subscriptionCount, apiKey: apiKeyCount }),
            mono: true,
          }"
          :health="healthSlot"
          @health-click="focusInspector"
        />

        <ProfilesQuickRail
          :profiles="profiles"
          :current-name="currentProfileName"
          i18n-prefix="claudeProfiles"
          :disabled="loading || isRefreshing || isSaving || confirmActionBusy"
          :busy-name="pendingAction?.kind === 'apply' ? pendingAction.name : null"
          :quick-switch="quickSwitch"
          :more-count="quickRailMoreCount"
          @apply="handleApply"
          @more="paletteOpen = true"
        />

        <ProfilesToolbar
          ref="toolbarRef"
          i18n-prefix="claudeProfiles.toolbar"
          :query="searchQuery"
          :status-filter="statusFilter"
          :tag-filter="tagFilter"
          :provider-filter="providerFilter"
          :sort-by="sortBy"
          :view-mode="viewMode"
          :result-count="filtered.length"
          :total="profiles.length"
          :all-tags="allTags"
          :all-providers="allProviders"
          @update:query="searchQuery = $event"
          @update:status-filter="statusFilter = $event"
          @update:tag-filter="tagFilter = $event"
          @update:provider-filter="providerFilter = $event"
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
            {{ $t('claudeProfiles.loadFailedTitle') }}
          </div>
          <div class="cp-state__hint">
            {{ loadError }}
          </div>
          <button
            type="button"
            class="cp-state__btn"
            @click="refreshProfiles()"
          >
            {{ $t('claudeProfiles.retry') }}
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
            {{ $t('claudeProfiles.refreshFailedTitle') }}
          </div>
          <div class="cp-state__hint">
            {{ refreshError }} · {{ $t('claudeProfiles.refreshFailedHint') }}
          </div>
          <button
            type="button"
            class="cp-state__btn"
            :disabled="isRefreshing"
            @click="refreshProfiles()"
          >
            {{ $t('claudeProfiles.retry') }}
          </button>
        </div>

        <div
          v-else-if="profiles.length === 0"
          class="cp-state"
        >
          <SIcon
            name="FolderOpen"
            size="w-7 h-7"
          />
          <div class="cp-state__title">
            {{ $t('claudeProfiles.emptyTitle') }}
          </div>
          <div class="cp-state__hint">
            {{ $t('claudeProfiles.emptyDesc') }}
          </div>
          <button
            type="button"
            class="cp-state__btn cp-state__btn--primary"
            @click="openAddForm()"
          >
            <SIcon
              name="Plus"
              size="w-3.5 h-3.5"
            />
            {{ $t('claudeProfiles.createProfile') }}
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
            {{ $t('claudeProfiles.searchEmptyTitle') }}
          </div>
          <div class="cp-state__hint">
            {{ $t('claudeProfiles.searchEmptyDesc') }}
          </div>
          <button
            type="button"
            class="cp-state__btn"
            @click="resetFilters()"
          >
            {{ $t('claudeProfiles.clearSearch') }}
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
                <span>{{ $t('claudeProfiles.fields.name') }}</span>
                <span>{{ $t('claudeProfiles.descLabel') }}</span>
                <span>{{ $t('claudeProfiles.fields.baseUrl') }}</span>
                <span>{{ $t('claudeProfiles.fields.model') }}</span>
                <span>{{ $t('claudeProfiles.fields.authMode') }}</span>
                <span>{{ $t('claudeProfiles.fields.tags') }}</span>
                <span class="cp-list-head__right">{{ $t('claudeProfiles.toolbar.actionsLabel') }}</span>
              </div>
              <ProfileListRow
                v-for="profile in section.profiles"
                :key="profile.name"
                v-bind="rowInteraction(profile.name)"
                :profile="profile"
                :descriptor="rowDescriptor"
                :is-current="profile.is_current"
                :disabled="rowsDisabled"
                :busy-action="busyActionFor(profile.name)"
                @apply="handleApply"
                @edit="openEditForm(findProfile($event))"
                @delete="handleDelete"
              />
            </div>
            <div
              v-else
              class="cp-grid"
            >
              <ClaudeProfileRow
                v-for="profile in section.profiles"
                :key="profile.name"
                v-bind="rowInteraction(profile.name)"
                :profile="profile"
                :search-query="trimmedSearchQuery"
                :disabled="rowsDisabled"
                :busy-action="busyActionFor(profile.name)"
                @apply="handleApply(profile.name)"
                @edit="openEditForm(profile)"
                @delete="handleDelete(profile.name)"
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
        i18n-prefix="claudeProfiles.inspector"
        :descriptor="inspectorDescriptor"
        :session-write-at="isPreviewingCurrent ? lastWriteHint : null"
        :selected-tag="tagFilter"
        @edit="openEditForm(findProfile($event))"
        @locate="locateProfile"
        @tag-select="onInspectorTagSelect"
      />
    </main>

    <ClaudeProfileEditorModal
      ref="editorModalRef"
      v-model="showForm"
      :form="form"
      :update-form-field="updateFormField"
      :parsed-form-tags="parsedFormTags"
      :is-editing="isEditing"
      :is-editing-current="isEditingCurrent"
      :editing-name="editingName"
      :saving="isSaving"
      :save-error="saveError"
      :selected-provider-template="selectedProviderTemplate"
      :selected-provider-endpoint="selectedProviderEndpoint"
      :template-draft="claudeTemplateDraft"
      @close="closeForm"
      @save="handleSave"
      @select-template="applyClaudeProviderTemplate"
      @manual-template="useManualProviderTemplate"
    />

    <ProfilesCommandPalette
      :open="paletteOpen"
      :profiles="profiles"
      :descriptor="paletteDescriptor"
      :actions="paletteActions"
      i18n-prefix="claudeProfiles.commandPalette"
      @update:open="paletteOpen = $event"
      @apply="handleApply"
    />

    <ProfilesRawEditorPanel
      v-if="showRawEditor"
      :get-raw="getClaudeProfilesRaw"
      :save-raw="saveClaudeProfilesRaw"
      @saved="handleRawSaved"
      @close="showRawEditor = false"
    />

    <ConfirmModal
      v-model:is-open="showConfirmModal"
      :type="confirmDialog.type"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :confirm-text="confirmDialog.confirmText"
      :cancel-text="$t('claudeProfiles.cancel')"
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
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import '@/styles/profiles-page.css'
import {
  addClaudeProfile,
  applyClaudeProfile,
  deleteClaudeProfile,
  exportClaudeProfiles,
  getCurrentEnvironment,
  listClaudeProfiles,
  updateClaudeProfile,
} from '@/api'
import { claudeProfileOff } from '@/api/domains/claude'
import { getClaudeProfilesRaw, saveClaudeProfilesRaw } from '@/api/domains/claude'
import ClaudeProfileEditorModal from '@/components/claude/ClaudeProfileEditorModal.vue'
import ClaudeProfileRow from '@/components/claude/ClaudeProfileRow.vue'
import ProfileDiffRows from '@/components/profiles/ProfileDiffRows.vue'
import ProfilesCommandPalette, { type ProfilesCommandPaletteAction, type ProfilesCommandPaletteDescriptor } from '@/components/profiles/ProfilesCommandPalette.vue'
import ProfilesInspector from '@/components/profiles/ProfilesInspector.vue'
import ProfilesSection from '@/components/profiles/ProfilesSection.vue'
import { useConfirmAction } from '@/composables/useConfirmAction'
import { useProfilesHotkeys } from '@/composables/useProfilesHotkeys'
import { useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import ProfilesHeader from '@/components/profiles/ProfilesHeader.vue'
import ProfilesRawEditorPanel from '@/components/profiles/ProfilesRawEditorPanel.vue'
import ProfileListRow from '@/components/profiles/ProfileListRow.vue'
import ProfilesQuickRail from '@/components/profiles/ProfilesQuickRail.vue'
import ProfilesStatStrip, { type ProfilesStatStripHealth } from '@/components/profiles/ProfilesStatStrip.vue'
import ProfilesToolbar, { type ProfilesViewMode } from '@/components/profiles/ProfilesToolbar.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'
import type { ClaudeProfileEditorForm } from '@/types/claudeProfileEditor'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { getErrorMessage } from '@/types/api'
import {
  applyClaudeTemplateToForm,
  buildClaudeProfileRequest,
  buildClaudeTemplateDraft,
  createClaudeProfileForm,
  fillClaudeProfileForm,
  parseClaudeProfileTags,
  resetClaudeProfileForm,
} from '@/utils/claudeProfileEditor'
import {
  createClaudeDiffFields,
  createClaudeInspectorDescriptor,
  createClaudeRowDescriptor,
  normalizeClaudeProfilesState,
} from '@/utils/claudeProfiles'
import { buildProfileDiff, type ProfileDiffRow } from '@/utils/profileDiff'
import {
  useClaudeProfilesFilter,
  type ClaudeProfilesSortBy,
  type ClaudeProfilesStatusFilter,
} from '@/composables/useClaudeProfilesFilter'
import { logger } from '@/utils/logger'
import { downloadTextFile } from '@/utils/download'
import { useUIStore } from '@/stores/ui'

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(true)
const isRefreshing = ref(false)
const isExporting = ref(false)
const loadError = ref<string | null>(null)
const refreshError = ref<string | null>(null)
const profiles = ref<ClaudeProfile[]>([])
const canOff = ref(false)
const profileNamesReady = ref(false)
const showForm = ref(false)
const isEditing = ref(false)
const isSaving = ref(false)
const saveError = ref<string | null>(null)
const editingName = ref('')
const searchQuery = ref('')
const statusFilter = ref<ClaudeProfilesStatusFilter>('all')
const tagFilter = ref<string | null>(null)
const providerFilter = ref<string | null>(null)
const selectedProviderTemplate = ref<string | null>(null)
const selectedProviderEndpoint = ref('')
const sortBy = ref<ClaudeProfilesSortBy>('recent')
const viewMode = ref<ProfilesViewMode>('card')
const paletteOpen = ref(false)
const lastWriteHint = ref<string | null>(null)
const showRawEditor = ref(false)
const rawLocal = ref(false)
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

const currentProfileRecord = computed(() => profiles.value.find(profile => profile.is_current) ?? null)
const currentProfileName = computed(() => currentProfileRecord.value?.name ?? null)
const providerUnsetLabel = computed(() => t('claudeProfiles.providerUnset'))
const isEditingCurrent = computed(() => isEditing.value && editingName.value === currentProfileRecord.value?.name)

// 平台策略：列表行与检查器的字段解析/文案/图标统一由 utils 组装
const rowDescriptor = computed(() => createClaudeRowDescriptor(t))
const inspectorDescriptor = createClaudeInspectorDescriptor(t)

// 快速切换：钉选（数字编号唯一来源）+ 最近使用，按平台键持久化
const quickSwitch = useProfilesQuickSwitch({
  platform: 'claude',
  getProfileNames: () => profileNamesReady.value
    ? profiles.value.map(profile => profile.name)
    : null,
  onPinLimit: () => uiStore.showWarning(t('claudeProfiles.pinLimitReached')),
})

const toolbarRef = ref<InstanceType<typeof ProfilesToolbar> | null>(null)
// Inspector 是泛型组件，InstanceType 无法解析，只取滚动定位需要的 $el
const inspectorRef = ref<{ $el?: unknown } | null>(null)
const editorModalRef = ref<InstanceType<typeof ClaudeProfileEditorModal> | null>(null)
const listRef = ref<HTMLElement | null>(null)

const form = reactive<ClaudeProfileEditorForm>(createClaudeProfileForm())

// 过滤/排序/分组逻辑下沉到 composable
const {
  allTags,
  allProviders,
  filtered,
  enabledList,
  disabledList,
} = useClaudeProfilesFilter({
  profiles,
  currentProfile: currentProfileName,
  query: searchQuery,
  statusFilter,
  tagFilter,
  providerFilter,
  sortBy,
  providerUnsetLabel,
})

// 认证分布（喂统计条）
const subscriptionCount = computed(
  () => profiles.value.filter(profile => (profile.auth_mode ?? 'subscription') === 'subscription').length,
)
const apiKeyCount = computed(
  () => profiles.value.filter(profile => profile.auth_mode === 'api_key').length,
)
const enabledProfilesCount = computed(
  () => profiles.value.filter(profile => profile.enabled !== false).length,
)

const trimmedSearchQuery = computed(() => searchQuery.value.trim())

// 启用/停用两个分组共用同一套行渲染，空分组不出现
const listSections = computed(() =>
  [
    { id: 'enabled', title: t('claudeProfiles.groups.enabled'), profiles: enabledList.value },
    { id: 'disabled', title: t('claudeProfiles.groups.disabled'), profiles: disabledList.value },
  ].filter(section => section.profiles.length > 0),
)
const rowsDisabled = computed(
  () => loading.value || isRefreshing.value || isSaving.value || confirmActionBusy.value,
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
    title: t('claudeProfiles.statStrip.healthTitle'),
    value: String(count),
    hint: count === 0
      ? t('claudeProfiles.statStrip.healthHintOk')
      : t('claudeProfiles.statStrip.healthHintIssues', { count }),
    warn: count > 0,
  }
})

// 预览目标：hover 优先于 focus，两者皆空时回落当前 profile
const hoveredName = ref<string | null>(null)
const focusedName = ref<string | null>(null)

const previewProfile = computed<ClaudeProfile | null>(() => {
  const name = hoveredName.value ?? focusedName.value
  if (name) {
    const match = profiles.value.find(profile => profile.name === name)
    if (match) return match
  }
  return currentProfileRecord.value
})

const isPreviewingCurrent = computed(
  () => !!previewProfile.value && previewProfile.value.name === currentProfileName.value,
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

/* ========================================================================
 * 快速切换条：钉选/最近 chip 之外的可用 profile 数走「+N more → ⌘K」
 * ======================================================================== */

const quickRailMoreCount = computed(() => {
  const shown = Math.min(
    quickSwitch.pinned.value.length + quickSwitch.recentNotPinned.value.length,
    8,
  )
  return Math.max(0, enabledProfilesCount.value - shown)
})

// 列表行只发 name，编辑需要完整 profile 记录
const findProfile = (name: string): ClaudeProfile => {
  const profile = profiles.value.find(item => item.name === name)
  if (!profile) {
    throw new Error(`Claude profile "${name}" not found`)
  }
  return profile
}

const parsedFormTags = computed(() => parseClaudeProfileTags(form.tagsInput) ?? [])
const claudeTemplateDraft = computed(() => buildClaudeTemplateDraft(form))

function updateFormField(field: keyof ClaudeProfileEditorForm, value: string | boolean) {
  if (field === 'enabled') {
    form.enabled = Boolean(value)
    return
  }

  if (field === 'auth_mode') {
    form.auth_mode = value === 'api_key' ? 'api_key' : 'subscription'
    return
  }

  form[field] = String(value) as ClaudeProfileEditorForm[typeof field]
}

const openAddForm = () => {
  resetClaudeProfileForm(form)
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  saveError.value = null
  isEditing.value = false
  editingName.value = ''
  showForm.value = true
}

const openEditForm = (profile: ClaudeProfile) => {
  fillClaudeProfileForm(form, profile)
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  saveError.value = null
  isEditing.value = true
  editingName.value = profile.name
  showForm.value = true
}

const closeForm = () => {
  if (isSaving.value) return

  showForm.value = false
  saveError.value = null
}

const useManualProviderTemplate = () => {
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyClaudeProviderTemplate = (selection: ProviderTemplateSelection) => {
  const switchedAuthMode = applyClaudeTemplateToForm(form, selection)

  selectedProviderTemplate.value = selection.template.id
  selectedProviderEndpoint.value = selection.endpoint || ''

  // 模板自动改写 auth_mode 时给出可见提示，避免静默变更鉴权语义
  if (switchedAuthMode) uiStore.showInfo(t('claudeProfiles.templateAuthModeSwitched'))

  editorModalRef.value?.scrollToSection('connection')
}

const resetFilters = () => {
  searchQuery.value = ''
  statusFilter.value = 'all'
  tagFilter.value = null
  providerFilter.value = null
}

// ===== 命令面板：策略注入(profile 判定/副标题) + 常用命令 =====
const paletteDescriptor: ProfilesCommandPaletteDescriptor<ClaudeProfile> = {
  isEnabled: profile => profile.enabled !== false,
  hint: profile => profile.description || profile.base_url || undefined,
}

const paletteActions = computed<ProfilesCommandPaletteAction[]>(() => {
  const actions: ProfilesCommandPaletteAction[] = [
    { id: '__add', icon: 'Plus', labelKey: 'claudeProfiles.commandPalette.actionAdd', handler: openAddForm },
    { id: '__reload', icon: 'RefreshCw', labelKey: 'claudeProfiles.commandPalette.actionReload', handler: () => { void refreshProfiles() } },
    { id: '__export', icon: 'Download', labelKey: 'claudeProfiles.commandPalette.actionExport', handler: () => { void handleExportProfiles() } },
  ]
  if (canOff.value) {
    actions.push({
      id: '__off',
      icon: 'Power',
      labelKey: 'claudeProfiles.commandPalette.actionOff',
      handler: handleOff,
    })
  }
  return actions
})

const markWrite = () => {
  lastWriteHint.value = new Date().toLocaleTimeString()
}

const loadProfiles = async (options: { preserveData?: boolean } = {}) => {
  const preserveData = options.preserveData === true

  if (preserveData) {
    isRefreshing.value = true
    refreshError.value = null
  } else {
    profileNamesReady.value = false
    loading.value = true
    loadError.value = null
  }

  try {
    const data = await listClaudeProfiles()
    const normalized = normalizeClaudeProfilesState(data.profiles || [], data.current_profile || null)

    canOff.value = data.can_off === true
    profiles.value = normalized.profiles
    profileNamesReady.value = true
    loadError.value = null
    refreshError.value = null

    if (normalized.warnings.length > 0) {
      logger.warn('Normalized inconsistent Claude profiles response', {
        currentProfile: normalized.currentProfile,
        warnings: normalized.warnings,
      })
    }
  } catch (error) {
    logger.error('Failed to load Claude profiles:', error)
    const message = getErrorMessage(error, t('claudeProfiles.loadFailed'))

    if (preserveData) {
      refreshError.value = message
    } else {
      profiles.value = []
      canOff.value = false
      loadError.value = message
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

const handleExportProfiles = async () => {
  isExporting.value = true

  try {
    const payload = await exportClaudeProfiles(true)
    downloadTextFile(payload.filename, payload.content, 'application/toml;charset=utf-8')
    uiStore.showSuccess(t('claudeProfiles.exportSuccess'))
  } catch (error) {
    logger.error('Failed to export Claude profiles:', error)
    uiStore.showError(getErrorMessage(error, t('claudeProfiles.exportFailed')))
  } finally {
    isExporting.value = false
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

const performSave = async () => {
  isSaving.value = true
  saveError.value = null

  try {
    const request = buildClaudeProfileRequest(form)
    const previousName = editingName.value

    if (isEditing.value) {
      await updateClaudeProfile(previousName, request)
      // 重命名跟随：钉选/最近列表里的旧名替换为新名，保持数字编号稳定
      if (request.name !== previousName) quickSwitch.renamePinned(previousName, request.name)
    } else {
      await addClaudeProfile(request)
    }

    showForm.value = false
    markWrite()
    await loadProfiles({ preserveData: profiles.value.length > 0 })
  } catch (error) {
    logger.error('Failed to save Claude profile:', error)
    saveError.value = getErrorMessage(error, t('claudeProfiles.operationFailed'))
  } finally {
    isSaving.value = false
  }
}

const handleSave = () => {
  const trimmedName = form.name.trim()
  if (!trimmedName) return

  // 重命名场景：name 改了。先做客户端唯一性校验 + 弹二次确认
  const isRenaming = isEditing.value && trimmedName !== editingName.value
  if (isRenaming) {
    const collision = profiles.value.some(p => p.name === trimmedName && p.name !== editingName.value)
    if (collision) {
      saveError.value = t('claudeProfiles.renameConflict', { name: trimmedName })
      return
    }

    confirmDiffRows.value = []
    pendingAction.value = null
    openConfirmDialog({
      title: t('claudeProfiles.renameConfirmTitle'),
      message: t('claudeProfiles.renameConfirmBody', { old: editingName.value, new: trimmedName }),
      confirmText: t('claudeProfiles.renameConfirmCta'),
      type: 'warning',
      action: performSave,
    })
    return
  }

  void performSave()
}

const handleDelete = (name: string) => {
  confirmDiffRows.value = []
  pendingAction.value = { name, kind: 'delete' }
  openConfirmDialog({
    title: t('claudeProfiles.deleteTooltip'),
    message: t('claudeProfiles.deleteConfirm', { name }),
    confirmText: t('claudeProfiles.deleteTooltip'),
    type: 'danger',
    footnote: t('claudeProfiles.confirmDeleteBackupFootnote'),
    action: async () => {
      try {
        await deleteClaudeProfile(name)
        markWrite()
        await loadProfiles({ preserveData: profiles.value.length > 0 })
      } catch (error) {
        logger.error('Failed to delete Claude profile:', error)
        uiStore.showError(getErrorMessage(error, t('claudeProfiles.deleteFailed')))
      } finally {
        pendingAction.value = null
      }
    },
  })
}

const handleOff = () => {
  if (!canOff.value) return
  confirmDiffRows.value = []
  openConfirmDialog({
    title: t('claudeProfiles.confirm.offTitle'),
    message: t('claudeProfiles.confirm.offMessage'),
    confirmText: t('claudeProfiles.actions.off'),
    type: 'warning',
    action: async () => {
      try {
        const result = await claudeProfileOff()
        markWrite()
        await loadProfiles({ preserveData: profiles.value.length > 0 })
        uiStore.showSuccess(t('claudeProfiles.messages.offSuccess'))
        const suppressorWarnings = result.remaining_suppressors.map(source =>
          `${source.kind} @ ${source.location} (${source.confidence}; ${source.ownership})`,
        )
        for (const warning of suppressorWarnings.length > 0 ? suppressorWarnings : result.warnings) {
          uiStore.showWarning(warning, 6000)
        }
      } catch (error) {
        logger.error('Failed to exit Claude profile mode:', error)
        uiStore.showError(getErrorMessage(error, t('claudeProfiles.messages.offFailed')))
      }
    },
  })
}

const handleApply = (name: string) => {
  const targetProfile = profiles.value.find(profile => profile.name === name)
  if (!targetProfile || targetProfile.is_current || targetProfile.enabled === false) return

  // 确认框内展示 base_url / model / auth_mode 三行「当前 → 目标」对比
  confirmDiffRows.value = buildProfileDiff(
    currentProfileRecord.value,
    targetProfile,
    createClaudeDiffFields(t),
  )
  pendingAction.value = { name, kind: 'apply' }

  openConfirmDialog({
    title: t('claudeProfiles.applyProfile'),
    message: t('claudeProfiles.confirmApply', { name }),
    confirmText: t('claudeProfiles.applyProfile'),
    type: 'warning',
    action: async () => {
      try {
        await applyClaudeProfile(name)
        quickSwitch.recordUse(name)
        markWrite()
        await loadProfiles({ preserveData: profiles.value.length > 0 })
      } catch (error) {
        logger.error('Failed to apply Claude profile:', error)
        uiStore.showError(getErrorMessage(error, t('claudeProfiles.applyFailed')))
      } finally {
        pendingAction.value = null
        confirmDiffRows.value = []
      }
    },
  })
}

// 当前激活的标签/Provider 若因数据变化而失效，自动回退到"全部"
watch([allTags, tagFilter], ([tags, tag]) => {
  if (tag && !tags.includes(tag)) tagFilter.value = null
})
watch([allProviders, providerFilter], ([providers, provider]) => {
  if (provider && !providers.some(item => item.key === provider)) providerFilter.value = null
})

// 预览目标被删除/重命名后立即回落到当前 profile
watch(profiles, (list) => {
  const names = new Set(list.map(profile => profile.name))
  if (hoveredName.value && !names.has(hoveredName.value)) hoveredName.value = null
  if (focusedName.value && !names.has(focusedName.value)) focusedName.value = null
})

watch(showForm, (isOpen) => {
  if (!isOpen) saveError.value = null
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
  getStableTargets: () => quickSwitch.stableTargets.value,
  onApply: handleApply,
})

onMounted(() => {
  void loadProfiles()
  void loadActiveEnvironment()
})
</script>

<style scoped>
.claude-profiles-banner {
  display: flex;
  align-items: center;
  gap: 0.875rem;
  margin-bottom: 0.875rem;
  padding: 0.875rem 1rem;
  color: var(--cp-ink-1);
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  border-left: 3px solid var(--cp-warn);
  border-radius: var(--radius-md);
}

.claude-profiles-banner > div {
  min-width: 0;
  flex: 1;
}

.claude-profiles-banner strong {
  color: var(--cp-ink-0);
  font-size: 0.875rem;
}

.claude-profiles-banner p {
  margin: 0.25rem 0 0;
  color: var(--cp-ink-2);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.claude-profiles-banner__action {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.75rem;
  color: var(--cp-accent);
  background: var(--cp-accent-soft);
  border: 1px solid var(--cp-accent-line);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.claude-profiles-banner__action:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

@media (width <= 720px) {
  .claude-profiles-banner {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .claude-profiles-banner__action {
    width: 100%;
    justify-content: center;
  }
}
</style>
