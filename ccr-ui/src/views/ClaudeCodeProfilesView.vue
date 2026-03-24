<template>
  <div class="claude-profiles-view">
    <div class="claude-profiles-view__shell">
      <header class="claude-profiles-view__header animate-slide-up">
        <div class="claude-profiles-view__hero">
          <div class="claude-profiles-view__breadcrumb">
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

          <div class="claude-profiles-view__hero-title-row">
            <div class="claude-profiles-view__hero-icon">
              <SIcon
                name="Layers"
                size="w-6 h-6"
              />
            </div>
            <div>
              <p class="claude-profiles-view__eyebrow">
                {{ $t('claudeProfiles.consoleEyebrow') }}
              </p>
              <h1 class="claude-profiles-view__title">
                {{ $t('claudeProfiles.title') }}
              </h1>
            </div>
          </div>

          <p class="claude-profiles-view__subtitle">
            {{ $t('claudeProfiles.subtitle') }}
          </p>
        </div>

        <div class="claude-profiles-view__header-actions">
          <RouterLink to="/claude-code">
            <button class="claude-profiles-view__header-button claude-profiles-view__header-button--secondary">
              <SIcon
                name="ArrowLeft"
                size="w-4 h-4"
              />
              {{ $t('claudeProfiles.back') }}
            </button>
          </RouterLink>

          <button
            type="button"
            class="claude-profiles-view__header-button claude-profiles-view__header-button--primary"
            @click="openAddForm()"
          >
            <SIcon
              name="Plus"
              size="w-4 h-4"
            />
            {{ $t('claudeProfiles.addProfile') }}
          </button>
        </div>
      </header>

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
            :sections="providerSections"
            :active-section-id="currentSectionId"
            @navigate="scrollToSection"
          />
        </aside>

        <main class="claude-profiles-view__main">
          <ClaudeProfilesProviderNav
            v-if="showNavigation"
            mobile
            :sections="providerSections"
            :active-section-id="currentSectionId"
            class="claude-profiles-view__mobile-nav"
            @navigate="scrollToSection"
          />

          <ClaudeProfilesOverview
            :current-profile-name="currentProfileName"
            :enabled-profiles-count="enabledProfilesCount"
            :profiles="profiles"
            :provider-sections-count="providerSections.length"
            :show-navigation="showNavigation"
            :total-profiles="profiles.length"
            @apply="handleApply"
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
                @click="loadProfiles()"
              >
                {{ $t('claudeProfiles.retry') }}
              </button>
            </div>
          </div>

          <div
            v-else-if="profiles.length === 0"
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

          <ClaudeProfilesSectionList
            v-else
            :provider-sections="providerSections"
            :register-section-ref="registerSectionRef"
            @apply="handleApply"
            @delete="handleDelete"
            @edit="openEditForm"
          />
        </main>
      </div>

      <BaseModal
        v-model="showForm"
        :description="modalDescription"
        :persistent="isSaving"
        :show-close="false"
        size="full"
        content-class="claude-profile-editor-modal !max-w-[1440px] !max-h-[92vh] rounded-[32px]"
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

        <div class="flex min-h-[620px] max-h-[calc(92vh-9rem)] flex-col overflow-hidden">
          <div
            ref="modalScrollRef"
            class="editor-scroll-area min-h-0 flex-1 overflow-y-auto pr-1"
            @scroll="syncActiveFormSection"
          >
            <div class="grid gap-5 xl:grid-cols-[320px_minmax(0,1fr)]">
              <ClaudeProfileEditorSidebar
                :active-form-section-id="activeFormSectionId"
                :enabled-badge-class="enabledBadgeClass"
                :form-enabled="form.enabled"
                :modal-preview-description="modalPreviewDescription"
                :modal-preview-title="modalPreviewTitle"
                :modal-section-items="modalSectionItems"
                :modal-status="modalStatus"
                :modal-status-class="modalStatusClass"
                :modal-summary-items="modalSummaryItems"
                :parsed-form-tags="parsedFormTags"
                @navigate="scrollToFormSection"
              />

              <ClaudeProfileEditorSections
                :editing-name="editingName"
                :form="form"
                :is-editing="isEditing"
                :modal-description="modalDescription"
                :modal-status="modalStatus"
                :modal-status-class="modalStatusClass"
                :monospace-field-class="monospaceFieldClass"
                :parsed-form-tags="parsedFormTags"
                :register-modal-section-ref="registerModalSectionRef"
                :save-error="saveError"
                :textarea-class="textareaClass"
                :text-field-class="textFieldClass"
                :update-form-field="updateFormField"
              />
            </div>
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
  listClaudeProfiles,
  updateClaudeProfile,
} from '@/api'
import ClaudeProfileEditorSections from '@/components/claude/ClaudeProfileEditorSections.vue'
import ClaudeProfileEditorSidebar from '@/components/claude/ClaudeProfileEditorSidebar.vue'
import ClaudeProfilesOverview from '@/components/claude/ClaudeProfilesOverview.vue'
import ClaudeProfilesProviderNav from '@/components/claude/ClaudeProfilesProviderNav.vue'
import ClaudeProfilesSectionList from '@/components/claude/ClaudeProfilesSectionList.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile, ClaudeProfileRequest, ClaudeProfilesResponse } from '@/types'
import type {
  ClaudeProfileEditorForm,
  ClaudeProfileEditorSectionItem,
  ClaudeProfileEditorSummaryItem,
  ClaudeProfileFormSectionId,
} from '@/types/claudeProfileEditor'
import { getErrorMessage } from '@/types/api'
import { createClaudeProfileSections } from '@/utils/claudeProfiles'
import { logger } from '@/utils/logger'
import { CLAUDE_PROFILE_FORM_SECTION_IDS } from '@/types/claudeProfileEditor'

