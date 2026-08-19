import type {
  GrokApiBackend,
  GrokCredentialAction,
  GrokProfileCreateRequest,
  GrokProfileDto,
  GrokProfileKindDto,
  GrokProfilePatch,
  GrokReasoningEffort,
} from '@/types'

export const GROK_REASONING_EFFORT_OPTIONS: readonly GrokReasoningEffort[] = [
  'none',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
]

export const GROK_API_BACKEND_OPTIONS: readonly GrokApiBackend[] = [
  'chat_completions',
  'responses',
  'messages',
]

export interface GrokProfileEditorForm {
  name: string
  description: string
  profileKind: GrokProfileKindDto
  baseUrl: string
  model: string
  provider: string
  enabled: boolean
  tagsInput: string
  apiBackend: '' | GrokApiBackend
  contextWindow: string
  supportsBackendSearch: boolean
  reasoningEffort: '' | GrokReasoningEffort
  credentialAction: GrokCredentialAction
  apiKey: string
  envKey: string
}

export type GrokProfileDirtyField = keyof GrokProfileEditorForm

const optionalText = (value: string): string | null => value.trim() || null

export const parseGrokTags = (value: string): string[] | null => {
  const tags = value
    .split(',')
    .map(tag => tag.trim())
    .filter(Boolean)
  return tags.length > 0 ? [...new Set(tags)] : null
}

const optionalPositiveInteger = (value: string): number | undefined => {
  const trimmed = value.trim()
  if (!trimmed) return undefined
  const parsed = Number(trimmed)
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined
}

export const createEmptyGrokForm = (): GrokProfileEditorForm => ({
  name: '',
  description: '',
  profileKind: 'third_party',
  baseUrl: '',
  model: 'grok-4.6',
  provider: '',
  enabled: true,
  tagsInput: 'work',
  apiBackend: 'responses',
  contextWindow: '500000',
  supportsBackendSearch: true,
  reasoningEffort: 'high',
  credentialAction: 'replace_api_key',
  apiKey: '',
  envKey: '',
})

export const fillGrokForm = (profile: GrokProfileDto): GrokProfileEditorForm => ({
  name: profile.name,
  description: profile.description ?? '',
  profileKind: profile.profile_kind,
  // Display-safe URLs are never copied into the write field.
  baseUrl: '',
  model: profile.model ?? '',
  provider: profile.provider ?? '',
  enabled: profile.enabled,
  tagsInput: profile.tags.join(', '),
  apiBackend: (profile.api_backend as GrokApiBackend | null) ?? '',
  contextWindow: profile.context_window ? String(profile.context_window) : '',
  supportsBackendSearch: profile.supports_backend_search ?? false,
  reasoningEffort: (profile.reasoning_effort as GrokReasoningEffort | null) ?? '',
  credentialAction: 'preserve',
  apiKey: '',
  envKey: '',
})

const addCredentialFields = (
  target: GrokProfileCreateRequest | GrokProfilePatch,
  form: GrokProfileEditorForm,
) => {
  target.credential_action = form.credentialAction
  if (form.credentialAction === 'replace_api_key') target.api_key = form.apiKey.trim()
  if (form.credentialAction === 'replace_env_key') target.env_key = form.envKey.trim()
}

export const buildGrokCreateRequest = (
  form: GrokProfileEditorForm,
): GrokProfileCreateRequest => {
  const request: GrokProfileCreateRequest = {
    name: form.name.trim(),
    description: optionalText(form.description),
    profile_kind: form.profileKind,
    provider: optionalText(form.provider),
    enabled: form.enabled,
    tags: parseGrokTags(form.tagsInput),
    credential_action: form.credentialAction,
  }

  if (form.model.trim()) request.model = form.model.trim()
  if (form.profileKind === 'third_party') {
    request.base_url = form.baseUrl.trim()
    if (form.apiBackend) request.api_backend = form.apiBackend
    const contextWindow = optionalPositiveInteger(form.contextWindow)
    if (contextWindow) request.context_window = contextWindow
    request.supports_backend_search = form.supportsBackendSearch
  }
  if (form.reasoningEffort) request.reasoning_effort = form.reasoningEffort
  if (form.profileKind === 'official') {
    request.credential_action = 'preserve'
  } else {
    addCredentialFields(request, form)
  }
  return request
}

export const buildGrokPatch = (
  form: GrokProfileEditorForm,
  dirtyFields: ReadonlySet<GrokProfileDirtyField>,
): GrokProfilePatch => {
  const patch: GrokProfilePatch = {}

  if (dirtyFields.has('name')) patch.name = form.name.trim()
  if (dirtyFields.has('description')) patch.description = optionalText(form.description)
  if (dirtyFields.has('provider')) patch.provider = optionalText(form.provider)
  if (dirtyFields.has('enabled')) patch.enabled = form.enabled
  if (dirtyFields.has('tagsInput')) patch.tags = parseGrokTags(form.tagsInput)
  if (dirtyFields.has('model')) patch.model = optionalText(form.model)
  if (dirtyFields.has('apiBackend')) patch.api_backend = form.apiBackend || null
  if (dirtyFields.has('contextWindow')) {
    patch.context_window = optionalPositiveInteger(form.contextWindow) ?? null
  }
  if (dirtyFields.has('supportsBackendSearch')) {
    patch.supports_backend_search = form.supportsBackendSearch
  }
  if (dirtyFields.has('reasoningEffort')) {
    patch.reasoning_effort = form.reasoningEffort || null
  }
  if (dirtyFields.has('baseUrl') && form.baseUrl.trim()) patch.base_url = form.baseUrl.trim()

  if (dirtyFields.has('profileKind')) {
    patch.profile_kind = form.profileKind
    if (form.profileKind === 'official') {
      patch.base_url = null
      patch.credential_action = 'clear'
    }
  }

  if (dirtyFields.has('credentialAction') || form.credentialAction !== 'preserve') {
    addCredentialFields(patch, form)
  }

  return patch
}
