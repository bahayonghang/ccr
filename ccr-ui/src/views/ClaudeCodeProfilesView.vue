<template>
  <div class="claude-profiles-view">
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
          }"
          :loading="loading || isRefreshing"
          :exporting="isExporting"
          @add="openAddForm"
          @export="handleExportProfiles"
          @reload="refreshProfiles"
        />

        <ProfilesStatStrip
          :current="currentProfileName"
          :total="profiles.length"
          :labels="{
            current: $t('claudeProfiles.currentProfile'),
            notSet: $t('claudeProfiles.notSet'),
            currentHint: $t('claudeProfiles.statStrip.profileSubtitle'),
            total: $t('claudeProfiles.totalCount'),
            totalHint: $t('claudeProfiles.statStrip.totalHint', { enabled: enabledProfilesCount, disabled: profiles.length - enabledProfilesCount }),
            lastWrite: $t('claudeProfiles.statStrip.lastWrite'),
            lastWriteHint: $t('claudeProfiles.statStrip.lastWriteHint'),
          }"
          :secondary="{
            icon: 'ShieldCheck',
            title: $t('claudeProfiles.statStrip.authTitle'),
            value: `${subscriptionCount} · ${apiKeyCount}`,
            hint: $t('claudeProfiles.statStrip.authSplit', { subscription: subscriptionCount, apiKey: apiKeyCount }),
            mono: true,
          }"
          :last-write="lastWriteHint"
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

        <template v-else>
          <ProfilesSection
            v-if="enabledList.length > 0"
            :title="$t('claudeProfiles.groups.enabled')"
            :count="enabledList.length"
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
                v-for="profile in enabledList"
                :key="profile.name"
                :profile="profile"
                :descriptor="rowDescriptor"
                :is-current="profile.is_current"
                :disabled="loading || isRefreshing || isSaving"
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
                v-for="profile in enabledList"
                :key="profile.name"
                :profile="profile"
                :provider-color="resolveProviderColor(profile.provider)"
                :search-query="trimmedSearchQuery"
                @apply="handleApply(profile.name)"
                @edit="openEditForm(profile)"
                @delete="handleDelete(profile.name)"
              />
            </div>
          </ProfilesSection>

          <ProfilesSection
            v-if="disabledList.length > 0"
            :title="$t('claudeProfiles.groups.disabled')"
            :count="disabledList.length"
          >
            <div
              v-if="viewMode === 'list'"
              class="cp-list"
            >
              <ProfileListRow
                v-for="profile in disabledList"
                :key="profile.name"
                :profile="profile"
                :descriptor="rowDescriptor"
                :is-current="profile.is_current"
                :disabled="loading || isRefreshing || isSaving"
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
                v-for="profile in disabledList"
                :key="profile.name"
                :profile="profile"
                :provider-color="resolveProviderColor(profile.provider)"
                :search-query="trimmedSearchQuery"
                @apply="handleApply(profile.name)"
                @edit="openEditForm(profile)"
                @delete="handleDelete(profile.name)"
              />
            </div>
          </ProfilesSection>
        </template>
      </div>

      <ProfilesContextRail
        :profiles="profiles"
        :current="currentProfileName"
        :active-profile="activeProfile"
        i18n-prefix="claudeProfiles.contextRail"
        :descriptor="railDescriptor"
        @edit="openEditForm(findProfile($event))"
      />
    </main>

    <BaseModal
      v-model="showForm"
      :persistent="isSaving"
      :show-close="false"
      size="xl"
      content-class="claude-profile-editor-modal !max-w-[980px] !max-h-[90vh] rounded-2xl"
    >
      <template #header="{ titleId }">
        <div class="editor-shell-header flex items-start justify-between gap-4">
          <div class="flex min-w-0 items-start gap-4">
            <div class="editor-hero-icon flex h-14 w-14 shrink-0 items-center justify-center rounded-lg">
              <SIcon
                name="Layers"
                size="w-7 h-7"
              />
            </div>
            <div class="min-w-0">
              <p class="editor-shell-eyebrow text-xs font-semibold uppercase tracking-[0.26em]">
                {{ modalEyebrow }}
              </p>
              <div class="mt-2 flex flex-wrap items-center gap-2">
                <h2
                  :id="titleId"
                  class="editor-shell-title text-2xl font-semibold tracking-tight"
                >
                  {{ modalTitle }}
                </h2>
                <span
                  class="editor-pill px-3 py-1 text-xs font-medium"
                  :class="modalStatusClass"
                >
                  {{ modalStatus }}
                </span>
              </div>
              <p class="editor-shell-description mt-2 max-w-3xl text-sm leading-6">
                {{ modalDescription }}
              </p>
            </div>
          </div>

          <button
            type="button"
            class="editor-close-button inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl transition-colors disabled:cursor-not-allowed disabled:opacity-50"
            :aria-label="$t('claudeProfiles.closeModal')"
            :disabled="isSaving"
            @click="closeForm"
          >
            <SIcon
              name="X"
              size="w-4 h-4"
            />
          </button>
        </div>
      </template>

      <div class="flex max-h-[calc(90vh-8rem)] flex-col overflow-hidden">
        <div class="editor-nav-rail mb-4 flex flex-wrap gap-2 border-b border-border-default/35 pb-4">
          <button
            v-for="section in modalSectionItems"
            :key="section.id"
            type="button"
            class="editor-nav-button inline-flex min-h-[40px] items-center gap-2 rounded-full px-3.5 py-2 text-sm transition-[background-color,border-color,transform] duration-200 hover:-translate-y-px"
            :class="activeFormSectionId === section.id
              ? 'editor-nav-button--active'
              : 'editor-nav-button--idle'"
            @click="scrollToFormSection(section.id)"
          >
            <span class="editor-nav-button__icon flex h-7 w-7 items-center justify-center rounded-full">
              <SIcon
                :name="section.icon"
                size="w-3.5 h-3.5"
              />
            </span>
            {{ section.title }}
          </button>
        </div>

        <div
          ref="modalScrollRef"
          class="editor-scroll-area min-h-0 flex-1 overflow-y-auto pr-1"
          @scroll="syncActiveFormSection"
        >
          <ProviderTemplateSelector
            class="mb-4"
            platform="claude"
            :selected-template-id="selectedProviderTemplate"
            :selected-endpoint="selectedProviderEndpoint"
            :draft-context="claudeTemplateDraft"
            label="Provider template"
            helper="Fill non-secret provider, endpoint, and model defaults from a reusable template."
            @select="applyClaudeProviderTemplate"
            @manual="useManualProviderTemplate"
          />

          <ClaudeProfileEditorSections
            :editing-name="editingName"
            :form="form"
            :is-editing="isEditing"
            :monospace-field-class="monospaceFieldClass"
            :parsed-form-tags="parsedFormTags"
            :register-modal-section-ref="registerModalSectionRef"
            :save-error="saveError"
            :textarea-class="textareaClass"
            :text-field-class="textFieldClass"
            :update-form-field="updateFormField"
          />
        </div>

        <div class="editor-footer mt-5 flex flex-col gap-3 pt-4 sm:flex-row sm:items-center sm:justify-between">
          <p class="text-sm text-text-secondary">
            {{ $t('claudeProfiles.modalFooterHint') }}
          </p>
          <div class="flex items-center justify-end gap-3">
            <button
              type="button"
              class="editor-button editor-button--secondary min-h-[44px] rounded-2xl px-5 py-2.5 text-sm disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="isSaving"
              @click="closeForm"
            >
              {{ $t('claudeProfiles.cancel') }}
            </button>
            <button
              type="button"
              class="editor-button editor-button--primary min-h-[44px] rounded-2xl px-5 py-2.5 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="!form.name.trim() || isSaving"
              @click="handleSave()"
            >
              <span class="inline-flex items-center gap-2">
                <SIcon
                  v-if="isSaving"
                  name="RefreshCw"
                  size="w-4 h-4"
                  class="animate-spin"
                />
                {{ isEditing ? $t('claudeProfiles.save') : $t('claudeProfiles.create') }}
              </span>
            </button>
          </div>
        </div>
      </div>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { computed, h, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch, type FunctionalComponent } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  addClaudeProfile,
  applyClaudeProfile,
  deleteClaudeProfile,
  exportClaudeProfiles,
  listClaudeProfiles,
  updateClaudeProfile,
} from '@/api'
import ClaudeProfileEditorSections from '@/components/claude/ClaudeProfileEditorSections.vue'
import ClaudeProfileRow from '@/components/claude/ClaudeProfileRow.vue'
import ProfilesContextRail, { type ContextRailDescriptor, type ContextRailActiveField } from '@/components/profiles/ProfilesContextRail.vue'
import { useClaudeProfilesInsights } from '@/composables/useClaudeProfilesInsights'
import ProfilesHeader from '@/components/profiles/ProfilesHeader.vue'
import ProfileListRow, { type ProfileRowDescriptor } from '@/components/profiles/ProfileListRow.vue'
import ProfilesStatStrip from '@/components/profiles/ProfilesStatStrip.vue'
import ProfilesToolbar, { type ProfilesViewMode } from '@/components/profiles/ProfilesToolbar.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import type { ClaudeProfile, ClaudeProfileRequest, ClaudeProfilesResponse } from '@/types'
import type {
  ClaudeProfileEditorForm,
  ClaudeProfileEditorSectionItem,
  ClaudeProfileFormSectionId,
} from '@/types/claudeProfileEditor'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { getErrorMessage } from '@/types/api'
import {
  normalizeClaudeProfilesState,
  resolveProviderColor,
} from '@/utils/claudeProfiles'
import {
  useClaudeProfilesFilter,
  type ClaudeProfilesSortBy,
  type ClaudeProfilesStatusFilter,
} from '@/composables/useClaudeProfilesFilter'
import { logger } from '@/utils/logger'
import { CLAUDE_PROFILE_FORM_SECTION_IDS } from '@/types/claudeProfileEditor'
import { downloadTextFile } from '@/utils/download'
import { useUIStore } from '@/stores/ui'
import { mapTemplateToClaudeProfilePatch } from '@/utils/providerTemplates'

