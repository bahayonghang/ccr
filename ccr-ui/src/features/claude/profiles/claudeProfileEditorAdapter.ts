import { claudeApi } from '@/api'
import type {
  ProfileEditorAdapter,
  ProfileEditorIssue,
  ProfileWriteOutcome,
} from '@/configs/profileEditorAdapter'
import { translate } from '@/i18n'
import type { ClaudeProfile } from '@/types'
import type { ClaudeProfileEditorForm } from '@/types/claudeProfileEditor'
import {
  buildClaudeProfileRequest,
  createClaudeProfileForm,
  fillClaudeProfileForm,
} from '@/utils/claudeProfileEditor'

const asForm = (form: unknown): ClaudeProfileEditorForm => form as ClaudeProfileEditorForm

const isApiKey = (form: unknown) => asForm(form).auth_mode === 'api_key'

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

export const claudeProfileEditorAdapter: ProfileEditorAdapter<
  ClaudeProfileEditorForm,
  ClaudeProfile
> = {
  createEmpty: createClaudeProfileForm,
  fromRecord: (record) => {
    const form = createClaudeProfileForm()
    fillClaudeProfileForm(form, record)
    form.auth_token = ''
    return form
  },
  sections: [
    {
      id: 'identity',
      titleKey: 'claudeProfiles.sections.basic.title',
      layout: 'grid',
      fields: [
        { key: 'name', labelKey: 'claudeProfiles.nameLabel', kind: 'mono-text', required: () => true },
        { key: 'description', labelKey: 'claudeProfiles.descLabel', kind: 'text' },
      ],
    },
    {
      id: 'connection',
      titleKey: 'claudeProfiles.sections.connection.title',
      layout: 'group',
      fields: [
        {
          key: 'auth_mode',
          labelKey: 'claudeProfiles.authModeLabel',
          kind: 'choice',
          options: ['subscription', 'api_key'],
        },
        {
          key: 'base_url',
          labelKey: 'claudeProfiles.baseUrlLabel',
          kind: 'mono-text',
          visible: isApiKey,
          required: isApiKey,
        },
        {
          key: 'auth_token',
          labelKey: 'claudeProfiles.authTokenLabel',
          kind: 'secret',
          visible: isApiKey,
          required: (form) => isApiKey(form),
        },
      ],
    },
    {
      id: 'runtime',
      titleKey: 'claudeProfiles.sections.auth.title',
      layout: 'grid',
      fields: [
        {
          key: 'default_sonnet_model',
          labelKey: 'claudeProfiles.defaultSonnetModelLabel',
          kind: 'choice',
          options: ['claude-sonnet-4-6', 'claude-opus-4-1', 'claude-haiku-4-5'],
        },
        {
          key: 'effort_level',
          labelKey: 'claudeProfiles.effortLevelLabel',
          kind: 'choice',
          options: ['low', 'medium', 'high', 'xhigh', 'max'],
        },
        { key: 'tagsInput', labelKey: 'claudeProfiles.tagsLabel', kind: 'multi-value', options: ['work', 'free', 'backup', 'test'] },
        { key: 'enabled', labelKey: 'claudeProfiles.enabledProfile', kind: 'boolean' },
      ],
    },
    {
      id: 'advanced',
      titleKey: 'claudeProfiles.advancedModelsTitle',
      layout: 'grid',
      advanced: true,
      fields: [
        { key: 'default_opus_model', labelKey: 'claudeProfiles.defaultOpusModelLabel', kind: 'text' },
        { key: 'default_haiku_model', labelKey: 'claudeProfiles.defaultHaikuModelLabel', kind: 'text' },
        { key: 'default_fable_model', labelKey: 'claudeProfiles.defaultFableModelLabel', kind: 'text' },
        { key: 'subagent_model', labelKey: 'claudeProfiles.subagentModelLabel', kind: 'text' },
        { key: 'api_timeout_ms', labelKey: 'claudeProfiles.apiTimeoutMsLabel', kind: 'number' },
        { key: 'provider', labelKey: 'claudeProfiles.providerLabel', kind: 'text' },
        { key: 'account', labelKey: 'claudeProfiles.accountLabel', kind: 'text' },
      ],
    },
  ],
  validate: (form, ctx) => {
    const issues: ProfileEditorIssue[] = []
    const name = form.name.trim()
    if (!name) {
      issues.push({
        section: 'identity',
        field: 'name',
        message: translate('profileEditor.validation.nameRequired'),
      })
    } else if (ctx.existingNames.includes(name) && name !== ctx.originalName) {
      issues.push({
        section: 'identity',
        field: 'name',
        message: translate('profileEditor.validation.nameDuplicate'),
      })
    }
    if (form.auth_mode === 'api_key' && !form.base_url.trim()) {
      issues.push({
        section: 'connection',
        field: 'base_url',
        message: translate('profileEditor.validation.baseUrlRequired'),
      })
    }
    if (form.auth_mode === 'api_key' && !ctx.isEditing && !form.auth_token.trim()) {
      issues.push({
        section: 'connection',
        field: 'auth_token',
        message: translate('profileEditor.validation.authTokenRequired'),
      })
    }
    return issues
  },
  submit: async (form, ctx) => {
    const payload = omitEmptyAuthToken(buildClaudeProfileRequest(form))
    try {
      if (ctx.isEditing && ctx.originalName) {
        await claudeApi.updateClaudeProfile(ctx.originalName, payload)
      } else {
        await claudeApi.addClaudeProfile(payload)
      }
      return { status: 'ok', appliedName: form.name.trim() }
    } catch (error) {
      return errorOf(error)
    }
  },
}
