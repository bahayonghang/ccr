<template>
  <div class="claude-profiles-view">
    <div class="claude-profiles-view__shell">
      <div class="claude-profiles-view__breadcrumb animate-slide-up">
        <RouterLink
          to="/claude-code"
          class="claude-profiles-view__breadcrumb-link"
        >
          Claude Code
        </RouterLink>
        <SIcon
          name="ChevronRight"
          size="w-3 h-3"
        />
        <span class="claude-profiles-view__breadcrumb-current">{{ $t('claudeProfiles.breadcrumbProfiles') }}</span>
      </div>

      <PageHeaderCard
        class="animate-slide-up"
        icon="Layers"
        tone="secondary"
        :title="$t('claudeProfiles.title')"
        :description="$t('claudeProfiles.subtitle')"
      >
        <template #meta>
          <span class="claude-profiles-view__eyebrow">
            {{ $t('claudeProfiles.consoleEyebrow') }}
          </span>
          <span
            v-if="showNavigation"
            class="claude-profiles-view__meta-chip"
          >
            {{ providerSectionsCountLabel }}
          </span>
        </template>

        <template #actions>
          <RouterLink
            to="/claude-code"
            class="claude-profiles-view__header-button claude-profiles-view__header-button--secondary"
          >
            <SIcon
              name="ArrowLeft"
              size="w-4 h-4"
            />
            {{ $t('claudeProfiles.back') }}
          </RouterLink>

          <button
            type="button"
            class="claude-profiles-view__header-button claude-profiles-view__header-button--secondary"
            :disabled="loading || isRefreshing || isSaving"
            @click="refreshProfiles()"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': loading || isRefreshing }"
            />
            {{ $t('common.refresh') }}
          </button>

          <button
            type="button"
            class="claude-profiles-view__header-button claude-profiles-view__header-button--secondary"
            :disabled="loading || isRefreshing || isSaving || isExporting"
            @click="handleExportProfiles()"
          >
            <SIcon
              name="Download"
              size="w-4 h-4"
            />
            {{ $t('common.export') }}
          </button>

          <button
            type="button"
            class="claude-profiles-view__header-button claude-profiles-view__header-button--primary"
            :disabled="isSaving"
            @click="openAddForm()"
          >
            <SIcon
              name="Plus"
              size="w-4 h-4"
            />
            {{ $t('claudeProfiles.addProfile') }}
          </button>
        </template>

        <ClaudeProfilesOverview
          :current-profile="currentProfileRecord"
          :provider-unset-label="providerUnsetLabel"
          :summary="overviewSummary"
        />
      </PageHeaderCard>

      <section
        v-if="showSearchRail"
        class="claude-profiles-view__search-rail animate-slide-up"
        style="animation-delay: 120ms"
      >
        <div class="claude-profiles-view__search-grid">
          <div class="claude-profiles-view__search-input-shell">
            <Input
              v-model="searchQuery"
              type="text"
              surface="status"
              :elevation="1"
              motion="subtle"
              density="compact"
              :full-width="true"
              :placeholder="$t('claudeProfiles.searchPlaceholder')"
            >
              <template #leading>
                <SIcon
                  name="Search"
                  size="w-4 h-4"
                />
              </template>
            </Input>
          </div>

          <div class="claude-profiles-view__search-meta">
            <span class="claude-profiles-view__search-chip">
              {{ searchProfilesCountLabel }}
            </span>
            <span class="claude-profiles-view__search-chip">
              {{ searchProvidersCountLabel }}
            </span>
            <button
              v-if="hasActiveSearch"
              type="button"
              class="claude-profiles-view__search-clear"
              @click="clearSearch()"
            >
              <SIcon
                name="RotateCcw"
                size="w-3.5 h-3.5"
              />
              {{ $t('claudeProfiles.clearSearch') }}
            </button>
          </div>
        </div>

        <p class="claude-profiles-view__search-hint">
          {{ $t('claudeProfiles.searchHint') }}
        </p>

        <section
          v-if="filteredProfiles.length > 0"
          class="claude-profiles-view__command-strip"
        >
          <div class="claude-profiles-view__command-strip-head">
            <div>
              <p class="claude-profiles-view__command-strip-title">
                {{ $t('claudeProfiles.quickSwitchStripTitle') }}
              </p>
              <p class="claude-profiles-view__command-strip-hint">
                {{ $t('claudeProfiles.quickSwitchStripHint') }}
              </p>
            </div>
            <span class="claude-profiles-view__command-strip-count">
              {{ quickSwitchStripCountLabel }}
            </span>
          </div>

          <div
            class="claude-profiles-view__command-strip-scroll"
            role="toolbar"
            :aria-label="$t('claudeProfiles.quickSwitch')"
          >
            <button
              v-for="profile in filteredProfiles"
              :key="profile.name"
              type="button"
              :disabled="profile.is_current || profile.enabled === false"
              class="claude-profiles-view__command-pill"
              :class="profile.is_current
                ? 'claude-profiles-view__command-pill--current'
                : (profile.enabled === false
                  ? 'claude-profiles-view__command-pill--disabled'
                  : 'claude-profiles-view__command-pill--idle')"
              :title="profile.provider?.trim() || providerUnsetLabel"
              @click="handleApply(profile.name)"
            >
              <span
                class="claude-profiles-view__command-pill-dot"
                :class="profile.is_current
                  ? 'claude-profiles-view__command-pill-dot--current'
                  : (profile.enabled === false
                    ? 'claude-profiles-view__command-pill-dot--disabled'
                    : 'claude-profiles-view__command-pill-dot--enabled')"
              />
              <span class="truncate">{{ profile.name }}</span>
            </button>
          </div>
        </section>
      </section>

      <div
        :class="[
          'claude-profiles-view__layout',
          showNavigation ? 'claude-profiles-view__layout--with-nav' : '',
        ]"
      >
        <aside
          v-if="showNavigation"
          class="claude-profiles-view__sidebar"
        >
          <ClaudeProfilesProviderNav
            :sections="visibleProviderSections"
            :active-section-id="currentSectionId"
            :provider-unset-label="providerUnsetLabel"
            @navigate="scrollToSection"
          />
        </aside>

        <main class="claude-profiles-view__main">
          <ClaudeProfilesProviderNav
            v-if="showNavigation"
            mobile
            :sections="visibleProviderSections"
            :active-section-id="currentSectionId"
            :provider-unset-label="providerUnsetLabel"
            class="claude-profiles-view__mobile-nav"
            @navigate="scrollToSection"
          />

          <div
            v-if="loading"
            class="flex items-center justify-center py-20"
          >
            <div class="h-8 w-8 rounded-full border-2 border-accent-secondary/30 border-t-accent-secondary animate-spin" />
          </div>

          <div
            v-else-if="loadError"
            class="rounded-[28px] border border-accent-danger/20 bg-accent-danger/5 p-6 animate-slide-up"
            style="animation-delay: 180ms"
          >
            <div class="flex items-start gap-4">
              <div class="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl bg-accent-danger/10 text-accent-danger">
                <SIcon
                  name="AlertTriangle"
                  size="w-5 h-5"
                />
              </div>
              <div class="min-w-0 flex-1">
                <h3 class="text-lg font-semibold text-text-primary">
                  {{ $t('claudeProfiles.loadFailedTitle') }}
                </h3>
                <p class="mt-1 break-words text-sm text-text-secondary">
                  {{ loadError }}
                </p>
              </div>
              <button
                type="button"
                class="rounded-2xl border border-accent-danger/25 bg-accent-danger/10 px-4 py-2 text-sm font-medium text-accent-danger transition-colors hover:bg-accent-danger/15"
                @click="refreshProfiles()"
              >
                {{ $t('claudeProfiles.retry') }}
              </button>
            </div>
          </div>

          <div
            v-else-if="refreshError"
            class="rounded-[24px] border border-accent-warning/20 bg-accent-warning/6 p-5 animate-slide-up"
            style="animation-delay: 160ms"
          >
            <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
              <div class="flex items-start gap-3">
                <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-accent-warning/10 text-accent-warning">
                  <SIcon
                    name="AlertCircle"
                    size="w-5 h-5"
                  />
                </div>
                <div class="min-w-0">
                  <h3 class="text-sm font-semibold text-text-primary">
                    {{ $t('claudeProfiles.refreshFailedTitle') }}
                  </h3>
                  <p class="mt-1 text-sm text-text-secondary">
                    {{ refreshError }}
                  </p>
                  <p class="mt-2 text-xs text-text-muted">
                    {{ $t('claudeProfiles.refreshFailedHint') }}
                  </p>
                </div>
              </div>

              <button
                type="button"
                class="self-start rounded-2xl border border-accent-warning/25 bg-accent-warning/10 px-4 py-2 text-sm font-medium text-accent-warning transition-colors hover:bg-accent-warning/15 disabled:cursor-not-allowed disabled:opacity-60"
                :disabled="isRefreshing"
                @click="refreshProfiles()"
              >
                <span class="inline-flex items-center gap-2">
                  <SIcon
                    name="RefreshCw"
                    size="w-4 h-4"
                    :class="{ 'animate-spin': isRefreshing }"
                  />
                  {{ $t('claudeProfiles.retry') }}
                </span>
              </button>
            </div>
          </div>

          <div
            v-if="!loadError && profiles.length === 0"
            class="py-20 text-center animate-slide-up"
            style="animation-delay: 200ms"
          >
            <div class="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-[28px] border border-border-default/50 bg-bg-surface/75 shadow-lg shadow-black/5">
              <SIcon
                name="FolderOpen"
                size="w-10 h-10"
                class="text-text-muted"
              />
            </div>
            <h3 class="mb-2 text-xl font-semibold text-text-primary">
              {{ $t('claudeProfiles.emptyTitle') }}
            </h3>
            <p class="mx-auto mb-6 max-w-xl text-text-secondary">
              {{ $t('claudeProfiles.emptyDesc') }}
            </p>
            <button
              type="button"
              class="inline-flex min-h-[44px] items-center justify-center rounded-2xl border border-accent-secondary/30 bg-accent-secondary/10 px-6 py-3 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/15 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
              @click="openAddForm()"
            >
              <SIcon
                name="Plus"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('claudeProfiles.createProfile') }}
            </button>
          </div>

          <div
            v-else-if="filteredProfiles.length === 0"
            class="py-20 text-center animate-slide-up"
            style="animation-delay: 200ms"
          >
            <div class="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-[28px] border border-border-default/50 bg-bg-surface/75 shadow-lg shadow-black/5">
              <SIcon
                name="SearchX"
                size="w-10 h-10"
                class="text-text-muted"
              />
            </div>
            <h3 class="mb-2 text-xl font-semibold text-text-primary">
              {{ $t('claudeProfiles.searchEmptyTitle') }}
            </h3>
            <p class="mx-auto mb-6 max-w-xl text-text-secondary">
              {{ $t('claudeProfiles.searchEmptyDesc') }}
            </p>
            <button
              type="button"
              class="inline-flex min-h-[44px] items-center justify-center rounded-2xl border border-accent-secondary/30 bg-accent-secondary/10 px-6 py-3 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/15 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
              @click="clearSearch()"
            >
              <SIcon
                name="RotateCcw"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('claudeProfiles.clearSearch') }}
            </button>
          </div>

          <ClaudeProfilesSectionList
            v-else
            :provider-sections="visibleProviderSections"
            :provider-unset-label="providerUnsetLabel"
            :register-section-ref="registerSectionRef"
            :search-query="trimmedSearchQuery"
            @apply="handleApply"
            @delete="handleDelete"
            @edit="openEditForm"
          />
        </main>
      </div>

      <BaseModal
        v-model="showForm"
        :persistent="isSaving"
        :show-close="false"
        size="xl"
        content-class="claude-profile-editor-modal !max-w-[980px] !max-h-[90vh] rounded-[32px]"
      >
        <template #header="{ titleId }">
          <div class="editor-shell-header flex items-start justify-between gap-4">
            <div class="flex min-w-0 items-start gap-4">
              <div class="editor-hero-icon flex h-14 w-14 shrink-0 items-center justify-center rounded-[20px]">
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
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch, type ComponentPublicInstance } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import {
  addClaudeProfile,
  applyClaudeProfile,
  deleteClaudeProfile,
  exportClaudeProfiles,
  listClaudeProfiles,
  updateClaudeProfile,
} from '@/api'
import ClaudeProfileEditorSections from '@/components/claude/ClaudeProfileEditorSections.vue'
import ClaudeProfilesOverview from '@/components/claude/ClaudeProfilesOverview.vue'
import ClaudeProfilesProviderNav from '@/components/claude/ClaudeProfilesProviderNav.vue'
import ClaudeProfilesSectionList from '@/components/claude/ClaudeProfilesSectionList.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import Input from '@/components/ui/Input.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import type { ClaudeProfile, ClaudeProfileRequest, ClaudeProfilesResponse } from '@/types'
import type {
  ClaudeProfileEditorForm,
  ClaudeProfileEditorSectionItem,
  ClaudeProfileFormSectionId,
} from '@/types/claudeProfileEditor'
import { getErrorMessage } from '@/types/api'
import {
  createClaudeProfilesOverviewSummary,
  createClaudeProfileSections,
  filterClaudeProfiles,
  normalizeClaudeProfilesState,
} from '@/utils/claudeProfiles'
import { logger } from '@/utils/logger'
import { CLAUDE_PROFILE_FORM_SECTION_IDS } from '@/types/claudeProfileEditor'
import { downloadTextFile } from '@/utils/download'
import { useUIStore } from '@/stores/ui'

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
const currentSectionId = ref<string | null>(null)
const searchQuery = ref('')
const sectionRefs = ref<Record<string, HTMLElement | null>>({})
const sectionObserver = ref<IntersectionObserver | null>(null)
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
  effort_level: '',
  provider: '',
  provider_type: '',
  account: '',
  tagsInput: '',
  enabled: true,
})

