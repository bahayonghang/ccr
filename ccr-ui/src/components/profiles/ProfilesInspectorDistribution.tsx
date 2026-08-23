import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui'
import type { ProfilesInspectorDescriptor, ProfilesInspectorProfile } from '@/utils/profileDescriptors'
import type { AuthModeBreakdownItem, ProviderBreakdownItem, TagFrequencyItem } from '@/utils/profilesInsights'

interface ProfilesInspectorDistributionProps<T extends ProfilesInspectorProfile> {
  i18nPrefix: string
  providerBreakdown: ProviderBreakdownItem[]
  authModeBreakdown: AuthModeBreakdownItem<string>[]
  topTags: TagFrequencyItem[]
  selectedTag: string | null
  descriptor: Pick<ProfilesInspectorDescriptor<T>, 'authModeLabel' | 'isDeprecatedMode'>
  onTagSelect: (tag: string) => void
}

export function ProfilesInspectorDistribution<T extends ProfilesInspectorProfile>({
  i18nPrefix,
  providerBreakdown,
  authModeBreakdown,
  topTags,
  selectedTag,
  descriptor,
  onTagSelect,
}: ProfilesInspectorDistributionProps<T>) {
  const t = useShellT()

  return (
    <details className="cp-inspector-card cp-inspector-details surface-card">
      <summary
        className="cp-inspector-card__head cp-inspector-details__summary"
        aria-label={t(`${i18nPrefix}.distributionTitle`)}
      >
        <SIcon name="BarChart3" size="w-3.5 h-3.5" className="cp-inspector-card__icon" />
        <span className="cp-inspector-card__title">{t(`${i18nPrefix}.distributionTitle`)}</span>
        <SIcon name="ChevronDown" size="w-3.5 h-3.5" className="cp-inspector-details__chevron" />
      </summary>

      <div className="cp-inspector-details__body">
        <div className="cp-inspector-section">
          <div className="cp-inspector-section__head">{t(`${i18nPrefix}.providerSection`)}</div>
          {providerBreakdown.length > 0 ? (
            <ul className="cp-inspector-bars" role="presentation">
              {providerBreakdown.map((item) => (
                <li key={item.provider} className="cp-inspector-bar">
                  <div className="cp-inspector-bar__label">
                    {item.provider === 'Unknown' ? t(`${i18nPrefix}.unknownProvider`) : item.provider}
                  </div>
                  <div className="cp-inspector-bar__track">
                    <div
                      className="cp-inspector-bar__fill"
                      style={{ width: `${Math.max(item.pct, 4)}%` }}
                    />
                  </div>
                  <div className="cp-inspector-bar__value">{item.count}</div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="cp-inspector-section__empty">—</div>
          )}
        </div>

        {authModeBreakdown.length > 0 ? (
          <div className="cp-inspector-section">
            <div className="cp-inspector-section__head">{t(`${i18nPrefix}.authSection`)}</div>
            <ul className="cp-inspector-bars" role="presentation">
              {authModeBreakdown.map((item) => (
                <li key={item.mode} className="cp-inspector-bar">
                  <div className="cp-inspector-bar__label">{descriptor.authModeLabel(item.mode)}</div>
                  <div className="cp-inspector-bar__track">
                    <div
                      className={
                        descriptor.isDeprecatedMode(item.mode)
                          ? 'cp-inspector-bar__fill cp-inspector-bar__fill--warn'
                          : 'cp-inspector-bar__fill'
                      }
                      style={{ width: `${Math.max(item.pct, 4)}%` }}
                    />
                  </div>
                  <div className="cp-inspector-bar__value">{item.count}</div>
                </li>
              ))}
            </ul>
          </div>
        ) : null}

        <div className="cp-inspector-section">
          <div className="cp-inspector-section__head">{t(`${i18nPrefix}.tagsSection`)}</div>
          {topTags.length > 0 ? (
            <div className="cp-inspector-tagcloud">
              {topTags.map((item) => (
                <button
                  key={item.tag}
                  type="button"
                  className="cp-inspector-tag cp-inspector-tag--count cp-inspector-tag--clickable"
                  aria-pressed={item.tag === selectedTag}
                  onClick={() => onTagSelect(item.tag)}
                >
                  <span>#{item.tag}</span>
                  <span className="cp-inspector-tag__count">{item.count}</span>
                </button>
              ))}
            </div>
          ) : (
            <div className="cp-inspector-section__empty">{t(`${i18nPrefix}.noTags`)}</div>
          )}
        </div>
      </div>
    </details>
  )
}
