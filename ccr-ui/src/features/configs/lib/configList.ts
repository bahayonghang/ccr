import type { ConfigItem } from '@/types'
import type { TranslateFunction } from '@/utils/tf'
import type { ConfigFilter, ConfigSort } from '../types'

export interface ConfigSummaryItem {
  key: ConfigFilter
  label: string
  count: number
  icon: string
  activeClass: string
  idleClass: string
}

const matchesFilter = (config: ConfigItem, filter: ConfigFilter): boolean => {
  if (filter === 'all') return true
  const type = config.provider_type?.toLowerCase() ?? ''
  if (filter === 'official_relay') return type.includes('official')
  if (filter === 'third_party_model') return type.includes('third')
  return !config.provider_type
}

const matchesQuery = (config: ConfigItem, query: string): boolean => {
  if (!query) return true
  return (
    config.name.toLowerCase().includes(query) ||
    (config.provider?.toLowerCase().includes(query) ?? false) ||
    (config.model?.toLowerCase().includes(query) ?? false)
  )
}

export function filterConfigs(input: {
  configs: ConfigItem[]
  filter: ConfigFilter
  searchQuery: string
  sort: ConfigSort
}): ConfigItem[] {
  const query = input.searchQuery.toLowerCase().trim()
  const list = input.configs.filter(
    (config) => matchesFilter(config, input.filter) && matchesQuery(config, query),
  )
  const sort = input.sort
  if (sort === 'usage_count') {
    list.sort((left, right) => (right.usage_count || 0) - (left.usage_count || 0))
    return list
  }
  if (sort === 'recent') {
    list.sort((left) => (left.is_current ? -1 : 1))
    return list
  }
  list.sort((left, right) => left.name.localeCompare(right.name))
  return list
}

export function currentConfigName(configs: ConfigItem[], fallback: string): string {
  return configs.find((config) => config.is_current)?.name ?? fallback
}

export function quickJumpConfigs(configs: ConfigItem[]): ConfigItem[] {
  const current = configs.filter((config) => config.is_current)
  const rest = configs.filter((config) => !config.is_current)
  return [...current, ...rest].slice(0, 8)
}

export function buildConfigSummary(configs: ConfigItem[], translate: TranslateFunction): ConfigSummaryItem[] {
  return [
    {
      key: 'all',
      label: translate('configs.filters.all'),
      count: configs.length,
      icon: 'LayoutGrid',
      activeClass: 'border-accent-primary bg-bg-elevated text-text-primary',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-accent-primary hover:text-text-primary',
    },
    {
      key: 'official_relay',
      label: translate('configs.filters.officialRelay'),
      count: configs.filter((config) => config.provider_type?.toLowerCase().includes('official')).length,
      icon: 'Zap',
      activeClass: 'border-accent-primary bg-bg-elevated text-text-primary',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-accent-primary hover:text-text-primary',
    },
    {
      key: 'third_party_model',
      label: translate('configs.filters.thirdPartyModel'),
      count: configs.filter((config) => config.provider_type?.toLowerCase().includes('third')).length,
      icon: 'Cpu',
      activeClass: 'border-accent-primary bg-bg-elevated text-text-primary',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-accent-primary hover:text-text-primary',
    },
    {
      key: 'uncategorized',
      label: translate('configs.filters.uncategorized'),
      count: configs.filter((config) => !config.provider_type).length,
      icon: 'HelpCircle',
      activeClass: 'border-accent-primary bg-bg-elevated text-text-primary',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-accent-primary hover:text-text-primary',
    },
  ]
}