const trimmedSearchQuery = computed(() => searchQuery.value.trim())
const hasActiveSearch = computed(() => trimmedSearchQuery.value.length > 0)
const currentProfileRecord = computed(() => profiles.value.find(profile => profile.is_current) ?? null)
const providerUnsetLabel = computed(() => t('claudeProfiles.providerUnset'))
const providerSections = computed(() => createClaudeProfileSections(profiles.value, providerUnsetLabel.value))
const filteredProfiles = computed(() => filterClaudeProfiles(profiles.value, trimmedSearchQuery.value))
const visibleProviderSections = computed(() => createClaudeProfileSections(filteredProfiles.value, providerUnsetLabel.value))
const overviewSummary = computed(() => createClaudeProfilesOverviewSummary(profiles.value, providerUnsetLabel.value))
const showSearchRail = computed(() => !loading.value && !loadError.value && profiles.value.length > 0)
const showNavigation = computed(() => !loading.value && !loadError.value && visibleProviderSections.value.length > 1)
const isEditingCurrent = computed(() => isEditing.value && editingName.value === currentProfileRecord.value?.name)
const providerSectionsCountLabel = computed(() => translateWithFallback(
  t,
  'claudeProfiles.providerSectionsCount',
  'Provider 分组 {count}',
  { count: providerSections.value.length },
))
const searchProfilesCountLabel = computed(() => translateWithFallback(
  t,
  'claudeProfiles.searchProfilesCount',
  '{matched} / {total} Profiles',
  {
    matched: filteredProfiles.value.length,
    total: profiles.value.length,
  },
))
const searchProvidersCountLabel = computed(() => translateWithFallback(
  t,
  'claudeProfiles.searchProvidersCount',
  '{matched} / {total} Providers',
  {
    matched: visibleProviderSections.value.length,
    total: providerSections.value.length,
  },
))
const quickSwitchStripCountLabel = computed(() => translateWithFallback(
  t,
  'claudeProfiles.quickSwitchStripCount',
  '{matched} 个候选',
  { matched: filteredProfiles.value.length },
))

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

