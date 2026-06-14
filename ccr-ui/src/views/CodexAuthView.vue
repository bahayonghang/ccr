<template>
  <div class="codex-auth-view">
    <div class="codex-auth-view__container">
      <div class="codex-auth-view__stack">
        <ModuleSubnav module="codex" />

        <main class="codex-auth-view__main">
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
                  {{
                    tf(
                      'codex.auth.managerSubtitle',
                      'Use one surface to add, import, switch, and review Codex accounts and model providers.'
                    )
                  }}
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
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                :disabled="loading"
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

              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                :disabled="!canSave"
                @click="handleSave"
              >
                <template #leading>
                  <SIcon
                    name="Save"
                    size="w-4 h-4"
                  />
                </template>
                {{ tf('codex.auth.actions.saveCurrent', 'Save current session') }}
              </Button>

              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                @click="openAddAccountModal()"
              >
                <template #leading>
                  <SIcon
                    name="Plus"
                    size="w-4 h-4"
                  />
                </template>
                {{ tf('codex.auth.actions.addAccount', 'Add account') }}
              </Button>
            </div>
          </div>

          <div class="codex-auth-view__status-grid">
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

            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell codex-auth-view__status-icon-shell--info"
                >
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

            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell"
                  :class="
                    currentAccount
                      ? 'bg-emerald-500/10 text-emerald-500'
                      : 'bg-gray-500/10 text-text-muted'
                  "
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
                    {{
                      currentAccount?.email ||
                        currentAccount?.name ||
                        $t('codex.auth.status.noAccount')
                    }}
                  </p>
                </div>
              </div>
            </Card>

            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell codex-auth-view__status-icon-shell--neutral"
                >
                  <SIcon
                    name="Globe"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ tf('codex.auth.status.providerCount', 'Model providers') }}
                  </p>
                  <p class="codex-auth-view__status-value">
                    {{ providers.length }}
                  </p>
                </div>
              </div>
            </Card>
          </div>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            padding="lg"
            class="codex-auth-view__segment-card"
          >
            <div class="codex-auth-view__segment-row">
              <button
                type="button"
                class="codex-auth-view__segment"
                :class="{ 'codex-auth-view__segment--active': activeManagerTab === 'accounts' }"
                @click="activeManagerTab = 'accounts'"
              >
                <SIcon
                  name="LayoutGrid"
                  size="w-4 h-4"
                />
                <span>{{ $t('codex.auth.accountOverview') }}</span>
                <span class="codex-auth-view__segment-count">{{ accounts.length }}</span>
              </button>
              <button
                type="button"
                class="codex-auth-view__segment"
                :class="{ 'codex-auth-view__segment--active': activeManagerTab === 'providers' }"
                @click="activeManagerTab = 'providers'"
              >
                <SIcon
                  name="Blocks"
                  size="w-4 h-4"
                />
                <span>{{ tf('codex.auth.providers.title', 'Model providers') }}</span>
                <span class="codex-auth-view__segment-count">{{ providers.length }}</span>
              </button>
            </div>
          </Card>


          <CodexAuthAccountsTab
            v-if="activeManagerTab === 'accounts'"
            v-model:search-query="searchQuery"
            v-model:status-filter="statusFilter"
            v-model:plan-filter="planFilter"
            v-model:sort-by="sortBy"
            :loading="loading"
            :accounts="accounts"
            :current-info="currentInfo"
            :can-manage-auth-accounts="canManageAuthAccounts"
            :profile-guard-message="profileGuardMessage"
            :auth-action-error="authActionError"
            :status-options="statusOptions"
            :plan-options="planOptions"
            :sort-options="sortOptions"
            :filtered-accounts="filteredAccounts"
            :filters-results-count="filtersResultsCount"
            :has-active-filters="hasActiveFilters"
            :quota-map="quotaMap"
            :quota-loading="quotaLoading"
            :busy-name="busyName"
            :busy-action="busyAction"
            :action-loading="actionLoading"
            :format-auth-method="formatAuthMethod"
            @clear-filters="clearFilters"
            @open-add-account="openAddAccountModal()"
            @switch="handleSwitch"
            @delete="handleDelete"
            @refresh="handleRefreshSingle"
            @tag="handleTag"
            @export="handleExport"
            @rename="handleRename"
          />

          <CodexAuthProvidersTab
            v-else
            :provider-form="providerForm"
            :provider-error="providerError"
            :provider-saving="providerSaving"
            :provider-loading="providerLoading"
            :providers="providers"
            :selected-provider-template="selectedProviderTemplate"
            :selected-provider-endpoint="selectedProviderEndpoint"
            :codex-template-draft="codexTemplateDraft"
            :format-provider-updated-at="formatProviderUpdatedAt"
            @update:provider-form="(val) => Object.assign(providerForm, val)"
            @reset-form="resetProviderForm"
            @apply-template="applyCodexProviderTemplate"
            @use-manual-template="useManualProviderTemplate"
            @save-provider="handleSaveProvider"
            @load-providers="loadProviders"
            @use-in-api-form="handleUseProviderInApiForm"
            @edit-provider="editProvider"
            @delete-provider="requestDeleteProvider"
          />

          <SaveCodexSessionModal
            v-model="showSaveForm"
            :current-info="currentInfo"
            :format-auth-method="formatAuthMethod"
            @saved="handleRefresh"
          />

          <AddCodexAccountModal
            v-model="showAddAccountModal"
            :providers="providers"
            :can-manage-auth-accounts="canManageAuthAccounts"
            :initial-method="addAccountInitialMethod"
            :preset-provider="addAccountPresetProvider"
            :refresh-on-mutation="handleRefresh"
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

          <RenameCodexAccountModal
            v-model="showRenameDialog"
            :account-name="renameTarget"
            @renamed="handleRefresh"
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
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import CodexAuthAccountsTab from './codex/tabs/CodexAuthAccountsTab.vue'
import CodexAuthProvidersTab from './codex/tabs/CodexAuthProvidersTab.vue'
import RenameCodexAccountModal from './codex/components/RenameCodexAccountModal.vue'
import SaveCodexSessionModal from './codex/components/SaveCodexSessionModal.vue'
import AddCodexAccountModal from './codex/components/AddCodexAccountModal.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useTf } from '@/composables/useTf'
import {
  listCodexProfiles,
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  switchCodexAuth,
  deleteCodexAuth,
  getCodexAllQuotas,
  codexListModelProviders,
  codexSaveModelProvider,
  codexDeleteModelProvider,
} from '@/api'
import {
  filterAndSortCodexAccounts,
  getLoginStateIcon,
  getLoginStateIconClass,
  getLoginStateTone,
  usesOpenAiAuthMode,
  type AccountPlanFilter,
  type AccountSort,
  type AccountStatusFilter,
} from './codex/codexAuthAccounts'
import type {
  CodexAccountQuota,
  CodexAuthAccountItem,
  CodexAuthCurrentInfo,
  CodexAuthCurrentResponse,
  CodexAuthListResponse,
  CodexModelProviderRecord,
  CodexModelProvidersResponse,
  CodexProfile,
  CodexProfilesResponse,
  LoginState,
} from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { logger } from '@/utils/logger'
import { extractErrorMessage } from '@/utils/errorHandler'
import { useUIStore } from '@/stores/ui'
import { mapTemplateToCodexProviderPatch } from '@/utils/providerTemplates'
import { REFRESH_TTL_MS } from '@/config/constants'

