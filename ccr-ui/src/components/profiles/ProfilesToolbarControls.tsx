import type { KeyboardEvent, RefObject } from 'react'
import { SIcon } from '@/ui'
import type { ProfilesSortBy, ProviderOption } from '@/utils/profilesFilter'
import type { TranslateFunction } from '@/utils/tf'
export type ProfilesViewMode = 'card' | 'list' | 'table'

export interface TagPillsProps {
  allTags: string[]
  tagFilter: string | null
  onUpdateTagFilter: (value: string | null) => void
}

export function TagPills({ allTags, tagFilter, onUpdateTagFilter }: TagPillsProps) {
  return (
    <>
      {allTags.map((tag) => (
        <button
          key={tag}
          type="button"
          className={tagFilter === tag ? 'cp-pill cp-pill--active' : 'cp-pill'}
          aria-pressed={tagFilter === tag}
          onClick={() => onUpdateTagFilter(tagFilter === tag ? null : tag)}
        >
          #{tag}
        </button>
      ))}
    </>
  )
}

export interface FiltersPopProps {
  i18nPrefix: string
  allTags: string[]
  tagFilter: string | null
  sortBy: ProfilesSortBy
  providerFilter: string | null
  allProviders?: ProviderOption[]
  showProvider: boolean
  activeFilterCount: number
  t: TranslateFunction
  popRef: RefObject<HTMLDivElement | null>
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void
  onUpdateTagFilter: (value: string | null) => void
  onUpdateProviderFilter: (value: string | null) => void
  onUpdateSortBy: (value: ProfilesSortBy) => void
  onClear: () => void
}

export function FiltersPop({
  i18nPrefix,
  allTags,
  tagFilter,
  sortBy,
  providerFilter,
  allProviders,
  showProvider,
  activeFilterCount,
  t,
  popRef,
  onKeyDown,
  onUpdateTagFilter,
  onUpdateProviderFilter,
  onUpdateSortBy,
  onClear,
}: FiltersPopProps) {
  return (
    <div
      ref={popRef}
      className="cp-filters__pop"
      role="dialog"
      aria-label={t(`${i18nPrefix}.filtersButton`)}
      onKeyDown={onKeyDown}
    >
      {allTags.length > 0 ? (
        <div className="cp-filters__section">
          <div className="cp-filters__label">{t(`${i18nPrefix}.tagGroupLabel`)}</div>
          <div className="cp-pill-row" role="group" aria-label={t(`${i18nPrefix}.tagGroupLabel`)}>
            <TagPills
              allTags={allTags}
              tagFilter={tagFilter}
              onUpdateTagFilter={onUpdateTagFilter}
            />
          </div>
        </div>
      ) : null}
      {showProvider ? (
        <div className="cp-filters__section">
          <div className="cp-filters__label">{t(`${i18nPrefix}.providerLabel`)}</div>
          <select
            value={providerFilter ?? ''}
            className="cp-toolbar__sort cp-filters__select"
            aria-label={t(`${i18nPrefix}.providerLabel`)}
            onChange={(event) => onUpdateProviderFilter(event.currentTarget.value || null)}
          >
            <option value="">{t(`${i18nPrefix}.providerAll`)}</option>
            {allProviders?.map((provider) => (
              <option key={provider.key} value={provider.key}>
                {provider.label}
              </option>
            ))}
          </select>
        </div>
      ) : null}
      <div className="cp-filters__section">
        <div className="cp-filters__label">{t(`${i18nPrefix}.sortLabel`)}</div>
        <select
          value={sortBy}
          className="cp-toolbar__sort cp-filters__select"
          aria-label={t(`${i18nPrefix}.sortLabel`)}
          onChange={(event) => onUpdateSortBy(event.currentTarget.value as ProfilesSortBy)}
        >
          <option value="recent">{t(`${i18nPrefix}.sortRecent`)}</option>
          <option value="name">{t(`${i18nPrefix}.sortName`)}</option>
          <option value="requests">{t(`${i18nPrefix}.sortRequests`)}</option>
          <option value="enabled">{t(`${i18nPrefix}.sortEnabled`)}</option>
        </select>
      </div>
      <div className="cp-filters__foot">
        <button type="button" className="cp-pill" disabled={activeFilterCount === 0} onClick={onClear}>
          {t(`${i18nPrefix}.clearAll`)}
        </button>
      </div>
    </div>
  )
}

export interface ViewSegmentProps {
  viewMode: ProfilesViewMode
  tableView: boolean
  i18nPrefix: string
  t: TranslateFunction
  onUpdateViewMode: (value: ProfilesViewMode) => void
}

export function ViewSegment({
  viewMode,
  tableView,
  i18nPrefix,
  t,
  onUpdateViewMode,
}: ViewSegmentProps) {
  const alt: ProfilesViewMode = tableView ? 'table' : 'list'
  return (
    <div className="cp-seg" role="group" aria-label={t(`${i18nPrefix}.viewLabel`)}>
      <button
        type="button"
        className={viewMode === 'card' ? 'cp-seg__btn cp-seg__btn--active' : 'cp-seg__btn'}
        title={t(`${i18nPrefix}.viewCard`)}
        aria-pressed={viewMode === 'card'}
        onClick={() => onUpdateViewMode('card')}
      >
        <SIcon name="Layers" size="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        className={viewMode === alt ? 'cp-seg__btn cp-seg__btn--active' : 'cp-seg__btn'}
        title={t(`${i18nPrefix}.${tableView ? 'viewTable' : 'viewList'}`)}
        aria-pressed={viewMode === alt}
        onClick={() => onUpdateViewMode(alt)}
      >
        <SIcon name="List" size="w-3.5 h-3.5" />
      </button>
    </div>
  )
}