const { t } = useI18n()

const loading = ref(true)
const loadError = ref<string | null>(null)
const profiles = ref<ClaudeProfile[]>([])
const currentProfile = ref<string | null>(null)
const showForm = ref(false)
const isEditing = ref(false)
const isSaving = ref(false)
const saveError = ref<string | null>(null)
const editingName = ref('')
const currentSectionId = ref<string | null>(null)
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
  base_url: '',
  auth_token: '',
  model: '',
  small_fast_model: '',
  provider: '',
  provider_type: '',
  account: '',
  tagsInput: '',
  enabled: true,
})

const currentProfileName = computed(() => currentProfile.value ?? profiles.value.find(profile => profile.is_current)?.name ?? null)
const enabledProfilesCount = computed(() => profiles.value.filter(profile => profile.enabled !== false).length)
const providerSections = computed(() => createClaudeProfileSections(profiles.value, t('claudeProfiles.providerUnset')))
const showNavigation = computed(() => !loading.value && !loadError.value && providerSections.value.length > 0)
const isEditingCurrent = computed(() => isEditing.value && editingName.value === currentProfileName.value)

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
const enabledBadgeClass = computed(() => (
  form.enabled ? 'editor-pill--success' : 'editor-pill--danger'
))

const textFieldClass = 'editor-input w-full rounded-[20px] px-4 py-3 text-sm'
const monospaceFieldClass = `${textFieldClass} editor-input--mono`
const textareaClass = `${textFieldClass} editor-input--textarea min-h-[116px] resize-y`

const displayFormValue = (value: string | null | undefined, fallback = t('claudeProfiles.notSet')): string => {
  const trimmed = value?.trim() ?? ''
  return trimmed || fallback
}

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
const modalPreviewTitle = computed(() => displayFormValue(form.name, modalTitle.value))
const modalPreviewDescription = computed(() => displayFormValue(form.description, t('claudeProfiles.descriptionFallback')))
const modalSummaryItems = computed<ClaudeProfileEditorSummaryItem[]>(() => [
  {
    label: t('claudeProfiles.providerLabel'),
    value: displayFormValue(form.provider, t('claudeProfiles.providerUnset')),
    icon: 'Globe',
  },
  {
    label: t('claudeProfiles.baseUrlLabel'),
    value: displayFormValue(form.base_url),
    icon: 'Webhook',
    mono: true,
  },
  {
    label: t('claudeProfiles.modelLabel'),
    value: displayFormValue(form.model),
    icon: 'Sparkles',
    mono: true,
  },
  {
    label: t('claudeProfiles.accountLabel'),
    value: displayFormValue(form.account),
    icon: 'User',
  },
  {
    label: t('claudeProfiles.authTokenLabel'),
    value: form.auth_token.trim() ? '********' : t('claudeProfiles.notSet'),
    icon: 'ShieldCheck',
    mono: true,
  },
])
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

  form[field] = String(value) as ClaudeProfileEditorForm[typeof field]
}

