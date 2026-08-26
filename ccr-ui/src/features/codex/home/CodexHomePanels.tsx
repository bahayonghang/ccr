import type { CodexDashboardUsageSummary } from '@/api'
import { EmptyState, buttonClass } from '@/ui'
import type { CodexDashboardActionItem, CodexDashboardInventoryItem } from '../dashboard-model'
import { panelCardClass } from '../ui-classes'
import { ManageRow, NextActionRow } from './CodexHomeCards'
import type { TranslateFunction } from '@/utils/tf'

interface UsageStripProps {
  t: TranslateFunction
  usageSummary: CodexDashboardUsageSummary | null
  overviewModel?: string | null
  usageLoading: boolean
  usageTotalRequests: string | number
  usageTotalTokens: string
  formatDateTime: (value?: string | null) => string
}

export function UsageStrip({
  t,
  usageSummary,
  overviewModel,
  usageLoading,
  usageTotalRequests,
  usageTotalTokens,
  formatDateTime,
}: UsageStripProps) {
  const model = usageSummary?.top_model?.model || overviewModel
  const modelLabel = model || (usageLoading ? t('codex.dashboard.usage.loading') : t('codex.dashboard.usage.unknownModel'))
  const activity = usageSummary?.last_activity_at
    ? formatDateTime(usageSummary.last_activity_at)
    : usageLoading
      ? t('codex.dashboard.usage.loading')
      : t('codex.dashboard.usage.noActivity')
  return (
    <div className="mt-4 grid grid-cols-2 gap-2 rounded-3xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] p-2 lg:grid-cols-4">
      <UsageChip label={t('codex.dashboard.usage.requests')} value={String(usageTotalRequests)} />
      <UsageChip label={t('codex.dashboard.usage.tokens')} value={usageTotalTokens} />
      <UsageChip label={t('codex.dashboard.usage.model')} value={modelLabel} />
      <UsageChip label={t('codex.dashboard.usage.lastActivity')} value={activity} />
    </div>
  )
}

function UsageChip({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-2xl px-3 py-2">
      <span className="block text-[0.68rem] font-semibold uppercase tracking-[0.12em] text-[color:var(--stage-text-quiet)]">{label}</span>
      <strong className="mt-1 block truncate text-sm font-semibold text-[color:var(--stage-text-primary)]">{value}</strong>
    </div>
  )
}

interface ActionConsoleProps {
  t: TranslateFunction
  error: string | null
  overviewMissing: boolean
  visibleNextActions: CodexDashboardActionItem[]
  overviewLoading: boolean
  onRefresh: () => void
}

export function ActionConsole({ t, error, overviewMissing, visibleNextActions, overviewLoading, onRefresh }: ActionConsoleProps) {
  return (
    <div className={`${panelCardClass} xl:col-span-3`}>
      <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--stage-text-quiet)]">{t('codex.dashboard.actionConsole.eyebrow')}</p>
      <h2 className="mt-1 text-lg font-semibold text-[color:var(--stage-text-primary)]">{t('codex.dashboard.actionConsole.title')}</h2>
      <p className="mb-4 max-w-xl text-sm leading-6 text-[color:var(--stage-text-secondary)]">{t('codex.dashboard.actionConsole.subtitle')}</p>
      {error && overviewMissing ? (
        <div className="mb-4 flex flex-col gap-3 rounded-3xl border border-accent-danger/20 bg-accent-danger/10 p-4 text-sm text-accent-danger lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="font-semibold">{t('codex.dashboard.error.title')}</p>
            <p className="mt-1 break-words">{error}</p>
          </div>
          <button type="button" className={buttonClass({ variant: 'ghost' })} onClick={onRefresh}>{t('codex.dashboard.header.refresh')}</button>
        </div>
      ) : null}
      {visibleNextActions.length > 0 ? (
        <div className="space-y-3">
          {visibleNextActions.map((action, index) => (
            <NextActionRow key={action.title} action={action} index={index} />
          ))}
        </div>
      ) : overviewLoading ? (
        <div className="space-y-3">
          <div className="h-28 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
          <div className="h-28 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
        </div>
      ) : (
        <EmptyState icon="Route" title={t('codex.dashboard.empty.actionsTitle')} description={t('codex.dashboard.empty.actionsDescription')} actionText={t('codex.dashboard.header.refresh')} actionIcon="RefreshCw" onAction={onRefresh} />
      )}
    </div>
  )
}

interface ManagePanelProps {
  t: TranslateFunction
  compactInventory: CodexDashboardInventoryItem[]
  overviewLoading: boolean
  onRefresh: () => void
}

export function ManagePanel({ t, compactInventory, overviewLoading, onRefresh }: ManagePanelProps) {
  return (
    <div className={`${panelCardClass} xl:col-span-2`}>
      <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--stage-text-quiet)]">{t('codex.dashboard.management.eyebrow')}</p>
      <h2 className="mt-1 text-lg font-semibold text-[color:var(--stage-text-primary)]">{t('codex.dashboard.management.title')}</h2>
      <p className="mb-3 max-w-xl text-sm leading-6 text-[color:var(--stage-text-secondary)]">{t('codex.dashboard.management.subtitle')}</p>
      {compactInventory.length > 0 ? (
        <div className="space-y-2">
          {compactInventory.map((item) => (
            <ManageRow key={item.key} item={item} />
          ))}
        </div>
      ) : overviewLoading ? (
        <div className="space-y-2">
          <div className="h-16 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
          <div className="h-16 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
        </div>
      ) : (
        <EmptyState icon="Folders" title={t('codex.dashboard.empty.managementTitle')} description={t('codex.dashboard.empty.managementDescription')} actionText={t('codex.dashboard.header.refresh')} actionIcon="RefreshCw" onAction={onRefresh} />
      )}
    </div>
  )
}
