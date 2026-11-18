import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { describe, expect, it } from 'vitest'
import {
  collectAdvisories,
  validateAllowlist,
  validateAuditReport,
} from '../../scripts/audit-dependencies.mjs'
import auditPolicy from '../../scripts/frontend-audit-allowlist.json'

const requireFromUi = createRequire(import.meta.url)

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

  it('covers every locked picomatch version in the temporary exception', () => {
    const lockfile = readFileSync(requireFromUi.resolve('../../bun.lock'), 'utf8')
    const resolvedVersions = new Set(
      Array.from(lockfile.matchAll(/:\s*\["picomatch@([^"]+)"/g), ([, version]) => version),
    )
    const picomatchException = auditPolicy.exceptions.find(({ package: name }) => name === 'picomatch')

    expect(picomatchException).toBeDefined()
    expect(resolvedVersions).toEqual(new Set(picomatchException?.patchedVersions))
  })

  it.each(['micromatch', 'tinyglobby', 'vite'])('blocks repeating extglobs for %s', (consumer) => {
    const requireFromConsumer = createRequire(requireFromUi.resolve(consumer))
    const match = requireFromConsumer('picomatch') as (pattern: string) => (input: string) => boolean

    expect(() => match('+(draft|final).md')).toThrow('Repeating extglobs are disabled by CCR')
    expect(() => match('*(draft|final).md')).toThrow('Repeating extglobs are disabled by CCR')
    expect(match('@(draft|final).md')('final.md')).toBe(true)
    expect(match('**/*.ts')('src/main.ts')).toBe(true)
    expect(match('**/*.ts')('src/main.js')).toBe(false)
  })
})