const textFieldClass = 'editor-input w-full rounded-[20px] px-4 py-3 text-sm'
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
  form.effort_level = ''
  form.provider = ''
  form.provider_type = ''
  form.account = ''
  form.tagsInput = ''
  form.enabled = true
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
  const rawEffort = profile.effort_level || ''
  form.effort_level = (VALID_EFFORT_LEVELS as readonly string[]).includes(rawEffort) ? rawEffort : ''
  form.provider = profile.provider || ''
  form.provider_type = profile.provider_type || ''
  form.account = profile.account || ''
  form.tagsInput = (profile.tags || []).join(', ')
  form.enabled = profile.enabled !== false
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

const resolveSectionElement = (target: Element | ComponentPublicInstance | null): HTMLElement | null => {
  if (!target) return null
  if (target instanceof HTMLElement) return target

  if ('$el' in target) {
    const { $el } = target
    return $el instanceof HTMLElement ? $el : null
  }

  return null
}

const registerSectionRef = (sectionId: string, target: Element | ComponentPublicInstance | null) => {
  const resolvedElement = resolveSectionElement(target)

  if (resolvedElement) {
    sectionRefs.value[sectionId] = resolvedElement
    return
  }

  delete sectionRefs.value[sectionId]
}

const registerModalSectionRef = (sectionId: ClaudeProfileFormSectionId, target: Element | ComponentPublicInstance | null) => {
  const resolvedElement = resolveSectionElement(target)

  modalSectionRefs.value[sectionId] = resolvedElement
}

