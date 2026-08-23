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
      activeClass: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-emerald-400/20 hover:text-text-primary',
    },
    {
      key: 'official_relay',
      label: translate('configs.filters.officialRelay'),
      count: configs.filter((config) => config.provider_type?.toLowerCase().includes('official')).length,
      icon: 'Zap',
      activeClass: 'border-cyan-400/30 bg-cyan-400/10 text-cyan-300',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-cyan-400/20 hover:text-text-primary',
    },
    {
      key: 'third_party_model',
      label: translate('configs.filters.thirdPartyModel'),
      count: configs.filter((config) => config.provider_type?.toLowerCase().includes('third')).length,
      icon: 'Cpu',
      activeClass: 'border-violet-400/30 bg-violet-400/10 text-violet-300',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-violet-400/20 hover:text-text-primary',
    },
    {
      key: 'uncategorized',
      label: translate('configs.filters.uncategorized'),
      count: configs.filter((config) => !config.provider_type).length,
      icon: 'HelpCircle',
      activeClass: 'border-amber-400/30 bg-amber-400/10 text-amber-300',
      idleClass:
        'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-amber-400/20 hover:text-text-primary',
    },
  ]
}

export function providerKind(config: ConfigItem): 'official' | 'third' | 'uncategorized' {
  const type = config.provider_type?.toLowerCase() || ''
  if (type.includes('official')) return 'official'
  if (type.includes('third')) return 'third'
  return 'uncategorized'
}
