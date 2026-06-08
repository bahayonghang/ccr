import type {
  ClaudeLegacyConfigTemplatePatch,
  ClaudeProfileTemplatePatch,
  CodexApiAccountTemplatePatch,
  CodexProviderTemplatePatch,
  CodexProviderTemplateOverride,
  OpenCodeProviderTemplatePatch,
  OpenCodeProviderTemplateOverride,
  ProviderTemplate,
  ProviderTemplateCategory,
  ProviderTemplateDraftContext,
  ProviderTemplateOption,
  ProviderTemplatePlatform,
  ProviderTemplatePlatformOverrides,
} from '@/types/providerTemplates'

export const CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY = 'ccr.providerTemplates.custom.v1'

export const PROVIDER_TEMPLATE_CATEGORY_LABELS: Record<ProviderTemplateCategory, string> = {
  official: 'Official',
  cn_official: 'CN official',
  aggregator: 'Aggregator',
  third_party: 'Third party',
  local: 'Local',
}

export const PROVIDER_TEMPLATE_CATEGORY_ORDER: Record<ProviderTemplateCategory, number> = {
  official: 0,
  cn_official: 1,
  aggregator: 2,
  third_party: 3,
  local: 4,
}

export const PROVIDER_TEMPLATE_PLATFORM_LABELS: Record<ProviderTemplatePlatform, string> = {
  claude: 'Claude Code',
  codex: 'Codex',
  opencode: 'OpenCode',
}

const SECRET_KEY_PATTERN =
  /^(api[_-]?key|x[_-]?api[_-]?key|auth[_-]?token|bearer[_-]?token|authorization|proxy[_-]?authorization|token|secret|password)$/i

export const compactString = (value: unknown) => (
  typeof value === 'string' ? value.trim() : ''
)

export const compactList = (values: Array<string | undefined | null>) => (
  [...new Set(values.map(compactString).filter(Boolean))]
)

export function parseListInput(input: string): string[] {
  return compactList(input.split(/[\n,]/))
}

export function formatListInput(values: string[] | undefined): string {
  return (values || []).join('\n')
}

export function safeJson(value: unknown): string {
  if (!value || (typeof value === 'object' && Object.keys(value).length === 0)) return '{}'
  return JSON.stringify(value, null, 2)
}

export function parseJsonObject(input: string): Record<string, unknown> {
  const text = input.trim()
  if (!text) return {}
  const parsed = JSON.parse(text)
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('JSON must be an object.')
  }
  return parsed as Record<string, unknown>
}

function hostFromUrl(url?: string) {
  const text = compactString(url)
  if (!text) return ''
  try {
    return new URL(text).host
  } catch {
    return text
      .replace(/^https?:\/\//, '')
      .replace(/\/.*$/, '')
  }
}

function stripSecretKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(stripSecretKeys)
  if (!value || typeof value !== 'object') return value

  return Object.fromEntries(
    Object
      .entries(value as Record<string, unknown>)
      .filter(([key]) => !SECRET_KEY_PATTERN.test(key))
      .map(([key, entry]) => [key, stripSecretKeys(entry)]),
  )
}

export function sanitizeProviderTemplate(template: ProviderTemplate): ProviderTemplate {
  const platforms: ProviderTemplatePlatformOverrides = {
    claude: template.platforms.claude
      ? stripSecretKeys(template.platforms.claude) as ProviderTemplatePlatformOverrides['claude']
      : undefined,
    codex: template.platforms.codex
      ? stripSecretKeys(template.platforms.codex) as ProviderTemplatePlatformOverrides['codex']
      : undefined,
    opencode: template.platforms.opencode
      ? stripSecretKeys(template.platforms.opencode) as ProviderTemplatePlatformOverrides['opencode']
      : undefined,
  }

  return {
    ...template,
    id: compactString(template.id),
    name: compactString(template.name),
    aliases: compactList(template.aliases || []),
    tags: compactList(template.tags || []),
    baseUrls: compactList(template.baseUrls || []),
    modelCatalog: compactList(template.modelCatalog || []),
    websiteUrl: compactString(template.websiteUrl) || undefined,
    apiKeyUrl: compactString(template.apiKeyUrl) || undefined,
    source: template.source || 'custom',
    platforms,
  }
}

