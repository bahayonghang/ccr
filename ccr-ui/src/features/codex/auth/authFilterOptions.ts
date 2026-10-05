import type { TranslateFunction } from '@/utils/tf'
import type { AccountPlanFilter, AccountSort, AccountStatusFilter } from '../codexAuthAccounts'

export function authFilterOptions(t: TranslateFunction) {
  return {
    statusOptions: [
      { value: 'all' as AccountStatusFilter, label: t('codex.auth.filters.statusOptions.all') },
      { value: 'current' as AccountStatusFilter, label: t('codex.auth.filters.statusOptions.current') },
      { value: 'virtual' as AccountStatusFilter, label: t('codex.auth.filters.statusOptions.virtual') },
      { value: 'attention' as AccountStatusFilter, label: t('codex.auth.filters.statusOptions.attention') },
    ],
    planOptions: [
      { value: 'all' as AccountPlanFilter, label: t('codex.auth.filters.planOptions.all') },
      { value: 'plus' as AccountPlanFilter, label: t('codex.auth.filters.planOptions.plus') },
      { value: 'pro' as AccountPlanFilter, label: t('codex.auth.filters.planOptions.pro') },
      { value: 'team' as AccountPlanFilter, label: t('codex.auth.filters.planOptions.team') },
      { value: 'unknown' as AccountPlanFilter, label: t('codex.auth.filters.planOptions.unknown') },
    ],
    sortOptions: [
      { value: 'saved_desc' as AccountSort, label: t('codex.auth.filters.sortOptions.savedDesc') },
      { value: 'used_desc' as AccountSort, label: t('codex.auth.filters.sortOptions.usedDesc') },
      { value: 'name_asc' as AccountSort, label: t('codex.auth.filters.sortOptions.nameAsc') },
    ],
  }
}
