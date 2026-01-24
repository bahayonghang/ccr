export interface SlashCommand {
  name: string
  command: string
  description: string
  folder: string
  enabled: boolean
}

export interface SlashCommandRequest {
  name: string
  command: string
  description: string
  folder: string
}

export interface FolderOption {
  label: string
  value: string
  icon: any
  count: number
}

export interface CommandStats {
  total: number
  enabled: number
  disabled: number
  byFolder: Record<string, number>
}

export interface PlatformConfig {
  api: {
    list: () => Promise<{ commands: SlashCommand[], folders: string[] }>
    add: (cmd: SlashCommandRequest) => Promise<void>
    update: (name: string, cmd: SlashCommandRequest) => Promise<void>
    delete: (name: string) => Promise<void>
    toggle: (name: string) => Promise<void>
  }
  i18n: {
    prefix: string
    breadcrumb?: {
      home: string
      platform: string
      current: string
    }
  }
  theme: 'claude-code' | 'css-variable'
  route: {
    homePath: string
    module: string
  }
  platform: {
    name: string
    displayName: string
  }
  features: {
    breadcrumb: boolean
    glassEffect: boolean
  }
}

export interface ThemeColors {
  module: string
  primary: string
  secondary: string
  muted: string
  bg: string
  bgSecondary: string
  bgTertiary: string
  accent: string
  accentBg: string
  accentBorder: string
}

export interface PlatformModuleCapabilities {
  config: boolean
  mcp: boolean
  profiles: boolean
  auth: boolean
  usage: boolean
  agents: boolean
  slash_commands: boolean
  skills: boolean
  plugins: boolean
  commands: boolean
}

export interface PlatformCapabilitiesResponse {
  platforms: Record<string, PlatformModuleCapabilities>
}
