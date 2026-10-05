// @vitest-environment node

import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const source = readFileSync(path.join(root, 'scripts/perf/soak-persist.mjs'), 'utf8')

describe('AC13 persist soak harness', () => {
  it('keeps a single raw CDP session and GCs before metrics', () => {
    expect(source).not.toMatch(/from ['"]playwright/)
    expect(source).not.toMatch(/chromium\.connectOverCDP/)
    expect(source).toContain('HeapProfiler.collectGarbage')
    expect(source).toContain('msedgewebview2')
    expect(source).toContain('spa-click-or-rr-push')
    expect(source).toContain('idx')
  })
})
