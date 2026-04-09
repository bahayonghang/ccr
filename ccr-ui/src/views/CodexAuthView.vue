<template>
  <div class="codex-auth-view">
    <div class="codex-auth-view__container">
      <div class="codex-auth-view__stack">
        <ModuleSubnav module="codex" />

        <main class="codex-auth-view__main">
          <!-- Header Section -->
          <div class="codex-auth-view__header">
            <div class="codex-auth-view__title-group">
              <div class="codex-auth-view__title-icon-shell">
                <SIcon
                  name="KeyRound"
                  size="w-6 h-6"
                  class="codex-auth-view__title-icon"
                />
              </div>
              <div>
                <h1 class="codex-auth-view__title">
                  {{ $t('codex.auth.title') }}
                </h1>
                <p class="codex-auth-view__subtitle">
                  {{ $t('codex.auth.subtitle') }}
                </p>
              </div>
            </div>

            <div class="codex-auth-view__actions">
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
                  {{ $t('codex.auth.backToCodex') }}
                </Button>
              </RouterLink>

              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                :disabled="!canSave"
                @click="handleSave"
              >
                <template #leading>
                  <SIcon
                    name="Save"
                    size="w-4 h-4"
                  />
                </template>
                {{ $t('codex.auth.saveAccount') }}
              </Button>
            </div>
          </div>

          <!-- Status Cards -->
          <div class="codex-auth-view__status-grid">
            <!-- Login State -->
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :gradient-border="true"
              :glow-color="loginStateColor"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell"
                  :class="loginStateIconClass"
                >
                  <SIcon
                    :name="loginStateIcon"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ $t('codex.auth.status.loginState') }}
                  </p>
                  <p class="codex-auth-view__status-value codex-auth-view__status-value--truncate">
                    {{ loginStateText }}
                  </p>
                </div>
              </div>
            </Card>

            <!-- Total Accounts -->
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :interactive="true"
              glow-color="primary"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div class="codex-auth-view__status-icon-shell codex-auth-view__status-icon-shell--info">
                  <SIcon
                    name="Users"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ $t('codex.auth.status.totalAccounts') }}
                  </p>
                  <p class="codex-auth-view__status-value">
                    {{ accounts.length }}
                  </p>
                </div>
              </div>
            </Card>

            <!-- Current Account -->
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :interactive="true"
              :glow-color="currentAccount ? 'success' : 'secondary'"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell"
                  :class="currentAccount ? 'bg-emerald-500/10 text-emerald-500' : 'bg-gray-500/10 text-text-muted'"
                >
                  <SIcon
                    name="UserCheck"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ $t('codex.auth.status.currentAccount') }}
                  </p>
                  <p class="codex-auth-view__status-value codex-auth-view__status-value--truncate">
                    {{ currentAccount?.name || $t('codex.auth.status.noAccount') }}
                  </p>
                </div>
              </div>
            </Card>
          </div>

          <!-- Current Session Info -->
          <Card
            v-if="currentInfo"
            surface="workspace"
            :elevation="2"
            motion="subtle"
            padding="lg"
          >
            <div class="codex-auth-view__section-header">
              <SIcon
                name="Info"
                size="w-5 h-5"
                class="codex-auth-view__section-icon"
              />
              <h3 class="codex-auth-view__section-title">
                {{ $t('codex.auth.currentSession') }}
              </h3>
            </div>
            <div class="codex-auth-view__session-grid">
              <div class="codex-auth-view__session-field">
                <span class="codex-auth-view__field-label">
                  {{ $t('codex.auth.fields.accountId') }}
                </span>
                <code class="codex-auth-view__field-code">
                  {{ currentInfo.account_id }}
                </code>
              </div>
              <div class="codex-auth-view__session-field">
                <span class="codex-auth-view__field-label">
                  {{ $t('codex.auth.fields.email') }}
                </span>
                <span class="codex-auth-view__field-value codex-auth-view__field-value--truncate">
                  {{ currentInfo.email || $t('codex.auth.status.notAvailable') }}
                </span>
              </div>
              <div class="codex-auth-view__session-field">
                <span class="codex-auth-view__field-label">
                  {{ $t('codex.auth.fields.tokenFreshness') }}
                </span>
                <div class="codex-auth-view__field-inline">
                  <span>{{ currentInfo.freshness_icon }}</span>
                  <span
                    class="codex-auth-view__field-value codex-auth-view__field-value--strong"
                    :class="freshnessClass(currentInfo.freshness)"
                  >
                    {{ currentInfo.freshness_description }}
                  </span>
                </div>
              </div>
              <div class="codex-auth-view__session-field">
                <span class="codex-auth-view__field-label">
                  {{ $t('codex.auth.fields.lastRefresh') }}
                </span>
                <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
                  {{ currentInfo.last_refresh || $t('codex.auth.status.notAvailable') }}
                </span>
              </div>
              <div class="codex-auth-view__session-field">
                <span class="codex-auth-view__field-label">
                  {{ $t('codex.auth.fields.expiresAt') }}
                </span>
                <div class="codex-auth-view__field-inline">
                  <span
                    v-if="currentInfo.is_expired"
                    class="codex-auth-view__expired-badge"
                  >
                    <SIcon
                      name="AlertTriangle"
                      size="w-3 h-3"
                    />
                    {{ $t('codex.auth.expired') }}
                  </span>
                  <span
                    v-else-if="currentInfo.expires_at"
                    class="codex-auth-view__field-value codex-auth-view__field-value--muted"
                  >
                    {{ formatExpiryDate(currentInfo.expires_at) }}
                  </span>
                  <span
                    v-else
                    class="codex-auth-view__field-value codex-auth-view__field-value--faint"
                  >
                    {{ $t('codex.auth.noExpiry') }}
                  </span>
                </div>
              </div>
            </div>
          </Card>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            padding="lg"
            :glow-color="canManageAuthAccounts ? 'success' : 'warning'"
          >
            <div class="codex-auth-view__guard">
              <div
                class="codex-auth-view__guard-icon-shell"
                :class="canManageAuthAccounts ? 'bg-emerald-500/10 text-emerald-400' : 'bg-yellow-500/10 text-yellow-400'"
              >
                <SIcon
                  name="AlertTriangle"
                  size="w-5 h-5"
                />
              </div>
              <div class="codex-auth-view__guard-body">
                <p class="codex-auth-view__guard-title">
                  {{ $t('codex.auth.profileGuard.title') }}
                </p>
                <p class="codex-auth-view__guard-message">
                  {{ profileGuardMessage }}
                </p>
                <p
                  v-if="authActionError"
                  class="codex-auth-view__guard-error"
                >
                  {{ authActionError }}
                </p>
              </div>
            </div>
          </Card>

          <!-- Account Overview -->
          <div class="codex-auth-view__overview-header">
            <h2 class="codex-auth-view__overview-title">
              <SIcon
                name="LayoutGrid"
                size="w-5 h-5"
                class="text-platform-codex"
              />
              {{ $t('codex.auth.accountOverview') }}
            </h2>
            <button
              class="hidden"
              @click="handleRefresh"
            />
            <Button
              variant="secondary"
              surface="status"
              density="compact"
              motion="subtle"
              @click="handleRefresh"
            >
              <template #leading>
                <SIcon
                  name="RefreshCw"
                  size="w-4 h-4"
                  :class="{ 'animate-spin': loading }"
                />
              </template>
              {{ $t('codex.auth.refresh') }}
            </Button>
          </div>

          <!-- Loading -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div class="w-12 h-12 rounded-full border-4 border-transparent border-t-accent-primary border-r-accent-secondary animate-spin" />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="accounts.length === 0"
            class="empty-state glass-effect rounded-2xl border border-white/5"
          >
            <div class="p-4 rounded-full glass-surface mb-4">
              <SIcon
                name="KeyRound"
                size="w-8 h-8"
                class="text-white/50"
              />
            </div>
            <p class="text-white/80">
              {{ $t('codex.auth.emptyState') }}
            </p>
            <p class="text-sm text-white/50 mt-2">
              {{ $t('codex.auth.emptyStateHint') }}
            </p>
          </div>

          <!-- Account Card Grid -->
          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 gap-4"
          >
            <CodexAccountCard
              v-for="account in accounts"
              :key="account.name"
              :account="account"
              :quota="quotaMap.get(account.name) ?? null"
              :quota-loading="quotaLoading"
              :is-current="account.is_current"
              :busy-action="busyName === account.name ? busyAction : null"
              :disabled="actionLoading"
              @switch="handleSwitch"
              @delete="handleDelete"
              @refresh="handleRefreshSingle"
              @tag="handleTag"
              @export="handleExport"
            />
          </div>

          <!-- Save Modal -->
          <BaseModal
            :model-value="showSaveForm"
            :title="$t('codex.auth.saveAccount')"
            :description="$t('codex.auth.subtitle')"
            size="lg"
            surface="glass"
            content-class="w-full max-w-lg max-h-[90vh] overflow-y-auto"
            @update:model-value="(value) => !value && handleCloseSaveForm()"
          >
            <template #header="{ titleId }">
              <!-- Modal Header -->
              <div class="px-6 py-4 border-b border-white/5 flex items-center justify-between sticky top-0 bg-white/5/95 backdrop-blur z-10">
                <h2
                  :id="titleId"
                  class="text-xl font-bold text-white"
                >
                  {{ $t('codex.auth.saveAccount') }}
                </h2>
                <Button
                  variant="ghost"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="handleCloseSaveForm"
                >
                  <template #leading>
                    <SIcon
                      name="X"
                      size="w-5 h-5"
                    />
                  </template>
                </Button>
              </div>
            </template>

            <!-- Modal Content -->
            <div class="p-6 space-y-6">
              <!-- Process Warning -->
              <div
                v-if="processWarning"
                class="p-4 rounded-lg bg-yellow-500/10 border border-yellow-500/30 text-yellow-600 dark:text-yellow-400"
              >
                <div class="flex items-start gap-3">
                  <SIcon
                    name="AlertTriangle"
                    size="w-5 h-5"
                    class="flex-shrink-0 mt-0.5"
                  />
                  <div>
                    <p class="font-medium">
                      {{ $t('codex.auth.processWarning') }}
                    </p>
                    <p class="text-sm mt-1 opacity-80">
                      {{ processWarning }}
                    </p>
                  </div>
                </div>
              </div>

              <div class="space-y-4">
                <div class="space-y-1.5">
                  <label class="text-sm font-semibold text-white/80">
                    {{ $t('codex.auth.fields.accountName') }} <span class="text-red-500">*</span>
                  </label>
                  <input
                    v-model="saveForm.name"
                    type="text"
                    class="input"
                    :placeholder="$t('codex.auth.placeholders.accountName')"
                  >
                </div>
                <div class="space-y-1.5">
                  <label class="text-sm font-semibold text-white/80">
                    {{ $t('codex.auth.fields.description') }}
                  </label>
                  <input
                    v-model="saveForm.description"
                    type="text"
                    class="input"
                    :placeholder="$t('codex.auth.placeholders.description')"
                  >
                </div>
                <div class="space-y-1.5">
                  <label class="text-sm font-semibold text-white/80">
                    {{ $t('codex.auth.fields.expiresAt') }}
                  </label>
                  <input
                    v-model="saveForm.expires_at"
                    type="datetime-local"
                    class="input"
                  >
                  <p class="text-xs text-white/50 mt-1">
                    {{ $t('codex.auth.expiresAtHint') }}
                  </p>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg glass-surface border border-white/5">
                  <input
                    id="forceOverwrite"
                    v-model="saveForm.force"
                    type="checkbox"
                    class="w-5 h-5 rounded border-white/10 text-accent-primary focus:ring-accent-primary/20"
                  >
                  <label
                    for="forceOverwrite"
                    class="text-sm font-medium text-white cursor-pointer select-none"
                  >
                    {{ $t('codex.auth.forceOverwrite') }}
                  </label>
                </div>
              </div>
            </div>

            <template #footer>
              <!-- Footer -->
              <div class="px-6 py-4 border-t border-white/5 flex justify-end gap-3 bg-white/5/50">
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="handleCloseSaveForm"
                >
                  {{ $t('codex.actions.cancel') }}
                </Button>
                <Button
                  variant="primary"
                  surface="card"
                  density="compact"
                  motion="standard"
                  :disabled="saving || !saveForm.name.trim()"
                  @click="handleConfirmSave"
                >
                  <template #leading>
                    <span
                      v-if="saving"
                      class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"
                    />
                  </template>
                  {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
                </Button>
              </div>
            </template>
          </BaseModal>

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
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onActivated, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import CodexAccountCard from '@/components/codex/CodexAccountCard.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import {
  listCodexProfiles,
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  saveCodexAuth,
  switchCodexAuth,
  deleteCodexAuth,
  detectCodexProcess,
  getCodexAllQuotas
} from '@/api'
import type {
  CodexAuthAccountItem,
  CodexAuthCurrentInfo,
  CodexAuthCurrentResponse,
  CodexAuthListResponse,
  CodexAuthProcessResponse,
  CodexProfile,
  CodexProfilesResponse,
  CodexAuthSaveRequest,
  CodexProfileAuthMode,
  LoginState,
  TokenFreshness,
  CodexAccountQuota
} from '@/types'
import { logger } from '@/utils/logger'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'CodexAuthView' })

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(false)
const saving = ref(false)
const actionLoading = ref(false)

