import type { GrokProfileEditorForm } from '@/utils/grokProfileEditor'
import type { TranslateFunction } from '@/utils/tf'

export type GrokEditorSectionId = 'identity' | 'connection' | 'runtime' | 'status'

export interface GrokEditorIssue {
  section: GrokEditorSectionId
  message: string
}

export const validateGrokEditor = (input: {
  form: GrokProfileEditorForm
  editingName: string | null
  hasExistingBaseUrl: boolean
  t: TranslateFunction
}): GrokEditorIssue[] => {
  const { form, editingName, hasExistingBaseUrl, t } = input
  const errors: GrokEditorIssue[] = []
  if (!form.name.trim()) {
    errors.push({ section: 'identity', message: t('grok.profiles.validation.nameRequired') })
  }
  if (form.profileKind !== 'third_party') return errors
  if (!form.model.trim()) {
    errors.push({ section: 'runtime', message: t('grok.profiles.validation.modelRequired') })
  }
  if (!form.baseUrl.trim() && !(editingName && hasExistingBaseUrl)) {
    errors.push({ section: 'connection', message: t('grok.profiles.validation.baseUrlRequired') })
  }
  if (!editingName && form.credentialAction === 'preserve') {
    errors.push({ section: 'connection', message: t('grok.profiles.validation.credentialRequired') })
  }
  if (form.credentialAction === 'replace_api_key' && !form.apiKey.trim()) {
    errors.push({ section: 'connection', message: t('grok.profiles.validation.apiKeyRequired') })
  }
  if (form.credentialAction === 'replace_env_key' && !form.envKey.trim()) {
    errors.push({ section: 'connection', message: t('grok.profiles.validation.envKeyRequired') })
  }
  if (!form.contextWindow.trim()) return errors
  const contextWindow = Number(form.contextWindow)
  if (!Number.isInteger(contextWindow) || contextWindow <= 0) {
    errors.push({ section: 'runtime', message: t('grok.profiles.validation.contextWindow') })
  }
  return errors
}
