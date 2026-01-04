import type { PlatformConfig } from '@/types/platform'

import { api } from '@/api/client'

// Claude Code 平台配置
export const claudeCodeConfig: PlatformConfig = {
  api: {
    list: async () => {
      const response = await api.get('/slash-commands')
      const data = response.data.data || response.data
      // Map 'disabled' field to 'enabled' for component compatibility
      const commands = (data.commands || []).map((cmd: any) => ({
        ...cmd,
        enabled: cmd.enabled !== undefined ? cmd.enabled : !cmd.disabled
      }))
      return { commands, folders: data.folders || [] }
    },
    add: async (cmd: any) => {
      await api.post('/slash-commands', cmd)
    },
    update: async (name: string, cmd: any) => {
      await api.put(`/slash-commands/${name}`, cmd)
    },
    delete: async (name: string) => {
      await api.delete(`/slash-commands/${name}`)
    },
    toggle: async (name: string) => {
      await api.put(`/slash-commands/${name}/toggle`)
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
      const response = await api.get('/codex/slash-commands')
      return response.data
    },
    add: async (cmd: any) => {
      await api.post('/codex/slash-commands', cmd)
    },
    update: async (name: string, cmd: any) => {
      await api.put(`/codex/slash-commands/${name}`, cmd)
    },
    delete: async (name: string) => {
      await api.delete(`/codex/slash-commands/${name}`)
    },
    toggle: async (name: string) => {
      await api.put(`/codex/slash-commands/${name}/toggle`)
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
      const response = await api.get('/gemini/slash-commands')
      return response.data
    },
    add: async (cmd: any) => {
      await api.post('/gemini/slash-commands', cmd)
    },
    update: async (name: string, cmd: any) => {
      await api.put(`/gemini/slash-commands/${name}`, cmd)
    },
    delete: async (name: string) => {
      await api.delete(`/gemini/slash-commands/${name}`)
    },
    toggle: async (name: string) => {
      await api.put(`/gemini/slash-commands/${name}/toggle`)
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
      const response = await api.get('/qwen/slash-commands')
      return response.data
    },
    add: async (cmd: any) => {
      await api.post('/qwen/slash-commands', cmd)
    },
    update: async (name: string, cmd: any) => {
      await api.put(`/qwen/slash-commands/${name}`, cmd)
    },
    delete: async (name: string) => {
      await api.delete(`/qwen/slash-commands/${name}`)
    },
    toggle: async (name: string) => {
      await api.put(`/qwen/slash-commands/${name}/toggle`)
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
      const response = await api.get('/iflow/slash-commands')
      return response.data
    },
    add: async (cmd: any) => {
      await api.post('/iflow/slash-commands', cmd)
    },
    update: async (name: string, cmd: any) => {
      await api.put(`/iflow/slash-commands/${name}`, cmd)
    },
    delete: async (name: string) => {
      await api.delete(`/iflow/slash-commands/${name}`)
    },
    toggle: async (name: string) => {
      await api.put(`/iflow/slash-commands/${name}/toggle`)
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