const accounts = ref<CodexAuthAccountItem[]>([])
const loginState = ref<LoginState>({ type: 'NotLoggedIn' })
const currentInfo = ref<CodexAuthCurrentInfo | null>(null)
const currentProfile = ref<CodexProfile | null>(null)
const authActionError = ref<string | null>(null)
const quotaMap = ref<Map<string, CodexAccountQuota>>(new Map())
const quotaLoading = ref(false)

const showSaveForm = ref(false)
const processWarning = ref<string | null>(null)
const busyName = ref<string | null>(null)
const busyAction = ref<'switch' | 'delete' | null>(null)
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

const saveForm = reactive({
  name: '',
  description: '',
  expires_at: '',
  force: false,
})

const usesOpenAiAuthMode = (authMode?: CodexProfileAuthMode | null) => {
  return authMode === 'openai_chatgpt' || authMode === 'openai_api_key'
}

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

// Computed properties
const currentAccount = computed(() => accounts.value.find(a => a.is_current))
const canManageAuthAccounts = computed(() => usesOpenAiAuthMode(currentProfile.value?.auth_mode))
const profileGuardMessage = computed(() => {
  if (!currentProfile.value) {
    return t('codex.auth.profileGuard.noCurrentProfile')
  }
  if (!canManageAuthAccounts.value) {
    return t('codex.auth.profileGuard.unsupportedProfile', {
      name: currentProfile.value.name,
      authMode: currentProfile.value.auth_mode || 'no_auth',
    })
  }
  return t('codex.auth.profileGuard.supportedProfile', {
    name: currentProfile.value.name,
    authMode: currentProfile.value.auth_mode || 'openai_chatgpt',
  })
})