interface ProfilesExportResponse {
  content: string
  filename: string
}

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(true)
const isRefreshing = ref(false)
const isExporting = ref(false)
const loadError = ref<string | null>(null)
const refreshError = ref<string | null>(null)
const profiles = ref<ClaudeProfile[]>([])
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
const lastWriteHint = ref<string | null>(null)

// 列表行平台策略：base_url/model/authMode 解析 + 操作文案 + 编辑图标
const rowDescriptor = computed<ProfileRowDescriptor<ClaudeProfile>>(() => ({
  baseUrl: (profile) => {
    const raw = profile.base_url?.trim()
    return raw && raw.length > 0 ? raw : t('claudeProfiles.officialBaseUrl')
  },
  // Claude 多模型：优先主模型，回退 sonnet/opus/haiku/subagent 映射
  model: (profile) =>
    profile.model?.trim()
    || profile.default_sonnet_model?.trim()
    || profile.default_opus_model?.trim()
    || profile.default_haiku_model?.trim()
    || profile.subagent_model?.trim()
    || '—',
  authMode: (profile) =>
    profile.auth_mode === 'api_key'
      ? t('claudeProfiles.authModeApiKey')
      : t('claudeProfiles.authModeSubscription'),
  editIcon: 'Pencil',
  labels: {
    apply: t('claudeProfiles.applyProfile'),
    edit: t('claudeProfiles.editTooltip'),
    delete: t('claudeProfiles.deleteTooltip'),
  },
}))

