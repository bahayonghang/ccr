import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

const migratedViewPaths = [
  '../src/components/MainLayout.vue',
  '../src/views/HomeView.vue',
  '../src/views/UsageDashboardView.vue',
  '../src/views/CodexView.vue',
]

const forbiddenLegacyUtilities = /\btext-white(?:\/|\b)|\bbg-white\/|\bborder-white\//
const forbiddenLegacyBranding = /pink-|purple-|neko-|cyber-grid/

describe('apple glass surface contract', () => {
  it.each(migratedViewPaths)('keeps %s free of legacy neon and novelty branding', async (relativePath) => {
    const absolutePath = fileURLToPath(new URL(relativePath, import.meta.url))
    const source = await readFile(absolutePath, 'utf8')

    expect(source).not.toMatch(forbiddenLegacyUtilities)
    expect(source).not.toMatch(forbiddenLegacyBranding)
  })

  it('maps the brand font token back to MapleBright for home and shell headings', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).toMatch(/--font-brand:\s*'MapleBright'/)
  })
})
