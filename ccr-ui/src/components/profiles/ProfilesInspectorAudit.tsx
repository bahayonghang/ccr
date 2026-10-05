import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui'
import type { ProfilesInspectorDescriptor, ProfilesInspectorProfile } from '@/utils/profileDescriptors'
import type { DuplicateRuntimeIssue, MissingFieldIssue } from '@/utils/profilesInsights'

interface ProfilesInspectorAuditProps<T extends ProfilesInspectorProfile> {
  i18nPrefix: string
  headingId: string
  totalIssueCount: number
  deprecatedAuthIssues: T[]
  missingFieldIssues: MissingFieldIssue<T, string>[]
  duplicateRuntimeIssues: DuplicateRuntimeIssue<T>[]
  descriptor: Pick<
    ProfilesInspectorDescriptor<T>,
    'deprecatedMessage' | 'missingMessage' | 'runtimeSummary'
  >
  onLocate: (name: string) => void
}

export function ProfilesInspectorAudit<T extends ProfilesInspectorProfile>({
  i18nPrefix,
  headingId,
  totalIssueCount,
  deprecatedAuthIssues,
  missingFieldIssues,
  duplicateRuntimeIssues,
  descriptor,
  onLocate,
}: ProfilesInspectorAuditProps<T>) {
  const t = useShellT()

  return (
    <section className="cp-inspector-card surface-card" aria-labelledby={headingId}>
      <header className="cp-inspector-card__head">
        <SIcon name="ShieldCheck" size="w-3.5 h-3.5" className="cp-inspector-card__icon" />
        <h3 id={headingId} className="cp-inspector-card__title">
          {t(`${i18nPrefix}.auditTitle`)}
        </h3>
        <span
          className={
            totalIssueCount > 0
              ? 'cp-inspector-card__count cp-inspector-card__count--warn'
              : 'cp-inspector-card__count'
          }
        >
          {totalIssueCount}
        </span>
      </header>

      {totalIssueCount === 0 ? (
        <div className="cp-inspector-clean">
          <SIcon name="CheckCircle" size="w-4 h-4" />
          <span>{t(`${i18nPrefix}.auditClean`)}</span>
        </div>
      ) : (
        <ul className="cp-inspector-issues">
          {deprecatedAuthIssues.map((profile) => (
            <li key={`dep-${profile.name}`}>
              <button
                type="button"
                className="cp-inspector-issue cp-inspector-issue--warn"
                aria-label={t(`${i18nPrefix}.locateAction`, { name: profile.name })}
                onClick={() => onLocate(profile.name)}
              >
                <SIcon name="AlertCircle" size="w-3.5 h-3.5" className="cp-inspector-issue__icon" />
                <div className="cp-inspector-issue__body">
                  <div className="cp-inspector-issue__name">{profile.name}</div>
                  <div className="cp-inspector-issue__msg">{descriptor.deprecatedMessage?.(profile)}</div>
                </div>
                <SIcon name="Target" size="w-3.5 h-3.5" className="cp-inspector-issue__locate" />
              </button>
            </li>
          ))}

          {missingFieldIssues.map((issue) => (
            <li key={`miss-${issue.profile.name}`}>
              <button
                type="button"
                className="cp-inspector-issue cp-inspector-issue--danger"
                aria-label={t(`${i18nPrefix}.locateAction`, { name: issue.profile.name })}
                onClick={() => onLocate(issue.profile.name)}
              >
                <SIcon name="AlertTriangle" size="w-3.5 h-3.5" className="cp-inspector-issue__icon" />
                <div className="cp-inspector-issue__body">
                  <div className="cp-inspector-issue__name">{issue.profile.name}</div>
                  <div className="cp-inspector-issue__msg">{descriptor.missingMessage(issue.missing)}</div>
                </div>
                <SIcon name="Target" size="w-3.5 h-3.5" className="cp-inspector-issue__locate" />
              </button>
            </li>
          ))}

          {duplicateRuntimeIssues.map((group) => (
            <li key={`dup-${group.key}`} className="cp-inspector-issue-group">
              <div className="cp-inspector-issue-group__head">
                <SIcon name="Copy" size="w-3.5 h-3.5" className="cp-inspector-issue__icon" />
                <span>
                  {t(`${i18nPrefix}.issues.duplicateRuntime`, { count: group.profiles.length })}
                </span>
              </div>
              {group.profiles.map((profile) => (
                <button
                  key={`dup-${group.key}-${profile.name}`}
                  type="button"
                  className="cp-inspector-issue cp-inspector-issue--info cp-inspector-issue--nested"
                  aria-label={t(`${i18nPrefix}.locateAction`, { name: profile.name })}
                  onClick={() => onLocate(profile.name)}
                >
                  <div className="cp-inspector-issue__body">
                    <div className="cp-inspector-issue__name">{profile.name}</div>
                    <div className="cp-inspector-issue__msg cp-inspector-issue__msg--mono">
                      {descriptor.runtimeSummary(profile)}
                    </div>
                  </div>
                  <SIcon name="Target" size="w-3.5 h-3.5" className="cp-inspector-issue__locate" />
                </button>
              ))}
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