const teardownSectionObserver = () => {
  sectionObserver.value?.disconnect()
  sectionObserver.value = null
}

const setupSectionObserver = () => {
  teardownSectionObserver()

  if (!showNavigation.value || typeof IntersectionObserver === 'undefined') return

  const elements = visibleProviderSections.value
    .map(section => sectionRefs.value[section.id])
    .filter((element): element is HTMLElement => !!element)

  if (elements.length === 0) return

  sectionObserver.value = new IntersectionObserver((entries) => {
    const visibleEntries = entries
      .filter(entry => entry.isIntersecting)
      .sort((left, right) => left.boundingClientRect.top - right.boundingClientRect.top)

    if (visibleEntries.length > 0) {
      currentSectionId.value = visibleEntries[0]?.target.id ?? currentSectionId.value
      return
    }

    const nearestPassedEntry = entries
      .filter(entry => entry.boundingClientRect.top <= 180)
      .sort((left, right) => right.boundingClientRect.top - left.boundingClientRect.top)[0]

    if (nearestPassedEntry) {
      currentSectionId.value = nearestPassedEntry.target.id
    }
  }, {
    rootMargin: '-18% 0px -58% 0px',
    threshold: [0.1, 0.45, 0.75],
  })

  elements.forEach(element => sectionObserver.value?.observe(element))
}

