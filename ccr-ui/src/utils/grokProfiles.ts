import type { Ref } from 'vue'
import type { GrokAuthModeDto, GrokProfileDto } from '@/types'
import type { ProfileRowDescriptor } from '@/components/profiles/ProfileListRow.vue'
import type {
  ProfilesInspectorDescriptor,
  ProfilesInspectorField,
} from '@/components/profiles/ProfilesInspector.vue'
import { useProfilesInsights, type ProfilesInsights } from '@/composables/useProfilesInsights'
import type { ProfileDiffField } from '@/utils/profileDiff'
import { formatBaseUrlDisplay } from '@/utils/text'

export const GROK_FIELD_PLACEHOLDER = '—'

type GrokTranslate = (
  key: string,
  values?: Record<string, string | number | boolean | null | undefined>,
) => string

type GrokMissingField = 'base_url' | 'model' | 'reasoning_effort'
type GrokProfilesInsights = ProfilesInsights<GrokProfileDto, GrokAuthModeDto, GrokMissingField>

export const grokAuthModeLabel = (t: GrokTranslate, mode?: GrokAuthModeDto | null) => (
  t(`grok.profiles.authModes.${mode ?? 'session'}`)
)

export const resolveGrokBaseUrl = (profile: GrokProfileDto, t: GrokTranslate) => (
  profile.base_url_display?.trim() || t('grok.profiles.officialBaseUrl')
)

const useGrokProfilesInsights = (profiles: Ref<GrokProfileDto[]>): GrokProfilesInsights => (
  useProfilesInsights<GrokProfileDto, GrokAuthModeDto, GrokMissingField>(profiles, {
    authModes: ['inline_api_key', 'env_key', 'session'],
    authModeOf: profile => profile.auth_mode,
    missingFieldsOf: (profile) => {
      const missing: GrokMissingField[] = []
      if (profile.profile_kind === 'third_party' && !profile.has_base_url) missing.push('base_url')
      if (profile.profile_kind === 'third_party' && !profile.model?.trim()) missing.push('model')
      if (!profile.reasoning_effort) missing.push('reasoning_effort')
      return missing
    },
    primaryRuntimeModel: profile => profile.model?.trim() ?? '',
  })
)

export const createGrokDiffFields = (t: GrokTranslate): ProfileDiffField<GrokProfileDto>[] => [
  {
    key: 'model',
    label: t('grok.profiles.fields.model'),
    value: profile => profile.model?.trim() ?? '',
  },
  {
    key: 'base_url_display',
    label: t('grok.profiles.fields.baseUrl'),
    value: profile => resolveGrokBaseUrl(profile, t),
  },
  {
    key: 'auth_mode',
    label: t('grok.profiles.fields.authMode'),
    value: profile => grokAuthModeLabel(t, profile.auth_mode),
  },
  {
    key: 'reasoning_effort',
    label: t('grok.profiles.fields.reasoningEffort'),
    value: profile => profile.reasoning_effort ?? '',
  },
]

export const createGrokRowDescriptor = (
  t: GrokTranslate,
): ProfileRowDescriptor<GrokProfileDto> => ({
  baseUrl: profile => formatBaseUrlDisplay(resolveGrokBaseUrl(profile, t)),
  model: profile => profile.model?.trim() || GROK_FIELD_PLACEHOLDER,
  authMode: profile => grokAuthModeLabel(t, profile.auth_mode),
  editIcon: 'Edit2',
  labels: {
    apply: t('grok.profiles.actions.apply'),
    edit: t('grok.profiles.actions.edit'),
    delete: t('grok.profiles.actions.delete'),
  },
})

const inspectorFields = (profile: GrokProfileDto, t: GrokTranslate): ProfilesInspectorField[] => {
  const fields: ProfilesInspectorField[] = [
    { label: t('grok.profiles.fields.baseUrl'), value: resolveGrokBaseUrl(profile, t) },
    {
      label: t('grok.profiles.fields.model'),
      value: profile.model?.trim() || GROK_FIELD_PLACEHOLDER,
      variant: 'accent',
    },
    {
      label: t('grok.profiles.fields.authMode'),
      value: grokAuthModeLabel(t, profile.auth_mode),
      variant: 'muted',
    },
  ]
  for (const [label, value] of [
    [t('grok.profiles.fields.apiBackend'), profile.api_backend],
    [t('grok.profiles.fields.reasoningEffort'), profile.reasoning_effort],
    [t('grok.profiles.fields.contextWindow'), profile.context_window?.toString()],
    [t('grok.profiles.fields.envKey'), profile.env_key],
  ] as const) {
    if (value) fields.push({ label, value, variant: 'muted' })
  }
  return fields
}

export const createGrokInspectorDescriptor = (
  t: GrokTranslate,
): ProfilesInspectorDescriptor<GrokProfileDto> => ({
  editIcon: 'Edit2',
  useInsights: useGrokProfilesInsights,
  activeFields: profile => inspectorFields(profile, t),
  diffFields: createGrokDiffFields(t),
  authModeLabel: mode => grokAuthModeLabel(t, mode as GrokAuthModeDto),
  isDeprecatedMode: () => false,
  missingMessage: missing => missing
    .map(field => t(`grok.profiles.inspector.issues.missing.${field}`))
    .join(' · '),
  runtimeSummary: profile => (
    `${profile.model?.trim() || GROK_FIELD_PLACEHOLDER} · ${profile.base_url_display?.trim() || GROK_FIELD_PLACEHOLDER}`
  ),
})
