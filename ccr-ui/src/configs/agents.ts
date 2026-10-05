import {
  addCodexAgent,
  addGeminiAgent,
  addOpenCodeAgent,
  deleteCodexAgent,
  deleteGeminiAgent,
  deleteOpenCodeAgent,
  listAgents,
  listCodexAgents,
  listGeminiAgents,
  listOpenCodeAgents,
  toggleGeminiAgent,
  updateCodexAgent,
  updateGeminiAgent,
  updateOpenCodeAgent,
} from '@/api'
import { addAgent, deleteAgent, toggleAgent, updateAgent } from '@/api/domains/claude'
import { surfaceNotify, type SurfaceNotify } from '@/configs/surfaceNotify'

export interface AgentRecord {
  id: string
  name: string
  description?: string
  enabled?: boolean
  folder?: string
  model?: string
}

export interface AgentDraft {
  name: string
  description?: string
  model?: string
  folder?: string
  body?: string
}

export interface AgentsFeatures {
  folders?: boolean
  toggle?: boolean
  projectContext?: boolean
  sources?: boolean
  copy?: boolean
  rename?: boolean
  tomlValidate?: boolean
}

export interface AgentsConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  parentPath: string
  features: AgentsFeatures
  notify: SurfaceNotify
  list: () => Promise<AgentRecord[]>
  create: (draft: AgentDraft) => Promise<void>
  update: (id: string, draft: AgentDraft) => Promise<void>
  remove: (id: string) => Promise<void>
  toggle?: (id: string) => Promise<void>
}

const asRecord = (value: unknown): Record<string, unknown> =>
  value && typeof value === 'object' ? (value as Record<string, unknown>) : {}

const toAgent = (value: unknown): AgentRecord | null => {
  const source = asRecord(value)
  const name = typeof source.name === 'string' ? source.name : ''
  if (!name) return null
  return {
    id: typeof source.path === 'string' ? source.path : name,
    name,
    description: typeof source.description === 'string' ? source.description : undefined,
    enabled: source.disabled === true ? false : source.enabled === false ? false : true,
    folder: typeof source.folder === 'string' ? source.folder : undefined,
    model: typeof source.model === 'string' ? source.model : undefined,
  }
}

const readAgents = (payload: unknown): unknown[] => {
  if (Array.isArray(payload)) return payload
  const source = asRecord(payload)
  if (Array.isArray(source.agents)) return source.agents
  return []
}

export const claudeAgentsConfig: AgentsConfig = {
  cacheKey: 'agents-claude',
  homePath: '/agents',
  module: 'config',
  i18nPrefix: 'agents',
  titleKey: 'agents.pageTitle',
  subtitleKey: 'agents.subtitle',
  parentPath: '/',
  features: { folders: true, toggle: true },
  notify: surfaceNotify,
  list: async () => readAgents(await listAgents()).map(toAgent).filter((item): item is AgentRecord => item !== null),
  create: async (draft) => {
    await addAgent({ name: draft.name, description: draft.description, model: draft.model })
  },
  update: async (id, draft) => {
    await updateAgent(id, { name: draft.name, description: draft.description, model: draft.model })
  },
  remove: async (id) => {
    await deleteAgent(id)
  },
  toggle: async (id) => {
    await toggleAgent(id)
  },
}

export const geminiAgentsConfig: AgentsConfig = {
  cacheKey: 'agents-gemini',
  homePath: '/antigravity/agents',
  module: 'antigravity',
  i18nPrefix: 'gemini.agents',
  titleKey: 'gemini.agents.pageTitle',
  subtitleKey: 'gemini.agents.subtitle',
  parentPath: '/antigravity',
  features: { folders: true, toggle: true },
  notify: surfaceNotify,
  list: async () => readAgents(await listGeminiAgents()).map(toAgent).filter((item): item is AgentRecord => item !== null),
  create: async (draft) => {
    await addGeminiAgent({ name: draft.name, description: draft.description, model: draft.model })
  },
  update: async (id, draft) => {
    await updateGeminiAgent(id, { name: draft.name, description: draft.description, model: draft.model })
  },
  remove: async (id) => {
    await deleteGeminiAgent(id)
  },
  toggle: async (id) => {
    await toggleGeminiAgent(id)
  },
}

export const codexAgentsConfig: AgentsConfig = {
  cacheKey: 'agents-codex',
  homePath: '/codex/agents',
  module: 'codex',
  i18nPrefix: 'codex.agents',
  titleKey: 'codex.agents.pageTitle',
  subtitleKey: 'codex.agents.subtitle',
  parentPath: '/codex',
  features: { projectContext: true, sources: true, copy: true, rename: true, tomlValidate: true },
  notify: surfaceNotify,
  list: async () => {
    const payload = await listCodexAgents()
    return readAgents(payload).map(toAgent).filter((item): item is AgentRecord => item !== null)
  },
  create: async (draft) => {
    await addCodexAgent({ name: draft.name, description: draft.description, body: draft.body ?? '' })
  },
  update: async (id, draft) => {
    await updateCodexAgent({ name: draft.name || id, description: draft.description, body: draft.body })
  },
  remove: async (id) => {
    await deleteCodexAgent(id)
  },
}

export const opencodeAgentsConfig: AgentsConfig = {
  cacheKey: 'agents-opencode',
  homePath: '/opencode/agents',
  module: 'opencode',
  i18nPrefix: 'opencode.agents',
  titleKey: 'opencode.agents.title',
  subtitleKey: 'opencode.agents.subtitle',
  parentPath: '/opencode',
  features: {},
  notify: surfaceNotify,
  list: async () => {
    const rows = await listOpenCodeAgents()
    return rows.map((row) => ({
      id: row.path || row.name,
      name: row.name,
      description: row.description,
      enabled: row.disable !== true,
      model: row.model,
    }))
  },
  create: async (draft) => {
    await addOpenCodeAgent({ name: draft.name, description: draft.description, model: draft.model, body: draft.body ?? '' })
  },
  update: async (id, draft) => {
    await updateOpenCodeAgent({ name: draft.name || id, description: draft.description, model: draft.model, body: draft.body })
  },
  remove: async (id) => {
    await deleteOpenCodeAgent(id)
  },
}

export const agentsConfigs = {
  claude: claudeAgentsConfig,
  gemini: geminiAgentsConfig,
  codex: codexAgentsConfig,
  opencode: opencodeAgentsConfig,
} as const
