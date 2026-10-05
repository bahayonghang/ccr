import { z } from 'zod'
import type {
  ProviderTemplate,
  ProviderTemplateDraftContext,
  ProviderTemplatePlatform,
  ProviderTemplatePlatformOverrides,
} from '@/types/providerTemplates'
import {
  createCustomProviderTemplateFromDraft,
  formatListInput,
  parseJsonObject,
  parseListInput,
  PROVIDER_TEMPLATE_PLATFORM_LABELS,
  safeJson,
  slugifyTemplateId,
} from '@/utils/providerTemplates'
import { tt } from '../locale'

export const PLATFORM_ITEMS: Array<{ id: ProviderTemplatePlatform; label: string }> = [
  { id: 'claude', label: 'Claude Code' },
  { id: 'codex', label: 'Codex' },
  { id: 'opencode', label: 'OpenCode' },
]

export const customTemplateSchema = z.object({
  id: z.string(),
  name: z.string().trim().min(1),
  category: z.enum(['official', 'cn_official', 'aggregator', 'third_party', 'local']),
  websiteUrl: z.string(),
  apiKeyUrl: z.string(),
  aliasesInput: z.string(),
  tagsInput: z.string(),
  baseUrlsInput: z.string(),
  modelCatalogInput: z.string(),
  platformClaude: z.boolean(),
  platformCodex: z.boolean(),
  platformOpencode: z.boolean(),
  overrideClaude: z.string(),
  overrideCodex: z.string(),
  overrideOpencode: z.string(),
})

export type CustomTemplateForm = z.infer<typeof customTemplateSchema>

export const emptyCustomTemplateForm = (): CustomTemplateForm => ({
  id: '',
  name: '',
  category: 'third_party',
  websiteUrl: '',
  apiKeyUrl: '',
  aliasesInput: '',
  tagsInput: '',
  baseUrlsInput: '',
  modelCatalogInput: '',
  platformClaude: false,
  platformCodex: false,
  platformOpencode: false,
  overrideClaude: '{}',
  overrideCodex: '{}',
  overrideOpencode: '{}',
})

const platformFlag = (
  values: CustomTemplateForm,
  platform: ProviderTemplatePlatform,
): boolean => {
  if (platform === 'claude') return values.platformClaude
  if (platform === 'codex') return values.platformCodex
  return values.platformOpencode
}

const overrideInput = (
  values: CustomTemplateForm,
  platform: ProviderTemplatePlatform,
): string => {
  if (platform === 'claude') return values.overrideClaude
  if (platform === 'codex') return values.overrideCodex
  return values.overrideOpencode
}

function formatPlatformOverrideInput(input: {
  template: ProviderTemplate | undefined
  platform: ProviderTemplatePlatform
  draft: ProviderTemplateDraftContext | null
  fromCurrent: boolean
}): string {
  const templateOverride = input.template?.platforms[input.platform]
  if (templateOverride) return safeJson(templateOverride)
  if (input.fromCurrent && input.draft?.platform === input.platform) return safeJson(input.draft.platformOverride)
  return '{}'
}

function selectedPlatformsOf(
  template: ProviderTemplate | undefined,
  currentPlatform: ProviderTemplatePlatform,
): Set<ProviderTemplatePlatform> {
  if (!template) return new Set([currentPlatform])
  return new Set(PLATFORM_ITEMS.filter((item) => Boolean(template.platforms[item.id])).map((item) => item.id))
}

function customNameOf(
  template: ProviderTemplate | undefined,
  draft: ProviderTemplateDraftContext | null,
  fromCurrent: boolean,
): string {
  if (template?.name) return template.name
  if (!fromCurrent) return ''
  return draft?.name ?? draft?.defaultName ?? ''
}

function firstDefined<T>(left: T | undefined, right: T | undefined, fallback: T): T {
  if (left !== undefined && left !== ('' as T)) return left
  if (right !== undefined && right !== ('' as T)) return right
  return fallback
}

function listField(left?: string[], right?: string[]): string {
  if (left && left.length > 0) return formatListInput(left)
  return formatListInput(right ?? [])
}

function listsOf(template: ProviderTemplate | undefined, draft: ProviderTemplateDraftContext | null) {
  return {
    aliasesInput: listField(template?.aliases, draft?.aliases),
    tagsInput: listField(template?.tags, draft?.tags),
    baseUrlsInput: listField(template?.baseUrls, draft?.baseUrls),
    modelCatalogInput: listField(template?.modelCatalog, draft?.modelCatalog),
  }
}