const claudeAuthModeLabel = (mode: string): string =>
  mode === 'api_key' ? t('claudeProfiles.authModeApiKey') : t('claudeProfiles.authModeSubscription')

// 上下文侧栏平台策略：洞察来源 + 当前 profile 字段列表 + 文案/图标
const railDescriptor: ContextRailDescriptor<ClaudeProfile> = {
  editIcon: 'Pencil',
  useInsights: useClaudeProfilesInsights,
  activeFields: (profile) => {
    const baseUrl = profile.base_url?.trim() || t('claudeProfiles.officialBaseUrl')
    // Claude 多模型：优先主模型，回退 sonnet/opus/haiku/subagent 映射
    const model = profile.model?.trim()
      || profile.default_sonnet_model?.trim()
      || profile.default_opus_model?.trim()
      || profile.default_haiku_model?.trim()
      || profile.subagent_model?.trim()
      || '—'
    const fields: ContextRailActiveField[] = [
      { label: t('claudeProfiles.fields.baseUrl'), value: baseUrl },
      { label: t('claudeProfiles.fields.model'), value: model, variant: 'accent' },
      {
        label: t('claudeProfiles.fields.authMode'),
        value: claudeAuthModeLabel(profile.auth_mode ?? 'subscription'),
        variant: 'muted',
      },
    ]
    if (profile.provider) {
      fields.push({ label: t('claudeProfiles.providerLabel'), value: profile.provider, variant: 'muted' })
    }
    if (profile.account) {
      fields.push({ label: t('claudeProfiles.accountLabel'), value: profile.account })
    }
    return fields
  },
  authModeLabel: claudeAuthModeLabel,
  isDeprecatedMode: () => false,
  missingMessage: missing =>
    missing
      .map(field =>
        field === 'base_url'
          ? t('claudeProfiles.contextRail.issues.missingBaseUrl')
          : field === 'model'
            ? t('claudeProfiles.contextRail.issues.missingModel')
            : t('claudeProfiles.contextRail.issues.missingAccount'),
      )
      .join(' · '),
  runtimeSummary: (profile) => {
    const model = profile.model?.trim()
      || profile.default_sonnet_model?.trim()
      || profile.default_opus_model?.trim()
      || '—'
    const baseUrl = profile.base_url?.trim() || '—'
    return `${model} · ${baseUrl}`
  },
}
const toolbarRef = ref<InstanceType<typeof ProfilesToolbar> | null>(null)
const modalScrollRef = ref<HTMLElement | null>(null)
const activeFormSectionId = ref<ClaudeProfileFormSectionId>('basic')
const modalSectionRefs = ref<Record<ClaudeProfileFormSectionId, HTMLElement | null>>({
  basic: null,
  connection: null,
  auth: null,
  status: null,
})

const form = reactive<ClaudeProfileEditorForm>({
  name: '',
  description: '',
  auth_mode: 'subscription',
  base_url: '',
  auth_token: '',
  default_opus_model: '',
  default_sonnet_model: '',
  default_haiku_model: '',
  subagent_model: '',
  custom_model_option: '',
  custom_model_option_name: '',
  effort_level: '',
  provider: '',
  provider_type: '',
  account: '',
  tagsInput: '',
  enabled: true,
})

const currentProfileRecord = computed(() => profiles.value.find(profile => profile.is_current) ?? null)
const currentProfileName = computed(() => currentProfileRecord.value?.name ?? null)
const providerUnsetLabel = computed(() => t('claudeProfiles.providerUnset'))
const isEditingCurrent = computed(() => isEditing.value && editingName.value === currentProfileRecord.value?.name)

