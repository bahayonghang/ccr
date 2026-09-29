import { translate } from '@/i18n'
import type { ProfileOutcome } from '@/types/generated/profiles/ProfileOutcome'

export function profileOutcome(value: unknown): ProfileOutcome | null {
  if (!value || typeof value !== 'object' || !('outcome' in value)) return null
  const candidate = value.outcome
  if (!candidate || typeof candidate !== 'object' || !('status' in candidate)) {
    throw new Error(translate('profilesSurface.recoveryRequired'))
  }
  const result = candidate as ProfileOutcome
  if (!['unchanged', 'applied', 'applied_with_warning', 'recovery_required'].includes(result.status)
    || typeof result.activation_committed !== 'boolean'
    || !Array.isArray(result.warnings)) {
    throw new Error(translate('profilesSurface.recoveryRequired'))
  }
  return result
}

/** Failure results stop subsequent actions. Warnings never repeat activation. */
export function requireProfileOutcome(value: unknown): ProfileOutcome | null {
  const result = profileOutcome(value)
  if (result?.status === 'recovery_required') throw new Error(translate('profilesSurface.recoveryRequired'))
  if (result?.status === 'unchanged' && !result.activation_committed) {
    throw new Error(translate('profilesSurface.unchangedFailure'))
  }
  return result
}

export function profileOutcomeWarning(value: unknown): string | undefined {
  return requireProfileOutcome(value)?.status === 'applied_with_warning'
    ? translate('profilesSurface.appliedWithWarning')
    : undefined
}
