export type Platform = 'claude-code' | 'codex' | 'gemini' | 'qwen' | 'qoder' | 'droid' | 'iflow' | 'opencode'
export type SkillOrigin = 'marketplace' | 'github' | 'repo' | 'local' | 'npx' | 'unknown'
export type SkillsTab = 'inventory' | 'sources' | 'marketplace'
export type SkillSource = 'all' | 'user' | 'plugin' | 'remote' | SkillOrigin
export type ImportSource = 'marketplace' | 'github' | 'local' | 'npx'

export interface PlatformTheme {
  displayName: string
  icon: string
  color: string
  tailwindColor: string
}

export const PLATFORM_CONFIG: Record<Platform, PlatformTheme> = {
  'claude-code': {
    displayName: 'Claude Code',
    icon: 'Code2',
    color: '#A78BFA',
    tailwindColor: 'purple-400',
  },
  codex: {
    displayName: 'Codex',
    icon: 'Sparkles',
    color: '#34D399',
    tailwindColor: 'emerald-400',
  },
  gemini: {
    displayName: 'Gemini CLI',
    icon: 'Gem',
    color: '#60A5FA',
    tailwindColor: 'blue-400',
  },
  qwen: {
    displayName: 'Qwen',
    icon: 'Zap',
    color: '#22D3EE',
    tailwindColor: 'cyan-400',
  },
  qoder: {
    displayName: 'Qoder',
    icon: 'Activity',
    color: '#FBBF24',
    tailwindColor: 'amber-400',
  },
  droid: {
    displayName: 'Droid',
    icon: 'Bot',
    color: '#F472B6',
    tailwindColor: 'pink-400',
  },
  iflow: {
    displayName: 'iFlow',
    icon: 'Workflow',
    color: '#FB923C',
    tailwindColor: 'orange-400',
  },
  opencode: {
    displayName: 'OpenCode',
    icon: 'Terminal',
    color: '#A3E635',
    tailwindColor: 'lime-400',
  },
}

export interface SkillInstallationRecord {
  id: string
  platformId: Platform
  platformName: string
  installPath: string
  installMode: 'copy'
  installedAt?: number
  isPrimary: boolean
}

export interface SkillRecord {
  id: string
  name: string
  description?: string
  category?: string
  tags: string[]
  version?: string
  author?: string
  origin: SkillOrigin
  sourceLabel?: string
  sourceRef?: string
  installCount: number
  installations: SkillInstallationRecord[]
  editableInstallations: string[]
}

export interface SkillSourceSkillRecord {
  id: string
  name: string
  description?: string
  category?: string
  tags: string[]
  installRef: string
}

export interface SkillSourceRecord {
  id: string
  type: 'git' | 'local'
  name: string
  description?: string
  location: string
  skillsRoot: string
  skillCount: number
  lastSyncedAt?: string
  health: 'ok' | 'error' | 'missing'
  skills: SkillSourceSkillRecord[]
}

export interface SkillPlatformSummary {
  id: Platform
  displayName: string
  globalSkillsDir: string
  detected: boolean
  installedCount: number
}

export interface SkillsInventoryResponse {
  skills: SkillRecord[]
  platforms: SkillPlatformSummary[]
  total: number
}

export interface SkillContent {
  skillId: string
  installationId: string
  name: string
  description?: string
  category?: string
  tags: string[]
  raw: string
  content: string
  skillDir: string
}

export interface SkillOperationResult {
  agent: string
  ok: boolean
  message?: string
}

export interface SkillOperationResponse {
  results: SkillOperationResult[]
}

export interface MarketplaceItem {
  package: string
  owner: string
  repo: string
  skill?: string
  skillsShUrl: string
  description?: string
  authorAvatar?: string
  stars?: number
}

export interface MarketplaceResponse {
  items: MarketplaceItem[]
  total: number
  page: number
  pageSize: number
  cached: boolean
}

export interface NpxStatus {
  available: boolean
  version?: string
  path?: string
}

export interface SkillFilters {
  search: string
  platform: Platform | 'all'
  origin?: SkillOrigin | 'all'
  category: string | null
  tags: string[]
  source: SkillSource
}

export interface SkillsRouteState {
  tab: SkillsTab
  selected: string | null
  mode: 'view' | 'edit'
  platform: Platform | 'all'
  origin: SkillOrigin | 'all'
  q: string
  page: number
  source: string | null
}

export interface SkillLogEntry {
  id: string
  action: string
  target: string
  status: 'pending' | 'success' | 'error'
  detail?: string
  timestamp: number
}

export interface SkillsInstallRequest {
  sourceKind: 'marketplace' | 'github' | 'local' | 'npx' | 'source'
  sourceRef: string
  sourceSkillId?: string
  targetPlatforms: Platform[]
  force?: boolean
}

export interface SkillsSyncRequest {
  skillId: string
  installationId?: string
  targetPlatforms: Platform[]
  force?: boolean
}

// Legacy compatibility types for still-existing components.
export interface PlatformSummary extends SkillPlatformSummary {
  display_name: string
  global_skills_dir: string
  installed_count: number
}

export interface UnifiedSkill {
  name: string
  description?: string
  skillDir: string
  platform: Platform
  platformName: string
  category?: string
  tags: string[]
  version?: string
  author?: string
  source?: SkillOrigin
  sourceUrl?: string
  installDate?: number
  commitHash?: string
}

export interface InstallProgress {
  phase: 'idle' | 'downloading' | 'installing' | 'done' | 'error'
  package: string
  message?: string
  startedAt: number
}

export interface SkillsStats {
  installed: number
  available: number
  activePlatforms: number
  totalPlatforms: number
}