const canSave = computed(() => {
  return canManageAuthAccounts.value && (loginState.value.type === 'LoggedInUnsaved' || loginState.value.type === 'LoggedInSaved')
})

const loginStateColor = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved': return 'success'
    case 'LoggedInUnsaved': return 'warning'
    case 'ApiKeyActive': return 'primary'
    case 'ProviderKeyActive': return 'primary'
    case 'Unknown': return 'warning'
    default: return 'danger'
  }
})

const loginStateIcon = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved': return 'UserCheck'
    case 'LoggedInUnsaved': return 'LogIn'
    case 'ApiKeyActive': return 'KeyRound'
    case 'ProviderKeyActive': return 'KeyRound'
    case 'Unknown': return 'AlertTriangle'
    default: return 'LogOut'
  }
})

const loginStateIconClass = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved': return 'bg-emerald-500/10 text-emerald-500'
    case 'LoggedInUnsaved': return 'bg-yellow-500/10 text-yellow-500'
    case 'ApiKeyActive': return 'bg-blue-500/10 text-blue-500'
    case 'ProviderKeyActive': return 'bg-blue-500/10 text-blue-500'
    case 'Unknown': return 'bg-yellow-500/10 text-yellow-500'
    default: return 'bg-red-500/10 text-red-500'
  }
})

