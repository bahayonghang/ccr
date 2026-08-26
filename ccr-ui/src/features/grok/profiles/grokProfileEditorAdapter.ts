import { grokApi } from '@/api'
import type {
  ProfileEditorAdapter,
  ProfileEditorIssue,
  ProfileWriteOutcome,
} from '@/configs/profileEditorAdapter'
import { translate } from '@/i18n'
import type { GrokProfileDto } from '@/types'
import {
  GROK_API_BACKEND_OPTIONS,
  GROK_REASONING_EFFORT_OPTIONS,
  buildGrokCreateRequest,
  buildGrokPatch,
  createEmptyGrokForm,
  fillGrokForm,
  type GrokProfileDirtyField,
  type GrokProfileEditorForm,
} from '@/utils/grokProfileEditor'
import { validateGrokEditor } from './grokEditorValidation'
import { mapGrokProfileWriteOutcome } from './grokProfileWriteOutcome'

const asForm = (form: unknown): GrokProfileEditorForm => form as GrokProfileEditorForm

const isOfficial = (form: unknown) => asForm(form).profileKind === 'official'

const isThirdParty = (form: unknown) => !isOfficial(form)

const dirtyFieldsOf = (dirty: ReadonlySet<string>): Set<GrokProfileDirtyField> => {
  const allowed = new Set(Object.keys(createEmptyGrokForm()))
  const next = new Set<GrokProfileDirtyField>()
  for (const key of dirty) {
    if (allowed.has(key)) next.add(key as GrokProfileDirtyField)
  }
  return next
}

const errorOf = (error: unknown): ProfileWriteOutcome => ({
  status: 'error',
  message: error instanceof Error ? error.message : String(error),
})

export const grokProfileEditorAdapter: ProfileEditorAdapter<
  GrokProfileEditorForm,
  GrokProfileDto
> = {
  createEmpty: createEmptyGrokForm,
  fromRecord: fillGrokForm,
  sections: [
    {
      id: 'identity',
      titleKey: 'grok.profiles.editor.identity',
      layout: 'grid',
      fields: [
        { key: 'name', labelKey: 'grok.profiles.fields.name', kind: 'mono-text', required: () => true },
        { key: 'description', labelKey: 'grok.profiles.fields.description', kind: 'text' },
        {
          key: 'profileKind',
          labelKey: 'grok.profiles.fields.profileKind',
          kind: 'text',
          readOnly: true,
        },
        {
          key: 'provider',
          labelKey: 'grok.profiles.fields.provider',
          kind: 'text',
          visible: isThirdParty,
        },
      ],
    },
    {
      id: 'connection',
      titleKey: 'grok.profiles.editor.connection',
      layout: 'group',
      fields: [
        {
          key: 'baseUrl',
          labelKey: 'grok.profiles.fields.baseUrl',
          kind: 'mono-text',
          visible: isThirdParty,
          required: isThirdParty,
        },
        {
          key: 'credentialAction',
          labelKey: 'grok.profiles.fields.credentialAction',
          kind: 'choice',
          options: ['preserve', 'replace_api_key', 'replace_env_key', 'clear'],
          visible: isThirdParty,
        },
        {
          key: 'apiKey',
          labelKey: 'grok.profiles.fields.apiKey',
          kind: 'secret',
          visible: (form) => isThirdParty(form) && asForm(form).credentialAction === 'replace_api_key',
          required: (form) => asForm(form).credentialAction === 'replace_api_key',
        },
        {
          key: 'envKey',
          labelKey: 'grok.profiles.fields.envKey',
          kind: 'mono-text',
          visible: (form) => isThirdParty(form) && asForm(form).credentialAction === 'replace_env_key',
          required: (form) => asForm(form).credentialAction === 'replace_env_key',
        },
      ],
    },
    {
      id: 'runtime',
      titleKey: 'grok.profiles.editor.runtime',
      layout: 'grid',
      fields: [
        {
          key: 'model',
          labelKey: 'grok.profiles.fields.model',
          kind: 'choice',
          options: ['grok-4.6', 'grok-4', 'grok-3'],
          required: isThirdParty,
        },
        {
          key: 'reasoningEffort',
          labelKey: 'grok.profiles.fields.reasoningEffort',
          kind: 'choice',
          options: GROK_REASONING_EFFORT_OPTIONS,
        },
        {
          key: 'apiBackend',
          labelKey: 'grok.profiles.fields.apiBackend',
          kind: 'choice',
          options: GROK_API_BACKEND_OPTIONS,
          visible: isThirdParty,
        },
        {
          key: 'contextWindow',
          labelKey: 'grok.profiles.fields.contextWindow',
          kind: 'number',
          visible: isThirdParty,
        },
        {
          key: 'supportsBackendSearch',
          labelKey: 'grok.profiles.fields.supportsBackendSearch',
          kind: 'boolean',
          visible: isThirdParty,
        },
      ],
    },
    {
      id: 'status',
      titleKey: 'grok.profiles.editor.status',
      layout: 'grid',
      fields: [
        { key: 'tagsInput', labelKey: 'grok.profiles.fields.tags', kind: 'multi-value', options: ['work', 'free', 'backup', 'test'] },
        { key: 'enabled', labelKey: 'grok.profiles.fields.enabled', kind: 'boolean' },
      ],
    },
  ],
  validate: (form, ctx) => {
    const issues: ProfileEditorIssue[] = validateGrokEditor({
      form,
      editingName: ctx.originalName,
      hasExistingBaseUrl: ctx.hasExistingBaseUrl,
      t: translate,
    }).map((issue) => ({ section: issue.section, message: issue.message }))
    return issues
  },
  submit: async (form, ctx) => {
    try {
      if (ctx.isEditing && ctx.originalName) {
        const response = await grokApi.updateGrokProfile(
          ctx.originalName,
          buildGrokPatch(form, dirtyFieldsOf(ctx.dirtyFields)),
        )
        return mapGrokProfileWriteOutcome(response)
      }
      const response = await grokApi.addGrokProfile(buildGrokCreateRequest(form))
      return mapGrokProfileWriteOutcome(response)
    } catch (error) {
      return errorOf(error)
    }
  },
}