const buildRequest = (): ClaudeProfileRequest => ({
  name: form.name.trim(),
  description: normalizeOptional(form.description),
  base_url: normalizeOptional(form.base_url),
  auth_token: normalizeOptional(form.auth_token),
  model: normalizeOptional(form.model),
  small_fast_model: normalizeOptional(form.small_fast_model),
  provider: normalizeOptional(form.provider),
  provider_type: normalizeOptional(form.provider_type),
  account: normalizeOptional(form.account),
  tags: parseTags(form.tagsInput),
  enabled: form.enabled,
})

const resetForm = () => {
  form.name = ''
  form.description = ''
  form.base_url = ''
  form.auth_token = ''
  form.model = ''
  form.small_fast_model = ''
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

const openEditForm = (profile: ClaudeProfile) => {
  form.name = profile.name
  form.description = profile.description || ''
  form.base_url = profile.base_url || ''
  form.auth_token = profile.auth_token || ''
  form.model = profile.model || ''
  form.small_fast_model = profile.small_fast_model || ''
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

  const elements = providerSections.value
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

const loadProfiles = async () => {
  loading.value = true
  loadError.value = null

  try {
    const data = await listClaudeProfiles<ClaudeProfilesResponse>()
    profiles.value = data.profiles || []
    currentProfile.value = data.current_profile || null
  } catch (error) {
    logger.error('Failed to load Claude profiles:', error)
    profiles.value = []
    currentProfile.value = null
    loadError.value = getErrorMessage(error, t('claudeProfiles.loadFailed'))
  } finally {
    loading.value = false
  }
}

const handleSave = async () => {
  if (!form.name.trim()) return

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
    await loadProfiles()
  } catch (error) {
    logger.error('Failed to save Claude profile:', error)
    saveError.value = getErrorMessage(error, t('claudeProfiles.operationFailed'))
  } finally {
    isSaving.value = false
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('claudeProfiles.confirmDelete', { name }))) return

  try {
    await deleteClaudeProfile(name)
    await loadProfiles()
  } catch (error) {
    logger.error('Failed to delete Claude profile:', error)
    alert(getErrorMessage(error, t('claudeProfiles.deleteFailed')))
  }
}

const handleApply = async (name: string) => {
  if (!confirm(t('claudeProfiles.confirmApply', { name }))) return

  try {
    await applyClaudeProfile(name)
    await loadProfiles()
  } catch (error) {
    logger.error('Failed to apply Claude profile:', error)
    alert(getErrorMessage(error, t('claudeProfiles.applyFailed')))
  }
}

