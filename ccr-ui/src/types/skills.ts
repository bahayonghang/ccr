export type Platform = string
export type SkillOrigin = 'marketplace' | 'github' | 'repo' | 'local' | 'npx' | 'unknown'
export type SkillsTab = 'library' | 'explore' | 'platforms' | 'sources' | 'inventory' | 'marketplace'
export type SkillSource = 'all' | 'user' | 'plugin' | 'remote' | SkillOrigin | `src_${string}`
export type ImportSource = 'marketplace' | 'github' | 'local' | 'npx'
export type SkillTargetStatus = 'ok' | 'pending' | 'error' | 'missing' | 'stale' | 'unknown'
export type SkillWorkflowStatus = 'idle' | 'pending' | 'success' | 'error'
export type SkillInstallStrategy = 'managedcopy' | 'directcopy' | 'directcli'

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

export interface SkillTargetRecord {
  id: string
  platformId: Platform
  platformName: string
  targetPath: string
  syncMode: 'copy'
  status: SkillTargetStatus
  syncedAt?: number
  lastError?: string
  isPrimary: boolean
}

export interface SkillLifecycleSummary {
  sourceRef?: string
  sourceLabel?: string
  sourceRevision?: string
  contentHash?: string
  lastSyncedAt?: number
  hasErrors: boolean
  targetCount: number
  healthyTargetCount: number
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
  targets: SkillTargetRecord[]
  lifecycle: SkillLifecycleSummary
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
  sharedDirGroup?: string
  installStrategy?: SkillInstallStrategy
  npxAgentKey?: string
  category?: string
  capabilities?: string[]
  sortOrder?: number
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

export interface SkillFileEntry {
  path: string
  size: number
  isDir: boolean
}

export interface SkillFileContent {
  skillId: string
  installationId: string
  path: string
  content: string
}

export interface SkillsChangePayload {
  paths: string[]
  affectsInventory: boolean
  affectsSources: boolean
  affectsMarketplace: boolean
}

export interface OnboardingCandidate {
  skillId: string
  name: string
  platformIds: Platform[]
  installationIds: string[]
  installationPaths: string[]
  reason: 'missing_source' | 'unknown_origin'
}

export interface SkillOperationResult {
  agent: string
  ok: boolean
  message?: string
}

export interface SkillWorkflowState {
  action: string
  target: string
  status: SkillWorkflowStatus
  targetPlatforms?: Platform[]
  results?: SkillOperationResult[]
  detail?: string
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
  selectedSkills?: string[]
  targetPlatforms: Platform[]
  force?: boolean
  scope?: 'global' | 'project'
  copyMode?: boolean
  allMode?: boolean
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

export interface NpxPlatformSupport {
  platformId: Platform
  platformName: string
  supported: boolean
  agentKey?: string
  reason?: string
}

export interface SkillsNpxCapabilities {
  available: boolean
  version?: string
  path?: string
  packageManager: string
  supportedFlags: string[]
  supportedPlatforms: NpxPlatformSupport[]
}

export interface SkillInstallReviewSource {
  sourceKind: string
  sourceRef: string
  sourceSkillId?: string
  resolvedName: string
  resolvedDirName: string
  origin: SkillOrigin
  description?: string
}

export interface SkillInstallReviewTarget {
  platformId: Platform
  platformName: string
  detected: boolean
  targetPath: string
  sharedDirGroup?: string
  installStrategy?: SkillInstallStrategy
  directNpxSupported: boolean
  npxAgentKey?: string
}

export interface SkillInstallCommandPreview {
  kind: string
  label: string
  command: string
  platforms: Platform[]
}

export interface SkillInstallReviewResponse {
  source: SkillInstallReviewSource
  targets: SkillInstallReviewTarget[]
  warnings: string[]
  commandPreviews: SkillInstallCommandPreview[]
  npx?: SkillsNpxCapabilities
}