export function mergeProviderTemplates(
  builtInTemplates: ProviderTemplate[],
  customTemplates: ProviderTemplate[],
): ProviderTemplate[] {
  const customById = new Map(customTemplates.map(template => [template.id, template]))
  const mergedBuiltIns = builtInTemplates.map(template => (
    customById.has(template.id)
      ? { ...template, id: `${template.id}-built-in` }
      : template
  ))

  return [...mergedBuiltIns, ...customTemplates]
}

export function readCustomProviderTemplates(storage: Storage | undefined = globalThis.localStorage): ProviderTemplate[] {
  if (!storage) return []

  try {
    const raw = storage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []

    return parsed
      .map(item => sanitizeProviderTemplate(item as ProviderTemplate))
      .filter(template => template.id && template.name && Object.keys(template.platforms).length > 0)
  } catch {
    return []
  }
}

export function writeCustomProviderTemplates(
  templates: ProviderTemplate[],
  storage: Storage | undefined = globalThis.localStorage,
) {
  if (!storage) return
  storage.setItem(
    CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY,
    JSON.stringify(templates.map(sanitizeProviderTemplate)),
  )
}

export function deleteCustomProviderTemplate(
  templates: ProviderTemplate[],
  id: string,
): ProviderTemplate[] {
  return templates.filter(template => template.id !== id)
}

export function upsertCustomProviderTemplate(
  templates: ProviderTemplate[],
  template: ProviderTemplate,
): ProviderTemplate[] {
  const sanitized = sanitizeProviderTemplate({
    ...template,
    source: 'custom',
    updatedAt: new Date().toISOString(),
    createdAt: template.createdAt || new Date().toISOString(),
  })
  const next = templates.filter(item => item.id !== sanitized.id)

  return [...next, sanitized].sort((a, b) => a.name.localeCompare(b.name))
}