// 过滤/排序/分组逻辑下沉到 composable
const {
  allTags,
  allProviders,
  filtered,
  enabledList,
  disabledList,
  activeProfile,
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

// 列表行只发 name，编辑需要完整 profile 记录
const findProfile = (name: string): ClaudeProfile => {
  const profile = profiles.value.find(item => item.name === name)
  if (!profile) {
    throw new Error(`Claude profile "${name}" not found`)
  }
  return profile
}

// 简易分组容器：标题 + 计数 + 内容插槽（functional component）
const ProfilesSection: FunctionalComponent<{ title: string, count: number }> = (
  props,
  { slots },
) => {
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

const modalEyebrow = computed(() => (
  isEditing.value
    ? t('claudeProfiles.modalEditEyebrow')
    : t('claudeProfiles.modalNewEyebrow')
))
const modalTitle = computed(() => (
  isEditing.value
    ? editingName.value || t('claudeProfiles.editProfileTitle')
    : t('claudeProfiles.newProfileTitle')
))
const modalDescription = computed(() => (
  isEditing.value
    ? t('claudeProfiles.modalEditDescription')
    : t('claudeProfiles.modalNewDescription')
))
const modalStatus = computed(() => {
  if (isEditingCurrent.value) return t('claudeProfiles.modalStatusCurrent')
  if (isEditing.value) return form.enabled ? t('claudeProfiles.modalStatusEditing') : t('claudeProfiles.modalStatusDisabled')
  return t('claudeProfiles.modalStatusDraft')
})
const modalStatusClass = computed(() => {
  if (isEditingCurrent.value) return 'editor-pill--current'
  if (isEditing.value && !form.enabled) return 'editor-pill--danger'
  if (isEditing.value) return 'editor-pill--info'
  return 'editor-pill--neutral'
})

const textFieldClass = 'editor-input w-full rounded-lg px-4 py-3 text-sm'
const monospaceFieldClass = `${textFieldClass} editor-input--mono`
const textareaClass = `${textFieldClass} editor-input--textarea min-h-[116px] resize-y`

const normalizeOptional = (value: string): string | undefined => {
  const trimmed = value.trim()
  return trimmed ? trimmed : undefined
}

const parseTags = (input: string): string[] | undefined => {
  const tags = input
    .split(',')
    .map(tag => tag.trim())
    .filter(Boolean)

  return tags.length > 0 ? tags : undefined
}

const parsedFormTags = computed(() => parseTags(form.tagsInput) ?? [])
const claudeTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'claude',
  defaultName: form.provider || form.name || 'Claude provider',
  name: form.provider || form.name,
  category: 'third_party',
  baseUrls: form.base_url.trim() ? [form.base_url.trim()] : [],
  modelCatalog: [
    form.default_opus_model,
    form.default_sonnet_model,
    form.default_haiku_model,
    form.subagent_model,
  ].filter(Boolean),
  platformOverride: {
    baseUrl: form.base_url,
    provider: form.provider,
    providerType: form.provider_type,
    defaultOpusModel: form.default_opus_model,
    defaultSonnetModel: form.default_sonnet_model,
    defaultHaikuModel: form.default_haiku_model,
    subagentModel: form.subagent_model,
    description: form.description,
  },
}))
const modalSectionItems = computed<ClaudeProfileEditorSectionItem[]>(() => ([
  {
    id: 'basic' as const,
    title: t('claudeProfiles.sections.basic.title'),
    description: t('claudeProfiles.sections.basic.description'),
    icon: 'Layers',
  },
  {
    id: 'connection' as const,
    title: t('claudeProfiles.sections.connection.title'),
    description: t('claudeProfiles.sections.connection.description'),
    icon: 'Globe',
  },
  {
    id: 'auth' as const,
    title: t('claudeProfiles.sections.auth.title'),
    description: t('claudeProfiles.sections.auth.description'),
    icon: 'ShieldCheck',
  },
  {
    id: 'status' as const,
    title: t('claudeProfiles.sections.status.title'),
    description: t('claudeProfiles.sections.status.description'),
    icon: 'SlidersHorizontal',
  },
]))

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

const buildRequest = (): ClaudeProfileRequest => ({
  name: form.name.trim(),
  description: normalizeOptional(form.description),
  auth_mode: form.auth_mode,
  base_url: normalizeOptional(form.base_url),
  auth_token: normalizeOptional(form.auth_token),
  model: null,
  small_fast_model: null,
  default_opus_model: normalizeOptional(form.default_opus_model) ?? null,
  default_sonnet_model: normalizeOptional(form.default_sonnet_model) ?? null,
  default_haiku_model: normalizeOptional(form.default_haiku_model) ?? null,
  subagent_model: normalizeOptional(form.subagent_model) ?? null,
  custom_model_option: normalizeOptional(form.custom_model_option) ?? null,
  custom_model_option_name: normalizeOptional(form.custom_model_option_name) ?? null,
  effort_level: normalizeOptional(form.effort_level) ?? null,
  provider: normalizeOptional(form.provider),
  provider_type: normalizeOptional(form.provider_type),
  account: normalizeOptional(form.account),
  tags: parseTags(form.tagsInput),
  enabled: form.enabled,
})

