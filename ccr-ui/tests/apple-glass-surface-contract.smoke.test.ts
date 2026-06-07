import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

const migratedViewPaths = [
  '../src/components/MainLayout.vue',
  '../src/views/DashboardView.vue',
  '../src/views/UsageDashboardView.vue',
  '../src/views/CodexView.vue',
]

const forbiddenLegacyUtilities = /\btext-white(?:\/|\b)|\bbg-white\/|\bborder-white\//
const forbiddenLegacyBranding = /pink-|purple-|neko-|cyber-grid/
const forbiddenLegacyFontStacks =
  /JetBrains Mono|Fira Code|Maple Mono|Cascadia Code|SFMono-Regular|ui-monospace|Menlo|Monaco|Consolas|Liberation Mono|Courier New/
const mochaOverridePattern = /html:root\[data-resolved-flavor="mocha"\]\s*{[\s\S]*?^}/m

async function collectSourceFiles(root: string): Promise<string[]> {
  const entries = await readdir(root, { withFileTypes: true })
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(root, entry.name)

      if (entry.isDirectory()) {
        return collectSourceFiles(fullPath)
      }

      return /\.(?:vue|css|ts|html)$/.test(entry.name) ? [fullPath] : []
    })
  )

  return files.flat()
}

describe('claude editorial surface contract', () => {
  it.each(migratedViewPaths)('keeps %s free of legacy novelty branding', async (relativePath) => {
    const absolutePath = fileURLToPath(new URL(relativePath, import.meta.url))
    const source = await readFile(absolutePath, 'utf8')

    expect(source).not.toMatch(forbiddenLegacyUtilities)
    expect(source).not.toMatch(forbiddenLegacyBranding)
  })

  it('maps global font tokens to MapleBright so visible copy stays consistent', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).toMatch(/--font-sans:\s*'MapleBright'/)
    expect(source).toMatch(/--font-brand:\s*'MapleBright'/)
    expect(source).toMatch(/--font-mono:\s*'MapleBright'/)
    const mochaOverride = source.match(mochaOverridePattern)?.[0] ?? ''

    expect(mochaOverride).toContain("--font-brand: 'SF Pro Display'")
    expect(mochaOverride).toContain("--font-mono: 'Cascadia Code'")
    expect(mochaOverride).toContain('--color-bg-base: var(--ctp-crust)')
    expect(source).not.toMatch(/#0071E3|#2997FF/)
  })

  it('drops deferred neko decorations from the runtime decoration layer', async () => {
    const source = await readFile('src/styles/deferred-decorations.css', 'utf8')

    expect(source).not.toMatch(/neko-decorations/)
  })

  it('keeps source files free of legacy monospace font stacks now that MapleBright is global', async () => {
    const testFilePath = fileURLToPath(import.meta.url)
    const testDir = path.dirname(testFilePath)
    const sourceRoot = path.resolve(testDir, '../src')
    const projectRoot = path.resolve(testDir, '..')
    const sourceFiles = await collectSourceFiles(sourceRoot)
    const filesToCheck = [...sourceFiles, path.join(projectRoot, 'index.html')]

    await Promise.all(
      filesToCheck.map(async (filePath) => {
        const source = (await readFile(filePath, 'utf8')).replace(mochaOverridePattern, '')
        expect(source, filePath).not.toMatch(forbiddenLegacyFontStacks)
      })
    )
  })
})