watch(providerSections, async (sections) => {
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

onMounted(loadProfiles)
onBeforeUnmount(teardownSectionObserver)
</script>

<style>
.claude-profile-editor-modal {
  --editor-shell-bg: linear-gradient(180deg, rgb(255 252 255 / 96%), rgb(255 244 249 / 92%));
  --editor-shell-border: rgb(var(--color-border-default-rgb) / 82%);
  --editor-shell-shadow: 0 28px 80px rgb(173 141 191 / 20%), 0 12px 32px rgb(104 70 123 / 10%);
  --editor-shell-highlight:
    radial-gradient(circle at top right, rgb(var(--color-accent-secondary-rgb) / 12%), transparent 40%),
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 10%), transparent 32%);
  --editor-panel-bg: rgb(255 251 253 / 82%);
  --editor-panel-muted-bg: linear-gradient(180deg, rgb(252 247 251 / 96%), rgb(247 239 248 / 90%));
  --editor-panel-head-bg: linear-gradient(180deg, rgb(255 252 254 / 98%), rgb(250 243 250 / 90%));
  --editor-input-bg: rgb(247 240 247 / 94%);
  --editor-input-bg-hover: rgb(253 247 252 / 98%);
  --editor-input-bg-focus: rgb(255 250 255 / 100%);
  --editor-input-border: rgb(var(--color-border-default-rgb) / 84%);
  --editor-input-border-strong: rgb(var(--color-accent-secondary-rgb) / 34%);
  --editor-hairline: rgb(var(--color-border-default-rgb) / 72%);
  --editor-hairline-soft: rgb(var(--color-border-default-rgb) / 46%);
  --editor-ink: rgb(var(--color-text-primary-rgb) / 96%);
  --editor-ink-muted: rgb(var(--color-text-secondary-rgb) / 90%);
  --editor-ink-soft: rgb(var(--color-text-muted-rgb) / 86%);
  --editor-placeholder: rgb(var(--color-text-muted-rgb) / 74%);
  --editor-panel-shadow: 0 20px 48px rgb(188 157 205 / 16%), inset 0 1px 0 rgb(255 255 255 / 64%);
  --editor-muted-shadow: 0 14px 34px rgb(188 157 205 / 12%), inset 0 1px 0 rgb(255 255 255 / 42%);
  --editor-ring: 0 0 0 3px rgb(var(--color-accent-secondary-rgb) / 14%);
  --editor-scrollbar-thumb: rgb(var(--color-accent-secondary-rgb) / 34%);
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
  --editor-shell-bg: linear-gradient(180deg, rgb(31 20 41 / 96%), rgb(24 15 33 / 94%));
  --editor-shell-border: rgb(123 98 149 / 42%);
  --editor-shell-shadow: 0 32px 90px rgb(8 4 12 / 58%), 0 18px 42px rgb(17 10 24 / 42%);
  --editor-shell-highlight:
    radial-gradient(circle at top right, rgb(var(--color-accent-secondary-rgb) / 14%), transparent 44%),
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 11%), transparent 34%);
  --editor-panel-bg: linear-gradient(180deg, rgb(46 31 60 / 86%), rgb(39 26 52 / 82%));
  --editor-panel-muted-bg: linear-gradient(180deg, rgb(58 41 73 / 90%), rgb(50 35 64 / 84%));
  --editor-panel-head-bg: linear-gradient(180deg, rgb(58 42 74 / 92%), rgb(47 33 62 / 82%));
  --editor-input-bg: rgb(69 51 86 / 88%);
  --editor-input-bg-hover: rgb(77 58 96 / 92%);
  --editor-input-bg-focus: rgb(84 64 103 / 96%);
  --editor-input-border: rgb(121 96 147 / 46%);
  --editor-input-border-strong: rgb(var(--color-accent-secondary-rgb) / 44%);
  --editor-hairline: rgb(123 98 149 / 42%);
  --editor-hairline-soft: rgb(123 98 149 / 28%);
  --editor-ink: rgb(253 242 248 / 98%);
  --editor-ink-muted: rgb(236 206 244 / 86%);
  --editor-ink-soft: rgb(203 169 221 / 78%);
  --editor-placeholder: rgb(196 165 214 / 70%);
  --editor-panel-shadow: 0 24px 60px rgb(6 3 10 / 42%), inset 0 1px 0 rgb(255 255 255 / 6%);
  --editor-muted-shadow: 0 16px 40px rgb(6 3 10 / 34%), inset 0 1px 0 rgb(255 255 255 / 5%);
  --editor-ring: 0 0 0 3px rgb(var(--color-accent-secondary-rgb) / 18%);
  --editor-scrollbar-thumb: rgb(var(--color-accent-secondary-rgb) / 48%);
  --editor-scrollbar-track: rgb(31 20 41 / 36%);
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
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: rgb(var(--color-accent-secondary-rgb) / 95%);
  box-shadow: 0 12px 24px rgb(var(--color-accent-secondary-rgb) / 12%);
}

.claude-profile-editor-modal .editor-shell-eyebrow {
  color: rgb(var(--color-accent-secondary-rgb) / 90%);
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

.claude-profile-editor-modal .editor-close-button:focus-visible,
.claude-profile-editor-modal .editor-button:focus-visible,
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
  box-shadow: var(--editor-muted-shadow);
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
  border-color: rgb(var(--color-accent-secondary-rgb) / 34%);
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 12%), rgb(var(--color-accent-secondary-rgb) / 8%));
  color: var(--editor-ink);
  box-shadow: 0 14px 32px rgb(var(--color-accent-secondary-rgb) / 12%);
}