defineOptions({ name: 'CodexAuthView' })

const { t } = useI18n()
const uiStore = useUIStore()

type ManagerTab = 'accounts' | 'providers'
type AddMethod = 'oauth' | 'token' | 'api' | 'local'

const loading = ref(false)
const actionLoading = ref(false)
const quotaLoading = ref(false)
const providerLoading = ref(false)
const providerSaving = ref(false)

const accounts = ref<CodexAuthAccountItem[]>([])
const providers = ref<CodexModelProviderRecord[]>([])
const loginState = ref<LoginState>({ type: 'NotLoggedIn' })
const currentInfo = ref<CodexAuthCurrentInfo | null>(null)
const currentProfile = ref<CodexProfile | null>(null)
const authActionError = ref<string | null>(null)
const providerError = ref<string | null>(null)
const quotaMap = ref<Map<string, CodexAccountQuota>>(new Map())

const activeManagerTab = ref<ManagerTab>('accounts')
const showSaveForm = ref(false)
const showAddAccountModal = ref(false)
// 添加账号弹窗的打开驱动：默认方式 + 可选的「使用提供商预填 API 表单」预设
const addAccountInitialMethod = ref<AddMethod>('oauth')
const addAccountPresetProvider = ref<CodexModelProviderRecord | null>(null)
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

const providerForm = reactive({
  id: '',
  name: '',
  baseUrl: '',
  websiteUrl: '',
  apiKeyUrl: '',
  apiKeyName: 'API Key',
  apiKey: '',
})
const selectedProviderTemplate = ref<string | null>(null)
const selectedProviderEndpoint = ref('')

const codexTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'codex',
  defaultName: providerForm.name || 'Codex provider',
  name: providerForm.name,
  websiteUrl: providerForm.websiteUrl,
  apiKeyUrl: providerForm.apiKeyUrl,
  category: 'third_party',
  baseUrls: providerForm.baseUrl.trim() ? [providerForm.baseUrl.trim()] : [],
  platformOverride: {
    baseUrl: providerForm.baseUrl,
    websiteUrl: providerForm.websiteUrl,
    apiKeyUrl: providerForm.apiKeyUrl,
  },
}))

const searchQuery = ref('')
const statusFilter = ref<AccountStatusFilter>('all')
const planFilter = ref<AccountPlanFilter>('all')
const sortBy = ref<AccountSort>('saved_desc')

const tf = useTf()

const currentAccount = computed(() => accounts.value.find((account) => account.is_current))
const canManageAuthAccounts = computed(() => usesOpenAiAuthMode(currentProfile.value?.auth_mode))

const profileGuardMessage = computed(() => {
  if (!currentProfile.value) {
    return t('codex.auth.profileGuard.noCurrentProfile')
  }
  if (!canManageAuthAccounts.value) {
    return tf(
      'codex.auth.profileGuard.unsupportedProfile',
      'Current profile "{name}" uses "{authMode}". Codex Auth account save/switch only works for OpenAI-auth current profiles.',
      {
        name: currentProfile.value.name,
        authMode: currentProfile.value.auth_mode || 'no_auth',
      }
    )
  }
  return tf(
    'codex.auth.profileGuard.supportedProfile',
    'Current profile "{name}" uses "{authMode}". Auth account save/switch is available.',
    {
      name: currentProfile.value.name,
      authMode: currentProfile.value.auth_mode || 'openai_chatgpt',
    }
  )
})

const canSave = computed(() => {
  return (
    canManageAuthAccounts.value &&
    (loginState.value.type === 'LoggedInUnsaved' || loginState.value.type === 'LoggedInSaved')
  )
})

const loginStateColor = computed(() => getLoginStateTone(loginState.value))

const loginStateIcon = computed(() => getLoginStateIcon(loginState.value))

const loginStateIconClass = computed(() => getLoginStateIconClass(loginState.value))

const loginStateText = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved':
      return tf('codex.auth.loginState.loggedInSaved', 'Logged in ({name})', {
        name: loginState.value.account_name,
      })
    case 'LoggedInUnsaved':
      return t('codex.auth.loginState.loggedInUnsaved')
    case 'ApiKeyActive':
      return t('codex.auth.loginState.apiKeyActive')
    case 'ProviderKeyActive':
      return tf('codex.auth.loginState.providerKeyActive', 'Provider Key ({envKey})', {
        envKey: loginState.value.env_key,
      })
    case 'Unknown':
      return tf('codex.auth.loginState.unknown', 'Unknown state ({type})', {
        type: loginState.value.raw_type,
      })
    default:
      return t('codex.auth.loginState.notLoggedIn')
  }
})

const statusOptions = computed(() => [
  { value: 'all' as const, label: t('codex.auth.filters.statusOptions.all') },
  { value: 'current' as const, label: t('codex.auth.filters.statusOptions.current') },
  { value: 'virtual' as const, label: t('codex.auth.filters.statusOptions.virtual') },
  { value: 'attention' as const, label: t('codex.auth.filters.statusOptions.attention') },
])

const planOptions = computed(() => [
  { value: 'all' as const, label: t('codex.auth.filters.planOptions.all') },
  { value: 'plus' as const, label: t('codex.auth.filters.planOptions.plus') },
  { value: 'pro' as const, label: t('codex.auth.filters.planOptions.pro') },
  { value: 'team' as const, label: t('codex.auth.filters.planOptions.team') },
  { value: 'unknown' as const, label: t('codex.auth.filters.planOptions.unknown') },
])

const sortOptions = computed(() => [
  { value: 'saved_desc' as const, label: t('codex.auth.filters.sortOptions.savedDesc') },
  { value: 'used_desc' as const, label: t('codex.auth.filters.sortOptions.usedDesc') },
  { value: 'name_asc' as const, label: t('codex.auth.filters.sortOptions.nameAsc') },
])

const hasActiveFilters = computed(() => {
  return (
    Boolean(searchQuery.value.trim()) ||
    statusFilter.value !== 'all' ||
    planFilter.value !== 'all' ||
    sortBy.value !== 'saved_desc'
  )
})

