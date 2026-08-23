import { useCallback, useEffect, useMemo, useState } from 'react'
import { deleteCodexAuth, getCodexAllQuotas, getCodexAuthCurrent, listCodexAuthAccounts, listCodexProfiles, switchCodexAuth } from '@/api'
import { codexAuthOff, codexProfileOff } from '@/api/domains/codex'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { CodexAccountQuota, CodexAuthAccountItem, CodexAuthCurrentInfo, CodexModelProviderRecord, CodexProfile, LoginState } from '@/types'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import type { TranslateFunction } from '@/utils/tf'
import type { CodexTf } from '../useCodexLocale'
import {
  filterAndSortCodexAccounts,
  usesOpenAiAuthMode,
  type AccountPlanFilter,
  type AccountSort,
  type AccountStatusFilter,
} from '../codexAuthAccounts'
import { useCodexProviders } from './useCodexProviders'

export type ManagerTab = 'accounts' | 'providers'
export type ConfirmState = {
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
  action: () => Promise<void>
} | null

export function useCodexAuthPage(t: TranslateFunction, tf: CodexTf) {
  const [loading, setLoading] = useState(false)
  const [actionLoading, setActionLoading] = useState(false)
  const [quotaLoading, setQuotaLoading] = useState(false)
  const [accounts, setAccounts] = useState<CodexAuthAccountItem[]>([])
  const [loginState, setLoginState] = useState<LoginState>({ type: 'NotLoggedIn' })
  const [currentInfo, setCurrentInfo] = useState<CodexAuthCurrentInfo | null>(null)
  const [currentProfile, setCurrentProfile] = useState<CodexProfile | null>(null)
  const [canOff, setCanOff] = useState(false)
  const [canAuthOff, setCanAuthOff] = useState(false)
  const [authActionError, setAuthActionError] = useState<string | null>(null)
  const [quotaMap, setQuotaMap] = useState<Map<string, CodexAccountQuota>>(new Map())
  const [activeManagerTab, setActiveManagerTab] = useState<ManagerTab>('accounts')
  const [showSaveForm, setShowSaveForm] = useState(false)
  const [showAddAccountModal, setShowAddAccountModal] = useState(false)
  const [addAccountInitialMethod, setAddAccountInitialMethod] = useState<'oauth' | 'token' | 'api' | 'local'>('oauth')
  const [addAccountPresetProvider, setAddAccountPresetProvider] = useState<CodexModelProviderRecord | null>(null)
  const [busyName, setBusyName] = useState<string | null>(null)
  const [busyAction, setBusyAction] = useState<'switch' | 'delete' | null>(null)
  const [confirmState, setConfirmState] = useState<ConfirmState>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<AccountStatusFilter>('all')
  const [planFilter, setPlanFilter] = useState<AccountPlanFilter>('all')
  const [sortBy, setSortBy] = useState<AccountSort>('saved_desc')
  const [showRenameDialog, setShowRenameDialog] = useState(false)
  const [renameTarget, setRenameTarget] = useState('')

  const openConfirmDialog = useCallback((options: NonNullable<ConfirmState>) => {
    setConfirmState(options)
  }, [])
  const providersApi = useCodexProviders({ t, openConfirmDialog, setActiveManagerTab })
  const loadProviders = providersApi.loadProviders

  const canManageAuthAccounts = usesOpenAiAuthMode(currentProfile?.auth_mode)
  const currentAccount = accounts.find((account) => account.is_current)
  const profileGuardMessage = !currentProfile
    ? t('codex.auth.profileGuard.noCurrentProfile')
    : canManageAuthAccounts
      ? tf('codex.auth.profileGuard.supportedProfile', 'Current profile "{name}" uses "{authMode}". Auth account save/switch is available.', {
          name: currentProfile.name,
          authMode: currentProfile.auth_mode || 'openai_chatgpt',
        })
      : tf('codex.auth.profileGuard.unsupportedProfile', 'Current profile "{name}" uses "{authMode}". Codex Auth account save/switch only works for OpenAI-auth current profiles.', {
          name: currentProfile.name,
          authMode: currentProfile.auth_mode || 'no_auth',
        })
  const canSave = canManageAuthAccounts && (loginState.type === 'LoggedInUnsaved' || loginState.type === 'LoggedInSaved')

  const formatAuthMethod = useCallback(
    (method?: string | null) => {
      if (method === 'chatgpt') return tf('codex.auth.authMethods.chatgpt', 'ChatGPT OAuth')
      if (method === 'api') return tf('codex.auth.authMethods.api', 'API Key')
      if (method === 'provider') return tf('codex.auth.authMethods.provider', 'Provider key')
      return tf('codex.auth.authMethods.unknown', 'Unknown')
    },
    [tf],
  )

  const loadAll = useCallback(async () => {
    setLoading(true)
    setAuthActionError(null)
    try {
      const [accountData, current, profileData, quotas] = await Promise.all([
        listCodexAuthAccounts(),
        getCodexAuthCurrent(),
        listCodexProfiles(),
        getCodexAllQuotas().catch(() => []),
      ])
      setAccounts(accountData.accounts || [])
      setLoginState(accountData.login_state ?? { type: 'NotLoggedIn' })
      setCanAuthOff(accountData.can_auth_off === true || current.can_auth_off === true)
      setCurrentInfo(current.logged_in && current.info ? current.info : null)
      setCanOff(profileData.can_off === true)
      setCurrentProfile(profileData.profiles?.find((profile) => profile.name === profileData.current_profile) || null)
      const map = new Map<string, CodexAccountQuota>()
      for (const quota of Array.isArray(quotas) ? quotas : []) map.set(quota.account_name, quota)
      setQuotaMap(map)
      await loadProviders()
    } catch (error) {
      logger.error('Failed to load codex auth:', error)
      surfaceNotify.error(extractErrorMessage(error) || t('codex.states.loadFailed'))
    } finally {
      setLoading(false)
    }
  }, [loadProviders, t])

  useEffect(() => {
    void loadAll()
  }, [loadAll])

  const handleAuthOff = useCallback(async () => {
    const ok = await surfaceNotify.confirm({
      title: t('auth.confirmOffTitle'),
      message: t('auth.confirmOffCodex'),
      confirmText: t('auth.off'),
      cancelText: t('common.cancel'),
      type: 'warning',
    })
    if (!ok) return
    try {
      setLoading(true)
      const result = await codexAuthOff()
      surfaceNotify.success(result.changed ? t('auth.offSuccess') : t('auth.offUnchanged'))
      for (const warning of result.warnings) surfaceNotify.warning(warning)
      await loadAll()
    } catch (error) {
      logger.error('Failed to log out Codex official session:', error)
      surfaceNotify.error(extractErrorMessage(error) || t('auth.offFailed'))
    } finally {
      setLoading(false)
    }
  }, [loadAll, t])

  const handleOff = useCallback(async () => {
    const ok = await surfaceNotify.confirm({
      title: t('codex.auth.off.title'),
      message: t('codex.auth.off.confirm'),
      confirmText: t('codex.auth.off.action'),
      cancelText: t('common.cancel'),
      type: 'warning',
    })
    if (!ok) return
    try {
      setLoading(true)
      await codexProfileOff()
      surfaceNotify.success(t('codex.auth.off.success'))
      await loadAll()
    } catch (error) {
      logger.error('Failed to exit Codex profile mode:', error)
      surfaceNotify.error(extractErrorMessage(error) || t('codex.auth.off.failed'))
    } finally {
      setLoading(false)
    }
  }, [loadAll, t])

  const handleSwitch = useCallback(
    (name: string) => {
      openConfirmDialog({
        title: t('codex.auth.switch'),
        message: tf('codex.auth.confirmSwitch', '确定要切换到账户 "{name}" 吗？', { name }),
        confirmText: t('codex.auth.switch'),
        type: 'warning',
        action: async () => {
          setBusyName(name)
          setBusyAction('switch')
          try {
            await switchCodexAuth(name)
            await loadAll()
            surfaceNotify.success(tf('codex.auth.feedback.switchSuccess', 'Switched account successfully.'))
          } catch (error) {
            const message = extractErrorMessage(error) || t('codex.states.saveFailed')
            setAuthActionError(message)
            surfaceNotify.error(message)
          } finally {
            setBusyName(null)
            setBusyAction(null)
          }
        },
      })
    },
    [loadAll, openConfirmDialog, t, tf],
  )

  const handleDelete = useCallback(
    (name: string) => {
      openConfirmDialog({
        title: t('codex.actions.delete'),
        message: tf('codex.auth.deleteConfirm', '确定要删除账户 "{name}" 吗？', { name }),
        confirmText: t('codex.actions.delete'),
        type: 'danger',
        action: async () => {
          setBusyName(name)
          setBusyAction('delete')
          try {
            await deleteCodexAuth(name)
            await loadAll()
            surfaceNotify.success(tf('codex.auth.feedback.deleteSuccess', 'Account deleted successfully.'))
          } catch (error) {
            const message = extractErrorMessage(error) || t('codex.states.deleteFailed')
            setAuthActionError(message)
            surfaceNotify.error(message)
          } finally {
            setBusyName(null)
            setBusyAction(null)
          }
        },
      })
    },
    [loadAll, openConfirmDialog, t, tf],
  )

  const handleComingSoon = useCallback((_name: string) => {
    surfaceNotify.success(t('codex.auth.featureComingSoon'))
  }, [t])
  const handleRename = useCallback(
    (name: string) => {
      if (!canManageAuthAccounts) {
        setAuthActionError(profileGuardMessage)
        return
      }
      setRenameTarget(name)
      setShowRenameDialog(true)
    },
    [canManageAuthAccounts, profileGuardMessage],
  )
  const openAddAccount = useCallback(() => {
    setAddAccountInitialMethod('oauth')
    setAddAccountPresetProvider(null)
    setShowAddAccountModal(true)
  }, [])
  const handleUseProviderInApiForm = useCallback((provider: CodexModelProviderRecord) => {
    setAddAccountInitialMethod('api')
    setAddAccountPresetProvider(provider)
    setShowAddAccountModal(true)
  }, [])
  const handleSave = useCallback(() => setShowSaveForm(true), [])
  const handleConfirm = useCallback(async () => {
    if (!confirmState) return
    setActionLoading(true)
    try {
      await confirmState.action()
    } finally {
      setActionLoading(false)
      setConfirmState(null)
    }
  }, [confirmState])
  const handleCancelConfirm = useCallback(() => setConfirmState(null), [])
  const clearFilters = useCallback(() => {
    setSearchQuery('')
    setStatusFilter('all')
    setPlanFilter('all')
    setSortBy('saved_desc')
  }, [])
  const loadQuotas = useCallback(async (_name?: string) => {
    setQuotaLoading(true)
    try {
      const data = await getCodexAllQuotas()
      const map = new Map<string, CodexAccountQuota>()
      for (const quota of data) map.set(quota.account_name, quota)
      setQuotaMap(map)
    } finally {
      setQuotaLoading(false)
    }
  }, [])

  const filteredAccounts = useMemo(
    () => filterAndSortCodexAccounts({ accounts, quotaMap, searchQuery, statusFilter, planFilter, sortBy }),
    [accounts, planFilter, quotaMap, searchQuery, sortBy, statusFilter],
  )

  return {
    loading,
    actionLoading,
    quotaLoading,
    accounts,
    loginState,
    currentInfo,
    canOff,
    canAuthOff,
    authActionError,
    quotaMap,
    activeManagerTab,
    setActiveManagerTab,
    showSaveForm,
    setShowSaveForm,
    showAddAccountModal,
    setShowAddAccountModal,
    addAccountInitialMethod,
    addAccountPresetProvider,
    busyName,
    busyAction,
    confirmState,
    searchQuery,
    setSearchQuery,
    statusFilter,
    setStatusFilter,
    planFilter,
    setPlanFilter,
    sortBy,
    setSortBy,
    showRenameDialog,
    setShowRenameDialog,
    renameTarget,
    providersApi,
    canManageAuthAccounts,
    currentAccount,
    profileGuardMessage,
    canSave,
    formatAuthMethod,
    loadAll,
    handleAuthOff,
    handleOff,
    handleSwitch,
    handleDelete,
    handleComingSoon,
    handleRename,
    openAddAccount,
    handleUseProviderInApiForm,
    handleSave,
    handleConfirm,
    handleCancelConfirm,
    clearFilters,
    loadQuotas,
    filteredAccounts,
  }
}