const scrollToSection = (sectionId: string) => {
  currentSectionId.value = sectionId
  sectionRefs.value[sectionId]?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

const clearSearch = () => {
  searchQuery.value = ''
}

// Ctrl/Cmd+K 聚焦搜索框快捷键
const handleGlobalKeydown = (event: KeyboardEvent) => {
  if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
    event.preventDefault()
    // 聚焦搜索区域内的 input 元素
    const searchEl = document.querySelector('.claude-profiles-view__search-input-shell input') as HTMLInputElement | null
    searchEl?.focus()
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

watch(visibleProviderSections, async (sections) => {
  const validSectionIds = new Set(sections.map(section => section.id))
  Object.keys(sectionRefs.value).forEach((sectionId) => {
    if (!validSectionIds.has(sectionId)) {
      delete sectionRefs.value[sectionId]
    }
  })

  currentSectionId.value = sections.find(section => section.id === currentSectionId.value)?.id ?? sections[0]?.id ?? null

  await nextTick()
  setupSectionObserver()
}, { flush: 'post' })

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
  teardownSectionObserver()
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
  backdrop-filter: blur(20px) saturate(135%);
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

.claude-profiles-view {
  position: relative;
  min-height: 100%;
  overflow: hidden;
  padding: 1.5rem;
}

.claude-profiles-view::before,
.claude-profiles-view::after {
  content: '';
  position: absolute;
  inset: auto;
  pointer-events: none;
  filter: blur(64px);
  opacity: 0.9;
}

.claude-profiles-view::before {
  top: -4rem;
  right: -2rem;
  width: 18rem;
  height: 18rem;
  background: radial-gradient(circle, rgb(var(--color-accent-secondary-rgb) / 16%), transparent 70%);
}

.claude-profiles-view::after {
  bottom: 8rem;
  left: -4rem;
  width: 22rem;
  height: 22rem;
  background: radial-gradient(circle, rgb(var(--color-platform-claude-rgb) / 10%), transparent 72%);
}

.claude-profiles-view__shell,
.claude-profiles-view__main {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
}

.claude-profiles-view__shell {
  max-width: 1680px;
  margin: 0 auto;
  gap: 1.6rem;
}

.claude-profiles-view__search-rail {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  padding: 1rem 1.125rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 48%);
  border-radius: 1.5rem;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 78%), rgb(var(--color-bg-surface-rgb) / 72%));
  box-shadow:
    0 18px 34px rgb(8 10 20 / 8%),
    inset 0 1px 0 rgb(255 255 255 / 8%);
  backdrop-filter: blur(18px) saturate(135%);
}

