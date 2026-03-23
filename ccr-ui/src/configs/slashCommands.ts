import type { PlatformConfig } from '@/types/platform'
import type { SlashCommand, SlashCommandRequest } from '@/types/platform'

import {
  listSlashCommands, addSlashCommand, updateSlashCommand, deleteSlashCommand, toggleSlashCommand,
  listCodexSlashCommands, addCodexSlashCommand, updateCodexSlashCommand, deleteCodexSlashCommand, toggleCodexSlashCommand,
  listGeminiSlashCommands, addGeminiSlashCommand, updateGeminiSlashCommand, deleteGeminiSlashCommand, toggleGeminiSlashCommand,
  listQwenSlashCommands, addQwenSlashCommand, updateQwenSlashCommand, deleteQwenSlashCommand, toggleQwenSlashCommand,
  listQoderCommands, addQoderCommand, updateQoderCommand, deleteQoderCommand, toggleQoderCommand
} from '@/api'

type UnknownRecord = Record<string, unknown>

function asRecord(value: unknown): UnknownRecord {
  return typeof value === 'object' && value !== null ? (value as UnknownRecord) : {}
}

function asStringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : []
}

function normalizeSlashCommand(value: unknown): SlashCommand {
  const source = asRecord(value)
  const enabled = typeof source.enabled === 'boolean'
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
      const data = await listSlashCommands<{ commands?: unknown[]; folders?: unknown }>()
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
    }
  },
  i18n: {
    prefix: 'slashCommands',
    breadcrumb: {
      home: 'slashCommands.breadcrumb.home',
      platform: 'slashCommands.breadcrumb.claudeCode',
      current: 'slashCommands.breadcrumb.slashCommands'
    }
  },
  theme: 'claude-code',
  route: {
    homePath: '/claude-code',
    module: 'claude-code'
  },
  platform: {
    name: 'claude-code',
    displayName: 'Claude Code'
  },
  features: {
    breadcrumb: true,
    glassEffect: true
  }
}

// Codex 平台配置
export const codexConfig: PlatformConfig = {
  api: {
    list: async () => {
      return await listCodexSlashCommands<{ commands: SlashCommand[]; folders: string[] }>()
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
    }
  },
  i18n: {
    prefix: 'codex.slashCommands'
  },
  theme: 'css-variable',
  route: {
    homePath: '/codex',
    module: 'codex'
  },
  platform: {
    name: 'codex',
    displayName: 'Codex'
  },
  features: {
    breadcrumb: false,
    glassEffect: false
  }
}

// Gemini CLI 平台配置
export const geminiConfig: PlatformConfig = {
  api: {
    list: async () => {
      return await listGeminiSlashCommands<{ commands: SlashCommand[]; folders: string[] }>()
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
    }
  },
  i18n: {
    prefix: 'gemini.slashCommands'
  },
  theme: 'css-variable',
  route: {
    homePath: '/gemini-cli',
    module: 'gemini-cli'
  },
  platform: {
    name: 'gemini-cli',
    displayName: 'Gemini CLI'
  },
  features: {
    breadcrumb: false,
    glassEffect: false
  }
}

// Qwen 平台配置
export const qwenConfig: PlatformConfig = {
  api: {
    list: async () => {
      return await listQwenSlashCommands<{ commands: SlashCommand[]; folders: string[] }>()
    },
    add: async (cmd: SlashCommandRequest) => {
      await addQwenSlashCommand(getRequestName(cmd), cmd)
    },
    update: async (name: string, cmd: SlashCommandRequest) => {
      await updateQwenSlashCommand(name, cmd)
    },
    delete: async (name: string) => {
      await deleteQwenSlashCommand(name)
    },
    toggle: async (name: string) => {
      await toggleQwenSlashCommand(name, true)
    }
  },
  i18n: {
    prefix: 'qwen.slashCommands'
  },
  theme: 'css-variable',
  route: {
    homePath: '/qwen',
    module: 'qwen'
  },
  platform: {
    name: 'qwen',
    displayName: 'Qwen'
  },
  features: {
    breadcrumb: false,
    glassEffect: false
  }
}

// Qoder 平台配置
export const qoderConfig: PlatformConfig = {
  api: {
    list: async () => {
      return await listQoderCommands<{ commands: SlashCommand[]; folders: string[] }>()
    },
    add: async (cmd: SlashCommandRequest) => {
      await addQoderCommand(getRequestName(cmd), cmd)
    },
    update: async (name: string, cmd: SlashCommandRequest) => {
      await updateQoderCommand(name, cmd)
    },
    delete: async (name: string) => {
      await deleteQoderCommand(name)
    },
    toggle: async (name: string) => {
      await toggleQoderCommand(name, true)
    }
  },
  i18n: {
    prefix: 'qoder.slashCommands'
  },
  theme: 'css-variable',
  route: {
    homePath: '/qoder',
    module: 'qoder'
  },
  platform: {
    name: 'qoder',
    displayName: 'Qoder'
  },
  features: {
    breadcrumb: false,
    glassEffect: false
  }
}

// 配置映射
export const platformConfigs = {
  'claude-code': claudeCodeConfig,
  'codex': codexConfig,
  'gemini-cli': geminiConfig,
  'qwen': qwenConfig,
  'qoder': qoderConfig
} as const

// 类型导出
export type PlatformName = keyof typeof platformConfigs
export type SlashCommandsConfig = typeof platformConfigs[PlatformName]