const loginStateText = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved':
      return t('codex.auth.loginState.loggedInSaved', { name: loginState.value.account_name })
    case 'LoggedInUnsaved':
      return t('codex.auth.loginState.loggedInUnsaved')
    case 'ApiKeyActive':
      return t('codex.auth.loginState.apiKeyActive')
    case 'ProviderKeyActive':
      return t('codex.auth.loginState.providerKeyActive', { envKey: loginState.value.env_key })
    case 'Unknown':
      return t('codex.auth.loginState.unknown', { type: loginState.value.raw_type })
    default:
      return t('codex.auth.loginState.notLoggedIn')
  }
})

// Helper functions
const freshnessClass = (freshness: TokenFreshness) => {
  switch (freshness) {
    case 'Fresh': return 'text-emerald-500'
    case 'Stale': return 'text-yellow-500'
    case 'Old': return 'text-orange-500'
    default: return 'text-text-muted'
  }
}

const formatExpiryDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr)
    return date.toLocaleString()
  } catch {
    return dateStr
  }
}

// Data loading
const loadCurrentProfile = async () => {
  try {
    const data = await listCodexProfiles<CodexProfilesResponse>()
    currentProfile.value = data.profiles.find(profile => profile.name === data.current_profile) || null
  } catch (error) {
    logger.error('Failed to load current codex profile:', error)
    currentProfile.value = null
  }
}