.claude-profiles-view__search-grid {
  display: grid;
  gap: 0.875rem;
}

.claude-profiles-view__search-input-shell {
  min-width: 0;
}

.claude-profiles-view__search-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.625rem;
  align-items: center;
}

.claude-profiles-view__search-chip,
.claude-profiles-view__search-clear {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  min-height: 2.25rem;
  padding: 0.45rem 0.85rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 48%);
  background: rgb(var(--color-bg-elevated-rgb) / 64%);
  font-size: 0.78rem;
  line-height: 1rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.claude-profiles-view__search-clear {
  border-color: rgb(var(--color-accent-secondary-rgb) / 26%);
  background: rgb(var(--color-accent-secondary-rgb) / 10%);
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
  transition: background-color 0.2s ease, border-color 0.2s ease, transform 0.2s ease;
}

.claude-profiles-view__search-clear:hover {
  border-color: rgb(var(--color-accent-secondary-rgb) / 34%);
  background: rgb(var(--color-accent-secondary-rgb) / 14%);
  transform: translateY(-1px);
}

.claude-profiles-view__search-hint {
  font-size: 0.82rem;
  line-height: 1.4;
  color: var(--color-text-muted);
}

.claude-profiles-view__command-strip {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
  padding-top: 0.15rem;
}

.claude-profiles-view__command-strip-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.claude-profiles-view__command-strip-title {
  font-size: 0.72rem;
  line-height: 1rem;
  font-weight: 700;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.claude-profiles-view__command-strip-hint {
  margin-top: 0.18rem;
  font-size: 0.8rem;
  line-height: 1.3;
  color: var(--color-text-secondary);
}

.claude-profiles-view__command-strip-count {
  display: inline-flex;
  align-items: center;
  min-height: 1.9rem;
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 45%);
  background: rgb(var(--color-bg-elevated-rgb) / 68%);
  font-size: 0.76rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.claude-profiles-view__command-strip-scroll {
  display: flex;
  gap: 0.55rem;
  overflow-x: auto;
  padding-bottom: 0.15rem;
  scrollbar-width: thin;
}

.claude-profiles-view__command-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  flex: 0 0 auto;
  min-height: 2rem;
  max-width: 15rem;
  padding: 0.38rem 0.8rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 45%);
  border-radius: 9999px;
  font-size: 0.8rem;
  line-height: 1rem;
  font-weight: 600;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    transform 0.2s ease;
}

.claude-profiles-view__command-pill--idle {
  background: rgb(var(--color-bg-elevated-rgb) / 68%);
  color: var(--color-text-secondary);
}

.claude-profiles-view__command-pill--idle:hover {
  border-color: rgb(var(--color-border-default-rgb) / 75%);
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  color: var(--color-text-primary);
  transform: translateY(-1px);
}

.claude-profiles-view__command-pill--current {
  border-color: rgb(var(--color-accent-secondary-rgb) / 28%);
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
}

.claude-profiles-view__command-pill--disabled {
  cursor: not-allowed;
  opacity: 0.58;
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
  color: var(--color-text-muted);
}

.claude-profiles-view__command-pill-dot {
  width: 0.42rem;
  height: 0.42rem;
  flex-shrink: 0;
  border-radius: 9999px;
}

.claude-profiles-view__command-pill-dot--current {
  background: rgb(var(--color-accent-secondary-rgb) / 100%);
}

.claude-profiles-view__command-pill-dot--enabled {
  background: rgb(var(--color-success-rgb) / 100%);
}

.claude-profiles-view__command-pill-dot--disabled {
  background: rgb(var(--color-danger-rgb) / 100%);
}

.claude-profiles-view__breadcrumb,
.claude-profiles-view__header-button {
  display: flex;
  align-items: center;
}