const filteredAccounts = computed(() =>
  filterAndSortCodexAccounts({
    accounts: accounts.value,
    quotaMap: quotaMap.value,
    searchQuery: searchQuery.value,
    statusFilter: statusFilter.value,
    planFilter: planFilter.value,
    sortBy: sortBy.value,
  })
)

const filtersResultsCount = computed(() =>
  tf('codex.auth.filters.resultsCount', '{shown} / {total} accounts', {
    shown: filteredAccounts.value.length,
    total: accounts.value.length,
  })
)

const formatAuthMethod = (method?: string | null) => {
  switch (method) {
    case 'chatgpt':
      return tf('codex.auth.authMethods.chatgpt', 'ChatGPT OAuth')
    case 'api':
      return tf('codex.auth.authMethods.api', 'API Key')
    case 'provider':
      return tf('codex.auth.authMethods.provider', 'Provider key')
    default:
      return tf('codex.auth.authMethods.unknown', 'Unknown')
  }
}

const formatProviderUpdatedAt = (value?: string | null, detailed = false) => {
  if (!value) return t('common.notAvailable')
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return detailed
    ? date.toLocaleString()
    : new Intl.DateTimeFormat('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      }).format(date)
}

const clearFilters = () => {
  searchQuery.value = ''
  statusFilter.value = 'all'
  planFilter.value = 'all'
  sortBy.value = 'saved_desc'
}

const loadCurrentProfile = async () => {
  try {
    const data = await listCodexProfiles<CodexProfilesResponse>()
    currentProfile.value =
      data.profiles.find((profile) => profile.name === data.current_profile) || null
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
    currentInfo.value = data.logged_in && data.info ? data.info : null
  } catch (error) {
    logger.error('Failed to load current auth info:', error)
  }
}

const loadQuotas = async () => {
  try {
    quotaLoading.value = true
    const data = await getCodexAllQuotas<CodexAccountQuota[]>()
    const map = new Map<string, CodexAccountQuota>()
    for (const quota of data) {
      map.set(quota.account_name, quota)
    }
    quotaMap.value = map
  } catch (error) {
    logger.error('Failed to fetch codex quotas:', error)
  } finally {
    quotaLoading.value = false
  }
}

const loadProviders = async () => {
  try {
    providerLoading.value = true
    providerError.value = null
    const data = await codexListModelProviders<CodexModelProvidersResponse>()
    providers.value = data.providers || []
  } catch (error) {
    logger.error('Failed to load codex providers:', error)
    providerError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.providers.loadFailed', 'Failed to load saved providers.')
  } finally {
    providerLoading.value = false
  }
}

const handleRefresh = async () => {
  await Promise.all([
    loadAccounts(),
    loadCurrentInfo(),
    loadCurrentProfile(),
    loadQuotas(),
    loadProviders(),
  ])
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

// 保存当前会话弹窗自身负责进程探测/表单/提交（SaveCodexSessionModal），
// 主视图仅在 canSave 允许时打开它（头部按钮已按 canSave 禁用）
const handleSave = () => {
  showSaveForm.value = true
}

const handleSwitch = async (name: string) => {
  authActionError.value = null
  openConfirmDialog({
    title: t('codex.auth.switch'),
    message: translateWithFallback(
      t,
      'codex.auth.confirmSwitch',
      '确定要切换到账户 "{name}" 吗？',
      { name }
    ),
    confirmText: t('codex.auth.switch'),
    type: 'warning',
    action: async () => {
      busyName.value = name
      busyAction.value = 'switch'
      try {
        await switchCodexAuth(name)
        await handleRefresh()
        uiStore.showSuccess(
          tf('codex.auth.feedback.switchSuccess', 'Switched account successfully.')
        )
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
    message: translateWithFallback(t, 'codex.auth.deleteConfirm', '确定要删除账户 "{name}" 吗？', {
      name,
    }),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      busyName.value = name
      busyAction.value = 'delete'
      try {
        await deleteCodexAuth(name)
        await handleRefresh()
        uiStore.showSuccess(
          tf('codex.auth.feedback.deleteSuccess', 'Account deleted successfully.')
        )
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

const handleRefreshSingle = async () => {
  await loadQuotas()
}

const handleTag = (_name: string) => {
  uiStore.showInfo(t('codex.auth.featureComingSoon'))
}

const handleExport = (_name: string) => {
  uiStore.showInfo(t('codex.auth.featureComingSoon'))
}

// ── 重命名账号 ──
// 弹窗自身负责表单/校验/提交（RenameCodexAccountModal），主视图仅做 profile 守卫 + 打开

const showRenameDialog = ref(false)
const renameTarget = ref('')

const handleRename = (name: string) => {
  if (!canManageAuthAccounts.value) {
    authActionError.value = profileGuardMessage.value
    return
  }
  renameTarget.value = name
  showRenameDialog.value = true
}

// 添加账号弹窗自身负责 OAuth / 导入 / API / 本地全部子流程（AddCodexAccountModal）；
// 主视图仅以默认方式或携带提供商预设打开它，成功后由 mutated 事件触发 handleRefresh
const openAddAccountModal = () => {
  addAccountInitialMethod.value = 'oauth'
  addAccountPresetProvider.value = null
  showAddAccountModal.value = true
}

const handleUseProviderInApiForm = (provider: CodexModelProviderRecord) => {
  addAccountInitialMethod.value = 'api'
  addAccountPresetProvider.value = provider
  showAddAccountModal.value = true
}

const resetProviderForm = () => {
  providerForm.id = ''
  providerForm.name = ''
  providerForm.baseUrl = ''
  providerForm.websiteUrl = ''
  providerForm.apiKeyUrl = ''
  providerForm.apiKeyName = 'API Key'
  providerForm.apiKey = ''
  providerError.value = null
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const editProvider = (provider: CodexModelProviderRecord) => {
  providerForm.id = provider.id
  providerForm.name = provider.name
  providerForm.baseUrl = provider.base_url
  providerForm.websiteUrl = provider.website_url || ''
  providerForm.apiKeyUrl = provider.api_key_url || ''
  providerForm.apiKeyName = provider.api_keys[0]?.name || 'API Key'
  providerForm.apiKey = provider.api_keys[0]?.api_key || ''
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  activeManagerTab.value = 'providers'
}

const useManualProviderTemplate = () => {
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyCodexProviderTemplate = (selection: ProviderTemplateSelection) => {
  const patch = mapTemplateToCodexProviderPatch(selection.template, selection.endpoint)

  selectedProviderTemplate.value = selection.template.id
  selectedProviderEndpoint.value = selection.endpoint || ''
  providerForm.name = patch.name || selection.template.name
  providerForm.baseUrl = patch.baseUrl || ''
  providerForm.websiteUrl = patch.websiteUrl || ''
  providerForm.apiKeyUrl = patch.apiKeyUrl || ''
  providerError.value = null
}

const handleSaveProvider = async () => {
  providerError.value = null
  if (!providerForm.name.trim()) {
    providerError.value = tf(
      'codex.auth.providers.validation.nameRequired',
      'Provider name is required.'
    )
    return
  }
  if (!providerForm.baseUrl.trim()) {
    providerError.value = tf(
      'codex.auth.providers.validation.baseUrlRequired',
      'Base URL is required.'
    )
    return
  }

  try {
    providerSaving.value = true
    await codexSaveModelProvider<{ provider: CodexModelProviderRecord }>({
      id: providerForm.id || undefined,
      name: providerForm.name.trim(),
      baseUrl: providerForm.baseUrl.trim(),
      websiteUrl: providerForm.websiteUrl.trim() || null,
      apiKeyUrl: providerForm.apiKeyUrl.trim() || null,
      apiKeyName: providerForm.apiKeyName.trim() || null,
      apiKey: providerForm.apiKey.trim() || null,
    })
    await loadProviders()
    resetProviderForm()
    uiStore.showSuccess(
      tf('codex.auth.providers.saveSuccess', 'Saved provider saved successfully.')
    )
  } catch (error) {
    providerError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.providers.saveFailed', 'Failed to save the saved provider.')
  } finally {
    providerSaving.value = false
  }
}

const requestDeleteProvider = (provider: CodexModelProviderRecord) => {
  openConfirmDialog({
    title: tf('codex.auth.providers.deleteTitle', 'Delete saved provider'),
    message: tf(
      'codex.auth.providers.deleteMessage',
      'Delete saved provider "{name}"? Stored API keys under this saved provider will also be removed.',
      { name: provider.name }
    ),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      try {
        await codexDeleteModelProvider(provider.id)
        await loadProviders()
        uiStore.showSuccess(
          tf('codex.auth.providers.deleteSuccess', 'Saved provider deleted successfully.')
        )
      } catch (error) {
        providerError.value =
          extractErrorMessage(error) ||
          tf('codex.auth.providers.deleteFailed', 'Failed to delete the saved provider.')
      }
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
/* codex-auth-view__* 结构样式已抽至全局共享层 src/styles/codex-auth-shared.css，
   供主视图、Accounts/Providers Tab 与各 Modal 共用，此处不再重复定义。 */

</style>