.claude-profile-editor-modal .editor-nav-button--active .editor-nav-button__icon {
  border-color: rgb(var(--color-accent-secondary-rgb) / 30%);
  background: rgb(var(--color-accent-secondary-rgb) / 14%);
  color: rgb(var(--color-accent-secondary-rgb) / 98%);
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
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
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
  border-color: rgb(var(--color-accent-secondary-rgb) / 20%);
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
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
  border-color: rgb(var(--color-accent-secondary-rgb) / 26%);
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 14%), rgb(var(--color-accent-secondary-rgb) / 10%));
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
  box-shadow: 0 12px 24px rgb(var(--color-accent-secondary-rgb) / 12%);
}

.claude-profile-editor-modal .editor-button--primary:hover {
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 20%), rgb(var(--color-accent-secondary-rgb) / 14%));
}

.claude-profiles-view {
  position: relative;
  min-height: 100%;
  overflow: hidden;
  padding: 1.5rem;
}

.claude-profiles-view__shell,
.claude-profiles-view__hero,
.claude-profiles-view__main {
  display: flex;
  flex-direction: column;
}

.claude-profiles-view__shell {
  max-width: 1680px;
  margin: 0 auto;
  gap: 2rem;
}

.claude-profiles-view__header,
.claude-profiles-view__breadcrumb,
.claude-profiles-view__hero-title-row,
.claude-profiles-view__header-actions,
.claude-profiles-view__header-button {
  display: flex;
  align-items: center;
}

.claude-profiles-view__header {
  justify-content: space-between;
  gap: 1.25rem;
}

.claude-profiles-view__hero {
  gap: 0.75rem;
}

.claude-profiles-view__breadcrumb {
  gap: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--text-secondary);
}

.claude-profiles-view__breadcrumb-link {
  transition: color 0.2s ease;
}

.claude-profiles-view__breadcrumb-link:hover {
  color: var(--text-primary);
}

.claude-profiles-view__breadcrumb-current {
  color: var(--text-primary);
}

.claude-profiles-view__hero-title-row {
  flex-wrap: wrap;
  gap: 0.75rem;
}

.claude-profiles-view__hero-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3rem;
  height: 3rem;
  border-radius: 1rem;
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
  box-shadow: 0 10px 25px rgb(96 70 160 / 18%);
}

.claude-profiles-view__eyebrow {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  letter-spacing: 0.28em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.claude-profiles-view__title {
  margin-top: 0.25rem;
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 700;
  letter-spacing: -0.025em;
  color: var(--text-primary);
}

.claude-profiles-view__subtitle {
  max-width: 48rem;
  font-size: 0.875rem;
  line-height: 1.5rem;
  color: var(--text-secondary);
}

.claude-profiles-view__header-actions {
  flex-wrap: wrap;
  gap: 0.75rem;
}

.claude-profiles-view__header-button {
  gap: 0.5rem;
  min-height: 44px;
  border: 1px solid;
  border-radius: 1rem;
  padding: 0.625rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.claude-profiles-view__header-button--secondary {
  border-color: rgb(var(--color-border-default-rgb) / 60%);
  background: rgb(var(--color-bg-surface-rgb) / 75%);
  color: var(--text-secondary);
}

.claude-profiles-view__header-button--secondary:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.claude-profiles-view__header-button--primary {
  border-color: rgb(var(--color-accent-secondary-rgb) / 35%);
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: rgb(var(--color-accent-secondary-rgb) / 100%);
  font-weight: 500;
}

.claude-profiles-view__header-button--primary:hover {
  background: rgb(var(--color-accent-secondary-rgb) / 18%);
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
  .claude-profiles-view__header {
    align-items: flex-end;
    flex-direction: row;
  }

  .claude-profiles-view__header-actions {
    justify-content: flex-end;
  }

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

@media (width >= 1024px) {
  .claude-profiles-view__title {
    font-size: 2.25rem;
    line-height: 2.5rem;
  }

  .claude-profiles-view__subtitle {
    font-size: 1rem;
  }
}
</style>
