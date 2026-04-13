/**
 * OpenCode 平台类型定义。
 *
 * 官方参考：
 * - https://opencode.ai/docs/config/
 * - https://opencode.ai/docs/agents/
 * - https://opencode.ai/docs/commands/
 * - https://opencode.ai/docs/skills/
 * - https://opencode.ai/docs/tools/
 * - https://opencode.ai/docs/plugins/
 */

export type OpenCodeScope = 'global' | 'project'
export type OpenCodeAgentMode = 'primary' | 'subagent' | 'all'
export type OpenCodeMcpType = 'local' | 'remote'
export type OpenCodeThemeType = 'light' | 'dark' | 'system'

export interface OpenCodeProviderOptions {
  apiKey?: string
  baseURL?: string
  timeout?: number | false
  chunkTimeout?: number
  setCacheKey?: boolean
  [key: string]: unknown
}

export interface OpenCodeModelLimit {
  context?: number
  output?: number
}

export interface OpenCodeModelConfig {
  name: string
  limit?: OpenCodeModelLimit
  [key: string]: unknown
}

export interface OpenCodeProviderConfig {
  id: string
  name?: string
  options?: OpenCodeProviderOptions
  models?: Record<string, OpenCodeModelConfig>
  enabled?: boolean
  disabled?: boolean
  [key: string]: unknown
}

export interface OpenCodeProviderRequest {
  id: string
  name?: string
  options?: OpenCodeProviderOptions
  models?: Record<string, OpenCodeModelConfig>
  enabled?: boolean
  disabled?: boolean
}

export interface OpenCodeMcpServer {
  id: string
  type: OpenCodeMcpType
  enabled?: boolean
  command?: string[]
  environment?: Record<string, string>
  url?: string
  headers?: Record<string, string>
  [key: string]: unknown
}

export interface OpenCodeMcpServerRequest {
  id: string
  type: OpenCodeMcpType
  enabled?: boolean
  command?: string[]
  environment?: Record<string, string>
  url?: string
  headers?: Record<string, string>
}

export interface OpenCodePluginPackage {
  name: string
}

export interface OpenCodePluginRequest {
  name: string
}

export interface OpenCodePermissionConfig {
  [key: string]: unknown
}

export interface OpenCodeServerConfig {
  port?: number
  hostname?: string
  mdns?: boolean
  mdnsDomain?: string
  cors?: string[]
  [key: string]: unknown
}

export interface OpenCodeAgent {
  name: string
  path: string
  scope: OpenCodeScope
  description?: string
  mode?: OpenCodeAgentMode
  model?: string
  temperature?: number
  topP?: number
  steps?: number
  hidden?: boolean
  disable?: boolean
  color?: string
  permission?: OpenCodePermissionConfig
  tools?: Record<string, unknown>
  body: string
  other?: Record<string, unknown>
  parseError?: string
}

export interface OpenCodeAgentRequest {
  name: string
  scope?: OpenCodeScope
  projectRoot?: string
  description?: string
  mode?: OpenCodeAgentMode
  model?: string
  temperature?: number
  topP?: number
  steps?: number
  hidden?: boolean
  disable?: boolean
  color?: string
  permission?: OpenCodePermissionConfig
  tools?: Record<string, unknown>
  body?: string
  [key: string]: unknown
}

export interface OpenCodeCommand {
  name: string
  path: string
  scope: OpenCodeScope
  description?: string
  agent?: string
  subtask?: boolean
  model?: string
  template: string
  other?: Record<string, unknown>
  parseError?: string
}

export interface OpenCodeCommandRequest {
  name: string
  scope?: OpenCodeScope
  projectRoot?: string
  description?: string
  agent?: string
  subtask?: boolean
  model?: string
  template?: string
  [key: string]: unknown
}

export interface OpenCodeTuiConfig {
  theme?: string
  keybinds?: Record<string, unknown>
  mouse?: boolean
  diff_style?: string
  scroll_speed?: number
  scroll_acceleration?: Record<string, unknown>
  [key: string]: unknown
}

export interface OpenCodeTheme {
  id: string
  name: string
  themeType: OpenCodeThemeType
}

export interface OpenCodeLocalPluginFile {
  name: string
  path: string
  scope: OpenCodeScope
  size: number
}

export interface OpenCodeSkillLocation {
  kind: string
  scope: OpenCodeScope
  path: string
  exists: boolean
  skillCount: number
  skills: string[]
}

export interface OpenCodeConfig {
  $schema?: string
  provider?: Record<string, Omit<OpenCodeProviderConfig, 'id'>>
  mcp?: Record<string, Omit<OpenCodeMcpServer, 'id'>>
  agent?: Record<string, Record<string, unknown>>
  command?: Record<string, Record<string, unknown>>
  plugin?: string[]
  model?: string
  small_model?: string
  default_agent?: string
  share?: 'manual' | 'auto' | 'disabled'
  snapshot?: boolean
  autoupdate?: boolean | 'notify'
  tools?: Record<string, unknown>
  permission?: OpenCodePermissionConfig
  server?: OpenCodeServerConfig
  instructions?: string[]
  enabled_providers?: string[]
  disabled_providers?: string[]
  [key: string]: unknown
}

export interface OpenCodeProviderPreset {
  id: string
  label: string
  description: string
}

export const OPENCODE_PROVIDER_PRESETS: OpenCodeProviderPreset[] = [
  {
    id: 'anthropic',
    label: 'Anthropic',
    description: 'Claude 4 / Sonnet / Haiku provider',
  },
  {
    id: 'openai',
    label: 'OpenAI',
    description: 'GPT and reasoning models',
  },
  {
    id: 'google',
    label: 'Google',
    description: 'Gemini provider',
  },
  {
    id: 'openai-compatible',
    label: 'OpenAI Compatible',
    description: 'Proxy or custom OpenAI-compatible endpoint',
  },
  {
    id: 'amazon-bedrock',
    label: 'Amazon Bedrock',
    description: 'AWS Bedrock runtime',
  },
]
