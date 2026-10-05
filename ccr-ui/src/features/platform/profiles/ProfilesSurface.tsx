import { useCallback, useMemo, useRef, useState, type ReactNode } from 'react'
import type { RawFileGetResult, RawProfilesSaveResult } from '@/api/domains/configRawTypes'
import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import {
  useProfilesHotkeys,
  useProfilesQuickSwitch,
} from '@/configs/profilesSurfaceRuntime'
import { surfaceNotify } from '@/configs/surfaceNotify'
import {
  ProfilesCommandPalette,
  ProfilesOffBanner,
  ProfilesPageHeader,
  ProfilesQuickRail,
  ProfilesRawEditorPanel,
  ProfilesStatStrip,
  ProfilesToolbar,
  type ProfilesCommandPaletteAction,
  type ProfilesToolbarHandle,
} from '@/features/platform/profiles/shared'
import { ProfilesSurfaceRecords } from './ProfilesSurfaceRecords'
import { useAppT } from '@/i18n'
import type { ProfilesSortBy } from '@/utils/profilesFilter'
import { useProfilesSurface } from './useProfilesSurface'

export interface ProfilesSurfaceRawSource {
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (
    content: string,
    token: string,
    force?: boolean,
  ) => Promise<RawProfilesSaveResult>
  refreshAll: () => Promise<void>
}

export interface ProfilesSurfaceProps {
  platformKey: string
  presentation: ProfilePresentationView
  records: readonly ProfileDisplayRecord[]
  current: string | null
  environmentLabel: string
  environmentOk: boolean
  loading?: boolean
  canOff: boolean
  commandPalette?: boolean
  onAdd: () => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
  onOff: () => Promise<void>
  onReload: () => void
  onExport?: () => void
  onToggle?: (name: string, enabled: boolean) => void
  onDelete?: (name: string) => void
  notice?: ReactNode
  rawSource?: ProfilesSurfaceRawSource
}

const mapSortBy = (sortBy: 'name' | 'usage'): ProfilesSortBy =>
  sortBy === 'usage' ? 'requests' : 'name'

const fromToolbarSort = (value: ProfilesSortBy): 'name' | 'usage' =>
  value === 'name' ? 'name' : 'usage'

const railShownCount = (pinned: number, recentNotPinned: number): number =>
  pinned + Math.min(recentNotPinned, Math.max(0, 8 - pinned))

