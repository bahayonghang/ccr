import type { TranslateFunction } from '@/utils/tf'

export interface ParsedMcpServer {
  name: string
  type: 'stdio' | 'http'
  command?: string
  args?: string[]
  url?: string
  env?: Record<string, string>
  headers?: Record<string, string>
}

export interface McpImportParseResult {
  servers: ParsedMcpServer[]
  error: string
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (typeof value !== 'object' || value === null) return null
  return value as Record<string, unknown>
}

function toStringMap(value: unknown): Record<string, string> | undefined {
  const record = asRecord(value)
  if (!record) return undefined
  return Object.fromEntries(Object.entries(record).map(([key, item]) => [key, String(item)]))
}

/** 单条失败时返回错误文案，成功返回服务器。 */
function parseOneServer(
  name: string,
  config: unknown,
  t: TranslateFunction,
): ParsedMcpServer | string {
  const cfg = asRecord(config) ?? {}
  const hasCommand = typeof cfg.command === 'string'
  const hasUrl = typeof cfg.url === 'string'
  if (!hasCommand && !hasUrl) {
    return t('mcp.manager.import.errors.missingCommandOrUrl', { name })
  }

  return {
    name,
    type: hasCommand ? 'stdio' : 'http',
    command: hasCommand ? String(cfg.command) : undefined,
    args: Array.isArray(cfg.args) ? cfg.args.map(String) : undefined,
    url: hasUrl ? String(cfg.url) : undefined,
    env: toStringMap(cfg.env),
    headers: toStringMap(cfg.headers),
  }
}

/** 解析 `mcpServers` 或顶层服务器对象。空串返回空结果，不报错。 */
export function parseMcpImportJson(value: string, t: TranslateFunction): McpImportParseResult {
  if (!value.trim()) return { servers: [], error: '' }

  let parsed: unknown
  try {
    parsed = JSON.parse(value)
  } catch {
    return { servers: [], error: t('mcp.manager.import.errors.invalidJson') }
  }

  const root = asRecord(parsed)
  if (!root) {
    return { servers: [], error: t('mcp.manager.import.errors.invalidFormat') }
  }

  const mcpServers = asRecord(root.mcpServers ?? parsed)
  if (!mcpServers) {
    return { servers: [], error: t('mcp.manager.import.errors.invalidFormat') }
  }

  const servers: ParsedMcpServer[] = []
  for (const [name, config] of Object.entries(mcpServers)) {
    const item = parseOneServer(name, config, t)
    if (typeof item === 'string') return { servers: [], error: item }
    servers.push(item)
  }

  return { servers, error: '' }
}