const resetForm = () => {
  form.name = ''
  form.description = ''
  form.auth_mode = 'subscription'
  form.base_url = ''
  form.auth_token = ''
  form.default_opus_model = ''
  form.default_sonnet_model = ''
  form.default_haiku_model = ''
  form.subagent_model = ''
  form.custom_model_option = ''
  form.custom_model_option_name = ''
  form.effort_level = ''
  form.provider = ''
  form.provider_type = ''
  form.account = ''
  form.tagsInput = ''
  form.enabled = true
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const prepareFormWorkspace = () => {
  saveError.value = null
  activeFormSectionId.value = 'basic'

  void nextTick(() => {
    modalScrollRef.value?.scrollTo({ top: 0 })
    syncActiveFormSection()
  })
}

const openAddForm = () => {
  resetForm()
  isEditing.value = false
  editingName.value = ''
  showForm.value = true
  prepareFormWorkspace()
}

const VALID_EFFORT_LEVELS = ['low', 'medium', 'high', 'xhigh', 'max'] as const

const openEditForm = (profile: ClaudeProfile) => {
  form.name = profile.name
  form.description = profile.description || ''
  form.auth_mode = profile.auth_mode || 'subscription'
  form.base_url = profile.base_url || ''
  form.auth_token = profile.auth_token || ''
  form.default_opus_model = profile.default_opus_model || ''
  form.default_sonnet_model = profile.default_sonnet_model || ''
  form.default_haiku_model = profile.default_haiku_model || ''
  form.subagent_model = profile.subagent_model || ''
  form.custom_model_option = profile.custom_model_option || ''
  form.custom_model_option_name = profile.custom_model_option_name || ''
  const rawEffort = profile.effort_level || ''
  form.effort_level = (VALID_EFFORT_LEVELS as readonly string[]).includes(rawEffort) ? rawEffort : ''
  form.provider = profile.provider || ''
  form.provider_type = profile.provider_type || ''
  form.account = profile.account || ''
  form.tagsInput = (profile.tags || []).join(', ')
  form.enabled = profile.enabled !== false
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  isEditing.value = true
  editingName.value = profile.name
  showForm.value = true
  prepareFormWorkspace()
}

const closeForm = () => {
  if (isSaving.value) return

  showForm.value = false
  saveError.value = null
  activeFormSectionId.value = 'basic'
}

const useManualProviderTemplate = () => {
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyClaudeProviderTemplate = (selection: ProviderTemplateSelection) => {
  const patch = mapTemplateToClaudeProfilePatch(selection.template, selection.endpoint)

  selectedProviderTemplate.value = selection.template.id
  selectedProviderEndpoint.value = selection.endpoint || ''
  form.base_url = patch.base_url || ''
  form.provider = patch.provider || selection.template.name
  form.provider_type = patch.provider_type || ''
  form.default_opus_model = patch.default_opus_model || ''
  form.default_sonnet_model = patch.default_sonnet_model || ''
  form.default_haiku_model = patch.default_haiku_model || ''
  form.subagent_model = patch.subagent_model || ''

  // 选用 provider 模板即意味着走第三方/中转端点 → 默认 api_key 鉴权，
  // 避免落到 subscription 被后端清空。
  if (patch.base_url) {
    form.auth_mode = 'api_key'
  }

  if (!form.name.trim()) {
    form.name = patch.suggestedName || selection.template.id
  }
  if (!form.description.trim() && patch.description) {
    form.description = patch.description
  }

  activeFormSectionId.value = 'connection'
}

type SectionRefTarget = Element | { $el?: unknown } | null

const resolveSectionElement = (target: SectionRefTarget): HTMLElement | null => {
  if (!target) return null
  if (target instanceof HTMLElement) return target

  if ('$el' in target) {
    const { $el } = target
    return $el instanceof HTMLElement ? $el : null
  }

  return null
}

const registerModalSectionRef = (sectionId: ClaudeProfileFormSectionId, target: SectionRefTarget) => {
  const resolvedElement = resolveSectionElement(target)

  modalSectionRefs.value[sectionId] = resolvedElement
}

const resetFilters = () => {
  searchQuery.value = ''
  statusFilter.value = 'all'
  tagFilter.value = null
  providerFilter.value = null
}

const isEditableTarget = (el: EventTarget | null): boolean => {
  if (!(el instanceof HTMLElement)) return false
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
}

// 快捷键：⌘K / 聚焦搜索框
const handleGlobalKeydown = (event: KeyboardEvent) => {
  if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
    event.preventDefault()
    toolbarRef.value?.focusSearch()
    return
  }
  if (event.key === '/' && !isEditableTarget(event.target)) {
    event.preventDefault()
    toolbarRef.value?.focusSearch()
  }
}

const syncActiveFormSection = () => {
  const container = modalScrollRef.value

  if (!container) return

  let nextSection: ClaudeProfileFormSectionId = 'basic'

  CLAUDE_PROFILE_FORM_SECTION_IDS.forEach((sectionId) => {
    const element = modalSectionRefs.value[sectionId]

    if (element && element.offsetTop - container.scrollTop <= 140) {
      nextSection = sectionId
    }
  })

  activeFormSectionId.value = nextSection
}

const scrollToFormSection = (sectionId: ClaudeProfileFormSectionId) => {
  const container = modalScrollRef.value
  const element = modalSectionRefs.value[sectionId]

  activeFormSectionId.value = sectionId

  if (!container || !element) return

  container.scrollTo({
    top: Math.max(element.offsetTop - 16, 0),
    behavior: 'smooth',
  })
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
    const data = await listClaudeProfiles<ClaudeProfilesResponse>()
    const normalized = normalizeClaudeProfilesState(data.profiles || [], data.current_profile || null)

    profiles.value = normalized.profiles
    loadError.value = null
    refreshError.value = null
    lastWriteHint.value = new Date().toLocaleTimeString()

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
    const payload = await exportClaudeProfiles<ProfilesExportResponse>(true)
    downloadTextFile(payload.filename, payload.content, 'application/toml;charset=utf-8')
    uiStore.showSuccess(t('claudeProfiles.exportSuccess'))
  } catch (error) {
    logger.error('Failed to export Claude profiles:', error)
    uiStore.showError(getErrorMessage(error, t('claudeProfiles.exportFailed')))
  } finally {
    isExporting.value = false
  }
}

const handleSave = async () => {
  const trimmedName = form.name.trim()
  if (!trimmedName) return

  // 重命名场景：name 改了。先做客户端唯一性校验 + 弹二次确认
  const isRenaming = isEditing.value && trimmedName !== editingName.value
  if (isRenaming) {
    const collision = profiles.value.some(p => p.name === trimmedName && p.name !== editingName.value)
    if (collision) {
      saveError.value = translateWithFallback(
        t,
        'claudeProfiles.renameConflict',
        '名称 "{name}" 已被其它 Profile 占用，请换一个名称。',
        { name: trimmedName },
      )
      return
    }

    const confirmed = confirm(translateWithFallback(
      t,
      'claudeProfiles.renameConfirmBody',
      '将 "{old}" 重命名为 "{new}"。旧名称会被删除；若当前激活，激活指针会自动迁移到新名。',
      { old: editingName.value, new: trimmedName },
    ))
    if (!confirmed) return
  }

  isSaving.value = true
  saveError.value = null

  try {
    const request = buildRequest()

    if (isEditing.value) {
      await updateClaudeProfile(editingName.value, request)
    } else {
      await addClaudeProfile(request)
    }

    showForm.value = false
    activeFormSectionId.value = 'basic'
    await loadProfiles({ preserveData: profiles.value.length > 0 })
  } catch (error) {
    logger.error('Failed to save Claude profile:', error)
    saveError.value = getErrorMessage(error, t('claudeProfiles.operationFailed'))
  } finally {
    isSaving.value = false
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(translateWithFallback(
    t,
    'claudeProfiles.deleteConfirm',
    '确定要删除 Profile "{name}" 吗？',
    { name },
  ))) return

  try {
    await deleteClaudeProfile(name)
    await loadProfiles({ preserveData: profiles.value.length > 0 })
  } catch (error) {
    logger.error('Failed to delete Claude profile:', error)
    alert(getErrorMessage(error, t('claudeProfiles.deleteFailed')))
  }
}

