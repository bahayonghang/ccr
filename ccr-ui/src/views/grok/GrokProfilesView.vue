<template>
  <PageShell class="profiles-view grok-profiles-view">
    <template #header>
      <ProfilesHeader
        icon="Folders"
        back-to="/grok"
        :labels="{
          title: t('grok.profiles.title'),
          subtitle: t('grok.profiles.subtitle'),
          back: t('grok.profiles.back'),
          reload: t('grok.profiles.actions.reload'),
          export: t('grok.profiles.actions.exportSummary'),
          add: t('grok.profiles.actions.add'),
          overflow: t('grok.profiles.overflowMenu'),
        }"
        :palette="{
          label: t('grok.profiles.commandPaletteButton'),
          shortcut: `${quickSwitch.modifier.value}K`,
          title: t('grok.profiles.commandPaletteShortcut'),
        }"
        :loading="loading || refreshing || localOnly"
        :exporting="exporting"
        :palette-open="paletteOpen"
        @add="handleAdd"
        @export="exportSummary"
        @reload="refreshProfiles"
        @open-palette="paletteOpen = true"
      />
    </template>

    <template #subnav>
      <ModuleSubnav module="grok" />
    </template>

    <main class="cp-shell">
      <div class="cp-main">
        <section
          v-if="localOnly"
          class="grok-profiles-banner grok-profiles-banner--warning"
        >
          <SIcon
            name="Laptop"
            size="w-5 h-5"
          />
          <div>
            <strong>{{ t('grok.dashboard.localOnly.title') }}</strong>
            <p>{{ t('grok.dashboard.localOnly.description') }}</p>
            <span>{{ t('grok.dashboard.localOnly.environment', { env: localOnlyEnvType || t('grok.states.unknown') }) }}</span>
          </div>
        </section>

        <template v-else>
          <section
            v-if="activation !== 'inactive'"
            class="grok-profiles-banner"
            :class="activation === 'active' ? 'grok-profiles-banner--active' : 'grok-profiles-banner--danger'"
          >
            <SIcon
              :name="activation === 'active' ? 'CircleCheck' : 'AlertTriangle'"
              size="w-5 h-5"
            />
            <div>
              <strong>{{ activationBannerTitle }}</strong>
              <p>{{ activationBannerDescription }}</p>
            </div>
            <button
              v-if="canOff"
              type="button"
              class="grok-profiles-banner__action"
              :disabled="rowsDisabled"
              @click="handleOff"
            >
              <SIcon
                name="Power"
                size="w-4 h-4"
              />
              {{ t('grok.profiles.actions.off') }}
            </button>
          </section>

          <section
            v-if="recovery"
            class="grok-profiles-banner grok-profiles-banner--danger"
            data-testid="rename-recovery"
          >
            <SIcon
              name="RotateCcw"
              size="w-5 h-5"
            />
            <div>
              <strong>{{ t(`grok.profiles.renameRecovery.${recovery.status}.title`) }}</strong>
              <p>{{ recovery.message }}</p>
            </div>
            <button
              type="button"
              class="grok-profiles-banner__action"
              data-testid="rename-recovery-action"
              :disabled="rowsDisabled"
              @click="runRecovery"
            >
              {{ t(`grok.profiles.renameRecovery.${recovery.status}.action`) }}
            </button>
          </section>

          <section
            v-if="unsafeDeleteRecovery"
            class="grok-profiles-banner grok-profiles-banner--danger"
            data-testid="unsafe-delete-recovery"
            role="alert"
          >
            <SIcon
              name="ShieldAlert"
              size="w-5 h-5"
            />
            <div>
              <strong>{{ t('grok.profiles.unsafeDelete.title', { name: unsafeDeleteRecovery.name }) }}</strong>
              <p>{{ unsafeDeleteRecovery.message }}</p>
              <span>{{ t('grok.profiles.unsafeDelete.manualRecovery', { path: '~/.grok/config.toml' }) }}</span>
            </div>
          </section>

          <ProfilesStatStrip
            :current="currentProfile"
            :total="profiles.length"
            :labels="{
              current: t('grok.profiles.statStrip.current'),
              notSet: t('grok.states.notSet'),
              currentHint: t('grok.profiles.statStrip.currentHint'),
              total: t('grok.profiles.statStrip.total'),
              totalHint: t('grok.profiles.statStrip.totalHint', { enabled: enabledCount, disabled: profiles.length - enabledCount }),
            }"
            :secondary="{
              icon: 'KeyRound',
              title: t('grok.profiles.statStrip.authMode'),
              value: currentAuthModeLabel,
              hint: t('grok.profiles.statStrip.authModeHint'),
              mono: false,
            }"
            :health="healthSlot"
            @health-click="focusInspector"
          />

          <ProfilesQuickRail
            :profiles="profiles"
            :current-name="currentProfile"
            i18n-prefix="grok.profiles"
            :disabled="rowsDisabled"
            :busy-name="pendingAction?.kind === 'apply' ? pendingAction.name : null"
            :quick-switch="quickSwitch"
            :more-count="quickRailMoreCount"
            @apply="handleApply"
            @more="paletteOpen = true"
          />

          <ProfilesToolbar
            ref="toolbarRef"
            i18n-prefix="grok.profiles.toolbar"
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
              {{ t('grok.profiles.loadFailedTitle') }}
            </div>
            <div class="cp-state__hint">
              {{ loadError }}
            </div>
            <button
              type="button"
              class="cp-state__btn"
              @click="refreshProfiles"
            >
              {{ t('grok.profiles.actions.retry') }}
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
              {{ t('grok.profiles.refreshFailedTitle') }}
            </div>
            <div class="cp-state__hint">
              {{ refreshError }}
            </div>
            <button
              type="button"
              class="cp-state__btn"
              :disabled="refreshing"
              @click="refreshProfiles"
            >
              {{ t('grok.profiles.actions.retry') }}
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
              {{ t('grok.profiles.empty.title') }}
            </div>
            <div class="cp-state__hint">
              {{ t('grok.profiles.empty.hint') }}
            </div>
            <button
              type="button"
              class="cp-state__btn cp-state__btn--primary"
              @click="handleAdd"
            >
              <SIcon
                name="Plus"
                size="w-4 h-4"
              />{{ t('grok.profiles.actions.add') }}
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
              {{ t('grok.profiles.empty.noResults', { query }) }}
            </div>
            <button
              type="button"
              class="cp-state__btn"
              @click="resetFilters"
            >
              {{ t('grok.profiles.empty.clearFilters') }}
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
                  <span>{{ t('grok.profiles.fields.name') }}</span>
                  <span>{{ t('grok.profiles.fields.description') }}</span>
                  <span>{{ t('grok.profiles.fields.baseUrl') }}</span>
                  <span>{{ t('grok.profiles.fields.model') }}</span>
                  <span>{{ t('grok.profiles.fields.authMode') }}</span>
                  <span>{{ t('grok.profiles.fields.tags') }}</span>
                  <span class="cp-list-head__right">{{ t('grok.profiles.toolbar.actionsLabel') }}</span>
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
                <GrokProfileCard
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
                  @toggle="handleToggle"
                />
              </div>
            </ProfilesSection>
          </div>
        </template>
      </div>

      <ProfilesInspector
        v-if="!localOnly"
        ref="inspectorRef"
        :profiles="profiles"
        :preview-profile="previewProfile"
        :current-profile="currentProfileRecord"
        i18n-prefix="grok.profiles.inspector"
        :descriptor="inspectorDescriptor"
        :session-write-at="isPreviewingCurrent ? lastWriteHint : null"
        :selected-tag="tagFilter"
        @edit="handleEdit"
        @locate="locateProfile"
        @tag-select="selectInspectorTag"
      />
    </main>

    <ProfilesCommandPalette
      :open="paletteOpen"
      :profiles="profiles"
      :descriptor="paletteDescriptor"
      :actions="paletteActions"
      i18n-prefix="grok.profiles.commandPalette"
      @update:open="paletteOpen = $event"
      @apply="handleApply"
    />

    <GrokProfileEditorModal
      v-model="showForm"
      :editing-name="editingName"
      :saving="saving"
      :error="saveError"
      :form="form"
      :update-field="updateFormField"
      :base-url-display="editingProfile?.base_url_display"
      :has-existing-base-url="editingProfile?.has_base_url"
      :current-auth-mode="editingProfile?.auth_mode"
      :current-env-key="editingProfile?.env_key"
      @save="handleSave"
    />

    <ConfirmModal
      v-model:is-open="showConfirmModal"
      :type="confirmDialog.type"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :confirm-text="confirmDialog.confirmText"
      :cancel-text="t('common.cancel')"
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
  </PageShell>
