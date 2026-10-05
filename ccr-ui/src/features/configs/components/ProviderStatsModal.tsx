import { useCallback, useMemo } from 'react'
import { BaseModal, SIcon } from '@/ui'
import { isZhLocale, t } from '../locale'
import type { ProviderSortMode } from '../types'
import { ProviderBar } from './ProviderBar'

interface ProviderStatsModalProps {
  visible: boolean
  providerUsage: Record<string, number>
  loading: boolean
  error: string | null
  sortMode: ProviderSortMode
  onClose: () => void
  onRefresh: () => void
  onUpdateSortMode: (value: ProviderSortMode) => void
}

const CHART_COLORS = [
  'var(--accent-success)',
  'var(--platform-gemini)',
  'var(--platform-codex)',
  'var(--accent-info)',
  'var(--accent-danger)',
]

export function ProviderStatsModal({
  visible,
  providerUsage,
  loading,
  error,
  sortMode,
  onClose,
  onRefresh,
  onUpdateSortMode,
}: ProviderStatsModalProps) {
  const entries = useMemo(() => Object.entries(providerUsage || {}), [providerUsage])
  const sorted = useMemo(() => {
    const next = [...entries]
    if (sortMode === 'count_asc') next.sort((left, right) => left[1] - right[1])
    else if (sortMode === 'name_asc') next.sort((left, right) => left[0].localeCompare(right[0]))
    else next.sort((left, right) => right[1] - left[1])
    return next
  }, [entries, sortMode])

  const maxCount = useMemo(() => {
    const values = entries.map(([, count]) => count)
    return values.length ? Math.max(...values) : 0
  }, [entries])
  const totalUsage = useMemo(() => entries.reduce((sum, [, count]) => sum + count, 0), [entries])
  const yTicks = useMemo(() => {
    const percents = [0, 25, 50, 75, 100]
    return percents.map((percent) => ({
      percent,
      value: maxCount === 0 ? 0 : Math.round((maxCount * percent) / 100),
    }))
  }, [maxCount])

  const sortLabel =
    sortMode === 'count_asc'
      ? t('configs.provider.sortModes.countAsc')
      : sortMode === 'name_asc'
        ? t('configs.provider.sortModes.nameAsc')
        : t('configs.provider.sortModes.countDesc')

  const handleSort = useCallback(
    (event: { target: EventTarget | null }) => {
      onUpdateSortMode((event.target as HTMLSelectElement).value as ProviderSortMode)
    },
    [onUpdateSortMode],
  )

  const handleOpen = useCallback(
    (open: boolean) => {
      if (!open) onClose()
    },
    [onClose],
  )

  return (
    <BaseModal
      modelValue={visible}
      size="5xl"
      surface="solid"
      title={t('configs.provider.stats')}
      onUpdateModelValue={handleOpen}
      onClose={onClose}
    >
      <div className="mb-4 flex items-center justify-between gap-4">
        <p className="text-xs text-text-muted">
          {t('configs.provider.totalProviders', { count: entries.length })} · {t('configs.provider.totalCalls', { count: totalUsage })}
        </p>
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-1 text-xs text-text-muted">
            <span>{t('configs.provider.sortLabel')}</span>
            <select
              value={sortMode}
              className="cursor-pointer rounded-xl border border-border-default bg-bg-elevated px-3 py-1.5 text-xs font-medium text-text-primary outline-none"
              onChange={handleSort}
            >
              <option value="count_desc">{t('configs.provider.sortCountDesc')}</option>
              <option value="count_asc">{t('configs.provider.sortCountAsc')}</option>
              <option value="name_asc">{t('configs.provider.sortNameAsc')}</option>
            </select>
          </label>
          <button
            type="button"
            className="flex items-center gap-1 rounded-xl border border-border-default bg-bg-elevated px-3 py-1.5 text-xs font-semibold text-text-primary"
            disabled={loading}
            onClick={onRefresh}
          >
            <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={loading ? 'animate-spin' : ''} />
            <span>{t('configs.provider.refreshStats')}</span>
          </button>
        </div>
      </div>
      {loading ? (
        <div className="flex items-center justify-center py-10">
          <div className="h-10 w-10 animate-spin rounded-full border-4 border-transparent border-t-accent-primary" />
        </div>
      ) : null}
      {!loading && error ? (
        <div className="rounded-lg border border-accent-danger bg-accent-danger/10 px-3 py-2 text-xs text-accent-danger">
          {t('configs.provider.loadFailed')}: {error}
        </div>
      ) : null}
      {!loading && !error && sorted.length === 0 ? (
        <div className="py-8 text-center text-xs text-text-muted">{t('configs.provider.noData')}</div>
      ) : null}
      {!loading && !error && sorted.length > 0 ? (
        <div className="flex h-[31.25rem] flex-col">
          <div className="mb-6 flex items-center gap-4 rounded-2xl border border-accent-primary/20 bg-accent-primary/10 p-4">
            <div className="rounded-xl bg-accent-primary/20 p-3 text-accent-primary">
              <SIcon name="BarChart3" size="w-6 h-6" />
            </div>
            <div>
              <div className="text-sm font-medium text-text-secondary">
                {t('configs.provider.totalProviders', { count: entries.length })}
              </div>
              <div className="text-2xl font-bold text-text-primary">
                {t('configs.provider.totalCalls', { count: totalUsage })}
              </div>
            </div>
            <div className="ml-auto text-right text-xs text-text-muted">
              {t('configs.provider.currentSort', { label: sortLabel })}
            </div>
          </div>
          <div className="relative flex min-h-0 flex-1 flex-col">
            <div className="pointer-events-none absolute inset-0 z-0 flex flex-col justify-between pb-8 pl-8">
              {[...yTicks].reverse().map((tick) => (
                <div key={`tick-${tick.percent}`} className="relative h-px w-full bg-border-default opacity-50">
                  <span className="absolute -top-2 -left-8 w-6 text-right text-[0.625rem] text-text-muted">
                    {tick.value}
                  </span>
                </div>
              ))}
            </div>
            <div className="z-10 flex-1 overflow-x-auto overflow-y-hidden pb-2 pl-8">
              <div className="flex h-full min-w-max items-end gap-4 px-4 pt-4">
                {sorted.map(([provider, count], index) => (
                  <ProviderBar
                    key={provider || `provider-${count}`}
                    provider={provider}
                    count={count}
                    maxCount={maxCount}
                    color={CHART_COLORS[index % CHART_COLORS.length]}
                    shareLabel={
                      isZhLocale()
                        ? `${count}次 (${maxCount ? ((count / totalUsage) * 100).toFixed(1) : '0'}%)`
                        : `${count} calls (${maxCount ? ((count / totalUsage) * 100).toFixed(1) : '0'}%)`
                    }
                  />
                ))}
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </BaseModal>
  )
}
