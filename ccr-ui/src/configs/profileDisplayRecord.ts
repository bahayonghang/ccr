export interface ProfileDisplayRecord {
  name: string
  description: string
  enabled: boolean
  /** 是否为当前应用的 profile */
  current: boolean
  tags: readonly string[]
  /** 与 presentation.fieldSlots 一一对应的四个展示值；空串由渲染层显示占位符 */
  slots: readonly [string, string, string, string]
  /** 搜索匹配用的合并小写文本，由 project 生成，不含凭据 */
  searchText: string
  /** 供应商去重 key；null 表示不计入供应商统计 */
  vendorKey: string | null
  /** 认证方式统计的分组 key（平台自定，如 subscription / openai_api_key / official） */
  authKey: string
  /** 认证方式统计与徽章的 i18n label key */
  authLabelKey: string
  /** 行内徽章。Grok 的 profile_kind 走这里 */
  badges: readonly {
    labelKey: string
    tone: 'neutral' | 'accent' | 'warning'
  }[]
  /** 排序维度（Filters 弹层的排序保留项） */
  sortKeys: { name: string; usageCount: number }
}

const DEFAULT_PORTS: Record<string, string> = {
  http: '80',
  https: '443',
}

const SCHEME_RE = /^[a-zA-Z][a-zA-Z0-9+.-]*:/

/**
 * 把 Base URL 规范化为供应商去重 key。
 * 空值、空白、非字符串或无法解析时返回 null。
 */
export function toVendorKey(baseUrl: unknown): string | null {
  if (typeof baseUrl !== 'string') return null
  const trimmed = baseUrl.trim()
  if (!trimmed) return null

  const candidate = SCHEME_RE.test(trimmed) ? trimmed : `https://${trimmed}`

  let parsed: URL
  try {
    parsed = new URL(candidate)
  } catch {
    return null
  }

  let hostname = parsed.hostname.trim().toLowerCase()
  if (!hostname) return null
  while (hostname.endsWith('.')) {
    hostname = hostname.slice(0, -1)
  }
  if (!hostname) return null

  const isIpv6 = hostname.includes(':')
  const host = isIpv6
    ? hostname.startsWith('[')
      ? hostname
      : `[${hostname}]`
    : hostname

  const scheme = parsed.protocol.replace(/:$/, '').toLowerCase()
  const port = parsed.port
  const defaultPort = DEFAULT_PORTS[scheme]
  const portSuffix = port && port !== defaultPort ? `:${port}` : ''

  return `${host}${portSuffix}`
}
