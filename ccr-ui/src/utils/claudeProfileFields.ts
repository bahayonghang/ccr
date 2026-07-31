import type { ClaudeProfile } from '@/types'

export const CLAUDE_FIELD_PLACEHOLDER = '—'

type ClaudeModelFields = Pick<
  ClaudeProfile,
  'model' | 'default_sonnet_model' | 'default_opus_model' | 'default_haiku_model' | 'subagent_model'
>

type ClaudeTranslate = (
  key: string,
  values?: Record<string, string | number | boolean | null | undefined>
) => string

/** Canonical Claude model fallback: model -> sonnet -> opus -> haiku -> subagent. */
export const resolveClaudePrimaryModel = (
  profile: ClaudeModelFields,
  fallback: string = CLAUDE_FIELD_PLACEHOLDER
): string =>
  profile.model?.trim() ||
  profile.default_sonnet_model?.trim() ||
  profile.default_opus_model?.trim() ||
  profile.default_haiku_model?.trim() ||
  profile.subagent_model?.trim() ||
  fallback

export const resolveClaudeDisplayBaseUrl = (
  profile: Pick<ClaudeProfile, 'base_url'>,
  t: ClaudeTranslate
): string => profile.base_url?.trim() || t('claudeProfiles.officialBaseUrl')
