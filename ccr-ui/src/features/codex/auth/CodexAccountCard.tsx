import { memo, useCallback } from 'react'
import type { CodexAccountQuota, CodexAuthAccountItem } from '@/types'
import { SIcon, cn } from '@/ui'
import { useAppT } from '@/i18n'

interface CodexAccountCardProps {
  account: CodexAuthAccountItem
  quota?: CodexAccountQuota | null
  quotaLoading?: boolean
  isCurrent?: boolean
  busyAction?: 'switch' | 'delete' | null
  disabled?: boolean
  onSwitch: (name: string) => void
  onDelete: (name: string) => void
  onRefresh: (name: string) => void
  onTag: (name: string) => void
  onExport: (name: string) => void
  onRename: (name: string) => void
}

const barColorClass = (pct: number) => {
  if (pct >= 60) return 'bg-accent-success'
  if (pct >= 30) return 'bg-accent-warning'
  return 'bg-accent-danger'
}

const textColorClass = (pct: number) => {
  if (pct >= 60) return 'text-accent-success'
  if (pct >= 30) return 'text-accent-warning'
  return 'text-accent-danger'
}

const planBadgeClass = (plan: string) => {
  const lower = plan.toLowerCase()
  if (lower === 'pro') return 'border-accent-secondary/30 bg-accent-secondary/20 text-accent-secondary'
  if (lower === 'plus') return 'border-accent-primary/30 bg-accent-primary/20 text-accent-primary'
  if (lower === 'team') return 'border-accent-success/30 bg-accent-success/20 text-accent-success'
  return 'border-border-default/15 bg-bg-elevated text-text-muted'
}

const authMethodLabel = (method?: string) => {
  if (method === 'chatgpt') return 'ChatGPT OAuth'
  if (method === 'api') return 'API Key'
  return 'Managed Auth'
}

const formatReset = (timestamp: number) => {
  const remaining = timestamp - Math.floor(Date.now() / 1000)
  if (remaining <= 0) return ''
  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)
  if (hours > 0) return `${hours}h${minutes}m`
  return `${minutes}m`
}