</template>

<script setup lang="ts">
import { computed, nextTick, onActivated, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import '@/styles/components/profiles-page.css'
import { grokApi } from '@/api'
import { getCurrentEnvironment } from '@/api/runtime/environment'
import GrokProfileCard from '@/components/grok/GrokProfileCard.vue'
import GrokProfileEditorModal from '@/components/grok/GrokProfileEditorModal.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageShell from '@/components/ui/PageShell.vue'
import ProfileDiffRows from '@/components/profiles/ProfileDiffRows.vue'
import ProfileListRow from '@/components/profiles/ProfileListRow.vue'
import ProfilesCommandPalette, {
  type ProfilesCommandPaletteAction,
  type ProfilesCommandPaletteDescriptor,
} from '@/components/profiles/ProfilesCommandPalette.vue'
import ProfilesHeader from '@/components/profiles/ProfilesHeader.vue'
import ProfilesInspector from '@/components/profiles/ProfilesInspector.vue'
import ProfilesQuickRail from '@/components/profiles/ProfilesQuickRail.vue'
import ProfilesSection from '@/components/profiles/ProfilesSection.vue'
import ProfilesStatStrip, { type ProfilesStatStripHealth } from '@/components/profiles/ProfilesStatStrip.vue'
import ProfilesToolbar, { type ProfilesViewMode } from '@/components/profiles/ProfilesToolbar.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useConfirmAction } from '@/composables/useConfirmAction'
import {
  useGrokProfilesFilter,
  type GrokProfilesSortBy,
  type GrokProfilesStatusFilter,
} from '@/composables/useGrokProfilesFilter'
import { useProfilesHotkeys } from '@/composables/useProfilesHotkeys'
import { useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import { REFRESH_TTL_MS } from '@/config/constants'
import { useUIStore } from '@/stores/ui'
import type {
  GrokActivationDto,
  GrokProfileActionResponse,
  GrokProfileDto,
} from '@/types'
import { getErrorMessage } from '@/types/api'
import { downloadTextFile } from '@/utils/download'
import {
  buildGrokCreateRequest,
  buildGrokPatch,
  createEmptyGrokForm,
  fillGrokForm,
  type GrokProfileDirtyField,
  type GrokProfileEditorForm,
} from '@/utils/grokProfileEditor'
import {
  createGrokDiffFields,
  createGrokInspectorDescriptor,
  createGrokRowDescriptor,
  grokAuthModeLabel,
} from '@/utils/grokProfiles'
import { buildProfileDiff, type ProfileDiffRow } from '@/utils/profileDiff'

defineOptions({ name: 'GrokProfilesView' })

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(false)
const refreshing = ref(false)
const saving = ref(false)
const exporting = ref(false)
const loadError = ref<string | null>(null)
const refreshError = ref<string | null>(null)
const saveError = ref<string | null>(null)
const localOnly = ref(false)
const localOnlyEnvType = ref<string | null>(null)
const profiles = ref<GrokProfileDto[]>([])
const profileNamesReady = ref(false)
const currentProfile = ref<string | null>(null)
const activation = ref<GrokActivationDto>('inactive')
const activationName = ref<string | null>(null)
const lastLoadedAt = ref(0)
const lastWriteHint = ref<string | null>(null)

const showForm = ref(false)
const editingName = ref<string | null>(null)
const editingProfile = ref<GrokProfileDto | null>(null)
const dirtyFields = new Set<GrokProfileDirtyField>()
const form = reactive(createEmptyGrokForm())

const paletteOpen = ref(false)
const query = ref('')
const statusFilter = ref<GrokProfilesStatusFilter>('all')
const tagFilter = ref<string | null>(null)
const sortBy = ref<GrokProfilesSortBy>('recent')
const viewMode = ref<ProfilesViewMode>('card')

const confirmDiffRows = ref<ProfileDiffRow[]>([])
const pendingAction = ref<{ name: string, kind: 'apply' | 'delete' } | null>(null)
const recovery = ref<{
  status: 'rename_apply_failed' | 'rename_cleanup_failed'
  oldName: string
  newName: string
  message: string
} | null>(null)
const unsafeDeleteRecovery = ref<{ name: string, message: string } | null>(null)
const {
  isOpen: showConfirmModal,
  dialog: confirmDialog,
  busy: confirmActionBusy,
  openConfirmDialog,
  executeConfirmedAction,
} = useConfirmAction()

const toolbarRef = ref<{ focusSearch: () => void } | null>(null)
const inspectorRef = ref<{ $el?: Element } | null>(null)
const listRef = ref<HTMLElement | null>(null)

const rowDescriptor = computed(() => createGrokRowDescriptor(t))
const inspectorDescriptor = createGrokInspectorDescriptor(t)
const insights = inspectorDescriptor.useInsights(profiles)

const quickSwitch = useProfilesQuickSwitch({
  platform: 'grok',
  getProfileNames: () => profileNamesReady.value ? profiles.value.map(profile => profile.name) : null,
  onPinLimit: () => uiStore.showWarning(t('grok.profiles.pinLimitReached')),
})

const { allTags, filtered, enabledList, disabledList } = useGrokProfilesFilter({
  profiles,
  currentProfile,
  query,
  statusFilter,
  tagFilter,
  sortBy,
})

const enabledCount = computed(() => profiles.value.filter(profile => profile.enabled).length)
const currentProfileRecord = computed(() => (
  profiles.value.find(profile => profile.name === currentProfile.value) ?? null
))
const currentAuthModeLabel = computed(() => (
  currentProfileRecord.value
    ? grokAuthModeLabel(t, currentProfileRecord.value.auth_mode)
    : t('grok.states.notSet')
))
const canOff = computed(() => activation.value === 'active' || activation.value === 'drifted')
const rowsDisabled = computed(() => (
  localOnly.value || loading.value || refreshing.value || saving.value || confirmActionBusy.value
))

const activationBannerTitle = computed(() => t(
  `grok.profiles.driftBanner.${activation.value}.title`,
  { name: activationName.value ?? t('grok.states.unknown') },
))
const activationBannerDescription = computed(() => t(
  `grok.profiles.driftBanner.${activation.value}.description`,
  { name: activationName.value ?? t('grok.states.unknown') },
))

const healthSlot = computed<ProfilesStatStripHealth>(() => ({
  title: t('grok.profiles.statStrip.healthTitle'),
  value: `${enabledCount.value}/${profiles.value.length}`,
  hint: t('grok.profiles.statStrip.healthSummary', {
    enabled: enabledCount.value,
    total: profiles.value.length,
    issues: insights.totalIssueCount.value,
  }),
  warn: insights.totalIssueCount.value > 0,
}))

const listSections = computed(() => [
  { id: 'enabled', title: t('grok.profiles.groups.enabled'), profiles: enabledList.value },
  { id: 'disabled', title: t('grok.profiles.groups.disabled'), profiles: disabledList.value },
].filter(section => section.profiles.length > 0))

const quickRailMoreCount = computed(() => {
  const shown = Math.min(quickSwitch.pinned.value.length + quickSwitch.recentNotPinned.value.length, 8)
  return Math.max(0, enabledCount.value - shown)
})

const hoveredName = ref<string | null>(null)
const focusedName = ref<string | null>(null)
const previewProfile = computed(() => {
  const name = hoveredName.value ?? focusedName.value
  return profiles.value.find(profile => profile.name === name) ?? currentProfileRecord.value
})
const isPreviewingCurrent = computed(() => (
  Boolean(previewProfile.value && previewProfile.value.name === currentProfile.value)
))

const rowInteraction = (name: string): Record<string, unknown> => ({
  'data-profile-name': name,
  onMouseenter: () => { hoveredName.value = name },
  onMouseleave: () => { if (hoveredName.value === name) hoveredName.value = null },
  onFocusin: () => { focusedName.value = name },
  onFocusout: (event: FocusEvent) => {
    const container = event.currentTarget as HTMLElement | null
    if ((event.relatedTarget as Node | null) && container?.contains(event.relatedTarget as Node)) return
    if (focusedName.value === name) focusedName.value = null
  },
})

const busyActionFor = (name: string): 'apply' | 'delete' | null => {
  if (!confirmActionBusy.value || pendingAction.value?.name !== name) return null
  return pendingAction.value.kind
}

const focusInspector = () => {
  if (inspectorRef.value?.$el instanceof HTMLElement) {
    inspectorRef.value.$el.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

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

const selectInspectorTag = (tag: string) => {
  tagFilter.value = tagFilter.value === tag ? null : tag
}

const markWrite = () => {
  lastWriteHint.value = new Date().toLocaleTimeString()
}

const setLocalOnly = (envType: string) => {
  localOnly.value = true
  localOnlyEnvType.value = envType
  profiles.value = []
  profileNamesReady.value = false
  currentProfile.value = null
  activation.value = 'inactive'
  activationName.value = null
  lastLoadedAt.value = 0
  unsafeDeleteRecovery.value = null
  paletteOpen.value = false
  showForm.value = false
}

const actionUnsupported = (response: GrokProfileActionResponse): boolean => {
  if (response.status !== 'unsupported_environment') return false
  setLocalOnly(response.env_type)
  return true
}

const loadProfiles = async (options: { preserveData?: boolean } = {}) => {
  const preserveData = options.preserveData === true && profiles.value.length > 0
  if (preserveData) {
    refreshing.value = true
    refreshError.value = null
  } else {
    loading.value = true
    profileNamesReady.value = false
    loadError.value = null
  }

  try {
    const environment = await getCurrentEnvironment()
    if (environment.env_type !== 'local') {
      setLocalOnly(environment.env_type)
      return
    }
    localOnly.value = false
    localOnlyEnvType.value = null

    const response = await grokApi.listGrokProfiles()
    if (response.status === 'unsupported_environment') {
      setLocalOnly(response.env_type)
      return
    }

    profiles.value = response.profiles
    profileNamesReady.value = true
    currentProfile.value = response.current_profile
    activation.value = response.activation
    activationName.value = response.activation_name
    lastLoadedAt.value = Date.now()
    loadError.value = null
    refreshError.value = null
  } catch (error) {
    const message = getErrorMessage(error, t('grok.profiles.messages.loadFailed'))
    if (preserveData) refreshError.value = message
    else {
      profiles.value = []
      loadError.value = message
      uiStore.showError(message)
    }
  } finally {
    loading.value = false
    refreshing.value = false
  }
}

const refreshProfiles = async () => loadProfiles({ preserveData: true })

const ensureLoaded = async () => {
  if (loading.value || refreshing.value) return
  if (lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) return
  await refreshProfiles()
}

const resetForm = () => {
  Object.assign(form, createEmptyGrokForm())
  dirtyFields.clear()
  saveError.value = null
  editingName.value = null
  editingProfile.value = null
}

const updateFormField = (field: keyof GrokProfileEditorForm, value: string | boolean) => {
  if (typeof form[field] === 'boolean') form[field] = Boolean(value) as never
  else form[field] = String(value) as never
  dirtyFields.add(field)
}

const handleAdd = () => {
  if (localOnly.value) return
  resetForm()
  showForm.value = true
}

const handleEdit = (name: string) => {
  if (localOnly.value) return
  const profile = profiles.value.find(item => item.name === name)
  if (!profile) return
  resetForm()
  editingName.value = name
  editingProfile.value = profile
  Object.assign(form, fillGrokForm(profile))
  dirtyFields.clear()
  showForm.value = true
}

const handleSave = async () => {
  const previousName = editingName.value
  let partialRenameMessage: string | null = null
  saveError.value = null
  saving.value = true
  try {
    const response = previousName
      ? await grokApi.updateGrokProfile(previousName, buildGrokPatch(form, dirtyFields))
      : await grokApi.addGrokProfile(buildGrokCreateRequest(form))
    if (actionUnsupported(response)) return

    if (response.status === 'rename_apply_failed' || response.status === 'rename_cleanup_failed') {
      partialRenameMessage = response.message
      recovery.value = {
        status: response.status,
        oldName: response.old_name,
        newName: response.new_name,
        message: response.message,
      }
      if (response.status === 'rename_cleanup_failed') {
        quickSwitch.renamePinned(response.old_name, response.new_name)
      }
    } else if (response.status === 'renamed') {
      quickSwitch.renamePinned(response.old_name, response.new_name)
      recovery.value = null
    } else if (response.status !== 'created' && response.status !== 'updated') {
      throw new Error(t('grok.profiles.messages.unexpectedResponse'))
    }

    showForm.value = false
    markWrite()
    await loadProfiles({ preserveData: true })
    if (partialRenameMessage) {
      uiStore.showWarning(partialRenameMessage)
    } else {
      uiStore.showSuccess(previousName
        ? t('grok.profiles.messages.updateSuccess')
        : t('grok.profiles.messages.createSuccess'))
    }
  } catch (error) {
    saveError.value = getErrorMessage(error, t('grok.profiles.messages.saveFailed'))
  } finally {
    saving.value = false
  }
}

const handleApply = (name: string) => {
  if (localOnly.value) return
  const target = profiles.value.find(profile => profile.name === name)
  if (!target || !target.enabled || target.name === currentProfile.value) return
  confirmDiffRows.value = buildProfileDiff(currentProfileRecord.value, target, createGrokDiffFields(t))
  pendingAction.value = { name, kind: 'apply' }
  openConfirmDialog({
    title: t('grok.profiles.confirm.applyTitle'),
    message: t('grok.profiles.confirm.applyMessage', { name }),
    confirmText: t('grok.profiles.actions.apply'),
    type: 'warning',
    action: async () => {
      try {
        const response = await grokApi.applyGrokProfile(name)
        if (actionUnsupported(response)) return
        if (response.status !== 'applied') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
        quickSwitch.recordUse(name)
        markWrite()
        await loadProfiles({ preserveData: true })
        uiStore.showSuccess(t('grok.profiles.messages.applySuccess', { name }))
      } catch (error) {
        uiStore.showError(getErrorMessage(error, t('grok.profiles.messages.applyFailed')))
      } finally {
        pendingAction.value = null
        confirmDiffRows.value = []
      }
    },
  })
}

const handleOff = () => {
  if (localOnly.value) return
  confirmDiffRows.value = []
  openConfirmDialog({
    title: t('grok.profiles.confirm.offTitle'),
    message: activation.value === 'drifted'
      ? t('grok.profiles.confirm.offDriftedMessage')
      : t('grok.profiles.confirm.offMessage'),
    confirmText: t('grok.profiles.actions.off'),
    type: 'warning',
    action: async () => {
      try {
        const response = await grokApi.grokProfileOff()
        if (actionUnsupported(response)) return
        if (response.status !== 'off') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
        markWrite()
        await loadProfiles({ preserveData: true })
        uiStore.showSuccess(t('grok.profiles.messages.offSuccess'))
      } catch (error) {
        uiStore.showError(getErrorMessage(error, t('grok.profiles.messages.offFailed')))
      }
    },
  })
}

const deleteProfile = async (name: string, force = false) => {
  const response = await grokApi.deleteGrokProfile(name, { force })
  if (actionUnsupported(response)) return
  if (response.status === 'deleted') {
    unsafeDeleteRecovery.value = null
    markWrite()
    await loadProfiles({ preserveData: true })
    uiStore.showSuccess(t('grok.profiles.messages.deleteSuccess', { name }))
    return
  }
  if (response.status !== 'blocked') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
  if (response.reason === 'unsafe_missing_entry_state') {
    unsafeDeleteRecovery.value = { name, message: response.message }
    return
  }
  if (force) throw new Error(response.message)
  unsafeDeleteRecovery.value = null
  window.setTimeout(() => {
    openConfirmDialog({
      title: t('grok.profiles.confirm.forceDeleteTitle'),
      message: t('grok.profiles.confirm.forceDeleteMessage', { name }),
      confirmText: t('grok.profiles.confirm.forceDeleteAction'),
      type: 'danger',
      action: async () => {
        try {
          await deleteProfile(name, true)
        } catch (error) {
          uiStore.showError(getErrorMessage(error, t('grok.profiles.messages.deleteFailed')))
        }
      },
    })
  }, 0)
}

const handleDelete = (name: string) => {
  if (localOnly.value) return
  unsafeDeleteRecovery.value = null
  pendingAction.value = { name, kind: 'delete' }
  confirmDiffRows.value = []
  openConfirmDialog({
    title: t('grok.profiles.confirm.deleteTitle'),
    message: t('grok.profiles.confirm.deleteMessage', { name }),
    confirmText: t('grok.profiles.actions.delete'),
    type: 'danger',
    action: async () => {
      try {
        await deleteProfile(name)
      } catch (error) {
        uiStore.showError(getErrorMessage(error, t('grok.profiles.messages.deleteFailed')))
      } finally {
        pendingAction.value = null
      }
    },
  })
}

const handleToggle = (name: string, enabled: boolean) => {
  if (localOnly.value) return
  openConfirmDialog({
    title: t(enabled ? 'grok.profiles.confirm.enableTitle' : 'grok.profiles.confirm.disableTitle'),
    message: t(enabled ? 'grok.profiles.confirm.enableMessage' : 'grok.profiles.confirm.disableMessage', { name }),
    confirmText: t(enabled ? 'grok.profiles.actions.enable' : 'grok.profiles.actions.disable'),
    type: enabled ? 'info' : 'warning',
    action: async () => {
      try {
        const response = await grokApi.updateGrokProfile(name, { enabled })
        if (actionUnsupported(response)) return
        if (response.status !== 'updated') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
        markWrite()
        await loadProfiles({ preserveData: true })
      } catch (error) {
        uiStore.showError(getErrorMessage(error, t('grok.profiles.messages.saveFailed')))
      }
    },
  })
}

const runRecovery = async () => {
  const pending = recovery.value
  if (!pending) return
  try {
    if (pending.status === 'rename_apply_failed') {
      const response = await grokApi.applyGrokProfile(pending.newName)
      if (actionUnsupported(response)) return
      if (response.status !== 'applied') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
      quickSwitch.renamePinned(pending.oldName, pending.newName)
      quickSwitch.recordUse(pending.newName)
    } else {
      const response = await grokApi.deleteGrokProfile(pending.oldName)
      if (actionUnsupported(response)) return
      if (response.status !== 'deleted') throw new Error(t('grok.profiles.messages.deleteFailed'))
    }
    recovery.value = null
    markWrite()
    await loadProfiles({ preserveData: true })
    uiStore.showSuccess(t('grok.profiles.messages.recoverySuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error, t('grok.profiles.messages.recoveryFailed')))
  }
}

const exportSummary = () => {
  if (localOnly.value) return
  exporting.value = true
  try {
    downloadTextFile(
      'grok-profiles-summary.json',
      `${JSON.stringify({ activation: activation.value, current_profile: currentProfile.value, profiles: profiles.value }, null, 2)}\n`,
      'application/json;charset=utf-8',
    )
    uiStore.showSuccess(t('grok.profiles.messages.exportSuccess'))
  } finally {
    exporting.value = false
  }
}

const resetFilters = () => {
  query.value = ''
  statusFilter.value = 'all'
  tagFilter.value = null
}

const paletteDescriptor: ProfilesCommandPaletteDescriptor<GrokProfileDto> = {
  isEnabled: profile => profile.enabled,
  hint: profile => profile.description || profile.base_url_display || undefined,
}

const paletteActions = computed<ProfilesCommandPaletteAction[]>(() => {
  if (localOnly.value) return []
  const actions: ProfilesCommandPaletteAction[] = [
    { id: '__add', icon: 'Plus', labelKey: 'grok.profiles.commandPalette.actionAdd', handler: handleAdd },
    { id: '__reload', icon: 'RefreshCw', labelKey: 'grok.profiles.commandPalette.actionReload', handler: () => { void refreshProfiles() } },
  ]
  if (canOff.value) {
    actions.push({ id: '__off', icon: 'Power', labelKey: 'grok.profiles.commandPalette.actionOff', handler: handleOff })
  }
  return actions
})

watch(showForm, (open) => {
  if (!open) resetForm()
})

watch([allTags, tagFilter], ([tags, tag]) => {
  if (tag && !tags.includes(tag)) tagFilter.value = null
})

watch(profiles, (list) => {
  const names = new Set(list.map(profile => profile.name))
  if (hoveredName.value && !names.has(hoveredName.value)) hoveredName.value = null
  if (focusedName.value && !names.has(focusedName.value)) focusedName.value = null
})

watch(showConfirmModal, (open) => {
  if (open || confirmActionBusy.value) return
  pendingAction.value = null
  confirmDiffRows.value = []
})

useProfilesHotkeys({
  paletteOpen,
  focusSearch: () => toolbarRef.value?.focusSearch(),
  getStableTargets: () => quickSwitch.stableTargets.value,
  onApply: handleApply,
})

onMounted(() => { void loadProfiles() })
onActivated(() => { void ensureLoaded() })
</script>

<style scoped>
.grok-profiles-view {
  --cp-icon-color: var(--color-platform-grok);
  --cp-icon-soft: rgb(var(--color-platform-grok-rgb) / 14%);
  --cp-icon-line: rgb(var(--color-platform-grok-rgb) / 35%);
}

.grok-profiles-banner {
  display: flex;
  align-items: center;
  gap: 0.875rem;
  margin-bottom: 0.875rem;
  padding: 0.875rem 1rem;
  color: var(--cp-ink-1);
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  border-left: 3px solid var(--cp-good);
  border-radius: var(--radius-md);
}

.grok-profiles-banner--warning { border-left-color: var(--cp-warn); }
.grok-profiles-banner--danger { border-left-color: var(--cp-danger); }
.grok-profiles-banner--active { border-left-color: var(--cp-good); }

.grok-profiles-banner > div {
  min-width: 0;
  flex: 1;
}

.grok-profiles-banner strong {
  color: var(--cp-ink-0);
  font-size: 0.875rem;
}

.grok-profiles-banner p {
  margin: 0.25rem 0 0;
  color: var(--cp-ink-2);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.grok-profiles-banner span {
  color: var(--cp-ink-3);
  font-size: 0.75rem;
}

.grok-profiles-banner__action {
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

.grok-profiles-banner__action:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

@media (width <= 720px) {
  .grok-profiles-banner {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .grok-profiles-banner__action {
    width: 100%;
    justify-content: center;
  }
}
</style>