const handleApply = async (name: string) => {
  const targetProfile = profiles.value.find(profile => profile.name === name)
  if (!targetProfile || targetProfile.is_current || targetProfile.enabled === false) return

  if (!confirm(translateWithFallback(
    t,
    'claudeProfiles.confirmApply',
    '确定要应用 Profile "{name}" 吗？这将同步更新当前 Claude 配置。',
    { name },
  ))) return

  try {
    await applyClaudeProfile(name)
    await loadProfiles({ preserveData: profiles.value.length > 0 })
  } catch (error) {
    logger.error('Failed to apply Claude profile:', error)
    alert(getErrorMessage(error, t('claudeProfiles.applyFailed')))
  }
}

// 当前激活的标签/Provider 若因数据变化而失效，自动回退到"全部"
watch([allTags, tagFilter], ([tags, tag]) => {
  if (tag && !tags.includes(tag)) tagFilter.value = null
})
watch([allProviders, providerFilter], ([providers, provider]) => {
  if (provider && !providers.some(item => item.key === provider)) providerFilter.value = null
})

watch(showForm, (isOpen) => {
  if (isOpen) return

  saveError.value = null
  activeFormSectionId.value = 'basic'
})

onMounted(() => {
  void loadProfiles()
  document.addEventListener('keydown', handleGlobalKeydown)
})
onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<style>
.claude-profile-editor-modal {
  --editor-shell-bg: linear-gradient(180deg, rgb(var(--color-bg-surface-rgb) / 96%), rgb(var(--color-bg-elevated-rgb) / 92%));
  --editor-shell-border: rgb(var(--color-border-default-rgb) / 72%);
  --editor-shell-shadow: 0 28px 80px rgb(var(--color-accent-primary-rgb) / 10%), 0 12px 32px rgb(var(--color-text-primary-rgb) / 8%);
  --editor-shell-highlight: radial-gradient(circle at top right, rgb(var(--color-accent-primary-rgb) / 10%), transparent 42%);
  --editor-panel-bg: rgb(var(--color-bg-surface-rgb) / 88%);
  --editor-panel-muted-bg: rgb(var(--color-bg-overlay-rgb) / 60%);
  --editor-panel-head-bg: rgb(var(--color-bg-elevated-rgb) / 96%);
  --editor-input-bg: rgb(var(--color-bg-elevated-rgb) / 94%);
  --editor-input-bg-hover: rgb(var(--color-bg-surface-rgb) / 96%);
  --editor-input-bg-focus: rgb(var(--color-bg-surface-rgb) / 100%);
  --editor-input-border: rgb(var(--color-border-default-rgb) / 80%);
  --editor-input-border-strong: rgb(var(--color-accent-primary-rgb) / 38%);
  --editor-hairline: rgb(var(--color-border-default-rgb) / 64%);
  --editor-hairline-soft: rgb(var(--color-border-default-rgb) / 40%);
  --editor-ink: rgb(var(--color-text-primary-rgb) / 96%);
  --editor-ink-muted: rgb(var(--color-text-secondary-rgb) / 90%);
  --editor-ink-soft: rgb(var(--color-text-muted-rgb) / 86%);
  --editor-placeholder: rgb(var(--color-text-muted-rgb) / 74%);
  --editor-panel-shadow: inset 0 1px 0 rgb(var(--color-bg-surface-rgb) / 60%), 0 12px 28px rgb(var(--color-text-primary-rgb) / 4%);
  --editor-muted-shadow: none;
  --editor-ring: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 16%);
  --editor-scrollbar-thumb: rgb(var(--color-accent-primary-rgb) / 34%);
  --editor-scrollbar-track: rgb(var(--color-bg-overlay-rgb) / 30%);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  background: var(--editor-shell-bg) !important;
  border: 1px solid var(--editor-shell-border) !important;
  box-shadow: var(--editor-shell-shadow) !important;
  color: var(--editor-ink);
}

:root[class~='dark'] .claude-profile-editor-modal,
[data-theme='dark'] .claude-profile-editor-modal {
  --editor-shell-shadow: 0 32px 90px rgb(0 0 0 / 56%), 0 18px 42px rgb(0 0 0 / 36%);
  --editor-panel-shadow: inset 0 1px 0 rgb(255 255 255 / 6%), 0 16px 32px rgb(0 0 0 / 24%);
  --editor-muted-shadow: none;
  --editor-scrollbar-track: rgb(var(--color-bg-base-rgb) / 36%);
}

.claude-profile-editor-modal::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--editor-shell-highlight);
  pointer-events: none;
  z-index: 0;
}

.claude-profile-editor-modal > * {
  position: relative;
  z-index: 1;
}

.claude-profile-editor-modal .text-text-primary {
  color: var(--editor-ink) !important;
}

.claude-profile-editor-modal .text-text-secondary {
  color: var(--editor-ink-muted) !important;
}

.claude-profile-editor-modal .text-text-muted {
  color: var(--editor-ink-soft) !important;
}

.claude-profile-editor-modal .editor-shell-header {
  border-bottom: 1px solid var(--editor-hairline-soft);
}

