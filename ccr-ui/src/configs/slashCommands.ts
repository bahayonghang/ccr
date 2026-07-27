import type { PlatformConfig } from '@/types/platform'
import type { SlashCommand, SlashCommandRequest } from '@/types/platform'

import {
  listSlashCommands,
  addSlashCommand,
  updateSlashCommand,
  deleteSlashCommand,
  toggleSlashCommand,
  listCodexSlashCommands,
  addCodexSlashCommand,
  updateCodexSlashCommand,
  deleteCodexSlashCommand,
  toggleCodexSlashCommand,
  listGeminiSlashCommands,
  addGeminiSlashCommand,
  updateGeminiSlashCommand,
  deleteGeminiSlashCommand,
  toggleGeminiSlashCommand,
} from '@/api'

import type { UnknownRecord } from '@/types/common'

function asRecord(value: unknown): UnknownRecord {
  return typeof value === 'object' && value !== null ? (value as UnknownRecord) : {}
}

function asStringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : []
}

function normalizeSlashCommand(value: unknown): SlashCommand {
  const source = asRecord(value)
  const enabled =
    typeof source.enabled === 'boolean'
      ? source.enabled
      : !(typeof source.disabled === 'boolean' ? source.disabled : false)

  return {
    name: String(source.name ?? ''),
    command: String(source.command ?? ''),
    description: String(source.description ?? ''),
    folder: String(source.folder ?? ''),
    enabled,
  }
}

function getRequestName(cmd: SlashCommandRequest): string {
  if (!cmd.name.trim()) {
    throw new Error('Slash command name is required')
  }
  return cmd.name
}

// Claude Code 平台配置
export const claudeCodeConfig: PlatformConfig = {
  api: {
    list: async () => {
      const data = await listSlashCommands()
      return {
        commands: (data.commands ?? []).map(normalizeSlashCommand),
        folders: asStringArray(data.folders),
      }
    },
    add: async (cmd: SlashCommandRequest) => {
      await addSlashCommand(getRequestName(cmd), cmd)
    },
    update: async (name: string, cmd: SlashCommandRequest) => {
      await updateSlashCommand(name, cmd)
    },
    delete: async (name: string) => {
      await deleteSlashCommand(name)
    },
    toggle: async (name: string) => {
      // TODO: toggleSlashCommand 需要 enabled 参数，此处默认传 true（后端应处理 toggle 逻辑）
      await toggleSlashCommand(name, true)
    },
  },
  i18n: {
    prefix: 'slashCommands',
    breadcrumb: {
      home: 'slashCommands.breadcrumb.home',
      platform: 'slashCommands.breadcrumb.claudeCode',
      current: 'slashCommands.breadcrumb.slashCommands',
    },
  },
  theme: 'claude-code',
  route: {
    homePath: '/claude-code',
    module: 'claude-code',
  },
  platform: {
    name: 'claude-code',
    displayName: 'Claude Code',
  },
  features: {
    breadcrumb: true,
    glassEffect: true,
  },
}

// Codex 平台配置
export const codexConfig: PlatformConfig = {
  api: {
    list: async () => {
      return await listCodexSlashCommands()
    },
    add: async (cmd: SlashCommandRequest) => {
      await addCodexSlashCommand(getRequestName(cmd), cmd)
    },
    update: async (name: string, cmd: SlashCommandRequest) => {
      await updateCodexSlashCommand(name, cmd)
    },
    delete: async (name: string) => {
      await deleteCodexSlashCommand(name)
    },
    toggle: async (name: string) => {
      await toggleCodexSlashCommand(name, true)
    },
  },
  i18n: {
    prefix: 'codex.slashCommands',
  },
  theme: 'css-variable',
  route: {
    homePath: '/codex',
    module: 'codex',
  },
  platform: {
    name: 'codex',
    displayName: 'Codex',
  },
  features: {
    breadcrumb: false,
    glassEffect: false,
  },
}

// Antigravity CLI 平台配置（内部 key 仍为 gemini）
export const geminiConfig: PlatformConfig = {
  api: {
    list: async () => {
      const data = asRecord(await listGeminiSlashCommands())
      return {
        commands: (Array.isArray(data.commands) ? data.commands : []).map(normalizeSlashCommand),
        folders: asStringArray(data.folders),
      }
    },
    add: async (cmd: SlashCommandRequest) => {
      await addGeminiSlashCommand(getRequestName(cmd), cmd)
    },
    update: async (name: string, cmd: SlashCommandRequest) => {
      await updateGeminiSlashCommand(name, cmd)
    },
    delete: async (name: string) => {
      await deleteGeminiSlashCommand(name)
    },
    toggle: async (name: string) => {
      await toggleGeminiSlashCommand(name, true)
    },
  },
  i18n: {
    prefix: 'gemini.slashCommands',
  },
  theme: 'css-variable',
  route: {
    homePath: '/antigravity',
    module: 'antigravity',
  },
  platform: {
    name: 'antigravity',
    displayName: 'Antigravity CLI',
  },
  features: {
    breadcrumb: false,
    glassEffect: false,
  },
}

// 配置映射
export const platformConfigs = {
  'claude-code': claudeCodeConfig,
  codex: codexConfig,
  antigravity: geminiConfig,
  'gemini-cli': geminiConfig,
} as const

// 类型导出
export type PlatformName = keyof typeof platformConfigs
export type SlashCommandsConfig = (typeof platformConfigs)[PlatformName]
