import { z } from 'zod'
import type { ConfigItem, UpdateConfigRequest } from '@/types'
import type { ProviderTemplateDraftContext } from '@/types/providerTemplates'

export const configFormSchema = z.object({
  name: z.string(),
  description: z.string(),
  base_url: z.string(),
  auth_token: z.string(),
  model: z.string(),
  small_fast_model: z.string(),
  provider: z.string(),
  provider_type: z.string(),
  account: z.string(),
  tagsInput: z.string(),
})

export const addConfigFormSchema = configFormSchema.extend({
  name: z.string().trim().min(1),
  base_url: z.string().trim().min(1),
  auth_token: z.string().trim().min(1),
})

export type ConfigFormValues = z.infer<typeof configFormSchema>

export const emptyConfigForm = (): ConfigFormValues => ({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  model: '',
  small_fast_model: '',
  provider: '',
  provider_type: '',
  account: '',
  tagsInput: '',
})

export function valuesFromConfig(config: Partial<ConfigItem>): ConfigFormValues {
  return {
    name: config.name ?? '',
    description: config.description ?? '',
    base_url: config.base_url ?? '',
    auth_token: config.auth_token ?? '',
    model: config.model ?? '',
    small_fast_model: config.small_fast_model ?? '',
    provider: config.provider ?? '',
    provider_type: config.provider_type ?? '',
    account: config.account ?? '',
    tagsInput: Array.isArray(config.tags) ? config.tags.join(', ') : '',
  }
}

export function parseTagsInput(tagsInput: string): string[] {
  return tagsInput
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean)
}

export function toUpdateRequest(values: ConfigFormValues, name: string): UpdateConfigRequest {
  const tags = parseTagsInput(values.tagsInput)
  return {
    name,
    description: values.description,
    base_url: values.base_url,
    auth_token: values.auth_token,
    model: values.model || undefined,
    small_fast_model: values.small_fast_model || undefined,
    provider: values.provider || undefined,
    provider_type: values.provider_type || undefined,
    account: values.account || undefined,
    tags: tags.length ? tags : undefined,
  }
}

export function draftContextFromValues(values: ConfigFormValues): ProviderTemplateDraftContext {
  return {
    platform: 'claude',
    defaultName: values.provider || values.name || 'Claude provider',
    name: values.provider || values.name,
    category: 'third_party',
    baseUrls: values.base_url.trim() ? [values.base_url.trim()] : [],
    modelCatalog: [values.model, values.small_fast_model].filter(Boolean),
    platformOverride: {
      baseUrl: values.base_url,
      provider: values.provider,
      providerType: values.provider_type,
      model: values.model,
      smallFastModel: values.small_fast_model,
      description: values.description,
    },
  }
}

export function isConfigFormDraft(value: unknown): value is ConfigFormValues {
  if (!value || typeof value !== 'object') return false
  const record = value as Record<string, unknown>
  return typeof record.name === 'string' && typeof record.auth_token === 'string'
}
