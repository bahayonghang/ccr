/**
 * Unified Skills Management Types
 * 统一 Skills 管理类型定义
 */

// Platform identifiers
export type Platform = 'claude-code' | 'codex' | 'gemini' | 'qwen' | 'iflow' | 'droid'

// Source type for filtering
export type SkillSource = 'all' | 'user' | 'plugin' | 'remote'

// Platform information
export interface PlatformInfo {
  id: Platform
  displayName: string
  globalSkillsDir: string
  detected: boolean
  installedCount: number
  icon: string
  color: string
}

// Platform configuration with colors and icons
export const PLATFORM_CONFIG: Record<Platform, { displayName: string; icon: string; color: string; tailwindColor: string }> = {
  'claude-code': {
    displayName: 'Claude Code',
    icon: 'Code2',
    color: '#A78BFA',
    tailwindColor: 'purple-400'
  },
  'codex': {
    displayName: 'Codex',
    icon: 'Settings',
    color: '#34D399',
    tailwindColor: 'emerald-400'
  },
  'gemini': {
    displayName: 'Gemini CLI',
    icon: 'Sparkles',
    color: '#38BDF8',
    tailwindColor: 'sky-400'
  },
  'qwen': {
    displayName: 'Qwen',
    icon: 'Zap',
    color: '#22D3EE',
    tailwindColor: 'cyan-400'
  },
  'iflow': {
    displayName: 'iFlow',
    icon: 'Activity',
    color: '#FBBF24',
    tailwindColor: 'amber-400'
  },
  'droid': {
    displayName: 'Droid',
    icon: 'Bot',
    color: '#F472B6',
    tailwindColor: 'pink-400'
  }
}

// Skill metadata from SKILL.md frontmatter
export interface SkillMetadata {
  author?: string
  version?: string
  license?: string
  category?: string
  tags?: string[]
  updatedAt?: string
}

// Unified skill with platform information
export interface UnifiedSkill {
  name: string
  description?: string
  skillDir: string
  platform: Platform
  platformName: string
  category?: string
  tags: string[]
  // Extended fields for UI
  isRemote?: boolean
  repository?: string
  // Metadata fields (from backend SkillInstallMeta + frontmatter)
  version?: string
  author?: string
  source?: string        // "marketplace" | "github" | "local"
  sourceUrl?: string
  installDate?: number   // Unix timestamp (ms)
  commitHash?: string
}

// Backend response for unified skills
export interface UnifiedSkillsResponse {
  skills: UnifiedSkill[]
  total: number
  platforms: PlatformSummary[]
}

// Platform summary from backend
export interface PlatformSummary {
  id: Platform
  display_name: string
  global_skills_dir: string
  detected: boolean
  installed_count: number
}

// Marketplace item from skills.sh
export interface MarketplaceItem {
  package: string
  owner: string
  repo: string
  skill?: string
  skillsShUrl: string
  // 新增（从 skills.sh HTML 解析）
  description?: string
  authorAvatar?: string    // 推导自 https://avatars.githubusercontent.com/{owner}
  stars?: number
}

// Marketplace response
export interface MarketplaceResponse {
  items: MarketplaceItem[]
  total: number
  cached: boolean
}

// Filter state
export interface SkillFilters {
  search: string
  source: SkillSource
  category: string | null
  tags: string[]
  platform: Platform | 'all'
}

// Install request
export interface InstallRequest {
  package: string
  agents: string[]
  force?: boolean
}

// Remove request
export interface RemoveRequest {
  skill: string
  agents: string[]
}

// Operation result for a single agent
export interface AgentOperationResult {
  agent: string
  ok: boolean
  message?: string
}

// Operation response
export interface OperationResponse {
  results: AgentOperationResult[]
}

// Content tab type
export type ContentTab = 'installed' | 'marketplace' | 'repositories'

// Stats for display
export interface SkillsStats {
  installed: number
  available: number
  activePlatforms: number
  totalPlatforms: number
}

// Skill content response from backend (full SKILL.md content)
export interface SkillContent {
  name: string
  description?: string
  category?: string
  tags: string[]
  content: string    // Markdown body (after frontmatter)
  raw: string        // Full raw SKILL.md content (for editing)
  skillDir: string
}

// === 多源安装相关类型 ===

// 安装来源类型
export type ImportSource = 'marketplace' | 'github' | 'local' | 'npx'

// GitHub URL 导入请求
export interface ImportGithubRequest {
  url: string
  agents: string[]
  force?: boolean
}

// 本地文件夹导入请求
export interface ImportLocalRequest {
  sourcePath: string
  agents: string[]
  skillName?: string
}

// npx skills 安装请求
export interface NpxInstallRequest {
  package: string
  agents: string[]
  global?: boolean
}

// 批量安装请求
export interface BatchInstallRequest {
  packages: string[]
  agents: string[]
  force?: boolean
}

// 批量安装单项结果
export interface BatchItemResult {
  package: string
  ok: boolean
  message?: string
}

// 批量安装响应
export interface BatchInstallResponse {
  total: number
  successCount: number
  failCount: number
  results: BatchItemResult[]
}

// npx 可用性状态
export interface NpxStatus {
  available: boolean
  version?: string
  path?: string
}

// npx 安装响应
export interface NpxInstallResponse {
  success: boolean
  method: string  // "npx" | "github_fallback"
  stdout?: string
  stderr?: string
  results: AgentOperationResult[]
}

// 安装进度状态
export type InstallPhase = 'idle' | 'downloading' | 'installing' | 'done' | 'error'

export interface InstallProgress {
  phase: InstallPhase
  package: string
  message?: string
  startedAt: number
}
