import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent,
} from 'react'
import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui'
import type { ProfilesSortBy, ProfilesStatusFilter, ProviderOption } from '@/utils/profilesFilter'
import './profiles-shared.css'

export type ProfilesViewMode = 'card' | 'list'

export interface ProfilesToolbarProps {
  query: string
  statusFilter: ProfilesStatusFilter
  tagFilter: string | null
  sortBy: ProfilesSortBy
  viewMode: ProfilesViewMode
  resultCount: number
  total: number
  allTags: string[]
  /** i18n key 前缀，例如 'claudeProfiles.toolbar' / 'codex.profiles.toolbar' */
  i18nPrefix: string
  /** provider 维度（Claude 用，Codex 省略 → 不渲染 provider 下拉） */
  providerFilter?: string | null
  allProviders?: ProviderOption[]
  onUpdateQuery: (value: string) => void
  onUpdateStatusFilter: (value: ProfilesStatusFilter) => void
  onUpdateTagFilter: (value: string | null) => void
  onUpdateProviderFilter: (value: string | null) => void
  onUpdateSortBy: (value: ProfilesSortBy) => void
  onUpdateViewMode: (value: ProfilesViewMode) => void
}

export interface ProfilesToolbarHandle {
  focusSearch: () => void
}

const FOCUSABLE_SELECTOR = 'button:not(:disabled), select, input, [tabindex]:not([tabindex="-1"])'

const focusableIn = (root: HTMLElement | null): HTMLElement[] =>
  Array.from(root?.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR) ?? [])

const trapTab = (event: KeyboardEvent<HTMLElement>, focusable: HTMLElement[]) => {
  if (focusable.length === 0) return
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  const active = document.activeElement
  if (event.shiftKey && active === first) {
    event.preventDefault()
    last?.focus()
    return
  }
  if (!event.shiftKey && active === last) {
    event.preventDefault()
    first?.focus()
  }
}

const moveByArrow = (event: KeyboardEvent<HTMLElement>, focusable: HTMLElement[]) => {
  const isArrow = ['ArrowDown', 'ArrowRight', 'ArrowUp', 'ArrowLeft'].includes(event.key)
  if (!isArrow || !(event.target instanceof HTMLButtonElement)) return
  if (focusable.length === 0) return
  const activeIndex = focusable.indexOf(document.activeElement as HTMLElement)
  if (activeIndex < 0) return
  const delta = event.key === 'ArrowDown' || event.key === 'ArrowRight' ? 1 : -1
  event.preventDefault()
  focusable[(activeIndex + delta + focusable.length) % focusable.length]?.focus()
}

