import { useCallback } from 'react'
import { SIcon } from '@/ui'
import type { UsageOpsAction, UsageOpsCockpitPresentation } from '@/views/usage/usageOpsCockpit'
import '../styles/usage-stale-banner.css'

interface UsageStaleBannerProps {
  presentation: UsageOpsCockpitPresentation
  onPrimaryAction: (action: UsageOpsAction) => void
  onSecondaryAction: () => void
}

export function UsageStaleBanner({
  presentation,
  onPrimaryAction,
  onSecondaryAction,
}: UsageStaleBannerProps) {
  const handlePrimary = useCallback(() => {
    if (presentation.primaryAction) onPrimaryAction(presentation.primaryAction)
  }, [onPrimaryAction, presentation.primaryAction])

  if (presentation.state === 'ready') return null

  const toneIcon =
    presentation.tone === 'info' ? 'RefreshCw' : presentation.tone === 'muted' ? 'AlertCircle' : 'AlertTriangle'

  return (
    <div
      className={`usage-stale-banner usage-stale-banner--${presentation.tone}`}
      role="status"
      data-usage-stale-banner
    >
      <SIcon name={toneIcon} size="w-4 h-4" className="usage-stale-banner__icon" />
      <div className="usage-stale-banner__copy">
        <p className="usage-stale-banner__title">
          <span>{presentation.title}</span>
          {presentation.freshnessAgeLabel ? (
            <span className="usage-stale-banner__age">{presentation.freshnessAgeLabel}</span>
          ) : null}
        </p>
        <p className="usage-stale-banner__detail">{presentation.detail}</p>
      </div>
      <div className="usage-stale-banner__actions">
        {presentation.primaryActionLabel ? (
          <button
            type="button"
            className="usage-stale-banner__action usage-stale-banner__action--primary"
            onClick={handlePrimary}
          >
            {presentation.primaryActionLabel}
          </button>
        ) : null}
        {presentation.secondaryActionLabel ? (
          <button type="button" className="usage-stale-banner__action" onClick={onSecondaryAction}>
            {presentation.secondaryActionLabel}
          </button>
        ) : null}
      </div>
    </div>
  )
}
