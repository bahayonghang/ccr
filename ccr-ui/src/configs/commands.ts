import {
  addOpenCodeCommand,
  deleteOpenCodeCommand,
  listCommands,
  listOpenCodeCommands,
  startCcrCommandJob,
  updateOpenCodeCommand,
} from '@/api'
import { surfaceNotify, type SurfaceNotify } from '@/configs/surfaceNotify'

export interface CommandRecord {
  id: string
  name: string
  description?: string
  enabled?: boolean
  template?: string
}

export interface CommandDraft {
  name: string
  description?: string
  template?: string
}

export interface CommandsFeatures {
  execute?: boolean
  history?: boolean
  clientSwitcher?: boolean
  templateCrud?: boolean
  builtinOverrideHint?: boolean
}

export interface CommandsConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  features: CommandsFeatures
  notify: SurfaceNotify
  list: (client?: string) => Promise<CommandRecord[]>
  execute?: (command: string) => Promise<void>
  create?: (draft: CommandDraft) => Promise<void>
  update?: (id: string, draft: CommandDraft) => Promise<void>
  remove?: (id: string) => Promise<void>
}

const asRecord = (value: unknown): Record<string, unknown> =>
  value && typeof value === 'object' ? (value as Record<string, unknown>) : {}

const toCommand = (value: unknown): CommandRecord | null => {
  const source = asRecord(value)
  const name = typeof source.name === 'string' ? source.name : typeof source.command === 'string' ? source.command : ''
  if (!name) return null
  const id = typeof source.id === 'string' ? source.id : typeof source.path === 'string' ? source.path : name
  return {
    id,
    name,
    description: typeof source.description === 'string' ? source.description : undefined,
    enabled: typeof source.enabled === 'boolean' ? source.enabled : undefined,
    template: typeof source.template === 'string' ? source.template : undefined,
  }
}

export const claudeCommandsConfig: CommandsConfig = {
  cacheKey: 'commands-ccr',
  homePath: '/commands',
  module: 'tools',
  i18nPrefix: 'commands',
  titleKey: 'commands.title',
  subtitleKey: 'commands.description',
  features: { execute: true, history: true, clientSwitcher: true },
  notify: surfaceNotify,
  list: async (client) => {
    const payload = await listCommands(client)
    const rows = Array.isArray(payload) ? payload : asRecord(payload).commands
    const list = Array.isArray(rows) ? rows : []
    return list.map(toCommand).filter((item): item is CommandRecord => item !== null)
  },
  execute: async (command) => {
    await startCcrCommandJob({ command })
  },
}

export const opencodeCommandsConfig: CommandsConfig = {
  cacheKey: 'commands-opencode',
  homePath: '/opencode/commands',
  module: 'opencode',
  i18nPrefix: 'opencode.commands',
  titleKey: 'opencode.commands.title',
  subtitleKey: 'opencode.commands.subtitle',
  features: { templateCrud: true, builtinOverrideHint: true },
  notify: surfaceNotify,
  list: async () => {
    const rows = await listOpenCodeCommands()
    return rows.map((row) => ({
      id: row.path || row.name,
      name: row.name,
      description: row.description,
      template: row.template,
    }))
  },
  create: async (draft) => {
    await addOpenCodeCommand({ name: draft.name, description: draft.description, template: draft.template ?? '' })
  },
  update: async (id, draft) => {
    await updateOpenCodeCommand({ name: draft.name || id, description: draft.description, template: draft.template })
  },
  remove: async (id) => {
    await deleteOpenCodeCommand(id)
  },
}

export const commandsConfigs = {
  claude: claudeCommandsConfig,
  opencode: opencodeCommandsConfig,
} as const
