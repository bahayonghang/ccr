import { useCallback, useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router'
import { useForm } from 'react-hook-form'
import {
  deleteClaudeAuth,
  getClaudeAuthCurrent,
  listClaudeAuthAccounts,
  saveClaudeAuth,
  switchClaudeAuth,
} from '@/api'
import { claudeAuthOff, claudeProfileOff, listClaudeProfiles } from '@/api/domains/claude'
import { surfaceNotify } from '@/configs/surfaceNotify'
import {
  AuthAccountsPanel,
  AuthCurrentPanel,
  AuthDiagnosis,
  AuthStatCard,
} from '@/features/claude/auth/AuthPanels'
import {
  authOwnershipLabel,
  currentProfileLabel,
  extractAuthError,
  formatAuthSource,
  loginStateLabel,
  runtimeModeLabel,
  authConfidenceLabel,
} from '@/features/claude/auth/authLabels'
import { ClaudeSubnav } from '@/features/claude/ClaudeSubnav'
import { t, tt } from '@/features/claude/locale'
import type {
  ClaudeAuthAccountItem,
  ClaudeAuthCurrentInfo,
  ClaudeAuthSaveRequest,
  ClaudeLoginState,
  ClaudeRuntimeSummary,
} from '@/types'
import { BaseModal, PageHeader, PageShell } from '@/ui'
import { logger } from '@/utils/logger'

interface SaveForm {
  name: string
  description: string
  force: boolean
}

const CREDENTIALS_FILE = '~/.claude/.credentials.json'

/** Claude 官方账号快照（OAuth 余量）。session/auth-off 语义与 BaseAuth 对齐。 */
export function ClaudeAuthView() {
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [busyName, setBusyName] = useState<string | null>(null)
  const [showSaveForm, setShowSaveForm] = useState(false)
  const [authActionError, setAuthActionError] = useState<string | null>(null)
  const [accounts, setAccounts] = useState<ClaudeAuthAccountItem[]>([])
  const [currentInfo, setCurrentInfo] = useState<ClaudeAuthCurrentInfo | null>(null)
  const [runtimeSummary, setRuntimeSummary] = useState<ClaudeRuntimeSummary | null>(null)
  const [loginState, setLoginState] = useState<ClaudeLoginState>({ type: 'NotLoggedIn' })
  const [canOff, setCanOff] = useState(false)
  const [canAuthOff, setCanAuthOff] = useState(false)
  const form = useForm<SaveForm>({ defaultValues: { name: '', description: '', force: false } })
  const { register, handleSubmit, reset } = form

  const refreshAll = useCallback(async () => {
    try {
      setLoading(true)
      setAuthActionError(null)
      const [accountsData, currentData, profilesData] = await Promise.all([
        listClaudeAuthAccounts(),
        getClaudeAuthCurrent(),
        listClaudeProfiles().catch(() => ({ can_off: false })),
      ])
      setAccounts(accountsData.accounts || [])
      setRuntimeSummary(accountsData.runtime_summary || currentData.runtime_summary)
      setLoginState(accountsData.login_state || currentData.login_state || { type: 'NotLoggedIn' })
      setCurrentInfo(currentData.info || null)
      setCanOff(profilesData.can_off === true)
      setCanAuthOff(accountsData.can_auth_off === true || currentData.can_auth_off === true)
    } catch (error) {
      logger.error('Failed to load Claude auth data:', error)
      const message = extractAuthError(error)
      setAuthActionError(message)
      surfaceNotify.error(message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void refreshAll()
  }, [refreshAll])

  const openSave = useCallback(() => setShowSaveForm(true), [])
  const closeSave = useCallback(() => setShowSaveForm(false), [])
  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) setShowSaveForm(false)
  }, [])

  const onSaveValid = useCallback(
    async (values: SaveForm) => {
      if (!values.name.trim()) {
        surfaceNotify.error(tt('账号名称不能为空', 'Account name is required'))
        return
      }
      try {
        setSaving(true)
        setAuthActionError(null)
        const payload: ClaudeAuthSaveRequest = {
          name: values.name.trim(),
          description: values.description.trim() || null,
          force: values.force,
        }
        await saveClaudeAuth(payload)
        surfaceNotify.success(tt('Claude 官方账号已保存', 'Claude official account saved'))
        setShowSaveForm(false)
        reset({ name: '', description: '', force: false })
        await refreshAll()
      } catch (error) {
        logger.error('Failed to save Claude auth:', error)
        const message = extractAuthError(error)
        setAuthActionError(message)
        surfaceNotify.error(message)
      } finally {
        setSaving(false)
      }
    },
    [refreshAll, reset],
  )
  const onSave = useMemo(() => handleSubmit(onSaveValid), [handleSubmit, onSaveValid])

  const handleAuthOff = useCallback(async () => {
    const confirmed = await surfaceNotify.confirm({
      title: t('auth.confirmOffTitle'),
      message: t('auth.confirmOffClaude'),
      confirmText: t('auth.off'),
      cancelText: t('common.cancel'),
      type: 'warning',
    })
    if (!confirmed) return
    try {
      setLoading(true)
      setAuthActionError(null)
      const result = await claudeAuthOff()
      surfaceNotify.success(result.changed ? t('auth.offSuccess') : t('auth.offUnchanged'))
      for (const warning of result.warnings) surfaceNotify.warning(warning)
      await refreshAll()
    } catch (error) {
      logger.error('Failed to log out Claude official session:', error)
      const message = extractAuthError(error)
      setAuthActionError(message)
      surfaceNotify.error(message || t('auth.offFailed'))
    } finally {
      setLoading(false)
    }
  }, [refreshAll])

  const handleProfileOff = useCallback(async () => {
    const confirmed = await surfaceNotify.confirm({
      title: tt('退出 Profile', 'Exit profile'),
      message: tt(
        '退出当前 Profile 并清理会压制官方登录的 CCR 运行时残留？已保存的账号不会删除。',
        'Exit the current profile and clear CCR leftovers that can suppress official login? Saved accounts stay.',
      ),
      confirmText: tt('退出 Profile', 'Exit profile'),
      cancelText: tt('取消', 'Cancel'),
      type: 'warning',
    })
    if (!confirmed) return
    try {
      setLoading(true)
      setAuthActionError(null)
      const result = await claudeProfileOff()
      surfaceNotify.success(tt('已退出 Profile 并清理登录残留', 'Exited profile mode and cleared login leftovers'))
      const warnings = result.remaining_suppressors.map((source) =>
        tt(
          `退出 Profile 后仍存在${authOwnershipLabel(source.ownership)}认证来源：${formatAuthSource(source)}（${authConfidenceLabel(source.confidence)}）`,
          `${authOwnershipLabel(source.ownership)} auth source remains after exit profile: ${formatAuthSource(source)} (${authConfidenceLabel(source.confidence)})`,
        ),
      )
      for (const warning of warnings.length > 0 ? warnings : result.warnings) surfaceNotify.warning(warning)
      await refreshAll()
    } catch (error) {
      logger.error('Failed to exit Claude profile mode:', error)
      const message = extractAuthError(error)
      setAuthActionError(message)
      surfaceNotify.error(message)
    } finally {
      setLoading(false)
    }
  }, [refreshAll])

  const handleSwitch = useCallback(
    async (name: string) => {
      const confirmed = await surfaceNotify.confirm({
        title: tt('切换官方账号', 'Switch official account'),
        message: tt(`确定切换到官方账号 "${name}" 吗？`, `Switch to official account "${name}"?`),
        confirmText: tt('切换', 'Switch'),
        cancelText: tt('取消', 'Cancel'),
        type: 'warning',
      })
      if (!confirmed) return
      try {
        setBusyName(name)
        setAuthActionError(null)
        const result = await switchClaudeAuth(name)
        const clearedCount = result.cleared_managed_sources.length
        surfaceNotify.success(
          clearedCount > 0
            ? tt(`已切换到 ${name}，并清理 ${clearedCount} 个 CCR 托管设置`, `Switched to ${name} and cleared ${clearedCount} CCR-managed setting(s)`)
            : tt(`已切换到 ${name}`, `Switched to ${name}`),
        )
        const structured = result.remaining_suppressors.map((source) =>
          tt(
            `切换后仍存在${authOwnershipLabel(source.ownership)}认证来源：${formatAuthSource(source)}（${authConfidenceLabel(source.confidence)}）`,
            `${authOwnershipLabel(source.ownership)} auth source remains after switching: ${formatAuthSource(source)} (${authConfidenceLabel(source.confidence)})`,
          ),
        )
        for (const warning of structured.length > 0 ? structured : result.warnings) surfaceNotify.warning(warning)
        await refreshAll()
      } catch (error) {
        logger.error('Failed to switch Claude auth:', error)
        const message = extractAuthError(error)
        setAuthActionError(message)
        surfaceNotify.error(message)
      } finally {
        setBusyName(null)
      }
    },
    [refreshAll],
  )

  const handleDelete = useCallback(
    async (name: string) => {
      const confirmed = await surfaceNotify.confirm({
        title: tt('删除官方账号', 'Delete official account'),
        message: tt(`确定删除官方账号 "${name}" 吗？`, `Delete official account "${name}"?`),
        confirmText: tt('删除', 'Delete'),
        cancelText: tt('取消', 'Cancel'),
        type: 'danger',
      })
      if (!confirmed) return
      try {
        setBusyName(name)
        setAuthActionError(null)
        await deleteClaudeAuth(name)
        surfaceNotify.success(tt(`已删除 ${name}`, `Deleted ${name}`))
        await refreshAll()
      } catch (error) {
        logger.error('Failed to delete Claude auth:', error)
        const message = extractAuthError(error)
        setAuthActionError(message)
        surfaceNotify.error(message)
      } finally {
        setBusyName(null)
      }
    },
    [refreshAll],
  )

  const header = (
    <PageHeader
      title={tt('官方账号管理', 'Official account management')}
      eyebrow={tt('Claude 官方订阅', 'Claude Official Subscription')}
      description={tt(
        `保存、切换、删除 Claude Code 官方订阅账号快照；切换会更新 ${CREDENTIALS_FILE}，并只清理 CCR 托管的 Profile 设置。`,
        `Save, switch, or delete Claude Code official subscription snapshots. Switching updates ${CREDENTIALS_FILE} and clears only CCR-managed profile settings.`,
      )}
      actions={
        <div className="flex flex-wrap gap-2">
          <Link to="/claude-code" className="inline-flex rounded-lg border border-border-default px-3 py-2 text-sm">
            {tt('返回 Claude Code', 'Back to Claude Code')}
          </Link>
          <button type="button" className="rounded-lg border border-border-default px-3 py-2 text-sm" disabled={loading} onClick={refreshAll}>
            {tt('刷新', 'Refresh')}
          </button>
          <button type="button" className="rounded-lg bg-accent-primary px-3 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]" disabled={saving} onClick={openSave}>
            {tt('保存当前登录', 'Save current login')}
          </button>
        </div>
      }
    />
  )

  return (
    <PageShell className="claude-auth-view" header={header} subnav={<ClaudeSubnav />}>
      <div className="grid gap-4">
        {authActionError ? (
          <div className="rounded-2xl border border-accent-danger/25 bg-accent-danger/10 px-4 py-3 text-accent-danger">
            {authActionError}
          </div>
        ) : null}
        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <AuthStatCard label={tt('登录状态', 'Login state')} value={loginStateLabel(loginState)} />
          <AuthStatCard label={tt('运行时模式', 'Runtime mode')} value={runtimeModeLabel(runtimeSummary)} />
          <AuthStatCard label={tt('当前 Profile', 'Current profile')} value={currentProfileLabel(runtimeSummary)} />
          <AuthStatCard label={tt('已保存账号', 'Saved accounts')} value={String(accounts.length)} />
        </section>
        {runtimeSummary ? (
          <AuthDiagnosis
            summary={runtimeSummary}
            canAuthOff={canAuthOff}
            canOff={canOff}
            loading={loading}
            onAuthOff={handleAuthOff}
            onProfileOff={handleProfileOff}
          />
        ) : null}
        {currentInfo ? <AuthCurrentPanel info={currentInfo} /> : null}
        <AuthAccountsPanel
          loading={loading}
          accounts={accounts}
          busyName={busyName}
          onSave={openSave}
          onSwitch={handleSwitch}
          onDelete={handleDelete}
        />
      </div>
      <BaseModal
        modelValue={showSaveForm}
        title={tt('保存当前官方登录', 'Save current official login')}
        size="md"
        surface="solid"
        onUpdateModelValue={handleOpenChange}
        onClose={closeSave}
        footer={
          <div className="flex w-full gap-4">
            <button type="button" className="flex-1 rounded-full border border-[color:var(--stage-border-soft)] px-4 py-3 text-sm" onClick={closeSave}>
              {tt('取消', 'Cancel')}
            </button>
            <button
              type="button"
              className="flex-1 rounded-full border border-accent-primary/25 bg-accent-primary/12 px-4 py-3 text-sm font-semibold text-accent-primary"
              disabled={saving}
              onClick={onSave}
            >
              {saving ? tt('保存中…', 'Saving...') : tt('保存', 'Save')}
            </button>
          </div>
        }
      >
        <p className="mt-2 text-sm leading-6 text-[color:var(--stage-text-secondary)]">
          {tt('当前必须已经通过 `claude login` 拿到官方登录，CCR 只负责保存快照和切换。', 'You must already have an official login from `claude login`. CCR only saves and switches snapshots.')}
        </p>
        <label className="mt-4 grid gap-2 text-sm font-semibold text-[color:var(--stage-text-primary)]">
          <span>{tt('账号名称', 'Account name')}</span>
          <input
            type="text"
            className="rounded-xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-3 py-3"
            placeholder={tt('例如 work / personal', 'e.g. work / personal')}
            {...register('name')}
          />
        </label>
        <label className="mt-4 grid gap-2 text-sm font-semibold text-[color:var(--stage-text-primary)]">
          <span>{tt('描述（可选）', 'Description (optional)')}</span>
          <input
            type="text"
            className="rounded-xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-3 py-3"
            placeholder={tt('例如 公司订阅 / 个人订阅', 'e.g. company plan / personal plan')}
            {...register('description')}
          />
        </label>
        <label className="mt-4 inline-flex items-center gap-2 text-sm text-[color:var(--stage-text-primary)]">
          <input type="checkbox" {...register('force')} />
          <span>{tt('覆盖同名账号', 'Overwrite same-name account')}</span>
        </label>
      </BaseModal>
    </PageShell>
  )
}
