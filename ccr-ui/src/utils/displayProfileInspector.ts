import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import type {
  ProfilesInspectorDescriptor,
  ProfilesInspectorField,
} from '@/utils/profileDescriptors'
import type { ProfileDiffField } from '@/utils/profileDiff'
import type { TranslateFunction } from '@/utils/tf'

const emptyInsights = () => ({
  providerBreakdown: [] as { provider: string; count: number; pct: number }[],
  authModeBreakdown: [] as { mode: string; count: number; pct: number }[],
  topTags: [] as { tag: string; count: number }[],
  deprecatedAuthIssues: [] as ProfileDisplayRecord[],
  missingFieldIssues: [] as { profile: ProfileDisplayRecord; missing: string[] }[],
  duplicateRuntimeIssues: [] as { key: string; profiles: ProfileDisplayRecord[] }[],
  totalIssueCount: 0,
})

const activeFieldsOf = (
  record: ProfileDisplayRecord,
  presentation: ProfilePresentationView,
  t: TranslateFunction,
): ProfilesInspectorField[] =>
  presentation.fieldSlots.map((slot, index) => ({
    label: t(slot.labelKey),
    value: record.slots[index] || '—',
    variant: slot.chip ? 'accent' : undefined,
  }))

const diffFieldsOf = (
  presentation: ProfilePresentationView,
  t: TranslateFunction,
): ProfileDiffField<ProfileDisplayRecord>[] =>
  presentation.fieldSlots.map((slot, index) => ({
    key: slot.labelKey,
    label: t(slot.labelKey),
    value: (record) => record.slots[index] || null,
  }))

/** 由 presentation 注入 Inspector，组件内不写平台分支。 */
export function makeDisplayInspectorDescriptor(
  presentation: ProfilePresentationView,
  t: TranslateFunction,
): ProfilesInspectorDescriptor<ProfileDisplayRecord> {
  return {
    editIcon: 'Pencil',
    useInsights: emptyInsights,
    activeFields: (record) => activeFieldsOf(record, presentation, t),
    diffFields: diffFieldsOf(presentation, t),
    authModeLabel: (mode) => t(`profilePresentation.auth.${mode}`),
    isDeprecatedMode: () => false,
    missingMessage: (missing) => missing.join(', '),
    runtimeSummary: (record) => record.name,
  }
}
