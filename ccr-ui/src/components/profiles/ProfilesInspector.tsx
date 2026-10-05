import { useMemo } from 'react'
import { useShellT } from '@/shell/i18n'
import type {
  ProfilesInspectorDescriptor,
  ProfilesInspectorField,
  ProfilesInspectorProfile,
} from '@/utils/profileDescriptors'
import { buildProfileDiff } from '@/utils/profileDiff'
import { ProfilesInspectorAudit } from './ProfilesInspectorAudit'
import { ProfilesInspectorDistribution } from './ProfilesInspectorDistribution'
import { ProfilesInspectorPreview } from './ProfilesInspectorPreview'
import './profiles-shared.css'

export type { ProfilesInspectorDescriptor, ProfilesInspectorField, ProfilesInspectorProfile }

export interface ProfilesInspectorProps<T extends ProfilesInspectorProfile> {
  profiles: T[]
  /** 预览目标（视图按 hoveredName ?? focusedName ?? current 解析后传入） */
  previewProfile: T | null
  /** 当前激活 profile（diff 的 from 侧） */
  currentProfile: T | null
  /** i18n key 前缀，指向 inspector 子对象，例如 'claudeProfiles.inspector' */
  i18nPrefix: string
  descriptor: ProfilesInspectorDescriptor<T>
  /** 本次会话最近一次写入时间（仅预览目标=当前时由视图传入） */
  sessionWriteAt?: string | null
  /** 当前生效的标签筛选（tag cloud aria-pressed 同步） */
  selectedTag?: string | null
  onEdit: (name: string) => void
  onLocate: (name: string) => void
  onTagSelect: (tag: string) => void
}

const PREVIEW_HEADING_ID = 'cp-inspector-preview-heading'
const AUDIT_HEADING_ID = 'cp-inspector-audit-heading'

/** Profiles 右侧检查器：预览 + Health Audit + Distribution。 */
export function ProfilesInspector<T extends ProfilesInspectorProfile>({
  profiles,
  previewProfile,
  currentProfile,
  i18nPrefix,
  descriptor,
  sessionWriteAt = null,
  selectedTag = null,
  onEdit,
  onLocate,
  onTagSelect,
}: ProfilesInspectorProps<T>) {
  const t = useShellT()
  const insights = useMemo(() => descriptor.useInsights(profiles), [descriptor, profiles])
  const previewFields = previewProfile ? descriptor.activeFields(previewProfile) : []
  const previewTags = previewProfile?.tags ?? []
  const isPreviewingCurrent = Boolean(
    previewProfile && currentProfile && previewProfile.name === currentProfile.name,
  )
  const diffRows = useMemo(() => {
    if (!previewProfile || !currentProfile || previewProfile.name === currentProfile.name) return []
    return buildProfileDiff(currentProfile, previewProfile, descriptor.diffFields)
  }, [currentProfile, descriptor.diffFields, previewProfile])
  const visibleAuthModeBreakdown = insights.authModeBreakdown.filter((item) => item.count > 0)

  return (
    <aside className="cp-inspector" aria-label={t(`${i18nPrefix}.ariaLabel`)}>
      <ProfilesInspectorPreview
        i18nPrefix={i18nPrefix}
        headingId={PREVIEW_HEADING_ID}
        previewProfile={previewProfile}
        isPreviewingCurrent={isPreviewingCurrent}
        previewFields={previewFields}
        previewTags={previewTags}
        diffRows={diffRows}
        sessionWriteAt={sessionWriteAt}
        editIcon={descriptor.editIcon}
        onEdit={onEdit}
      />

      <ProfilesInspectorAudit
        i18nPrefix={i18nPrefix}
        headingId={AUDIT_HEADING_ID}
        totalIssueCount={insights.totalIssueCount}
        deprecatedAuthIssues={insights.deprecatedAuthIssues}
        missingFieldIssues={insights.missingFieldIssues}
        duplicateRuntimeIssues={insights.duplicateRuntimeIssues}
        descriptor={descriptor}
        onLocate={onLocate}
      />

      <ProfilesInspectorDistribution
        i18nPrefix={i18nPrefix}
        providerBreakdown={insights.providerBreakdown}
        authModeBreakdown={visibleAuthModeBreakdown}
        topTags={insights.topTags}
        selectedTag={selectedTag}
        descriptor={descriptor}
        onTagSelect={onTagSelect}
      />
    </aside>
  )
}
