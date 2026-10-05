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
import { FiltersPop, TagPills, ViewSegment, type ProfilesViewMode } from './ProfilesToolbarControls'
import './profiles-shared.css'

export type { ProfilesViewMode } from './ProfilesToolbarControls'

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
  /** 为真时第二段控件写入 table 而非 list */
  tableView?: boolean
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
      tableView = false,
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
      <div className="cp-toolbar surface-workspace" data-testid="profiles-toolbar">
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

        <div className="cp-pill-row" role="group" aria-label={t(`${i18nPrefix}.tagGroupLabel`)}>
          <button
            type="button"
            className={tagFilter === null ? 'cp-pill cp-pill--active' : 'cp-pill'}
            aria-pressed={tagFilter === null}
            onClick={() => onUpdateTagFilter(null)}
          >
            {t('profilesSurface.allTags', { count: total })}
          </button>
          <TagPills
            allTags={allTags}
            tagFilter={tagFilter}
            onUpdateTagFilter={onUpdateTagFilter}
          />
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
            <FiltersPop
              i18nPrefix={i18nPrefix}
              allTags={allTags}
              tagFilter={tagFilter}
              sortBy={sortBy}
              providerFilter={providerFilter}
              allProviders={allProviders}
              showProvider={showProvider}
              activeFilterCount={activeFilterCount}
              t={t}
              popRef={filtersPopRef}
              onKeyDown={onFiltersKeydown}
              onUpdateTagFilter={onUpdateTagFilter}
              onUpdateProviderFilter={onUpdateProviderFilter}
              onUpdateSortBy={onUpdateSortBy}
              onClear={clearAllFilters}
            />
          ) : null}
        </div>

        <div className="cp-toolbar__right">
          <span className="cp-toolbar__meta">
            {resultCount}/{total}
          </span>

          <ViewSegment
            viewMode={viewMode}
            tableView={tableView}
            i18nPrefix={i18nPrefix}
            t={t}
            onUpdateViewMode={onUpdateViewMode}
          />
        </div>
      </div>
    )
  },
)
