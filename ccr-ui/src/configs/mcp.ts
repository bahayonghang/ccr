import {
  addCodexMcpServer,
  addGeminiMcpServer,
  addOpenCodeMcpServer,
  deleteCodexMcpServer,
  deleteGeminiMcpServer,
  deleteOpenCodeMcpServer,
  listCodexMcpServers,
  listGeminiMcpServers,
  listOpenCodeMcpServers,
  updateCodexMcpServer,
  updateGeminiMcpServer,
  updateOpenCodeMcpServer,
} from '@/api'
import { surfaceNotify, type SurfaceNotify } from '@/configs/surfaceNotify'
import type { GeminiMcpServerRequest } from '@/types'

export interface McpServerRecord {
  id: string
  name: string
  command?: string
  url?: string
  args?: string[]
  env?: Record<string, string>
  cwd?: string
  timeout?: number
  trust?: boolean
  includeTools?: string[]
  headers?: Record<string, string>
  enabled?: boolean
  transport?: 'stdio' | 'http'
  startupTimeoutMs?: number
  bearerTokenEnv?: string
  enabledTools?: string[]
  disabledTools?: string[]
}

export interface McpDraft {
  name: string
  command?: string
  url?: string
  args?: string[]
  env?: Record<string, string>
  cwd?: string
  timeout?: number
  trust?: boolean
  includeTools?: string[]
  headers?: Record<string, string>
  enabled?: boolean
  transport?: 'stdio' | 'http'
  startupTimeoutMs?: number
  bearerTokenEnv?: string
}

export interface McpFeatures {
  httpCreate?: boolean
  stdioCreate?: boolean
  statsStrip?: boolean
  authInjection?: boolean
  toolScope?: boolean
  startupPolicy?: boolean
  enabledFlag?: boolean
}

export interface McpConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  parentPath: string
  features: McpFeatures
  notify: SurfaceNotify
  list: () => Promise<McpServerRecord[]>
  create: (draft: McpDraft) => Promise<void>
  update: (id: string, draft: McpDraft) => Promise<void>
  remove: (id: string) => Promise<void>
}

export const geminiMcpConfig: McpConfig = {
  cacheKey: 'mcp-gemini',
  homePath: '/antigravity/mcp',
  module: 'antigravity',
  i18nPrefix: 'gemini.mcp',
  titleKey: 'gemini.mcp.pageTitle',
  subtitleKey: 'gemini.mcp.subtitle',
  parentPath: '/antigravity',
  features: { httpCreate: true, stdioCreate: true },
  notify: surfaceNotify,
  list: async () => {
    const servers = await listGeminiMcpServers()
    return servers.map((server) => ({
      id: server.name,
      name: server.name,
      command: server.command,
      url: server.url,
      args: server.args,
      env: server.env,
      cwd: server.cwd,
      timeout: server.timeout,
      trust: server.trust,
      includeTools: server.includeTools,
      transport: server.url ? 'http' : 'stdio',
    }))
  },
  create: async (draft) => {
    const request: GeminiMcpServerRequest = {
      name: draft.name,
      command: draft.command,
      args: draft.args,
      env: draft.env,
      cwd: draft.cwd,
      timeout: draft.timeout,
      trust: draft.trust,
      includeTools: draft.includeTools,
      url: draft.url,
    }
    await addGeminiMcpServer(request)
  },
  update: async (id, draft) => {
    const request: GeminiMcpServerRequest = {
      name: draft.name,
      command: draft.command,
      args: draft.args,
      env: draft.env,
      cwd: draft.cwd,
      timeout: draft.timeout,
      trust: draft.trust,
      includeTools: draft.includeTools,
      url: draft.url,
    }
    await updateGeminiMcpServer(id, request)
  },
  remove: async (id) => {
    await deleteGeminiMcpServer(id)
  },
}