export function providerTemplateSearchText(template: ProviderTemplate, platform?: ProviderTemplatePlatform) {
  const platformOverride = platform ? template.platforms[platform] : null

  return [
    template.id,
    template.name,
    ...(template.aliases || []),
    template.category,
    PROVIDER_TEMPLATE_CATEGORY_LABELS[template.category],
    ...(template.tags || []),
    template.websiteUrl,
    template.apiKeyUrl,
    hostFromUrl(template.websiteUrl),
    ...(template.baseUrls || []),
    ...(template.baseUrls || []).map(hostFromUrl),
    ...(template.modelCatalog || []),
    ...(platformOverride && 'baseUrl' in platformOverride ? [platformOverride.baseUrl] : []),
    ...(platformOverride && 'baseURL' in platformOverride ? [platformOverride.baseURL] : []),
    ...(platformOverride && 'models' in platformOverride
      ? Object.keys(platformOverride.models || {})
      : []),
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()
}

export function getTemplatesForPlatform(
  templates: ProviderTemplate[],
  platform: ProviderTemplatePlatform,
): ProviderTemplate[] {
  return templates
    .filter(template => Boolean(template.platforms[platform]))
    .sort((a, b) => {
      const categoryDelta =
        PROVIDER_TEMPLATE_CATEGORY_ORDER[a.category] - PROVIDER_TEMPLATE_CATEGORY_ORDER[b.category]
      if (categoryDelta !== 0) return categoryDelta
      if ((a.source || 'built_in') !== (b.source || 'built_in')) {
        return (a.source || 'built_in') === 'custom' ? -1 : 1
      }
      return a.name.localeCompare(b.name)
    })
}

export function resolveTemplateBaseUrls(
  template: ProviderTemplate,
  platform: ProviderTemplatePlatform,
): string[] {
  const override = template.platforms[platform]
  const platformBase = override && 'baseUrl' in override
    ? override.baseUrl
    : override && 'baseURL' in override
      ? override.baseURL
      : undefined

  return compactList([platformBase, ...(template.baseUrls || [])])
}

export function resolveTemplateEndpoint(
  template: ProviderTemplate,
  platform: ProviderTemplatePlatform,
  endpoint?: string,
): string | undefined {
  const endpoints = resolveTemplateBaseUrls(template, platform)
  const selected = compactString(endpoint)

  if (selected && endpoints.includes(selected)) return selected
  return endpoints[0]
}

export function buildProviderTemplateOptions(
  templates: ProviderTemplate[],
  platform: ProviderTemplatePlatform,
): ProviderTemplateOption[] {
  return getTemplatesForPlatform(templates, platform).flatMap((template) => {
    const endpoints = resolveTemplateBaseUrls(template, platform)
    const sourceLabel = (template.source || 'built_in') === 'custom' ? 'Custom' : 'Built-in'
    const categoryLabel = PROVIDER_TEMPLATE_CATEGORY_LABELS[template.category]
    const endpointOptions = endpoints.length > 0 ? endpoints : [undefined]

    return endpointOptions.map((endpoint, endpointIndex) => {
      const subtitleParts = compactList([
        endpoint,
        template.websiteUrl ? hostFromUrl(template.websiteUrl) : undefined,
        (template.modelCatalog || []).slice(0, 2).join(', '),
      ])

      return {
        id: `${template.id}:${endpoint || 'default'}:${endpointIndex}`,
        template,
        platform,
        endpoint,
        label: template.name,
        subtitle: subtitleParts.join(' · ') || categoryLabel,
        sourceLabel,
        categoryLabel,
        searchText: providerTemplateSearchText(template, platform),
      }
    })
  })
}

function fallbackPlatformOverride(
  template: Pick<ProviderTemplate, 'name' | 'baseUrls' | 'websiteUrl' | 'apiKeyUrl' | 'modelCatalog'>,
  platform: ProviderTemplatePlatform,
): NonNullable<ProviderTemplatePlatformOverrides[ProviderTemplatePlatform]> {
  const baseUrl = template.baseUrls?.[0]

  if (platform === 'claude') {
    return {
      baseUrl,
      provider: template.name,
      providerType: 'third_party_model',
      defaultSonnetModel: template.modelCatalog?.[0],
      defaultHaikuModel: template.modelCatalog?.[1] || template.modelCatalog?.[0],
      subagentModel: template.modelCatalog?.[1] || template.modelCatalog?.[0],
    }
  }

  if (platform === 'codex') {
    return {
      baseUrl,
      websiteUrl: template.websiteUrl,
      apiKeyUrl: template.apiKeyUrl,
      modelCatalog: template.modelCatalog,
    } satisfies CodexProviderTemplateOverride
  }

  return {
    id: slugifyTemplateId(template.name),
    name: template.name,
    npm: baseUrl ? '@ai-sdk/openai-compatible' : undefined,
    baseURL: baseUrl,
  } satisfies OpenCodeProviderTemplateOverride
}

export function createCustomProviderTemplateFromDraft(
  draft: ProviderTemplateDraftContext,
  selectedPlatforms: ProviderTemplatePlatform[],
  values: {
    id?: string
    name: string
    aliases?: string[]
    tags?: string[]
    category: ProviderTemplateCategory
    websiteUrl?: string
    apiKeyUrl?: string
    baseUrls?: string[]
    modelCatalog?: string[]
    existing?: ProviderTemplate
    platformOverrides?: ProviderTemplatePlatformOverrides
  },
): ProviderTemplate {
  const template: ProviderTemplate = {
    id: slugifyTemplateId(values.id || values.name || draft.defaultName || 'custom-provider'),
    name: compactString(values.name) || draft.defaultName || 'Custom provider',
    aliases: compactList(values.aliases || draft.aliases || []),
    tags: compactList(values.tags || draft.tags || []),
    category: values.category,
    websiteUrl: compactString(values.websiteUrl) || compactString(draft.websiteUrl) || undefined,
    apiKeyUrl: compactString(values.apiKeyUrl) || compactString(draft.apiKeyUrl) || undefined,
    baseUrls: compactList(values.baseUrls || draft.baseUrls || []),
    modelCatalog: compactList(values.modelCatalog || draft.modelCatalog || []),
    source: 'custom',
    platforms: {},
    createdAt: values.existing?.createdAt,
  }

  for (const platform of selectedPlatforms) {
    const suppliedOverride = values.platformOverrides?.[platform]
    if (suppliedOverride && Object.keys(suppliedOverride).length > 0) {
      template.platforms[platform] = suppliedOverride as never
      continue
    }

    if (platform === draft.platform) {
      template.platforms[platform] = draft.platformOverride as never
      continue
    }

    template.platforms[platform] =
      values.existing?.platforms[platform] ||
      fallbackPlatformOverride(template, platform) as never
  }

  return sanitizeProviderTemplate(template)
}

export function mapTemplateToClaudeProfilePatch(
  template: ProviderTemplate,
  endpoint?: string,
): ClaudeProfileTemplatePatch {
  const override = template.platforms.claude
  if (!override) return {}

  const baseUrl = endpoint || override.baseUrl || resolveTemplateEndpoint(template, 'claude')
  const model = override.model || template.modelCatalog?.[0]
  const smallFastModel = override.smallFastModel || template.modelCatalog?.[1] || model

  return {
    base_url: baseUrl,
    provider: override.provider || template.name,
    provider_type: override.providerType,
    default_opus_model: override.defaultOpusModel,
    default_sonnet_model: override.defaultSonnetModel || model,
    default_haiku_model: override.defaultHaikuModel || smallFastModel,
    subagent_model: override.subagentModel || smallFastModel,
    description: override.description,
    suggestedName: template.id,
  }
}

export function mapTemplateToClaudeLegacyConfigPatch(
  template: ProviderTemplate,
  endpoint?: string,
): ClaudeLegacyConfigTemplatePatch {
  const override = template.platforms.claude
  if (!override) return {}

  const baseUrl = endpoint || override.baseUrl || resolveTemplateEndpoint(template, 'claude')
  const model = override.model || override.defaultSonnetModel || template.modelCatalog?.[0]
  const smallFastModel =
    override.smallFastModel ||
    override.defaultHaikuModel ||
    override.subagentModel ||
    template.modelCatalog?.[1] ||
    model

  return {
    base_url: baseUrl,
    model,
    small_fast_model: smallFastModel,
    provider: override.provider || template.name,
    provider_type: override.providerType,
    description: override.description,
    suggestedName: template.id,
  }
}

export function mapTemplateToCodexProviderPatch(
  template: ProviderTemplate,
  endpoint?: string,
): CodexProviderTemplatePatch {
  const override = template.platforms.codex
  if (!override) return {}

  return {
    name: template.name,
    baseUrl: endpoint || override.baseUrl || resolveTemplateEndpoint(template, 'codex'),
    websiteUrl: override.websiteUrl || template.websiteUrl,
    apiKeyUrl: override.apiKeyUrl || template.apiKeyUrl,
  }
}

export function mapTemplateToCodexApiAccountPatch(
  template: ProviderTemplate,
  endpoint?: string,
): CodexApiAccountTemplatePatch {
  const providerPatch = mapTemplateToCodexProviderPatch(template, endpoint)

  return {
    providerName: providerPatch.name,
    apiBaseUrl: providerPatch.baseUrl,
  }
}

export function mapTemplateToOpenCodeProviderPatch(
  template: ProviderTemplate,
  endpoint?: string,
): OpenCodeProviderTemplatePatch {
  const override = template.platforms.opencode
  if (!override) return {}

  return {
    id: override.id || slugifyTemplateId(template.name),
    name: override.name || template.name,
    npm: override.npm,
    baseURL: endpoint || override.baseURL || resolveTemplateEndpoint(template, 'opencode'),
    modelsJson: safeJson(override.models || {}),
    extraOptionsJson: safeJson(stripSecretKeys(override.extraOptions || {})),
    rootExtraJson: safeJson(stripSecretKeys(override.rootExtra || {})),
  }
}

export function slugifyTemplateId(value: string): string {
  const slug = compactString(value)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

  return slug || 'custom-provider'
}