.claude-profile-editor-modal .editor-hero-icon,
.claude-profile-editor-modal .editor-summary-icon,
.claude-profile-editor-modal .editor-section-icon {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: rgb(var(--color-accent-primary-rgb) / 100%);
  box-shadow: 0 12px 24px rgb(var(--color-accent-primary-rgb) / 12%);
}

.claude-profile-editor-modal .editor-shell-eyebrow {
  color: rgb(var(--color-accent-primary-rgb) / 90%);
}

.claude-profile-editor-modal .editor-shell-title {
  color: var(--editor-ink);
}

.claude-profile-editor-modal .editor-shell-description {
  color: var(--editor-ink-muted);
}

.claude-profile-editor-modal .editor-close-button {
  border: 1px solid var(--editor-hairline);
  background: rgb(var(--color-bg-elevated-rgb) / 70%);
  color: var(--editor-ink-soft);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 5%);
}

.claude-profile-editor-modal .editor-close-button:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--editor-ink);
}

.claude-profile-editor-modal .editor-icon-button {
  display: inline-flex;
  height: 36px;
  width: 36px;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--editor-hairline-soft);
  border-radius: 14px;
  background: rgb(var(--color-bg-elevated-rgb) / 56%);
  color: var(--editor-ink-muted);
  transition: background-color 180ms ease, border-color 180ms ease, color 180ms ease, transform 180ms ease;
}

.claude-profile-editor-modal .editor-icon-button:hover {
  border-color: var(--editor-hairline);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  color: var(--editor-ink);
  transform: translateY(-1px);
}

.claude-profile-editor-modal .editor-icon-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
  transform: none;
}

.claude-profile-editor-modal .editor-close-button:focus-visible,
.claude-profile-editor-modal .editor-button:focus-visible,
.claude-profile-editor-modal .editor-icon-button:focus-visible,
.claude-profile-editor-modal .editor-input:focus-visible,
.claude-profile-editor-modal .editor-nav-button:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
  outline-offset: 2px;
  box-shadow: var(--editor-ring);
}

.claude-profile-editor-modal .editor-scroll-area {
  scrollbar-color: var(--editor-scrollbar-thumb) var(--editor-scrollbar-track);
}

.claude-profile-editor-modal .editor-scroll-area::-webkit-scrollbar {
  width: 10px;
}

.claude-profile-editor-modal .editor-scroll-area::-webkit-scrollbar-track {
  background: var(--editor-scrollbar-track);
  border-radius: 999px;
}

.claude-profile-editor-modal .editor-scroll-area::-webkit-scrollbar-thumb {
  background: var(--editor-scrollbar-thumb);
  border-radius: 999px;
}

.claude-profile-editor-modal .editor-panel {
  border: 1px solid var(--editor-hairline);
  background: var(--editor-panel-bg);
  box-shadow: var(--editor-panel-shadow);
}

.claude-profile-editor-modal .editor-panel-head {
  border-color: var(--editor-hairline-soft);
  background: var(--editor-panel-head-bg);
}

.claude-profile-editor-modal .editor-panel-muted,
.claude-profile-editor-modal .editor-info-card,
.claude-profile-editor-modal .editor-inline-card,
.claude-profile-editor-modal .editor-empty-hint {
  border: 1px solid var(--editor-hairline-soft);
  background: var(--editor-panel-muted-bg);
}

.claude-profile-editor-modal .editor-info-icon {
  border: 1px solid var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  color: var(--editor-ink-muted);
}

.claude-profile-editor-modal .editor-nav-button {
  border: 1px solid var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 34%);
  color: var(--editor-ink-muted);
}

.claude-profile-editor-modal .editor-nav-button:hover {
  border-color: var(--editor-hairline);
  background: var(--editor-panel-muted-bg);
  color: var(--editor-ink);
}

.claude-profile-editor-modal .editor-nav-button__icon {
  border: 1px solid var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 56%);
  color: var(--editor-ink-soft);
}

.claude-profile-editor-modal .editor-nav-button--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 34%);
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 12%), rgb(var(--color-accent-primary-rgb) / 8%));
  color: var(--editor-ink);
  box-shadow: 0 14px 32px rgb(var(--color-accent-primary-rgb) / 12%);
}

.claude-profile-editor-modal .editor-nav-button--active .editor-nav-button__icon {
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: rgb(var(--color-accent-primary-rgb) / 100%);
}

.claude-profile-editor-modal .editor-tag,
.claude-profile-editor-modal .editor-inline-chip {
  border: 1px solid var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 50%);
}

.claude-profile-editor-modal .editor-banner {
  border: 1px solid rgb(var(--color-danger-rgb) / 22%);
  background: linear-gradient(180deg, rgb(var(--color-danger-rgb) / 12%), rgb(var(--color-danger-rgb) / 6%));
  box-shadow: 0 18px 40px rgb(var(--color-danger-rgb) / 8%);
}

.claude-profile-editor-modal .editor-banner__icon {
  background: rgb(var(--color-danger-rgb) / 12%);
  color: rgb(var(--color-danger-rgb) / 100%);
}

.claude-profile-editor-modal .editor-banner--warn {
  border-color: rgb(var(--color-warning-rgb) / 28%);
  background: linear-gradient(180deg, rgb(var(--color-warning-rgb) / 12%), rgb(var(--color-warning-rgb) / 6%));
  box-shadow: 0 18px 40px rgb(var(--color-warning-rgb) / 8%);
}

.claude-profile-editor-modal .editor-banner--warn .editor-banner__icon {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: rgb(var(--color-warning-rgb) / 100%);
}

