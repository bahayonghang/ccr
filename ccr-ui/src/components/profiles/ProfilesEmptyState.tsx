import { useShellT } from '@/shell/i18n'
import './profiles-shared.css'

export interface ProfilesEmptyStateProps {
  variant: 'no-profiles' | 'no-results'
  query: string
  tagFilter: string | null
  providerFilter: string | null
  onClear: () => void
  onAdd: () => void
}

const filterHint = (query: string, tagFilter: string | null, providerFilter: string | null) => {
  const parts: string[] = []
  if (query) parts.push(`“${query}”`)
  if (tagFilter) parts.push(`#${tagFilter}`)
  if (providerFilter) parts.push(providerFilter)
  return parts.join(' ')
}

/** 无配置 / 无筛选结果两种空态。 */
export function ProfilesEmptyState({
  variant,
  query,
  tagFilter,
  providerFilter,
  onClear,
  onAdd,
}: ProfilesEmptyStateProps) {
  const t = useShellT()
  const isEmpty = variant === 'no-profiles'
  return (
    <div className="cp-empty" data-testid="profiles-empty">
      <div className="cp-glyph cp-empty__glyph">?</div>
      <h2>
        {isEmpty ? t('profilesSurface.empty.noProfiles') : t('profilesSurface.empty.noResults')}
      </h2>
      <p>
        {isEmpty
          ? t('profilesSurface.empty.noProfilesHint')
          : t('profilesSurface.empty.noResultsHint', {
              query,
              filters: filterHint(query, tagFilter, providerFilter),
            })}
      </p>
      <div className="cp-empty__actions">
        {isEmpty ? null : (
          <button
            type="button"
            className="cp-btn cp-btn--ghost"
            data-testid="profiles-clear-filters"
            onClick={onClear}
          >
            {t('profilesSurface.empty.clearFilters')}
          </button>
        )}
        <button type="button" className="cp-btn cp-btn--primary" onClick={onAdd}>
          {t('profilesSurface.newProfile')}
        </button>
      </div>
    </div>
  )
}