export function fillCustomForm(input: {
  currentPlatform: ProviderTemplatePlatform
  draft: ProviderTemplateDraftContext | null
  template?: ProviderTemplate
  fromCurrent?: boolean
}): CustomTemplateForm {
  const fromCurrent = input.fromCurrent === true
  const template = input.template
  const draft = input.draft
  const name = customNameOf(template, draft, fromCurrent)
  const selected = selectedPlatformsOf(template, input.currentPlatform)
  const lists = listsOf(template, draft)
  return {
    name,
    id: firstDefined(template?.id, undefined, slugifyTemplateId(name)),
    category: firstDefined(template?.category, draft?.category, 'third_party'),
    websiteUrl: firstDefined(template?.websiteUrl, draft?.websiteUrl, ''),
    apiKeyUrl: firstDefined(template?.apiKeyUrl, draft?.apiKeyUrl, ''),
    ...lists,
    platformClaude: selected.has('claude'),
    platformCodex: selected.has('codex'),
    platformOpencode: selected.has('opencode'),
    overrideClaude: formatPlatformOverrideInput({ template, platform: 'claude', draft, fromCurrent }),
    overrideCodex: formatPlatformOverrideInput({ template, platform: 'codex', draft, fromCurrent }),
    overrideOpencode: formatPlatformOverrideInput({ template, platform: 'opencode', draft, fromCurrent }),
  }
}

export function draftForCustomSave(
  platform: ProviderTemplatePlatform,
  existing: ProviderTemplate | undefined,
  draftContext: ProviderTemplateDraftContext | null,
): ProviderTemplateDraftContext | null {
  if (existing?.platforms[platform]) {
    return {
      platform,
      defaultName: existing.name,
      name: existing.name,
      category: existing.category,
      websiteUrl: existing.websiteUrl,
      apiKeyUrl: existing.apiKeyUrl,
      aliases: existing.aliases,
      tags: existing.tags,
      baseUrls: existing.baseUrls,
      modelCatalog: existing.modelCatalog,
      platformOverride: existing.platforms[platform] as never,
    }
  }
  return draftContext
}

function parsePlatformOverrides(
  values: CustomTemplateForm,
  selectedPlatforms: ProviderTemplatePlatform[],
): { platformOverrides?: ProviderTemplatePlatformOverrides; error?: string } {
  const platformOverrides: ProviderTemplatePlatformOverrides = {}
  for (const platform of selectedPlatforms) {
    const parsed = parseOneOverride(values, platform)
    if (parsed.error) return parsed
    if (parsed.override) platformOverrides[platform] = parsed.override
  }
  return { platformOverrides }
}

function parseOneOverride(
  values: CustomTemplateForm,
  platform: ProviderTemplatePlatform,
): { override?: ProviderTemplatePlatformOverrides[ProviderTemplatePlatform]; error?: string } {
  try {
    const override = parseJsonObject(overrideInput(values, platform))
    if (Object.keys(override).length === 0) return {}
    return { override: override as ProviderTemplatePlatformOverrides[ProviderTemplatePlatform] }
  } catch (error) {
    const message = error instanceof Error ? error.message : tt('JSON 无效。', 'Invalid JSON.')
    const label = PROVIDER_TEMPLATE_PLATFORM_LABELS[platform]
    return {
      error: tt(`${label} override JSON 无效。${message}`, `${label} override JSON is invalid. ${message}`),
    }
  }
}

export function buildCustomTemplate(input: {
  values: CustomTemplateForm
  draft: ProviderTemplateDraftContext
  existing?: ProviderTemplate
}): { template?: ProviderTemplate; error?: string } {
  const selectedPlatforms = PLATFORM_ITEMS.filter((item) => platformFlag(input.values, item.id)).map(
    (item) => item.id,
  )
  if (selectedPlatforms.length === 0) {
    return { error: tt('至少选择一个平台。', 'Select at least one platform.') }
  }

  const parsed = parsePlatformOverrides(input.values, selectedPlatforms)
  if (parsed.error) return parsed
  const platformOverrides = parsed.platformOverrides ?? {}

  return {
    template: createCustomProviderTemplateFromDraft(input.draft, selectedPlatforms, {
      id: input.values.id,
      name: input.values.name.trim(),
      aliases: parseListInput(input.values.aliasesInput),
      tags: parseListInput(input.values.tagsInput),
      category: input.values.category,
      websiteUrl: input.values.websiteUrl,
      apiKeyUrl: input.values.apiKeyUrl,
      baseUrls: parseListInput(input.values.baseUrlsInput),
      modelCatalog: parseListInput(input.values.modelCatalogInput),
      existing: input.existing,
      platformOverrides,
    }),
  }
}