.claude-profiles-view__breadcrumb {
  width: fit-content;
  gap: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-secondary);
}

.claude-profiles-view__breadcrumb-link {
  transition: color 0.2s ease;
}

.claude-profiles-view__breadcrumb-link:hover {
  color: var(--color-text-primary);
}

.claude-profiles-view__breadcrumb-current {
  color: var(--color-text-primary);
}

.claude-profiles-view__eyebrow {
  display: inline-flex;
  align-items: center;
  min-height: 1.75rem;
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-platform-claude-rgb) / 22%);
  background: rgb(var(--color-platform-claude-rgb) / 8%);
  color: rgb(var(--color-platform-claude-rgb));
  font-size: 0.72rem;
  line-height: 1rem;
  font-weight: 700;
  letter-spacing: 0.22em;
  text-transform: uppercase;
}

.claude-profiles-view__meta-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.75rem;
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 45%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  font-weight: 500;
}

.claude-profiles-view__header-button {
  gap: 0.5rem;
  min-height: 44px;
  border: 1px solid;
  border-radius: 1rem;
  padding: 0.625rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.claude-profiles-view__header-button--secondary {
  border-color: rgb(var(--color-border-default-rgb) / 60%);
  background: rgb(var(--color-bg-surface-rgb) / 75%);
  color: var(--color-text-secondary);
}

.claude-profiles-view__header-button--secondary:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-primary);
}

.claude-profiles-view__header-button--primary {
  border-color: rgb(var(--color-accent-secondary-rgb) / 35%);
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 14%), rgb(var(--color-accent-secondary-rgb) / 10%));
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
  box-shadow: 0 12px 24px rgb(var(--color-accent-secondary-rgb) / 12%);
}

.claude-profiles-view__header-button--primary:hover {
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 20%), rgb(var(--color-accent-secondary-rgb) / 14%));
}

.claude-profiles-view__header-button--primary:focus-visible {
  outline: 2px solid rgb(var(--color-accent-secondary-rgb) / 20%);
  outline-offset: 2px;
}

.claude-profiles-view__layout {
  display: grid;
  gap: 1.5rem;
}

.claude-profiles-view__main {
  min-width: 0;
  gap: 1.5rem;
}

.claude-profiles-view__sidebar {
  display: none;
}

.claude-profiles-view__mobile-nav {
  display: block;
}

@media (width >= 1024px) {
  .claude-profiles-view {
    padding: 2.5rem;
  }
}

@media (width >= 1280px) {
  .claude-profiles-view__layout--with-nav {
    grid-template-columns: 18rem minmax(0, 1fr);
  }

  .claude-profiles-view__sidebar {
    display: block;
  }

  .claude-profiles-view__mobile-nav {
    display: none;
  }
}

@media (width >= 960px) {
  .claude-profiles-view__search-grid {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
  }

  .claude-profiles-view__search-meta {
    justify-content: flex-end;
  }
}

@media (width < 768px) {
  .claude-profiles-view__meta-chip {
    display: none;
  }

  .claude-profiles-view__search-rail {
    padding: 0.9rem;
  }

  .claude-profiles-view__command-strip-head {
    align-items: flex-start;
  }
}

/* ── Overview 与列表的视觉分隔 ── */
.claude-profiles-view .page-header-card__body {
  padding-top: 1rem;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 18%);
}

/* ── 搜索高亮样式 ── */
.profile-search-highlight {
  background: rgb(var(--color-accent-secondary-rgb) / 18%);
  color: inherit;
  border-radius: 2px;
  padding: 0 1px;
}

/* ── 键盘快捷键提示 ── */
.claude-profiles-view__search-hint kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.5em;
  padding: 0.1em 0.35em;
  border: 1px solid rgb(var(--color-border-default-rgb) / 45%);
  border-radius: 4px;
  background: rgb(var(--color-bg-elevated-rgb) / 60%);
  font-family: var(--font-mono);
  font-size: 0.72em;
  line-height: 1.3;
  color: var(--color-text-muted);
  box-shadow: 0 1px 0 rgb(0 0 0 / 8%);
}

/* ── reduced motion 降级 ── */
@media (prefers-reduced-motion: reduce) {
  .profile-search-highlight {
    transition: none;
  }
}
</style>
