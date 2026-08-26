import { memo, useCallback, useMemo } from 'react'
import { SIcon, buttonClass } from '@/ui'
import { t } from '../locale'
import type { ConfigFilter, ConfigSort } from '../types'
import { FilterChip } from './FilterChip'
import '../styles/config-filters.css'

interface ConfigFiltersProps {
  currentFilter: ConfigFilter
  currentSort: ConfigSort
  onUpdateFilter: (value: ConfigFilter) => void
  onUpdateSort: (value: ConfigSort) => void
  onShowProviderStats: () => void
  onAddConfig: () => void
}

const SORT_ICONS: Record<ConfigSort, string> = {
  name: 'FileText',
  usage_count: 'TrendingUp',
  recent: 'Clock',
}

export const ConfigFilters = memo(function ConfigFilters({
  currentFilter,
  currentSort,
  onUpdateFilter,
  onUpdateSort,
  onShowProviderStats,
  onAddConfig,
}: ConfigFiltersProps) {
  const filters = useMemo(
    () => [
      { type: 'all' as const, label: t('configs.filters.all'), icon: 'LayoutGrid', iconColor: 'text-emerald-400' },
      {
        type: 'official_relay' as const,
        label: t('configs.filters.officialRelay'),
        icon: 'Zap',
        iconColor: 'text-cyan-400',
      },
      {
        type: 'third_party_model' as const,
        label: t('configs.filters.thirdPartyModel'),
        icon: 'Cpu',
        iconColor: 'text-violet-400',
      },
      {
        type: 'uncategorized' as const,
        label: t('configs.filters.uncategorized'),
        icon: 'HelpCircle',
        iconColor: 'text-amber-400',
      },
    ],
    [],
  )

  const handleSort = useCallback(
    (event: { target: EventTarget | null }) => {
      const value = (event.target as HTMLSelectElement).value as ConfigSort
      onUpdateSort(value)
    },
    [onUpdateSort],
  )

  return (
    <div className="mb-6 flex items-center gap-4">
      <div className="glass-filter-container flex flex-1 gap-2 rounded-2xl p-1.5">
        {filters.map((filter) => (
          <FilterChip
            key={filter.type}
            type={filter.type}
            label={filter.label}
            icon={filter.icon}
            iconColor={filter.iconColor}
            active={currentFilter === filter.type}
            onSelect={onUpdateFilter}
          />
        ))}
      </div>
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <label className="whitespace-nowrap text-sm font-medium text-text-primary">{t('configs.sort.label')}</label>
          <div className="relative">
            <select value={currentSort} className="sort-select appearance-none rounded-xl py-2.5 pr-8 pl-9 text-sm font-semibold" onChange={handleSort}>
              <option value="name">{t('configs.sort.name')}</option>
              <option value="usage_count">{t('configs.sort.usageCount')}</option>
              <option value="recent">{t('configs.sort.recent')}</option>
            </select>
            <SIcon
              name={SORT_ICONS[currentSort]}
              size="w-4 h-4"
              className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-accent-primary"
            />
            <SIcon
              name="ChevronDown"
              size="w-4 h-4"
              className="pointer-events-none absolute top-1/2 right-2.5 -translate-y-1/2 text-text-muted"
            />
          </div>
        </div>
        <button
          type="button"
          className="stats-btn flex items-center gap-1.5 rounded-xl px-3 py-2.5 text-xs font-semibold transition-colors duration-200"
          onClick={onShowProviderStats}
        >
          <SIcon name="BarChart3" size="w-4 h-4" />
          <span>{t('configs.provider.title')}</span>
        </button>
        <button
          type="button"
          className={buttonClass({ variant: 'primary', className: 'add-btn flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm font-bold transition-[background-color,transform] duration-200 hover:scale-105' })}
          onClick={onAddConfig}
        >
          <SIcon name="PlusCircle" size="w-4 h-4" />
          <span>{t('configs.buttons.add')}</span>
        </button>
      </div>
    </div>
  )
})