const formatDateTime = (raw?: string | null) => {
  if (!raw) return '—'
  const date = new Date(raw)
  if (Number.isNaN(date.getTime())) return raw
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

const ActionButton = memo(function ActionButton({
  title,
  disabled,
  icon,
  spinning,
  onClick,
}: {
  title: string
  disabled?: boolean
  icon: string
  spinning?: boolean
  onClick: () => void
}) {
  return (
    <button type="button" className="rounded-lg p-2 text-text-ghost hover:bg-bg-surface/70 hover:text-text-secondary" title={title} disabled={disabled} onClick={onClick}>
      <SIcon name={icon} size="w-4 h-4" className={spinning ? 'animate-spin' : undefined} />
    </button>
  )
})

function QuotaBlock({ quota, quotaLoading }: { quota?: CodexAccountQuota | null; quotaLoading?: boolean }) {
  const t = useAppT()
  if (quotaLoading) {
    return (
      <div className="mb-3 space-y-3">
        <div className="h-1.5 animate-pulse rounded-full bg-bg-elevated" />
        <div className="h-1.5 animate-pulse rounded-full bg-bg-elevated" />
      </div>
    )
  }
  if (quota?.quota) {
    return (
      <div className="mb-3 space-y-2.5">
        <QuotaBar label={t('codex.auth.hourlyQuota')} pct={quota.quota.hourly_percentage} reset={quota.quota.hourly_reset_time} />
        <QuotaBar label={t('codex.auth.weeklyQuota')} pct={quota.quota.weekly_percentage} />
      </div>
    )
  }
  if (quota?.error) {
    return <div className="mb-3 truncate rounded-lg border border-accent-danger/20 bg-accent-danger/10 px-2 py-1.5 text-xs text-accent-danger">{quota.error}</div>
  }
  return <div className="mb-3 text-xs text-text-disabled italic">{t('codex.auth.quotaNotQueried')}</div>
}

function QuotaBar({ label, pct, reset }: { label: string; pct: number; reset?: number }) {
  return (
    <div>
      <div className="mb-1 flex items-center justify-between">
        <span className="text-xs text-text-muted">{label}</span>
        <span className={cn('font-mono text-xs font-semibold', textColorClass(pct))}>{pct}%</span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-bg-elevated">
        <div className={cn('h-full origin-left rounded-full', barColorClass(pct))} style={{ transform: `scaleX(${Math.min(pct, 100) / 100})` }} />
      </div>
      {reset ? <span className="mt-0.5 block text-right text-[0.6875rem] text-text-disabled">{formatReset(reset)}</span> : null}
    </div>
  )
}

export const CodexAccountCard = memo(function CodexAccountCard({
  account,
  quota,
  quotaLoading,
  isCurrent,
  busyAction,
  disabled,
  onSwitch,
  onDelete,
  onRefresh,
  onTag,
  onExport,
  onRename,
}: CodexAccountCardProps) {
  const t = useAppT()
  const handleSwitch = useCallback(() => onSwitch(account.name), [account.name, onSwitch])
  const handleDelete = useCallback(() => onDelete(account.name), [account.name, onDelete])
  const handleRefresh = useCallback(() => onRefresh(account.name), [account.name, onRefresh])
  const handleTag = useCallback(() => onTag(account.name), [account.name, onTag])
  const handleExport = useCallback(() => onExport(account.name), [account.name, onExport])
  const handleRename = useCallback(() => onRename(account.name), [account.name, onRename])

  return (
    <article className={cn('rounded-2xl border border-border-default/15 bg-bg-surface p-4', isCurrent && 'border-l-2 border-l-accent-primary')}>
      <div className="mb-2 flex items-start justify-between gap-2">
        <span className="truncate text-base font-semibold text-text-primary">{account.email || account.name}</span>
        <div className="flex flex-wrap items-center justify-end gap-1.5">
          {isCurrent ? (
            <span className="rounded-full bg-accent-success/15 px-2 py-0.5 text-[0.625rem] font-semibold text-accent-success">
              {t('codex.auth.currentBadge')}
            </span>
          ) : null}
          {quota?.quota?.plan_type ? (
            <span className={cn('rounded-full border px-2 py-0.5 text-[0.625rem] font-semibold uppercase tracking-wider', planBadgeClass(quota.quota.plan_type))}>
              {quota.quota.plan_type}
            </span>
          ) : null}
        </div>
      </div>
      <div className="mb-3 space-y-0.5">
        <div className="flex items-center gap-2 text-sm text-text-muted">
          <span className="font-mono font-medium">{account.name}</span>
          {account.is_virtual ? <span className="text-text-ghost">({t('codex.auth.virtual')})</span> : null}
        </div>
        {account.description ? <p className="truncate text-xs text-text-ghost">{account.description}</p> : null}
        <div className="flex flex-wrap items-center gap-1.5 text-[0.6875rem] text-text-muted">
          <span className="rounded-md border border-border-default/15 px-1.5 py-0.5">{authMethodLabel(account.auth_method)}</span>
          {account.api_provider_name ? <span className="rounded-md border border-border-default/15 px-1.5 py-0.5">{account.api_provider_name}</span> : null}
        </div>
      </div>
      <QuotaBlock quota={quota} quotaLoading={quotaLoading} />
      <div className="flex items-center justify-between gap-3 border-t border-border-default/10 pt-2">
        <p className="truncate text-xs text-text-disabled">{formatDateTime(account.last_used)}</p>
        <div className="flex items-center gap-1">
          <ActionButton title={t('codex.auth.tagAccount')} disabled={disabled} icon="Tag" onClick={handleTag} />
          {account.is_virtual ? null : <ActionButton title={t('codex.auth.renameAccount')} disabled={disabled} icon="Pencil" onClick={handleRename} />}
          {isCurrent ? null : (
            <ActionButton title={t('codex.auth.switch')} disabled={disabled} icon={busyAction === 'switch' ? 'RefreshCw' : 'Play'} spinning={busyAction === 'switch'} onClick={handleSwitch} />
          )}
          <ActionButton title={t('codex.auth.refreshQuota')} disabled={disabled} icon="RefreshCw" onClick={handleRefresh} />
          <ActionButton title={t('codex.auth.exportAccount')} disabled={disabled} icon="Upload" onClick={handleExport} />
          {account.is_virtual ? null : (
            <ActionButton title={t('codex.actions.delete')} disabled={disabled} icon={busyAction === 'delete' ? 'RefreshCw' : 'Trash2'} spinning={busyAction === 'delete'} onClick={handleDelete} />
          )}
        </div>
      </div>
    </article>
  )
})
