import { USAGE_PLATFORM_IDS, type UsagePlatform } from '@/types/usage'
import { isUsagePlatform, usageSourceFallbackLabel } from '@/views/usage/usageSources'

// 来源键 → 颜色 token 名（token 名用 kebab case，见 DESIGN.md Platform Identity）
const SOURCE_TOKEN_NAMES = {
  claude: 'claude',
  codex: 'codex',
  opencode: 'opencode',
  antigravity: 'antigravity',
  kimi_code: 'kimi-code',
  pi: 'pi',
  grok: 'grok',
  zcode: 'zcode',
  deepseek_harness: 'deepseek-harness',
} satisfies Record<UsagePlatform, string>

/** 周趋势堆叠顺序（自下而上），图例同序；"其他" 固定在最上层。 */
export const INSIGHTS_STACK_ORDER: readonly UsagePlatform[] = [
  'claude',
  'zcode',
  'grok',
  'deepseek_harness',
  'antigravity',
  'kimi_code',
  'opencode',
  'codex',
  'pi',
]

/** "其他" 合并系列的键；不与任何来源键冲突。 */
export const INSIGHTS_OTHER_KEY = '__other__'

export const OTHER_COLOR = 'var(--color-chart-other)'

export const isMappedSource = (source: string): source is UsagePlatform => isUsagePlatform(source)

/** 同量平局时的来源顺序（与 SourceKind::ALL 一致）；未登记来源排在最后。 */
export const sourceRank = (source: string) => {
  const index = (USAGE_PLATFORM_IDS as readonly string[]).indexOf(source)
  return index === -1 ? USAGE_PLATFORM_IDS.length : index
}

export const sourceColor = (source: string) =>
  isMappedSource(source) ? `var(--color-platform-${SOURCE_TOKEN_NAMES[source]})` : OTHER_COLOR

/** 来源显示名：i18n `usage.platforms.<key>`，缺词条时回退固定名；未登记来源显示原始键。 */
export const sourceLabel = (source: string, t: (key: string) => string) => {
  if (!isMappedSource(source)) return source
  const key = `usage.platforms.${source}`
  const label = t(key)
  return label && label !== key ? label : usageSourceFallbackLabel(source)
}