.claude-profile-editor-modal .editor-input {
  border: 1px solid var(--editor-input-border);
  background: var(--editor-input-bg);
  color: var(--editor-ink);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 5%);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, color 180ms ease;
}

.claude-profile-editor-modal .editor-input::placeholder {
  color: var(--editor-placeholder);
}

.claude-profile-editor-modal .editor-input:hover {
  border-color: var(--editor-hairline);
  background: var(--editor-input-bg-hover);
}

.claude-profile-editor-modal .editor-input:focus {
  border-color: var(--editor-input-border-strong);
  background: var(--editor-input-bg-focus);
  outline: none;
  box-shadow: var(--editor-ring);
}

.claude-profile-editor-modal .editor-input:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
  outline-offset: 2px;
}

.claude-profile-editor-modal .editor-input:disabled,
.claude-profile-editor-modal .editor-input[readonly] {
  background: rgb(var(--color-bg-elevated-rgb) / 42%);
  color: var(--editor-ink-soft);
}

.claude-profile-editor-modal .editor-input--mono {
  font-family: var(--font-mono);
  letter-spacing: 0.01em;
}

.claude-profile-editor-modal .editor-input--textarea {
  line-height: 1.65;
}

.claude-profile-editor-modal input[type='checkbox'] {
  border-color: var(--editor-input-border);
  background: rgb(var(--color-bg-elevated-rgb) / 62%);
  color: rgb(var(--color-accent-primary-rgb) / 100%);
}

.claude-profile-editor-modal input[type='checkbox']:focus {
  box-shadow: var(--editor-ring);
}

.claude-profile-editor-modal .editor-pill {
  border: 1px solid transparent;
}

.claude-profile-editor-modal .editor-pill--neutral {
  border-color: var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 52%);
  color: var(--editor-ink-muted);
}

.claude-profile-editor-modal .editor-pill--current,
.claude-profile-editor-modal .editor-pill--info {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: rgb(var(--color-accent-primary-rgb) / 100%);
}

.claude-profile-editor-modal .editor-pill--success {
  border-color: rgb(var(--color-success-rgb) / 20%);
  background: rgb(var(--color-success-rgb) / 14%);
  color: rgb(var(--color-success-rgb) / 100%);
}

.claude-profile-editor-modal .editor-pill--danger {
  border-color: rgb(var(--color-danger-rgb) / 24%);
  background: rgb(var(--color-danger-rgb) / 12%);
  color: rgb(var(--color-danger-rgb) / 100%);
}

.claude-profile-editor-modal .editor-footer {
  position: sticky;
  bottom: 0;
  border-top: 1px solid var(--editor-hairline-soft);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 72%), rgb(var(--color-bg-elevated-rgb) / 92%));
  box-shadow: 0 -12px 32px rgb(0 0 0 / 4%);
}

:root[class~='dark'] .claude-profile-editor-modal .editor-footer,
[data-theme='dark'] .claude-profile-editor-modal .editor-footer {
  box-shadow: 0 -16px 36px rgb(6 3 10 / 28%);
}

.claude-profile-editor-modal .editor-button {
  border: 1px solid transparent;
  transition: background-color 180ms ease, border-color 180ms ease, color 180ms ease, box-shadow 180ms ease, transform 180ms ease;
}

.claude-profile-editor-modal .editor-button:hover {
  transform: translateY(-1px);
}

.claude-profile-editor-modal .editor-button--secondary {
  border-color: var(--editor-hairline);
  background: rgb(var(--color-bg-elevated-rgb) / 68%);
  color: var(--editor-ink-muted);
}

.claude-profile-editor-modal .editor-button--secondary:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  color: var(--editor-ink);
}

.claude-profile-editor-modal .editor-button--primary {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-primary-rgb) / 12%));
  color: rgb(var(--color-accent-primary-rgb) / 100%);
  box-shadow: 0 12px 24px rgb(var(--color-accent-primary-rgb) / 14%);
}

.claude-profile-editor-modal .editor-button--primary:hover {
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 24%), rgb(var(--color-accent-primary-rgb) / 16%));
}


/* ===========================================================
   作用域设计令牌：仅在本视图内生效，子组件靠继承解析 --cp-*
   主色用暖中性 accent-secondary（与统一身份色系统一致）
   =========================================================== */
.claude-profiles-view {
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

  /* 主色 → 暖中性 accent-secondary */
  --cp-accent: var(--color-accent-secondary);
  --cp-accent-soft: rgb(var(--color-accent-secondary-rgb) / 14%);
  --cp-accent-line: rgb(var(--color-accent-secondary-rgb) / 35%);
  --cp-accent-hover: var(--color-accent-secondary-hover);
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
  gap: 16px;
  align-items: start;
}

@media (width >= 1280px) {
  .cp-shell {
    grid-template-columns: minmax(0, 1fr) 320px;
  }
}

.cp-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
}

/* 分组容器 */
.cp-section {
  margin-bottom: 18px;
}

.cp-section__head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.cp-section__icon {
  color: var(--cp-ink-3);
}

.cp-section__title {
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--cp-ink-2);
}

.cp-section__count {
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--cp-bg-3);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-3);
  font-family: var(--cp-mono);
  font-size: 10.5px;
}

.cp-section__body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* 卡片视图栅格 */
.cp-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 10px;
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
  font-size: 9.5px;
  letter-spacing: 0.8px;
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
  font-size: 14px;
  font-weight: 600;
  color: var(--cp-ink-0);
}

.cp-state__hint {
  font-size: 12px;
  color: var(--cp-ink-2);
  max-width: 420px;
}

.cp-state__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
  padding: 7px 14px;
  border-radius: 7px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-3);
  color: var(--cp-ink-1);
  font-size: 12.5px;
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
</style>
