import { codexApi } from '@/api'
import type {
  ProfileEditorAdapter,
  ProfileEditorIssue,
  ProfileWriteOutcome,
} from '@/configs/profileEditorAdapter'
import { translate } from '@/i18n'
import type { CodexProfile, CodexProfileAuthMode } from '@/types'
import {
  AVAILABLE_AUTH_MODES,
  CUSTOM_MODEL_OPTION,
  DEPRECATED_AUTH_MODES,
  REASONING_EFFORT_OPTIONS,
  buildCodexProfileRequest,
  codexProfileToEditorForm,
  createCodexProfileEditorForm,
  resolveModelSelection,
  usesOpenAiAuthMode,
  type CodexProfileEditorForm,
} from '@/utils/codexProfileEditor'

export type CodexEditorForm = CodexProfileEditorForm & { model: string }

const AUTH_MODES: readonly CodexProfileAuthMode[] = [...AVAILABLE_AUTH_MODES, ...DEPRECATED_AUTH_MODES]
const MODEL_CATALOG = ['gpt-5.6-sol', 'gpt-5.6-high', 'gpt-5.6-mini'] as const

const resolvedModelOf = (form: CodexEditorForm): string => {
  const selection = resolveModelSelection(form.model, [...MODEL_CATALOG])
  return selection.selectedModelOption === CUSTOM_MODEL_OPTION
    ? selection.customModelInput
    : selection.selectedModelOption
}

const asForm = (form: unknown): CodexEditorForm => form as CodexEditorForm

const requiresBaseUrl = (form: unknown) => !usesOpenAiAuthMode(asForm(form).auth_mode)

const requiresSecretMode = (form: unknown) => {
  const mode = asForm(form).auth_mode
  return (
    mode === 'openai_api_key' || mode === 'provider_env_key' || mode === 'provider_bearer_token'
  )
}

const requiresEnvKey = (form: unknown) => asForm(form).auth_mode === 'provider_env_key'

const omitEmptyAuthToken = (request: object): object => {
  const payload = { ...request } as Record<string, unknown>
  const token = payload.auth_token
  if (typeof token !== 'string' || !token.trim()) delete payload.auth_token
  return payload
}

const errorOf = (error: unknown): ProfileWriteOutcome => ({
  status: 'error',
  message: error instanceof Error ? error.message : String(error),
})

export const codexProfileEditorAdapter: ProfileEditorAdapter<CodexEditorForm, CodexProfile> = {
  createEmpty: () => ({ ...createCodexProfileEditorForm(), model: '' }),
  fromRecord: (record) => ({
    ...codexProfileToEditorForm(record),
    model: record.model ?? '',
    auth_token: '',
  }),
  sections: [
    {
      id: 'identity',
      titleKey: 'codex.profiles.sections.identity',
      layout: 'grid',
      fields: [
        { key: 'name', labelKey: 'codex.profiles.profileName', kind: 'mono-text', required: () => true },
        { key: 'description', labelKey: 'codex.profiles.description', kind: 'text' },
      ],
    },
    {
      id: 'auth',
      titleKey: 'codex.profiles.sections.authentication',
      layout: 'group',
      fields: [
        {
          key: 'auth_mode',
          labelKey: 'codex.profiles.fields.authMode',
          kind: 'choice',
          options: AUTH_MODES,
        },
        {
          key: 'base_url',
          labelKey: 'codex.profiles.baseUrl',
          kind: 'mono-text',
          visible: requiresBaseUrl,
          required: requiresBaseUrl,
        },
        {
          key: 'auth_token',
          labelKey: 'codex.profiles.authToken',
          kind: 'secret',
          visible: requiresSecretMode,
          required: requiresSecretMode,
        },
        {
          key: 'env_key',
          labelKey: 'codex.profiles.fields.envKey',
          kind: 'mono-text',
          visible: requiresEnvKey,
          required: requiresEnvKey,
        },
      ],
    },
    {
      id: 'runtime',
      titleKey: 'codex.profiles.sections.runtime',
      layout: 'grid',
      fields: [
        {
          key: 'model',
          labelKey: 'codex.profiles.model',
          kind: 'choice',
          options: MODEL_CATALOG,
          required: () => true,
        },
        { key: 'wire_api', labelKey: 'codex.profiles.fields.wireApi', kind: 'choice', options: ['responses', 'chat'] },
        {
          key: 'model_reasoning_effort',
          labelKey: 'codex.profiles.fields.reasoningEffort',
          kind: 'choice',
          options: [...REASONING_EFFORT_OPTIONS],
        },
        {
          key: 'tags_input',
          labelKey: 'codex.profiles.fields.tags',
          kind: 'multi-value',
          options: ['work', 'free', 'backup', 'test'],
        },
        { key: 'enabled', labelKey: 'codex.profiles.fields.enabled', kind: 'boolean' },
      ],
    },
    {
      id: 'advanced',
      titleKey: 'codex.profiles.sections.metadata',
      layout: 'grid',
      advanced: true,
      fields: [
        { key: 'model_catalog_json', labelKey: 'codex.profiles.fields.modelCatalogJson', kind: 'mono-text' },
        {
          key: 'preferred_auth_method',
          labelKey: 'codex.profiles.fields.preferredAuthMethod',
          kind: 'text',
          visible: (form) => asForm(form).auth_mode === 'provider_bearer_token',
        },
        {
          key: 'forced_login_method',
          labelKey: 'codex.profiles.fields.forcedLoginMethod',
          kind: 'text',
          visible: (form) => asForm(form).auth_mode === 'provider_bearer_token',
        },
        { key: 'provider', labelKey: 'codex.profiles.provider', kind: 'text' },
      ],
    },
  ],
  validate: (form, ctx) => {
    const issues: ProfileEditorIssue[] = []
    const resolvedModel = resolvedModelOf(form)
    if (!form.name.trim()) {
      issues.push({
        section: 'identity',
        field: 'name',
        message: translate('codex.profiles.validation.nameRequired'),
      })
    }
    if (requiresBaseUrl(form) && !form.base_url.trim()) {
      issues.push({
        section: 'auth',
        field: 'base_url',
        message: translate('codex.profiles.validation.baseUrlRequired'),
      })
    }
    if (requiresSecretMode(form) && !ctx.isEditing && !form.auth_token.trim()) {
      issues.push({
        section: 'auth',
        field: 'auth_token',
        message: translate('codex.profiles.validation.authTokenRequired'),
      })
    }
    if (requiresEnvKey(form) && !form.env_key.trim()) {
      issues.push({
        section: 'auth',
        field: 'env_key',
        message: translate('codex.profiles.validation.envKeyRequired'),
      })
    }
    if (!resolvedModel) {
      issues.push({
        section: 'runtime',
        field: 'model',
        message: translate('codex.profiles.validation.modelRequired'),
      })
    }
    return issues
  },
  submit: async (form, ctx) => {
    const payload = omitEmptyAuthToken(buildCodexProfileRequest(form, resolvedModelOf(form)))
    try {
      if (ctx.isEditing && ctx.originalName) {
        await codexApi.updateCodexProfile(ctx.originalName, payload)
      } else {
        await codexApi.addCodexProfile(payload)
      }
      return { status: 'ok', appliedName: form.name.trim() }
    } catch (error) {
      return errorOf(error)
    }
  },
}
