import type { PlatformConfig } from '@/types/platform'

import {
  listSlashCommands, addSlashCommand, updateSlashCommand, deleteSlashCommand, toggleSlashCommand,
  listCodexSlashCommands, addCodexSlashCommand, updateCodexSlashCommand, deleteCodexSlashCommand, toggleCodexSlashCommand,
  listGeminiSlashCommands, addGeminiSlashCommand, updateGeminiSlashCommand, deleteGeminiSlashCommand, toggleGeminiSlashCommand,
  listQwenSlashCommands, addQwenSlashCommand, updateQwenSlashCommand, deleteQwenSlashCommand, toggleQwenSlashCommand,
  listIflowSlashCommands, addIflowSlashCommand, updateIflowSlashCommand, deleteIflowSlashCommand, toggleIflowSlashCommand
} from '@/api'

// Claude Code 平台配置
export const claudeCodeConfig: PlatformConfig = {
  api: {
    list: async () => {
      const data = await listSlashCommands()
      // Map 'disabled' field to 'enabled' for component compatibility
      const commands = (data.commands || []).map((cmd: any) => ({
        ...cmd,
        enabled: cmd.enabled !== undefined ? cmd.enabled : !cmd.disabled
      }))
      return { commands, folders: data.folders || [] }
    },
    add: async (cmd: any) => {
      await addSlashCommand(cmd.name, cmd)
    },
    update: async (name: string, cmd: any) => {
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
      return await listCodexSlashCommands()
    },
    add: async (cmd: any) => {
      await addCodexSlashCommand(cmd.name, cmd)
    },
    update: async (name: string, cmd: any) => {
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
      return await listGeminiSlashCommands()
    },
    add: async (cmd: any) => {
      await addGeminiSlashCommand(cmd.name, cmd)
    },
    update: async (name: string, cmd: any) => {
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
      return await listQwenSlashCommands()
    },
    add: async (cmd: any) => {
      await addQwenSlashCommand(cmd.name, cmd)
    },
    update: async (name: string, cmd: any) => {
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

// iFlow 平台配置
export const iflowConfig: PlatformConfig = {
  api: {
    list: async () => {
      return await listIflowSlashCommands()
    },
    add: async (cmd: any) => {
      await addIflowSlashCommand(cmd.name, cmd)
    },
    update: async (name: string, cmd: any) => {
      await updateIflowSlashCommand(name, cmd)
    },
    delete: async (name: string) => {
      await deleteIflowSlashCommand(name)
    },
    toggle: async (name: string) => {
      await toggleIflowSlashCommand(name, true)
    }
  },
  i18n: {
    prefix: 'iflow.slashCommands'
  },
  theme: 'css-variable',
  route: {
    homePath: '/iflow',
    module: 'iflow'
  },
  platform: {
    name: 'iflow',
    displayName: 'iFlow'
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
  'iflow': iflowConfig
} as const

// 类型导出
export type PlatformName = keyof typeof platformConfigs
export type SlashCommandsConfig = typeof platformConfigs[PlatformName]
