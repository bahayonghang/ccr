/**
 * skills_ext 版本历史 / 回收站 / 启用禁用 类型定义
 * 对应 Rust `ccr_skills::skills_ext::{versioning, lcs, trash}` 并通过
 * `#[serde(rename_all = "camelCase")]` 保证字段名一致。
 */

export type SnapshotSource = 'auto' | 'manual'

export interface VersionMeta {
  id: string
  skillPath: string
  skillName: string
  timestamp: string // ISO 8601
  message: string
  source: SnapshotSource
  contentHash: string
}

export interface Version extends VersionMeta {
  content: string
  files: Record<string, string>
}

export type DiffLineKind = 'same' | 'add' | 'remove'

export interface DiffLine {
  kind: DiffLineKind
  oldLine?: number
  newLine?: number
  content: string
}

export interface DiffStats {
  additions: number
  deletions: number
  unchanged: number
}

/** P2-3: LCS 超行截断信息，UI 据此显示警告 banner */
export interface TruncationInfo {
  truncated: boolean
  totalOldLines: number
  totalNewLines: number
  limit: number
}

export interface DiffResult {
  oldVersion: VersionMeta
  newVersion: VersionMeta
  lines: DiffLine[]
  stats: DiffStats
  truncation: TruncationInfo
}

export interface TrashEntry {
  id: string
  skillName: string
  originalPath: string
  deletedAt: string // ISO 8601
  expiresAt: string // ISO 8601
}

// ============================================================================
// Phase 7 — taxonomy / conflicts / health
// ============================================================================

export type MatchSource = 'frontmatter' | 'keyword' | 'fallback'

export interface Classification {
  skillId: string
  categoryId: string
  matchedBy: MatchSource
}

export interface CategorySummary {
  id: string
  nameEn: string
  nameZh: string
  icon: string
  count: number
  skillIds: string[]
}

export interface SkillRef {
  id: string
  name: string
}

export interface MergeSuggestion {
  categoryId: string
  categoryName: string
  reason: string
  skills: [SkillRef, SkillRef]
  similarity: number
}

export interface ConflictGroup {
  name: string
  skillIds: string[]
  realPaths: string[]
}

export interface HealthReport {
  total: number
  conflicts: number
  mergeSuggestions: number
  disabled: number
  pluginLocations: number
}

export interface TaxonomyInput {
  id: string
  name: string
  description?: string
  frontmatterCategory?: string | null
  realPath?: string | null
}

export interface TaxonomyResponse {
  classifications: Classification[]
  categories: CategorySummary[]
  mergeSuggestions: MergeSuggestion[]
  conflicts: ConflictGroup[]
  health: HealthReport
}
