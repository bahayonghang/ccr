import { claudePresets } from '@/configs/providerPresets'
import { CHECKIN_CATALOG_PROVIDER_TEMPLATES } from '@/configs/providersCatalog'
import { OPENCODE_PROVIDER_PRESETS } from '@/types/opencode'
import type {
  ClaudeProviderTemplateOverride,
  CodexProviderTemplateOverride,
  OpenCodeProviderTemplateOverride,
  ProviderTemplate,
  ProviderTemplateCategory,
} from '@/types/providerTemplates'

const categoryLabels: Record<ProviderTemplateCategory, string> = {
  official: 'Official',
  cn_official: 'CN official',
  aggregator: 'Aggregator',
  third_party: 'Third party',
  local: 'Local',
}

const hostFromUrl = (url?: string) => {
  if (!url) return ''
  try {
    return new URL(url).host
  } catch {
    return url.replace(/^https?:\/\//, '').replace(/\/.*$/, '')
  }
}

const dedupe = (values: Array<string | undefined>) => [
  ...new Set(values.map((value) => value?.trim()).filter(Boolean) as string[]),
]

const commonAliases: Record<string, string[]> = {
  'claude-official': ['anthropic', 'claude', 'claude code'],
  deepseek: ['deep seek', '深度求索'],
  'zhipu-glm': ['zhipu', 'glm', 'bigmodel', '智谱'],
  'zai-glm': ['z.ai', 'zai', 'glm'],
  bailian: ['dashscope', 'aliyun', '阿里云', '百炼'],
  'bailian-coding': ['dashscope coding', 'aliyun coding', '百炼 coding'],
  kimi: ['moonshot', '月之暗面'],
  'kimi-coding': ['moonshot coding'],
  stepfun: ['step', '阶跃星辰'],
  longcat: ['long cat'],
  minimax: ['mini max'],
  'minimax-en': ['mini max en'],
  'doubao-seed': ['doubao', 'volcengine', 'ark', '火山引擎'],
  bailing: ['ling', '支付宝'],
  'xiaomi-mimo': ['mimo', 'xiaomi', '小米'],
  modelscope: ['model scope', '魔搭'],
  aihubmix: ['ai hub mix'],
  siliconflow: ['silicon flow', '硅基流动'],
  'siliconflow-en': ['silicon flow en'],
  dmxapi: ['dmx'],
  compshare: ['modelverse'],
  openrouter: ['open router'],
  novita: ['novita ai'],
  nvidia: ['nim'],
}

const codexOverrides: Record<string, CodexProviderTemplateOverride> = {
  'claude-official': {
    baseUrl: 'https://api.anthropic.com/v1',
    websiteUrl: 'https://www.anthropic.com',
    apiKeyUrl: 'https://console.anthropic.com/settings/keys',
    modelCatalog: ['claude-sonnet-4-5', 'claude-haiku-4-5'],
  },
  deepseek: {
    baseUrl: 'https://api.deepseek.com/',
    websiteUrl: 'https://platform.deepseek.com',
    apiKeyUrl: 'https://platform.deepseek.com/api_keys',
    modelCatalog: ['deepseek-v4-flash'],
    model: 'deepseek-v4-flash',
  },
  kimi: {
    baseUrl: 'https://api.moonshot.cn/v1',
    websiteUrl: 'https://platform.moonshot.cn/console',
    apiKeyUrl: 'https://platform.moonshot.cn/console/api-keys',
    modelCatalog: ['kimi-k2.5', 'moonshot-v1-auto'],
  },
  openrouter: {
    baseUrl: 'https://openrouter.ai/api/v1',
    websiteUrl: 'https://openrouter.ai',
    apiKeyUrl: 'https://openrouter.ai/keys',
    modelCatalog: ['anthropic/claude-sonnet-4.6', 'openai/gpt-5.1'],
  },
  siliconflow: {
    baseUrl: 'https://api.siliconflow.cn/v1',
    websiteUrl: 'https://siliconflow.cn',
    apiKeyUrl: 'https://cloud.siliconflow.cn/account/ak',
    modelCatalog: ['deepseek-ai/DeepSeek-V3.2', 'Qwen/Qwen3-Coder'],
  },
  'siliconflow-en': {
    baseUrl: 'https://api.siliconflow.com/v1',
    websiteUrl: 'https://siliconflow.com',
    apiKeyUrl: 'https://cloud.siliconflow.com/account/ak',
    modelCatalog: ['deepseek-ai/DeepSeek-V3.2', 'Qwen/Qwen3-Coder'],
  },
  modelscope: {
    baseUrl: 'https://api-inference.modelscope.cn/v1',
    websiteUrl: 'https://modelscope.cn',
    apiKeyUrl: 'https://modelscope.cn/my/myaccesstoken',
    modelCatalog: ['ZhipuAI/GLM-5', 'Qwen/Qwen3-Coder'],
  },
  minimax: {
    baseUrl: 'https://api.minimaxi.com/v1',
    websiteUrl: 'https://platform.minimaxi.com',
    apiKeyUrl: 'https://platform.minimaxi.com/user-center/basic-information/interface-key',
    modelCatalog: ['MiniMax-M2.5'],
  },
  'minimax-en': {
    baseUrl: 'https://api.minimax.io/v1',
    websiteUrl: 'https://platform.minimax.io',
    apiKeyUrl: 'https://platform.minimax.io/user-center/basic-information/interface-key',
    modelCatalog: ['MiniMax-M2.5'],
  },
  zhipu: {
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    websiteUrl: 'https://open.bigmodel.cn',
    apiKeyUrl: 'https://open.bigmodel.cn/usercenter/proj-mgmt/apikeys',
    modelCatalog: ['glm-5', 'glm-4.5'],
  },
  'zhipu-glm': {
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    websiteUrl: 'https://open.bigmodel.cn',
    apiKeyUrl: 'https://open.bigmodel.cn/usercenter/proj-mgmt/apikeys',
    modelCatalog: ['glm-5', 'glm-4.5'],
  },
  doubao: {
    baseUrl: 'https://ark.cn-beijing.volces.com/api/v3',
    websiteUrl: 'https://www.volcengine.com/product/doubao',
    apiKeyUrl: 'https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey',
    modelCatalog: ['doubao-seed-2-0-code-preview-latest'],
  },
  'doubao-seed': {
    baseUrl: 'https://ark.cn-beijing.volces.com/api/v3',
    websiteUrl: 'https://www.volcengine.com/product/doubao',
    apiKeyUrl: 'https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey',
    modelCatalog: ['doubao-seed-2-0-code-preview-latest'],
  },
  aihubmix: {
    baseUrl: 'https://aihubmix.com/v1',
    websiteUrl: 'https://aihubmix.com',
    apiKeyUrl: 'https://aihubmix.com',
  },
  dmxapi: {
    baseUrl: 'https://www.dmxapi.cn/v1',
    websiteUrl: 'https://www.dmxapi.cn',
    apiKeyUrl: 'https://www.dmxapi.cn',
  },
}

const extraCodexTemplates: ProviderTemplate[] = [
  {
    id: 'openai',
    name: 'OpenAI',
    aliases: ['chatgpt', 'gpt'],
    category: 'official',
    websiteUrl: 'https://platform.openai.com',
    apiKeyUrl: 'https://platform.openai.com/api-keys',
    tags: ['codex', 'openai-compatible'],
    baseUrls: ['https://api.openai.com/v1'],
    modelCatalog: ['gpt-5.1', 'gpt-5.1-codex-max', 'o3'],
    isOfficial: true,
    source: 'built_in',
    platforms: {
      codex: {
        baseUrl: 'https://api.openai.com/v1',
        websiteUrl: 'https://platform.openai.com',
        apiKeyUrl: 'https://platform.openai.com/api-keys',
        modelCatalog: ['gpt-5.1', 'gpt-5.1-codex-max', 'o3'],
      },
      opencode: {
        id: 'openai',
        name: 'OpenAI',
      },
    },
  },
  {
    id: 'azure-openai',
    name: 'Azure OpenAI',
    aliases: ['azure', 'microsoft'],
    category: 'official',
    websiteUrl: 'https://azure.microsoft.com/products/ai-services/openai-service',
    apiKeyUrl: 'https://portal.azure.com',
    tags: ['codex', 'openai-compatible'],
    baseUrls: ['https://{resource}.openai.azure.com/openai'],
    modelCatalog: ['gpt-5.1', 'gpt-4.1'],
    isOfficial: true,
    source: 'built_in',
    platforms: {
      codex: {
        baseUrl: 'https://{resource}.openai.azure.com/openai',
        websiteUrl: 'https://azure.microsoft.com/products/ai-services/openai-service',
        apiKeyUrl: 'https://portal.azure.com',
        modelCatalog: ['gpt-5.1', 'gpt-4.1'],
      },
    },
  },
  {
    id: 'local-openai-compatible',
    name: 'Local OpenAI Compatible',
    aliases: ['ollama', 'lm studio', 'local gateway'],
    category: 'local',
    tags: ['codex', 'opencode', 'openai-compatible'],
    baseUrls: ['http://127.0.0.1:11434/v1', 'http://127.0.0.1:1234/v1'],
    modelCatalog: ['qwen3-coder', 'gpt-oss'],
    source: 'built_in',
    platforms: {
      codex: {
        baseUrl: 'http://127.0.0.1:11434/v1',
        modelCatalog: ['qwen3-coder', 'gpt-oss'],
      },
      opencode: {
        id: 'openai',
        name: 'Local OpenAI Compatible',
        npm: '@ai-sdk/openai-compatible',
        baseURL: 'http://127.0.0.1:11434/v1',
      },
    },
  },
]

const claudeTemplateFromPreset = (
  preset: (typeof claudePresets.presets)[number]
): ProviderTemplate => {
  const claudeOverride: ClaudeProviderTemplateOverride = {
    baseUrl: preset.base_url,
    provider: preset.provider || preset.name,
    providerType: preset.provider_type,
    model: preset.model,
    smallFastModel: preset.small_fast_model,
    defaultSonnetModel: preset.model,
    defaultHaikuModel: preset.small_fast_model || preset.model,
    subagentModel: preset.small_fast_model || preset.model,
    claudeCodeAutoCompactWindow: preset.claude_code_auto_compact_window,
    apiTimeoutMs: preset.api_timeout_ms,
    claudeCodeDisableNonessentialTraffic:
      preset.claude_code_disable_nonessential_traffic,
    description: preset.description,
  }
  const codexOverride = codexOverrides[preset.id]
  const baseUrls = dedupe([preset.base_url, codexOverride?.baseUrl])
  const modelCatalog = dedupe([
    preset.model,
    preset.small_fast_model,
    ...(codexOverride?.modelCatalog || []),
  ])

  return {
    id: preset.id,
    name: preset.name,
    aliases: dedupe([
      preset.provider,
      hostFromUrl(preset.websiteUrl),
      hostFromUrl(preset.base_url),
      ...(commonAliases[preset.id] || []),
    ]),
    category: preset.category,
    websiteUrl: preset.websiteUrl,
    apiKeyUrl: preset.apiKeyUrl,
    tags: dedupe([
      categoryLabels[preset.category],
      preset.provider_type,
      codexOverride ? 'codex' : undefined,
      'claude',
    ]),
    baseUrls,
    modelCatalog,
    isOfficial: preset.category === 'official',
    isPartner: preset.isPartner,
    source: 'built_in',
    platforms: {
      claude: claudeOverride,
      ...(codexOverride ? { codex: codexOverride } : {}),
    },
  }
}

const opencodeTemplateFromPreset = (
  preset: (typeof OPENCODE_PROVIDER_PRESETS)[number]
): ProviderTemplate => {
  const override: OpenCodeProviderTemplateOverride = {
    id: preset.providerId || preset.id,
    name: preset.label,
    npm: preset.npm,
  }

  return {
    id: `opencode-${preset.id}`,
    name: preset.label,
    aliases: dedupe([preset.id, preset.providerId, preset.npm]),
    category: preset.id === 'openai-compatible' ? 'third_party' : 'official',
    tags: ['opencode'],
    source: 'built_in',
    platforms: {
      opencode: override,
    },
  }
}

const mergeTemplates = (templates: ProviderTemplate[]) => {
  const merged = new Map<string, ProviderTemplate>()

  for (const template of templates) {
    const existing = merged.get(template.id)
    if (!existing) {
      merged.set(template.id, template)
      continue
    }

    merged.set(template.id, {
      ...existing,
      aliases: dedupe([...(existing.aliases || []), ...(template.aliases || [])]),
      tags: dedupe([...(existing.tags || []), ...(template.tags || [])]),
      baseUrls: dedupe([...(existing.baseUrls || []), ...(template.baseUrls || [])]),
      modelCatalog: dedupe([...(existing.modelCatalog || []), ...(template.modelCatalog || [])]),
      platforms: {
        ...existing.platforms,
        ...template.platforms,
      },
    })
  }

  return [...merged.values()]
}

export const BUILT_IN_PROVIDER_TEMPLATES: ProviderTemplate[] = mergeTemplates([
  ...claudePresets.presets.map(claudeTemplateFromPreset),
  ...extraCodexTemplates,
  ...OPENCODE_PROVIDER_PRESETS.map(opencodeTemplateFromPreset),
  // 共享站点目录（providers-catalog.json）中带 platforms 块的签到公益站
  ...CHECKIN_CATALOG_PROVIDER_TEMPLATES,
])
