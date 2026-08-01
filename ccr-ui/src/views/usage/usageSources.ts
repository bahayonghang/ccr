import { USAGE_PLATFORM_IDS, type UsagePlatform } from '@/types/usage'

export interface UsageSourceDefinition {
  id: UsagePlatform
  fallbackLabel: string
}

const fallbackLabels = {
  claude: 'Claude',
  codex: 'Codex',
  opencode: 'OpenCode',
  antigravity: 'Antigravity CLI',
  kimi_code: 'Kimi Code',
  pi: 'Pi / Oh My Pi',
  grok: 'Grok Build',
} satisfies Record<UsagePlatform, string>

export const USAGE_SOURCE_DEFINITIONS: readonly UsageSourceDefinition[] =
  USAGE_PLATFORM_IDS.map((id) => ({ id, fallbackLabel: fallbackLabels[id] }))

const sourceIds = new Set<string>(USAGE_PLATFORM_IDS)

export const isUsagePlatform = (value: string): value is UsagePlatform => sourceIds.has(value)

export const usageSourceFallbackLabel = (source: string) =>
  isUsagePlatform(source) ? fallbackLabels[source] : source
