import { memo, useCallback } from 'react'
import { tt } from '@/features/claude/locale'
import {
  authConfidenceLabel,
  authEvidenceLabel,
  authOwnershipLabel,
  authSourceKindLabel,
  authSourceLocationLabel,
  formatAuthDate,
  formatAuthSource,
  unobservableLabels,
} from '@/features/claude/auth/authLabels'
import type {
  ClaudeAuthAccountItem,
  ClaudeAuthCurrentInfo,
  ClaudeAuthSourceObservation,
  ClaudeRuntimeSummary,
} from '@/types'
import { EmptyState } from '@/ui'

const panelClass =
  'rounded-2xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-elevated)] px-4 py-4'
const ghostBtn =
  'inline-flex items-center justify-center rounded-full border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-4 py-3 text-sm font-semibold text-[color:var(--stage-text-primary)] disabled:cursor-not-allowed disabled:opacity-50'

export const AuthStatCard = memo(function AuthStatCard({ label, value }: { label: string; value: string }) {
  return (
    <article className={panelClass}>
      <p className="text-xs font-semibold text-[color:var(--stage-text-quiet)]">{label}</p>
      <p className="mt-1 text-lg font-bold text-[color:var(--stage-text-primary)]">{value}</p>
    </article>
  )
})

export function AuthDiagnosis({
  summary,
  canAuthOff,
  canOff,
  loading,
  onAuthOff,
  onProfileOff,
}: {
  summary: ClaudeRuntimeSummary
  canAuthOff: boolean
  canOff: boolean
  loading: boolean
  onAuthOff: () => void
  onProfileOff: () => void
}) {
  const diagnosis = summary.auth_diagnosis
  const visible = diagnosis.observations.filter((source) => source.suppresses_subscription)
  const presumed = diagnosis.presumed_effective_source
  const unobservable = unobservableLabels(diagnosis.unobservable)
  const warning = visible.length > 0
  return (
    <section className={panelClass} data-testid="claude-auth-diagnosis">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-bold text-[color:var(--stage-text-primary)]">
            {tt('认证来源诊断', 'Auth source diagnosis')}
          </h2>
          <p className="text-sm leading-5 text-[color:var(--stage-text-secondary)]">
            {tt('范围限于当前 CCR 进程和已解析的用户级文件。', 'Scope is limited to this CCR process and the resolved user-level files.')}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          {canAuthOff ? (
            <button type="button" className={ghostBtn} data-testid="claude-auth-off" disabled={loading} onClick={onAuthOff}>
              {tt('退出官方登录', 'Sign out official')}
            </button>
          ) : null}
          {canOff ? (
            <button type="button" className={ghostBtn} data-testid="claude-auth-profile-off" disabled={loading} onClick={onProfileOff}>
              {tt('退出 Profile', 'Exit profile')}
            </button>
          ) : null}
          <span
            className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${
              warning
                ? 'border-accent-warning/40 bg-accent-warning/10'
                : 'border-accent-success/40 bg-accent-success/10'
            }`}
          >
            {warning
              ? tt(`${visible.length} 个可见竞争来源`, `${visible.length} visible competing source(s)`)
              : tt('未发现可见竞争来源', 'No visible competing source')}
          </span>
        </div>
      </div>
      <dl className="mt-4 grid gap-4 border-y border-[color:var(--stage-border-soft)] py-4 md:grid-cols-3">
        <div>
          <dt className="text-xs font-semibold text-[color:var(--stage-text-quiet)]">{tt('当前推定来源', 'Presumed source')}</dt>
          <dd className="mt-1 text-[color:var(--stage-text-primary)]" data-testid="claude-auth-presumed-source">
            {presumed ? formatAuthSource(presumed) : tt('未解析或存在同级歧义', 'Unresolved or same-priority ambiguity')}
          </dd>
        </div>
        <div>
          <dt className="text-xs font-semibold text-[color:var(--stage-text-quiet)]">{tt('置信度', 'Confidence')}</dt>
          <dd className="mt-1 text-[color:var(--stage-text-primary)]">
            {presumed ? authConfidenceLabel(presumed.confidence) : '-'}
          </dd>
        </div>
        <div>
          <dt className="text-xs font-semibold text-[color:var(--stage-text-quiet)]">{tt('API Key 批准记录', 'API key response state')}</dt>
          <dd className="mt-1 text-[color:var(--stage-text-primary)]">
            {diagnosis.custom_api_key_responses_present
              ? tt('存在，仅作解释', 'Present, context only')
              : tt('未观察到', 'Not observed')}
          </dd>
        </div>
      </dl>
      {visible.length > 0 ? (
        <div className="mt-2">
          {visible.map((source) => (
            <SourceRow key={`${source.kind}-${source.location}-${source.ownership}`} source={source} />
          ))}
        </div>
      ) : null}
      <details className="mt-4 text-xs leading-5 text-[color:var(--stage-text-secondary)]">
        <summary className="w-fit cursor-pointer font-semibold text-[color:var(--stage-text-primary)]">
          {tt(`${unobservable.length} 个不可观测层`, `${unobservable.length} unobservable layer(s)`)}
        </summary>
        <ul className="mt-2 list-disc pl-5">
          {unobservable.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </details>
    </section>
  )
}

const SourceRow = memo(function SourceRow({ source }: { source: ClaudeAuthSourceObservation }) {
  return (
    <div className="grid gap-2 border-b border-[color:var(--stage-border-soft)] py-3 md:grid-cols-[minmax(13rem,1fr)_minmax(18rem,1.3fr)] md:items-center">
      <div className="flex flex-wrap items-center gap-2">
        <strong className="text-[color:var(--stage-text-primary)]">{authSourceKindLabel(source.kind)}</strong>
        <span className="text-xs text-[color:var(--stage-text-secondary)]">{authSourceLocationLabel(source.location)}</span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-full border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-2 py-0.5 text-xs text-[color:var(--stage-text-secondary)]">
          {authConfidenceLabel(source.confidence)}
        </span>
        <span className="rounded-full border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-2 py-0.5 text-xs text-[color:var(--stage-text-secondary)]">
          {authEvidenceLabel(source.evidence)}
        </span>
        <span className="rounded-full border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-2 py-0.5 text-xs text-[color:var(--stage-text-secondary)]">
          {authOwnershipLabel(source.ownership)}
        </span>
      </div>
    </div>
  )
})

export function AuthCurrentPanel({ info }: { info: ClaudeAuthCurrentInfo }) {
  const rows = [
    { label: tt('邮箱', 'Email'), value: info.email || '-' },
    { label: tt('账号 UUID', 'Account UUID'), value: info.account_uuid || '-' },
    { label: tt('订阅类型', 'Subscription type'), value: info.subscription_type || '-' },
    { label: tt('计费类型', 'Billing type'), value: info.billing_type || '-' },
    { label: tt('速率档位', 'Rate tier'), value: info.rate_limit_tier || '-' },
    { label: tt('Access Token 到期', 'Access token expiry'), value: info.expires_at ? formatAuthDate(info.expires_at) : '-' },
  ]
  return (
    <section className={panelClass}>
      <h2 className="text-lg font-bold text-[color:var(--stage-text-primary)]">
        {tt('当前运行时官方登录', 'Current runtime official login')}
      </h2>
      <div className="mt-4 grid gap-4 md:grid-cols-3">
        {rows.map((row) => (
          <div key={row.label}>
            <p className="text-sm text-[color:var(--stage-text-secondary)]">{row.label}</p>
            <p className="mt-1 break-words text-[0.95rem] text-[color:var(--stage-text-primary)]">{row.value}</p>
          </div>
        ))}
      </div>
    </section>
  )
}

const AccountRow = memo(function AccountRow({
  account,
  busy,
  onSwitch,
  onDelete,
}: {
  account: ClaudeAuthAccountItem
  busy: boolean
  onSwitch: (name: string) => void
  onDelete: (name: string) => void
}) {
  const handleSwitch = useCallback(() => {
    onSwitch(account.name)
  }, [account.name, onSwitch])
  const handleDelete = useCallback(() => {
    onDelete(account.name)
  }, [account.name, onDelete])
  const state = account.is_current
    ? tt('当前生效', 'Active now')
    : account.is_logged_in
      ? tt('已登录', 'Logged in')
      : tt('已保存', 'Saved')
  return (
    <tr>
      <td className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 align-top">
        <div className="flex items-center gap-3">
          <span>{account.name}</span>
          {account.is_current ? (
            <span className="rounded-full border border-accent-primary/25 bg-accent-primary/10 px-2 py-0.5 text-xs font-bold text-accent-primary">
              {tt('当前', 'Current')}
            </span>
          ) : null}
        </div>
        {account.description ? (
          <p className="text-sm text-[color:var(--stage-text-secondary)]">{account.description}</p>
        ) : null}
      </td>
      <td className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 align-top">{account.email || '-'}</td>
      <td className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 align-top">{account.subscription_type || '-'}</td>
      <td className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 align-top">
        {account.expires_at ? formatAuthDate(account.expires_at) : '-'}
      </td>
      <td className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 align-top">{state}</td>
      <td className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 align-top">
        <div className="flex items-center gap-3">
          <button type="button" className={ghostBtn} disabled={busy} onClick={handleSwitch}>
            {tt('切换', 'Switch')}
          </button>
          <button type="button" className={`${ghostBtn} text-accent-danger`} disabled={busy} onClick={handleDelete}>
            {tt('删除', 'Delete')}
          </button>
        </div>
      </td>
    </tr>
  )
})

export function AuthAccountsPanel({
  loading,
  accounts,
  busyName,
  onSave,
  onSwitch,
  onDelete,
}: {
  loading: boolean
  accounts: ClaudeAuthAccountItem[]
  busyName: string | null
  onSave: () => void
  onSwitch: (name: string) => void
  onDelete: (name: string) => void
}) {
  return (
    <section className={panelClass}>
      <h2 className="text-lg font-bold text-[color:var(--stage-text-primary)]">
        {tt('已保存账号快照', 'Saved account snapshots')}
      </h2>
      <p className="text-sm leading-5 text-[color:var(--stage-text-secondary)]">
        {tt('每个快照都保存当前 `claudeAiOauth`，切换时不会改写', 'Each snapshot keeps the current `claudeAiOauth`, and switching will not rewrite')}{' '}
        <code>~/.claude.json</code>
        {tt('。', '.')}
      </p>
      {loading ? (
        <div className="px-4 py-8 text-center text-[color:var(--stage-text-secondary)]">
          {tt('正在加载账号信息…', 'Loading account details...')}
        </div>
      ) : accounts.length === 0 ? (
        <EmptyState
          icon="User"
          title={tt('尚未保存任何官方账号快照。', 'No official account snapshots saved yet.')}
          actionText={tt('保存当前登录', 'Save current login')}
          actionIcon="Plus"
          onAction={onSave}
        />
      ) : (
        <div className="mt-4 overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              <tr>
                <th className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 text-left text-xs font-medium text-[color:var(--stage-text-quiet)]">{tt('名称', 'Name')}</th>
                <th className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 text-left text-xs font-medium text-[color:var(--stage-text-quiet)]">{tt('邮箱', 'Email')}</th>
                <th className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 text-left text-xs font-medium text-[color:var(--stage-text-quiet)]">{tt('订阅', 'Subscription')}</th>
                <th className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 text-left text-xs font-medium text-[color:var(--stage-text-quiet)]">{tt('到期', 'Expiry')}</th>
                <th className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 text-left text-xs font-medium text-[color:var(--stage-text-quiet)]">{tt('状态', 'State')}</th>
                <th className="border-b border-[color:var(--stage-border-soft)] px-3 py-3 text-left text-xs font-medium text-[color:var(--stage-text-quiet)]">{tt('操作', 'Actions')}</th>
              </tr>
            </thead>
            <tbody>
              {accounts.map((account) => (
                <AccountRow
                  key={account.name}
                  account={account}
                  busy={busyName === account.name}
                  onSwitch={onSwitch}
                  onDelete={onDelete}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
