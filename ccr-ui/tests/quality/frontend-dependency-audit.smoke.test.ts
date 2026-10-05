import { describe, expect, it } from 'vitest'
import {
  collectAdvisories,
  validateAllowlist,
  validateAuditReport,
} from '../../scripts/audit-dependencies.mjs'

const exception = {
  id: 'GHSA-mh99-v99m-4gvg',
  package: 'brace-expansion',
  owner: 'frontend-platform',
  rationale: 'Legacy majors are locally patched to a compatible safe implementation.',
  expires: '2026-08-31',
  patchedVersions: ['1.1.16', '2.1.2'],
}

const report = {
  'brace-expansion': [
    {
      url: 'https://github.com/advisories/GHSA-mh99-v99m-4gvg',
      severity: 'high',
    },
  ],
}

describe('frontend dependency audit policy', () => {
  it('accepts only a complete, non-expired bounded exception', () => {
    expect(
      validateAllowlist(
        { maxActiveExceptions: 1, exceptions: [exception] },
        new Date('2026-07-27T00:00:00Z'),
      ),
    ).toEqual([])
  })

  it('rejects expired, excess, unexpected, and stale exceptions', () => {
    const expired = { ...exception, expires: '2026-07-26' }
    expect(
      validateAllowlist(
        { maxActiveExceptions: 0, exceptions: [expired] },
        new Date('2026-07-27T00:00:00Z'),
      ),
    ).toEqual(expect.arrayContaining([
      expect.stringContaining('exceed limit'),
      expect.stringContaining('expired'),
    ]))

    expect(validateAuditReport({ other: report['brace-expansion'] }, { exceptions: [exception] }))
      .toEqual(expect.arrayContaining([
        expect.stringContaining('package other does not match'),
      ]))
    expect(validateAuditReport({}, { exceptions: [exception] }))
      .toEqual([expect.stringContaining('stale exception')])
  })

  it('extracts the GHSA identifier from structured Bun audit output', () => {
    expect(collectAdvisories(report)).toEqual([
      expect.objectContaining({
        id: 'GHSA-mh99-v99m-4gvg',
        package: 'brace-expansion',
        severity: 'high',
      }),
    ])
  })
})
