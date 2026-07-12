import type {
  CodexProfile,
  CodexProfileAuthMode,
  CodexProfileRequest,
  OpenAiLoginMethod,
} from '@/types'

export const CUSTOM_MODEL_OPTION = '__custom__'

export const AVAILABLE_AUTH_MODES: CodexProfileAuthMode[] = ['openai_api_key', 'no_auth']
export const DEPRECATED_AUTH_MODES: CodexProfileAuthMode[] = ['openai_chatgpt', 'provider_env_key']
export const REASONING_EFFORT_OPTIONS = ['minimal', 'low', 'medium', 'high', 'xhigh'] as const

export interface CodexProfileEditorForm {
  name: string
  description: string
  base_url: string
  auth_token: string
  provider: string
  provider_type: string
  tags_input: string
  enabled: boolean
  wire_api: string
  env_key: string
  requires_openai_auth: boolean
  auth_mode: CodexProfileAuthMode
  openai_login_method: OpenAiLoginMethod | null
  model_reasoning_effort: string
}

export const authModeToLoginMethod = (
  authMode: CodexProfileAuthMode,
): OpenAiLoginMethod | undefined => {
  switch (authMode) {
    case 'openai_chatgpt':
      return 'chatgpt'
    case 'openai_api_key':
      return 'api'
    default:
      return undefined
  }
}

export const usesOpenAiAuthMode = (authMode: CodexProfileAuthMode) => {
  return authMode === 'openai_chatgpt' || authMode === 'openai_api_key'
}

export const isDeprecatedAuthMode = (authMode?: CodexProfileAuthMode | null) => {
  return authMode ? DEPRECATED_AUTH_MODES.includes(authMode) : false
}

export const normalizeModelName = (value?: string | null) => value?.trim() || ''

export const buildCodexProfileModelCatalog = (
  presets: string[],
  currentModel?: string | null,
) => {
  const catalog = presets
    .map(normalizeModelName)
    .filter((model, index, models) => Boolean(model) && models.indexOf(model) === index)
  const normalizedCurrentModel = normalizeModelName(currentModel)

  if (normalizedCurrentModel && !catalog.includes(normalizedCurrentModel)) {
    catalog.push(normalizedCurrentModel)
  }

  return catalog
}

export const createCodexProfileEditorForm = (): CodexProfileEditorForm => ({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  provider: '',
  provider_type: '',
  tags_input: '',
  enabled: true,
  wire_api: '',
  env_key: '',
  requires_openai_auth: false,
  auth_mode: 'no_auth',
  openai_login_method: null,
  model_reasoning_effort: '',
})

export const codexProfileToEditorForm = (profile: CodexProfile): CodexProfileEditorForm => ({
  name: profile.name,
  description: profile.description || '',
  base_url: profile.base_url || '',
  auth_token: profile.auth_token || '',
  provider: profile.provider || '',
  provider_type: profile.provider_type || '',
  tags_input: (profile.tags || []).join(', '),
  enabled: profile.enabled !== false,
  wire_api: profile.wire_api || '',
  env_key: profile.env_key || '',
  requires_openai_auth:
    profile.requires_openai_auth ?? usesOpenAiAuthMode(profile.auth_mode || 'no_auth'),
  auth_mode: profile.auth_mode || 'no_auth',
  openai_login_method:
    profile.openai_login_method || authModeToLoginMethod(profile.auth_mode || 'no_auth') || null,
  model_reasoning_effort: profile.model_reasoning_effort || '',
})

export const resolveModelSelection = (model: string | null | undefined, catalog: string[]) => {
  const normalizedModel = normalizeModelName(model)
  if (normalizedModel && catalog.includes(normalizedModel)) {
    return {
      selectedModelOption: normalizedModel,
      customModelInput: '',
    }
  }

  return {
    selectedModelOption: CUSTOM_MODEL_OPTION,
    customModelInput: normalizedModel,
  }
}

export const normalizeOptionalText = (value: string) => {
  const trimmed = value.trim()
  return trimmed ? trimmed : null
}

export const parseTagsInput = (raw: string): string[] | null => {
  const tags = raw
    .split(',')
    .map(tag => tag.trim())
    .filter(Boolean)

  return tags.length > 0 ? tags : null
}

export const buildCodexProfileRequest = (
  form: CodexProfileEditorForm,
  resolvedModel: string,
): CodexProfileRequest => ({
  name: form.name.trim(),
  description: normalizeOptionalText(form.description),
  base_url: normalizeOptionalText(form.base_url),
  auth_token: normalizeOptionalText(form.auth_token),
  model: resolvedModel,
  small_fast_model: null,
  provider: normalizeOptionalText(form.provider),
  provider_type: normalizeOptionalText(form.provider_type),
  tags: parseTagsInput(form.tags_input),
  enabled: form.enabled,
  wire_api: normalizeOptionalText(form.wire_api),
  env_key: normalizeOptionalText(form.env_key),
  requires_openai_auth: usesOpenAiAuthMode(form.auth_mode),
  auth_mode: form.auth_mode,
  openai_login_method: authModeToLoginMethod(form.auth_mode) ?? null,
  model_reasoning_effort: normalizeOptionalText(form.model_reasoning_effort),
  extra: null,
})
