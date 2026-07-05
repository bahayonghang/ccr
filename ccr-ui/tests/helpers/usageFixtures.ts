/**
 * Shared usage fixture builders for smoke tests.
 * V2 usage 类型切到 ts-rs 生成类型后字段全部必填，旧的"部分字段字面量"无法通过 type-check。
 * 这里为反复出现的 DTO 提供完整对象构造器：默认值取中性值（0 / null / [] / 'priced'），
 * 测试只在 overrides 里写自己关心的字段，断言语义不受默认值影响。
 */
import type {
  ModelStat,
  SessionIndexJobSnapshot,
  UsageArchiveDiagnostics,
  UsageFeatureCapability,
  UsageFreshnessProjection,
  UsageImportJobSnapshot,
  UsageReadinessProjection,
  UsageRecordV2,
  UsageSnapshotProjection,
  UsageSourceHealth,
} from '@/types/usage'

/**
 * 生成类型把 wire 上 skip_serializing_if 的 Option 字段表达为 `field?: T`（缺键，无 null）。
 * 但既有测试 fixture 在运行时用 null 驱动断言（如 `expect(...).toBeNull()`）。
 * 为不改任何断言语义，override 允许显式传 null，保持运行时值与旧 fixture 完全一致，
 * 仅在返回处收窄回生成类型。
 */
type NullableOverrides<T> = { [K in keyof T]?: T[K] | null }

export const makeModelStat = (overrides: Partial<ModelStat> = {}): ModelStat => ({
  model: 'claude-sonnet',
  request_count: 0,
  total_tokens: 0,
  total_cost: 0,
  input_tokens: 0,
  output_tokens: 0,
  cache_read_tokens: 0,
  cache_creation_tokens: 0,
  cost_with_cache: 0,
  cost_without_cache: 0,
  cache_savings: 0,
  pricing_status: 'priced',
  pricing_source: null,
  pricing_rate: null,
  ...overrides,
})

export const makeFreshness = (
  overrides: Partial<UsageFreshnessProjection> = {},
): UsageFreshnessProjection => ({
  state: 'fresh',
  latest_completed_at: null,
  age_seconds: null,
  stale_after_seconds: 86_400,
  ...overrides,
})

export const makeReadiness = (
  overrides: Partial<UsageReadinessProjection> = {},
): UsageReadinessProjection => ({
  state: 'ready',
  next_action: null,
  detail: '',
  has_live_sources: true,
  has_missing_sources: false,
  has_deleted_sources: false,
  active_usage_import: false,
  active_session_index: false,
  recent_completed_at: null,
  ...overrides,
})

export const makeSourceHealth = (
  overrides: Partial<UsageSourceHealth> = {},
): UsageSourceHealth => ({
  source: 'claude',
  state: 'live',
  live_sources: 0,
  missing_sources: 0,
  deleted_sources: 0,
  recent_completed_at: null,
  history_completed_at: null,
  freshness: makeFreshness(),
  ...overrides,
})

export const makeArchiveDiagnostics = (
  overrides: Partial<UsageArchiveDiagnostics> = {},
): UsageArchiveDiagnostics => ({
  archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
  live_sources: 0,
  missing_sources: 0,
  deleted_sources: 0,
  archived_sessions: 0,
  recent_completed_at: null,
  history_completed_at: null,
  source_health: [],
  freshness: makeFreshness(),
  readiness: makeReadiness(),
  ...overrides,
})

export const makeSnapshotProjection = (
  overrides: Partial<UsageSnapshotProjection> = {},
): UsageSnapshotProjection => ({
  generated_at: '2026-01-01T00:00:00Z',
  platform_scope: 'all',
  start_date: null,
  end_date: null,
  cache_ttl_seconds: 30,
  freshness: makeFreshness(),
  readiness: makeReadiness(),
  source_health: [],
  drilldown: {
    dimensions: [],
    supports_logs: true,
    supports_projects: true,
    supports_sessions: true,
  },
  ...overrides,
})

export const makeFeatureCapability = (
  overrides: Partial<UsageFeatureCapability> = {},
): UsageFeatureCapability => ({
  supported: true,
  reason: null,
  detail: null,
  ...overrides,
})

export const makeUsageRecord = (overrides: Partial<UsageRecordV2> = {}): UsageRecordV2 => ({
  id: 'record-1',
  platform: 'codex',
  project_path: 'D:/repo',
  record_json: '{}',
  recorded_at: '2026-01-01T00:00:00Z',
  source_id: 'source-1',
  model: null,
  input_tokens: 0,
  output_tokens: 0,
  cache_read_tokens: 0,
  cache_creation_tokens: 0,
  cost_with_cache_usd: 0,
  cost_without_cache_usd: 0,
  pricing_status: 'priced',
  pricing_source: null,
  ...overrides,
})

export const makeUsageImportJobSnapshot = (
  overrides: NullableOverrides<UsageImportJobSnapshot> = {},
): UsageImportJobSnapshot =>
  ({
    job_id: 'usage-import-job',
    status: 'running',
    stage: 'importing_recent',
    platform_scope: 'all',
    recent_window_days: 30,
    files_total: 0,
    files_scanned: 0,
    files_imported: 0,
    records_imported: 0,
    records_skipped: 0,
    history_cursor_hit: false,
    live_sources: 0,
    missing_sources: 0,
    deleted_sources: 0,
    started_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    warnings: [],
    results: [],
    ...overrides,
  }) as UsageImportJobSnapshot

export const makeSessionIndexJobSnapshot = (
  overrides: NullableOverrides<SessionIndexJobSnapshot> = {},
): SessionIndexJobSnapshot =>
  ({
    job_id: 'session-index-job',
    status: 'running',
    stage: 'indexing',
    platforms_total: 0,
    platforms_completed: 0,
    files_total: 0,
    files_scanned: 0,
    sessions_added: 0,
    sessions_updated: 0,
    errors: 0,
    started_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    warnings: [],
    ...overrides,
  }) as SessionIndexJobSnapshot
