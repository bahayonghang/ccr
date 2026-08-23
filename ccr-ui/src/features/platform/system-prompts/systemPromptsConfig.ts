export type SystemPromptPlatform = 'claude' | 'codex' | 'gemini' | 'opencode'

export interface SystemPromptsFeatures {
  hierarchyNote?: boolean
  geminiNote?: boolean
  showRules?: boolean
  limitHint?: boolean
}

export interface SystemPromptsConfig {
  platform: SystemPromptPlatform
  module: string
  features: SystemPromptsFeatures
}

export const systemPromptsConfigs: Record<SystemPromptPlatform, SystemPromptsConfig> = {
  claude: {
    platform: 'claude',
    module: 'claude-code',
    features: { hierarchyNote: true, showRules: true },
  },
  codex: {
    platform: 'codex',
    module: 'codex',
    features: { limitHint: true },
  },
  gemini: {
    platform: 'gemini',
    module: 'antigravity',
    features: { geminiNote: true },
  },
  opencode: {
    platform: 'opencode',
    module: 'opencode',
    features: {},
  },
}
