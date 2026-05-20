// Router parameter type definitions for CCR UI

// CLI client type. The persisted Antigravity/Gemini client id remains
// `gemini` so command bookmarks and usage keys stay compatible.
export type CliClient = 'ccr' | 'claude' | 'gemini'

// Route parameter interfaces
export interface AgentDetailRouteParams {
  name: string
}

export interface CheckinAccountDashboardRouteParams {
  accountId: string
}

export interface CommandsRouteParams {
  client?: CliClient
}

/**
 * Safely extract a string parameter from route params.
 * Handles both string and string[] types that vue-router may return.
 * @param param - The route parameter value (string | string[] | undefined)
 * @returns The extracted string or null if invalid
 */
export function extractStringParam(param: string | string[] | undefined): string | null {
  if (typeof param === 'string' && param.trim()) {
    return param.trim()
  }
  if (Array.isArray(param) && param.length > 0 && typeof param[0] === 'string') {
    return param[0].trim() || null
  }
  return null
}

/**
 * Normalize CLI client parameter from route.
 * Handles legacy/platform aliases and validates client type.
 * @param clientParam - The route parameter value
 * @returns The normalized CliClient or null if invalid
 */
export function normalizeCliClient(clientParam: string | string[] | undefined): CliClient | null {
  const client = extractStringParam(clientParam)
  if (!client) return null

  // Handle alias
  const normalized = client.toLowerCase()

  if (normalized === 'claude-code') return 'claude'
  if (
    normalized === 'agy' ||
    normalized === 'antigravity' ||
    normalized === 'antigravity-cli' ||
    normalized === 'gemini-cli'
  ) {
    return 'gemini'
  }

  // Validate
  const validClients: CliClient[] = ['ccr', 'claude', 'gemini']
  return validClients.includes(normalized as CliClient) ? (normalized as CliClient) : null
}
