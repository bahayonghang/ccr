import type { IconName } from '@/config/icons'

export type HomePlatformMode = 'cli' | 'managed'

export interface HomeQuickAction {
  title: string
  desc: string
  path: string
  icon: IconName
  tone: 'command' | 'config' | 'sync' | 'usage'
}

export interface HomePlatformRecord {
  title: string
  desc: string
  path: string
  icon: IconName
  iconClass: string
  platformKey: string
  usageKey?: 'claude' | 'codex' | 'gemini' | 'opencode'
  role: string
  mode: HomePlatformMode
  isRuntimeCli: boolean
}

export type HomeUsageMetric = 'sessions' | 'requests' | 'tokens'