export const codexMcpConfig: McpConfig = {
  cacheKey: 'mcp-codex',
  homePath: '/codex/mcp',
  module: 'codex',
  i18nPrefix: 'codex.mcp',
  titleKey: 'codex.mcp.title',
  subtitleKey: 'codex.mcp.subtitle',
  parentPath: '/codex',
  features: {
    httpCreate: true,
    stdioCreate: true,
    statsStrip: true,
    authInjection: true,
    toolScope: true,
    startupPolicy: true,
    enabledFlag: true,
  },
  notify: surfaceNotify,
  list: async () => {
    const payload = await listCodexMcpServers()
    return payload.servers.map((server) => ({
      id: server.name,
      name: server.name,
      command: server.command ?? undefined,
      url: server.url ?? undefined,
      args: server.args ?? undefined,
      env: server.env ?? undefined,
      cwd: server.cwd ?? undefined,
      timeout: server.startup_timeout_ms ?? undefined,
      includeTools: server.enabled_tools ?? undefined,
      headers: server.http_headers ?? undefined,
      enabled: server.enabled ?? undefined,
      transport: server.transport ?? (server.url ? 'http' : 'stdio'),
      startupTimeoutMs: server.startup_timeout_ms ?? undefined,
      bearerTokenEnv: server.bearer_token_env_var ?? undefined,
      enabledTools: server.enabled_tools ?? undefined,
      disabledTools: server.disabled_tools ?? undefined,
    }))
  },
  create: async (draft) => {
    await addCodexMcpServer(draft.name, {
      command: draft.command,
      url: draft.url,
      args: draft.args,
      env: draft.env,
      cwd: draft.cwd,
      startup_timeout_ms: draft.startupTimeoutMs,
      bearer_token_env_var: draft.bearerTokenEnv,
      enabled_tools: draft.includeTools,
      http_headers: draft.headers,
      enabled: draft.enabled,
    })
  },
  update: async (id, draft) => {
    await updateCodexMcpServer(id, {
      command: draft.command,
      url: draft.url,
      args: draft.args,
      env: draft.env,
      cwd: draft.cwd,
      startup_timeout_ms: draft.startupTimeoutMs,
      bearer_token_env_var: draft.bearerTokenEnv,
      enabled_tools: draft.includeTools,
      http_headers: draft.headers,
      enabled: draft.enabled,
    })
  },
  remove: async (id) => {
    await deleteCodexMcpServer(id)
  },
}

export const opencodeMcpConfig: McpConfig = {
  cacheKey: 'mcp-opencode',
  homePath: '/opencode/mcp',
  module: 'opencode',
  i18nPrefix: 'opencode.mcp',
  titleKey: 'opencode.mcp.title',
  subtitleKey: 'opencode.mcp.subtitle',
  parentPath: '/opencode',
  features: { httpCreate: true, stdioCreate: true, enabledFlag: true },
  notify: surfaceNotify,
  list: async () => {
    const servers = await listOpenCodeMcpServers()
    return servers.map((server) => ({
      id: server.id,
      name: server.id,
      command: Array.isArray(server.command) ? server.command[0] : undefined,
      args: Array.isArray(server.command) ? server.command.slice(1) : undefined,
      url: server.url,
      env: server.environment,
      headers: server.headers,
      enabled: server.enabled,
      transport: server.type === 'remote' ? 'http' : 'stdio',
    }))
  },
  create: async (draft) => {
    const command = draft.command ? [draft.command, ...(draft.args ?? [])] : undefined
    await addOpenCodeMcpServer(draft.name, {
      type: draft.url ? 'remote' : 'local',
      enabled: draft.enabled,
      command,
      environment: draft.env,
      url: draft.url,
      headers: draft.headers,
    })
  },
  update: async (id, draft) => {
    const command = draft.command ? [draft.command, ...(draft.args ?? [])] : undefined
    await updateOpenCodeMcpServer(id, {
      type: draft.url ? 'remote' : 'local',
      enabled: draft.enabled,
      command,
      environment: draft.env,
      url: draft.url,
      headers: draft.headers,
    })
  },
  remove: async (id) => {
    await deleteOpenCodeMcpServer(id)
  },
}

export const mcpConfigs = {
  gemini: geminiMcpConfig,
  codex: codexMcpConfig,
  opencode: opencodeMcpConfig,
} as const
