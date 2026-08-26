import { describe, expect, it } from 'vitest'
import type {
  ImportAllUsageResponse,
  UsageImportResult,
  UsageImportJobSnapshot,
} from '@/types/usage'
import {
  buildImportSummary,
  normalizeImportResponse,
  normalizeUserVisibleImportJob,
  toUserVisibleImportResult,
} from '@/utils/usageImportNormalization'

const result = (overrides: Partial<UsageImportResult> = {}): UsageImportResult => ({
  platform: 'codex',
  files_processed: 1,
  records_imported: 1,
  records_skipped: 0,
  duration_ms: 10,
  completed: true,
  error: null,
  is_optional_absent: false,
  ...overrides,
})

describe('usage import normalization helpers', () => {
  it('turns typed optional-absent results into user-visible non-errors', () => {
    expect(
      toUserVisibleImportResult(
        result({
          platform: 'opencode',
          files_processed: 0,
          records_imported: 0,
          completed: false,
          error: 'missing optional db',
          is_optional_absent: true,
        })
      )
    ).toMatchObject({
      platform: 'opencode',
      completed: true,
      error: null,
    })
  })

  it('rebuilds import summaries after normalizing visible results', () => {
    const response: ImportAllUsageResponse = {
      results: [
        result({ records_imported: 4, files_processed: 2 }),
        result({
          platform: 'opencode',
          completed: false,
          error: 'missing',
          is_optional_absent: true,
          records_imported: 0,
          files_processed: 0,
        }),
      ],
      summary: {
        success_count: 0,
        failure_count: 2,
        imported_records: 0,
        processed_files: 0,
        has_partial: true,
      },
    }

    expect(normalizeImportResponse(response).summary).toEqual({
      success_count: 2,
      failure_count: 0,
      imported_records: 4,
      processed_files: 2,
      has_partial: false,
    })
  })

  it('normalizes single-platform responses with platform override and partial detection', () => {
    expect(
      normalizeImportResponse(
        result({
          platform: 'unknown',
          files_processed: 2,
          records_imported: 0,
          records_skipped: 2,
        }),
        'antigravity'
      )
    ).toEqual({
      results: [expect.objectContaining({ platform: 'antigravity' })],
      summary: {
        success_count: 1,
        failure_count: 0,
        imported_records: 0,
        processed_files: 2,
        has_partial: true,
      },
    })
  })

  it('normalizes active import job snapshots without losing job metadata', () => {
    const job: UsageImportJobSnapshot = {
      job_id: 'job-1',
      status: 'running',
      stage: 'importing_recent',
      platform_scope: 'all',
      recent_window_days: 14,
      files_total: 0,
      files_scanned: 0,
      files_imported: 0,
      records_imported: 0,
      records_skipped: 0,
      history_cursor_hit: false,
      live_sources: 0,
      missing_sources: 0,
      deleted_sources: 0,
      started_at: '2026-05-19T00:00:00Z',
      updated_at: '2026-05-19T00:00:01Z',
      warnings: [],
      results: [result({ completed: false, error: 'missing', is_optional_absent: true })],
    }

    expect(normalizeUserVisibleImportJob(job)).toMatchObject({
      job_id: 'job-1',
      results: [expect.objectContaining({ completed: true, error: null })],
    })
  })

  it('keeps direct summary behavior available for store job snapshots', () => {
    expect(
      buildImportSummary([
        result({ records_imported: 3, files_processed: 1 }),
        result({ completed: false, error: 'boom', records_imported: 0, files_processed: 1 }),
      ])
    ).toEqual({
      success_count: 1,
      failure_count: 1,
      imported_records: 3,
      processed_files: 2,
      has_partial: true,
    })
  })
})