/** 粘性工具条：搜索 / 状态pill / 标签pill / (可选)Provider下拉 / 排序 / 视图 / 结果数。 */
export const ProfilesToolbar = forwardRef<ProfilesToolbarHandle, ProfilesToolbarProps>(
  function ProfilesToolbar(props, ref) {
    const {
      query,
      statusFilter,
      tagFilter,
      sortBy,
      viewMode,
      resultCount,
      total,
      allTags,
      i18nPrefix,
      providerFilter = null,
      allProviders,
      onUpdateQuery,
      onUpdateStatusFilter,
      onUpdateTagFilter,
      onUpdateProviderFilter,
      onUpdateSortBy,
      onUpdateViewMode,
    } = props
    const t = useShellT()
    const searchRef = useRef<HTMLInputElement | null>(null)
    const filtersBtnRef = useRef<HTMLButtonElement | null>(null)
    const filtersPopRef = useRef<HTMLDivElement | null>(null)
    const [filtersOpen, setFiltersOpen] = useState(false)

    useImperativeHandle(ref, () => ({
      focusSearch: () => searchRef.current?.focus(),
    }))

    const activeFilterCount = (tagFilter ? 1 : 0) + (providerFilter ? 1 : 0) + (sortBy !== 'recent' ? 1 : 0)

    useEffect(() => {
      if (!filtersOpen) return
      focusableIn(filtersPopRef.current)[0]?.focus()
      const onDocumentPointerDown = (event: MouseEvent) => {
        const target = event.target as Node
        if (filtersPopRef.current?.contains(target)) return
        if (filtersBtnRef.current?.contains(target)) return
        setFiltersOpen(false)
      }
      document.addEventListener('mousedown', onDocumentPointerDown)
      return () => document.removeEventListener('mousedown', onDocumentPointerDown)
    }, [filtersOpen])

    const closeFilters = (restoreFocus: boolean) => {
      setFiltersOpen(false)
      if (restoreFocus) filtersBtnRef.current?.focus()
    }

    const onFiltersKeydown = (event: KeyboardEvent<HTMLElement>) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        event.stopPropagation()
        closeFilters(true)
        return
      }
      const focusable = focusableIn(filtersPopRef.current)
      if (event.key === 'Tab') {
        trapTab(event, focusable)
        return
      }
      moveByArrow(event, focusable)
    }

    const clearAllFilters = () => {
      onUpdateTagFilter(null)
      onUpdateProviderFilter(null)
      onUpdateSortBy('recent')
      closeFilters(true)
    }

    const statusOptions = useMemo(
      () =>
        [
          { id: 'all' as const, label: t(`${i18nPrefix}.statusAll`) },
          { id: 'active' as const, label: t(`${i18nPrefix}.statusActive`) },
          { id: 'enabled' as const, label: t(`${i18nPrefix}.statusEnabled`) },
          { id: 'disabled' as const, label: t(`${i18nPrefix}.statusDisabled`) },
        ] satisfies { id: ProfilesStatusFilter; label: string }[],
      [i18nPrefix, t],
    )

    const onQueryInput = (event: FormEvent<HTMLInputElement>) => {
      onUpdateQuery(event.currentTarget.value)
    }

    const showProvider = Boolean(allProviders && allProviders.length > 1)
    const triggerClass =
      activeFilterCount > 0 || filtersOpen ? 'cp-pill cp-filters__trigger cp-pill--active' : 'cp-pill cp-filters__trigger'

    return (
      <div className="cp-toolbar surface-workspace">
        <div className="cp-search">
          <SIcon name="Search" size="w-3.5 h-3.5" className="cp-search__icon" />
          <input
            ref={searchRef}
            value={query}
            placeholder={t(`${i18nPrefix}.searchPlaceholder`)}
            aria-label={t(`${i18nPrefix}.searchPlaceholder`)}
            className="cp-search__input"
            onInput={onQueryInput}
          />
          <kbd className="cp-search__kbd">/</kbd>
        </div>

        <span className="cp-toolbar__sep" />

        <div className="cp-pill-row" role="group" aria-label={t(`${i18nPrefix}.statusGroupLabel`)}>
          {statusOptions.map((opt) => (
            <button
              key={opt.id}
              type="button"
              className={statusFilter === opt.id ? 'cp-pill cp-pill--active' : 'cp-pill'}
              aria-pressed={statusFilter === opt.id}
              onClick={() => onUpdateStatusFilter(opt.id)}
            >
              {opt.label}
            </button>
          ))}
        </div>

        <span className="cp-toolbar__sep" />

        <div className="cp-filters">
          <button
            ref={filtersBtnRef}
            type="button"
            className={triggerClass}
            aria-expanded={filtersOpen}
            aria-haspopup="dialog"
            onClick={() => setFiltersOpen((open) => !open)}
          >
            <SIcon name="SlidersHorizontal" size="w-3.5 h-3.5" />
            {t(`${i18nPrefix}.filtersButton`)}
            {activeFilterCount > 0 ? <span className="cp-filters__badge">{activeFilterCount}</span> : null}
            <SIcon name="ChevronDown" size="w-3 h-3" />
          </button>

          {filtersOpen ? (
            <div
              ref={filtersPopRef}
              className="cp-filters__pop"
              role="dialog"
              aria-label={t(`${i18nPrefix}.filtersButton`)}
              onKeyDown={onFiltersKeydown}
            >
              {allTags.length > 0 ? (
                <div className="cp-filters__section">
                  <div className="cp-filters__label">{t(`${i18nPrefix}.tagGroupLabel`)}</div>
                  <div className="cp-pill-row" role="group" aria-label={t(`${i18nPrefix}.tagGroupLabel`)}>
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
                <button
                  type="button"
                  className="cp-pill"
                  disabled={activeFilterCount === 0}
                  onClick={clearAllFilters}
                >
                  {t(`${i18nPrefix}.clearAll`)}
                </button>
              </div>
            </div>
          ) : null}
        </div>

        <div className="cp-toolbar__right">
          <span className="cp-toolbar__meta">
            {resultCount}/{total}
          </span>

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
              className={viewMode === 'list' ? 'cp-seg__btn cp-seg__btn--active' : 'cp-seg__btn'}
              title={t(`${i18nPrefix}.viewList`)}
              aria-pressed={viewMode === 'list'}
              onClick={() => onUpdateViewMode('list')}
            >
              <SIcon name="List" size="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    )
  },
)
