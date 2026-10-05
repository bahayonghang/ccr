import { useCallback, useEffect, useRef, useState, type ChangeEvent } from 'react'
import { Link } from 'react-router'
import { PillToggleGroup, SIcon } from '@/ui'
import type { UsageRangePreset } from '@/views/usage/dateWindow'
import type { DashboardMetaItem } from '@/views/usage/usageOverviewInsights'
import { USAGE_SOURCE_DEFINITIONS } from '@/views/usage/usageSources'
import { useUsageT } from '../translate'
import '../styles/usage-dashboard-toolbar.css'

interface UsageDashboardToolbarProps {
  selectedPlatform: string
  selectedRange: UsageRangePreset
  importButtonLabel: string
  importing: boolean
  runtimeUnavailable: boolean
  metaItems: DashboardMetaItem[]
  onPlatformChange: (value: string) => void
  onRangeChange: (value: UsageRangePreset) => void
  onImport: () => void
}

const RANGE_OPTIONS: Array<{ value: UsageRangePreset; key: string }> = [
  { value: 'today', key: 'usage.dashboard.range.today' },
  { value: 'this_week', key: 'usage.dashboard.range.thisWeek' },
  { value: 'this_month', key: 'usage.dashboard.range.thisMonth' },
  { value: 'last_30d', key: 'usage.dashboard.range.last30' },
  { value: 'all_time', key: 'usage.dashboard.range.allTime' },
]

export function UsageDashboardToolbar({
  selectedPlatform,
  selectedRange,
  importButtonLabel,
  importing,
  runtimeUnavailable,
  metaItems,
  onPlatformChange,
  onRangeChange,
  onImport,
}: UsageDashboardToolbarProps) {
  const t = useUsageT()
  const [metaOpen, setMetaOpen] = useState(false)
  const metaRootRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    const handleClick = (event: MouseEvent) => {
      if (!metaRootRef.current?.contains(event.target as Node)) setMetaOpen(false)
    }
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setMetaOpen(false)
    }
    document.addEventListener('click', handleClick)
    document.addEventListener('keydown', handleEscape)
    return () => {
      document.removeEventListener('click', handleClick)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [])

  const handlePlatform = useCallback((event: ChangeEvent<HTMLSelectElement>) => {
    onPlatformChange(event.target.value)
  }, [onPlatformChange])

  const toggleMeta = useCallback(() => setMetaOpen((open) => !open), [])

  if (runtimeUnavailable) return <header className="usage-dashboard-toolbar" />

  return (
    <header className="usage-dashboard-toolbar">
      <div className="usage-dashboard-toolbar__actions">
        <label className="usage-dashboard-toolbar__field">
          <span>{t('usage.dashboard.toolbar.platform')}</span>
          <select
            value={selectedPlatform}
            className="usage-dashboard-toolbar__select"
            onChange={handlePlatform}
          >
            <option value="">{t('usage.dashboard.allPlatforms')}</option>
            {USAGE_SOURCE_DEFINITIONS.map((source) => {
              const key = `usage.platforms.${source.id}`
              const translated = t(key)
              return (
                <option key={source.id} value={source.id}>
                  {translated === key ? source.fallbackLabel : translated}
                </option>
              )
            })}
          </select>
        </label>
        <div className="usage-dashboard-toolbar__field usage-dashboard-toolbar__field--segmented">
          <span>{t('usage.dashboard.toolbar.window')}</span>
          <PillToggleGroup
            options={RANGE_OPTIONS.map((option) => ({ value: option.value, label: t(option.key) }))}
            value={selectedRange}
            onValueChange={onRangeChange}
          />
        </div>
        <Link className="usage-dashboard-toolbar__pricing-link" to="/pricing">
          {t('usage.dashboard.toolbar.pricing')}
        </Link>
        {metaItems.length > 0 ? (
          <div ref={metaRootRef} className="usage-dashboard-toolbar__meta">
            <button
              type="button"
              className="usage-dashboard-toolbar__meta-trigger"
              aria-expanded={metaOpen}
              onClick={toggleMeta}
            >
              <SIcon name="Database" size="w-3.5 h-3.5" />
              <span>{t('usage.dashboard.toolbar.dataSource')}</span>
              <SIcon
                name="ChevronDown"
                size="w-3 h-3"
                className={['usage-dashboard-toolbar__meta-chevron', metaOpen ? 'usage-dashboard-toolbar__meta-chevron--open' : '']
                  .filter(Boolean)
                  .join(' ')}
              />
            </button>
            {metaOpen ? (
              <div
                className="usage-dashboard-toolbar__meta-popover"
                role="group"
                aria-label={t('usage.dashboard.toolbar.dataSource')}
              >
                {metaItems.map((item) => (
                  <span key={item.id} className="usage-dashboard-toolbar__meta-chip">
                    <span className="usage-dashboard-toolbar__meta-label">{item.label}</span>
                    <strong className="usage-dashboard-toolbar__meta-value">{item.value}</strong>
                  </span>
                ))}
              </div>
            ) : null}
          </div>
        ) : null}
        <button
          type="button"
          className="usage-dashboard-toolbar__import"
          disabled={importing}
          onClick={onImport}
        >
          {importButtonLabel}
        </button>
      </div>
    </header>
  )
}