/** 统一 Profile 列表页装配：筛选与视图状态本地持有，数据由调用方注入。 */
export function ProfilesSurface(props: ProfilesSurfaceProps) {
  const {
    platformKey,
    presentation,
    records,
    current,
    environmentLabel,
    environmentOk,
    loading = false,
    canOff,
    commandPalette = false,
    onAdd,
    onEdit,
    onApply,
    onOff,
    onReload,
    onExport,
    onToggle,
    onDelete,
    notice,
    rawSource,
  } = props
  const t = useAppT()
  const toolbarRef = useRef<ProfilesToolbarHandle>(null)
  const [paletteOpen, setPaletteOpen] = useState(false)
  const surface = useProfilesSurface({ platformKey, records, current })
  const {
    query,
    setQuery,
    tagFilter,
    setTagFilter,
    providerFilter,
    setProviderFilter,
    statusFilter,
    setStatusFilter,
    sortBy,
    setSortBy,
    viewMode,
    setViewMode,
    filtered,
    stats,
    allTags,
    allProviders,
    sourceMode,
    setSourceMode,
    inspectorOpen,
    setInspectorOpen,
    setFocusedName,
    previewRecord,
    currentRecord,
    clearFilters,
  } = surface

  const names = useMemo(() => records.map((record) => record.name), [records])
  const getProfileNames = useCallback(() => names, [names])
  const onPinLimit = useCallback(() => {
    surfaceNotify.warning(t('profilesSurface.pinLimitReached'))
  }, [t])
  const quickSwitch = useProfilesQuickSwitch({
    platform: platformKey,
    getProfileNames,
    onPinLimit,
  })
  const recordUse = quickSwitch.recordUse

  const focusSearch = useCallback(() => {
    toolbarRef.current?.focusSearch()
  }, [])
  const getStableTargets = useCallback(
    () => quickSwitch.stableTargets,
    [quickSwitch.stableTargets],
  )
  const applyAndRecord = useCallback(
    (name: string) => {
      onApply(name)
      recordUse(name)
    },
    [onApply, recordUse],
  )

  useProfilesHotkeys({
    paletteOpen,
    setPaletteOpen,
    focusSearch,
    getStableTargets,
    onApply: applyAndRecord,
  })

  const paletteDescriptor = useMemo(
    () => ({
      isEnabled: (record: ProfileDisplayRecord) => record.enabled,
      hint: (record: ProfileDisplayRecord) => record.description || undefined,
    }),
    [],
  )
  const runOff = useCallback(() => {
    void onOff()
  }, [onOff])
  const paletteActions = useMemo((): ProfilesCommandPaletteAction[] => {
    const actions: ProfilesCommandPaletteAction[] = [
      { id: 'add', icon: 'Plus', labelKey: 'profilesSurface.newProfile', handler: onAdd },
    ]
    if (canOff) {
      actions.push({
        id: '__off',
        icon: 'PowerOff',
        labelKey: 'profilesSurface.offAction',
        handler: runOff,
      })
    }
    return actions
  }, [canOff, onAdd, runOff])

  const enterSource = useCallback(() => {
    void surfaceNotify
      .confirm({
        title: t('profilesSurface.sourceWarningTitle'),
        message: t('profilesSurface.sourceWarningMessage'),
        confirmText: t('profilesSurface.sourceContinue'),
        cancelText: t('common.cancel'),
        type: 'warning',
      })
      .then((ok) => {
        if (ok) setSourceMode(true)
      })
  }, [setSourceMode, t])

  const onSourceSaved = useCallback(() => {
    setSourceMode(false)
    void rawSource?.refreshAll()
  }, [rawSource, setSourceMode])

  const onSourceClose = useCallback(() => {
    setSourceMode(false)
  }, [setSourceMode])

  const onToggleInspector = useCallback(() => {
    setInspectorOpen(!inspectorOpen)
  }, [inspectorOpen, setInspectorOpen])

  const onOpenPalette = useCallback(() => {
    setPaletteOpen(true)
  }, [])

  const onUpdateViewMode = useCallback(
    (mode: 'card' | 'list' | 'table') => {
      if (mode === 'list') return
      setViewMode(mode)
    },
    [setViewMode],
  )

  const onUpdateSortBy = useCallback(
    (value: ProfilesSortBy) => {
      setSortBy(fromToolbarSort(value))
    },
    [setSortBy],
  )

  const statLabels = useMemo(
    () => ({
      total: t('profilesSurface.stats.total'),
      vendors: t('profilesSurface.stats.vendors', { count: stats.vendorCount }),
      running: t('profilesSurface.stats.running'),
      runningHint: t('profilesSurface.stats.runningHint'),
      notApplied: t('profilesSurface.stats.notApplied'),
      tags: t('profilesSurface.stats.tags'),
      auth: t('profilesSurface.stats.auth'),
    }),
    [stats.vendorCount, t],
  )

  const moreCount = Math.max(
    0,
    records.length - railShownCount(quickSwitch.pinned.length, quickSwitch.recentNotPinned.length),
  )
  const inspectorToggleKey = inspectorOpen
    ? 'profilesSurface.inspectorClosed'
    : 'profilesSurface.inspectorOpen'

  if (sourceMode && rawSource) {
    return (
      <ProfilesRawEditorPanel
        getRaw={rawSource.getRaw}
        saveRaw={rawSource.saveRaw}
        onSaved={onSourceSaved}
        onClose={onSourceClose}
      />
    )
  }

  return (
    <div
      className="cp-surface"
      data-testid="profiles-surface"
      data-platform={presentation.key}
      data-can-off={canOff ? 'true' : 'false'}
    >
      <ProfilesPageHeader
        presentation={presentation}
        environmentLabel={environmentLabel}
        environmentOk={environmentOk}
        loading={loading}
        onAdd={onAdd}
        onReload={onReload}
        onExport={onExport}
        onEditSource={rawSource ? enterSource : undefined}
      />
      {notice}
      <ProfilesOffBanner canOff={canOff} currentName={current} onOff={onOff} />
      <ProfilesStatStrip current={current} stats={stats} labels={statLabels} />
      <div data-testid="profiles-quick-rail">
        <ProfilesQuickRail
          profiles={[...records]}
          currentName={current}
          i18nPrefix="profilesSurface"
          quickSwitch={quickSwitch}
          moreCount={moreCount}
          onApply={applyAndRecord}
          onMore={onOpenPalette}
        />
      </div>
      <ProfilesToolbar
        ref={toolbarRef}
        query={query}
        statusFilter={statusFilter}
        tagFilter={tagFilter}
        sortBy={mapSortBy(sortBy)}
        viewMode={viewMode}
        resultCount={filtered.length}
        total={records.length}
        allTags={[...allTags]}
        i18nPrefix="profilesSurface.toolbar"
        providerFilter={providerFilter}
        allProviders={allProviders}
        tableView
        onUpdateQuery={setQuery}
        onUpdateStatusFilter={setStatusFilter}
        onUpdateTagFilter={setTagFilter}
        onUpdateProviderFilter={setProviderFilter}
        onUpdateSortBy={onUpdateSortBy}
        onUpdateViewMode={onUpdateViewMode}
      />
      <ProfilesSurfaceRecords
        records={records}
        filtered={filtered}
        presentation={presentation}
        viewMode={viewMode}
        inspectorOpen={inspectorOpen}
        query={query}
        tagFilter={tagFilter}
        providerFilter={providerFilter}
        previewRecord={previewRecord}
        currentRecord={currentRecord}
        t={t}
        onAdd={onAdd}
        onEdit={onEdit}
        onApply={applyAndRecord}
        onToggle={onToggle}
        onDelete={onDelete}
        onSelect={setFocusedName}
        onClearFilters={clearFilters}
        onTagSelect={setTagFilter}
      />
      <button
        type="button"
        className="cp-inspector-toggle"
        data-testid="profiles-inspector-toggle"
        onClick={onToggleInspector}
      >
        {t(inspectorToggleKey)}
      </button>
      {commandPalette ? (
        <ProfilesCommandPalette
          open={paletteOpen}
          profiles={[...records]}
          descriptor={paletteDescriptor}
          actions={paletteActions}
          i18nPrefix="profilesSurface.commandPalette"
          onUpdateOpen={setPaletteOpen}
          onApply={applyAndRecord}
        />
      ) : null}
      {commandPalette ? (
        <button
          type="button"
          className="sr-only"
          data-testid="profiles-open-palette"
          onClick={onOpenPalette}
        >
          {t('profilesSurface.commandPalette.title')}
        </button>
      ) : null}
    </div>
  )
}
