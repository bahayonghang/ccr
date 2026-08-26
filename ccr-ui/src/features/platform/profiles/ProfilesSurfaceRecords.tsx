import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import {
  ProfileCardGrid,
  ProfileTable,
  ProfilesEmptyState,
  ProfilesInspector,
} from '@/features/platform/profiles/shared'
import { makeDisplayInspectorDescriptor } from '@/utils/displayProfileInspector'
import type { TranslateFunction } from '@/utils/tf'

export interface ProfilesSurfaceRecordsProps {
  records: readonly ProfileDisplayRecord[]
  filtered: readonly ProfileDisplayRecord[]
  presentation: ProfilePresentationView
  viewMode: 'card' | 'list' | 'table'
  inspectorOpen: boolean
  query: string
  tagFilter: string | null
  providerFilter: string | null
  previewRecord: ProfileDisplayRecord | null
  currentRecord: ProfileDisplayRecord | null
  t: TranslateFunction
  onAdd: () => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
  onToggle?: (name: string, enabled: boolean) => void
  onDelete?: (name: string) => void
  onSelect: (name: string) => void
  onClearFilters: () => void
  onTagSelect: (tag: string) => void
}

/** 列表区：空态 / 表格 / 卡片 + Inspector。 */
export function ProfilesSurfaceRecords(props: ProfilesSurfaceRecordsProps) {
  const {
    records,
    filtered,
    presentation,
    viewMode,
    inspectorOpen,
    query,
    tagFilter,
    providerFilter,
    previewRecord,
    currentRecord,
    t,
    onAdd,
    onEdit,
    onApply,
    onToggle,
    onDelete,
    onSelect,
    onClearFilters,
    onTagSelect,
  } = props
  const showEmpty = filtered.length === 0
  const emptyVariant = records.length === 0 ? 'no-profiles' : 'no-results'
  const inspectorDescriptor = makeDisplayInspectorDescriptor(presentation, t)

  return (
    <div className="cp-surface__body">
      <div className="cp-surface__list" data-testid="profiles-list">
        {showEmpty ? (
          <ProfilesEmptyState
            variant={emptyVariant}
            query={query}
            tagFilter={tagFilter}
            providerFilter={providerFilter}
            onClear={onClearFilters}
            onAdd={onAdd}
          />
        ) : null}
        {!showEmpty && viewMode === 'table' ? (
          <ProfileTable
            records={filtered}
            presentation={presentation}
            onSelect={onSelect}
            onEdit={onEdit}
            onApply={onApply}
            onToggle={onToggle}
            onDelete={onDelete}
          />
        ) : null}
        {!showEmpty && viewMode !== 'table' ? (
          <ProfileCardGrid
            records={filtered}
            presentation={presentation}
            inspectorOpen={inspectorOpen}
            onSelect={onSelect}
            onEdit={onEdit}
            onApply={onApply}
            onToggle={onToggle}
            onDelete={onDelete}
          />
        ) : null}
      </div>
      {inspectorOpen ? (
        <div data-testid="profiles-inspector">
          <ProfilesInspector
            profiles={[...records]}
            previewProfile={previewRecord}
            currentProfile={currentRecord}
            i18nPrefix="profilesSurface.inspector"
            descriptor={inspectorDescriptor}
            selectedTag={tagFilter}
            onEdit={onEdit}
            onLocate={onSelect}
            onTagSelect={onTagSelect}
          />
        </div>
      ) : null}
    </div>
  )
}