const loadAccounts = async () => {
  try {
    loading.value = true
    authActionError.value = null
    const data = await listCodexAuthAccounts<CodexAuthListResponse>()
    accounts.value = data.accounts || []
    loginState.value = data.login_state
  } catch (error) {
    logger.error('Failed to load codex auth accounts:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const loadCurrentInfo = async () => {
  try {
    const data = await getCodexAuthCurrent<CodexAuthCurrentResponse>()
    if (data.logged_in && data.info) {
      currentInfo.value = data.info
    } else {
      currentInfo.value = null
    }
  } catch (error) {
    logger.error('Failed to load current auth info:', error)
  }
}

const loadQuotas = async () => {
  try {
    quotaLoading.value = true
    const data = await getCodexAllQuotas<CodexAccountQuota[]>()
    const map = new Map<string, CodexAccountQuota>()
    for (const q of data) {
      map.set(q.account_name, q)
    }
    quotaMap.value = map
  } catch (e) {
    logger.error('Failed to fetch codex quotas:', e)
  } finally {
    quotaLoading.value = false
  }
}

const handleRefresh = async () => {
  await Promise.all([loadAccounts(), loadCurrentInfo(), loadCurrentProfile(), loadQuotas()])
  lastLoadedAt.value = Date.now()
}

const ensureLoaded = async (force = false) => {
  if (loading.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) {
    return
  }
  await handleRefresh()
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

// Actions
const handleSave = async () => {
  authActionError.value = null
  if (!canManageAuthAccounts.value) {
    authActionError.value = profileGuardMessage.value
    return
  }

  // Check for running Codex processes
  try {
    const processInfo = await detectCodexProcess<CodexAuthProcessResponse>()
    if (processInfo.has_running_process) {
      processWarning.value = processInfo.warning || t('codex.auth.processDetected', { pids: processInfo.pids.join(', ') })
    } else {
      processWarning.value = null
    }
  } catch {
    processWarning.value = null
  }

  // Reset form
  saveForm.name = ''
  saveForm.description = ''
  saveForm.expires_at = ''
  saveForm.force = false
  showSaveForm.value = true
}

const handleCloseSaveForm = () => {
  showSaveForm.value = false
  processWarning.value = null
}

const handleConfirmSave = async () => {
  authActionError.value = null
  if (!saveForm.name.trim()) {
    uiStore.showError(t('codex.auth.validation.nameRequired'))
    return
  }

  try {
    saving.value = true
    // Convert local datetime to ISO 8601 UTC format if provided
    let expiresAt: string | undefined
    if (saveForm.expires_at) {
      const localDate = new Date(saveForm.expires_at)
      expiresAt = localDate.toISOString()
    }

    const payload: CodexAuthSaveRequest = {
      name: saveForm.name.trim(),
      description: saveForm.description.trim() || undefined,
      expires_at: expiresAt,
      force: saveForm.force,
    }
    await saveCodexAuth(payload)
    handleCloseSaveForm()
    await handleRefresh()
    uiStore.showSuccess(t('codex.auth.saveAccount'))
  } catch (error) {
    logger.error('Failed to save auth:', error)
    authActionError.value = extractErrorMessage(error) || t('codex.states.saveFailed')
    uiStore.showError(authActionError.value)
  } finally {
    saving.value = false
  }
}

const handleSwitch = async (name: string) => {
  authActionError.value = null
  openConfirmDialog({
    title: t('codex.auth.switch'),
    message: translateWithFallback(
      t,
      'codex.auth.confirmSwitch',
      '确定要切换到账户 "{name}" 吗？',
      { name },
    ),
    confirmText: t('codex.auth.switch'),
    type: 'warning',
    action: async () => {
      busyName.value = name
      busyAction.value = 'switch'
      try {
        await switchCodexAuth(name)
        await handleRefresh()
        uiStore.showSuccess(t('codex.auth.switch'))
      } catch (error) {
        logger.error('Failed to switch auth:', error)
        authActionError.value = extractErrorMessage(error) || t('codex.states.saveFailed')
        uiStore.showError(authActionError.value)
      } finally {
        busyName.value = null
        busyAction.value = null
      }
    },
  })
}

const handleDelete = async (name: string) => {
  authActionError.value = null
  openConfirmDialog({
    title: t('codex.actions.delete'),
    message: translateWithFallback(
      t,
      'codex.auth.deleteConfirm',
      '确定要删除账户 "{name}" 吗？',
      { name },
    ),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      busyName.value = name
      busyAction.value = 'delete'
      try {
        await deleteCodexAuth(name)
        await handleRefresh()
        uiStore.showSuccess(t('codex.actions.delete'))
      } catch (error) {
        logger.error('Failed to delete auth:', error)
        authActionError.value = extractErrorMessage(error) || t('codex.states.deleteFailed')
        uiStore.showError(authActionError.value)
      } finally {
        busyName.value = null
        busyAction.value = null
      }
    },
  })
}

const handleRefreshSingle = async (_name: string) => {
  await loadQuotas()
}

const handleTag = (_name: string) => {
  uiStore.showInfo(t('codex.auth.featureComingSoon'))
}

const handleExport = (_name: string) => {
  uiStore.showInfo(t('codex.auth.featureComingSoon'))
}

onMounted(async () => {
  await ensureLoaded(true)
})

onActivated(() => {
  void ensureLoaded(false)
})
</script>

<style scoped>
.codex-auth-view {
  min-height: 100%;
  padding: 1.5rem;
}

.codex-auth-view__container {
  max-width: 1800px;
  margin: 0 auto;
}

.codex-auth-view__stack {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  margin-top: 1.5rem;
}

.codex-auth-view__main {
  display: flex;
  width: 100%;
  min-width: 0;
  flex-direction: column;
  gap: 1.5rem;
}

.codex-auth-view__header,
.codex-auth-view__overview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.codex-auth-view__title-group,
.codex-auth-view__actions,
.codex-auth-view__overview-title,
.codex-auth-view__section-header,
.codex-auth-view__status-row,
.codex-auth-view__field-inline {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.codex-auth-view__title-icon-shell,
.codex-auth-view__guard-icon-shell {
  border-radius: 0.75rem;
  padding: 0.5rem;
}

.codex-auth-view__title-icon-shell {
  background: rgb(var(--platform-codex-rgb, 245 158 11) / 10%);
}

.codex-auth-view__title-icon,
.codex-auth-view__section-icon {
  color: var(--platform-codex, #f59e0b);
}

.codex-auth-view__title {
  color: rgb(255 255 255 / 100%);
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
}

.codex-auth-view__subtitle {
  margin-top: 0.25rem;
  color: rgb(255 255 255 / 80%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__status-grid,
.codex-auth-view__session-grid {
  display: grid;
  gap: 1rem;
}

.codex-auth-view__status-grid {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.codex-auth-view__status-card {
  position: relative;
  overflow: hidden;
}

.codex-auth-view__status-icon-shell {
  border-radius: 0.75rem;
  padding: 0.75rem;
  transition: transform 0.3s ease, color 0.3s ease, background-color 0.3s ease;
}

.codex-auth-view__status-card:hover .codex-auth-view__status-icon-shell {
  transform: scale(1.1);
}

.codex-auth-view__status-icon-shell--info {
  background: rgb(99 102 241 / 10%);
  color: rgb(99 102 241 / 100%);
}

.codex-auth-view__status-label,
.codex-auth-view__field-label {
  margin-bottom: 0.25rem;
  color: rgb(255 255 255 / 50%);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.codex-auth-view__status-value,
.codex-auth-view__overview-title {
  color: rgb(255 255 255 / 100%);
  font-size: 1.25rem;
  line-height: 1.75rem;
  font-weight: 700;
}

.codex-auth-view__status-value--truncate,
.codex-auth-view__field-value--truncate,
.codex-auth-view__field-code {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.codex-auth-view__section-title,
.codex-auth-view__guard-title {
  color: rgb(255 255 255 / 100%);
  font-size: 1rem;
  line-height: 1.5rem;
  font-weight: 600;
}

.codex-auth-view__session-grid {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.codex-auth-view__session-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.codex-auth-view__field-code {
  border: 1px solid rgb(255 255 255 / 5%);
  border-radius: 0.5rem;
  padding: 0.25rem 0.5rem;
  color: rgb(255 255 255 / 100%);
  font-family: var(--font-mono, 'Maple Mono', monospace);
}

.codex-auth-view__field-value--muted {
  color: rgb(255 255 255 / 80%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__field-value--faint {
  color: rgb(255 255 255 / 50%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__field-value--strong {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.codex-auth-view__expired-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid rgb(239 68 68 / 20%);
  border-radius: 0.375rem;
  background: rgb(239 68 68 / 10%);
  padding: 0.125rem 0.5rem;
  color: rgb(239 68 68 / 100%);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
}

.codex-auth-view__guard {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.codex-auth-view__guard-icon-shell {
  margin-top: 0.125rem;
}

.codex-auth-view__guard-body {
  min-width: 0;
}

.codex-auth-view__guard-message {
  margin-top: 0.25rem;
  color: rgb(255 255 255 / 70%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__guard-error {
  margin-top: 0.75rem;
  border: 1px solid rgb(239 68 68 / 20%);
  border-radius: 0.5rem;
  background: rgb(239 68 68 / 10%);
  padding: 0.5rem 0.75rem;
  color: rgb(252 165 165 / 100%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

@media (width >= 768px) {
  .codex-auth-view__status-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .codex-auth-view__session-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width >= 1280px) {
  .codex-auth-view__session-grid {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
}

@media (width <= 900px) {
  .codex-auth-view__header,
  .codex-auth-view__overview-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .codex-auth-view__actions {
    flex-wrap: wrap;
  }
}
</style>
