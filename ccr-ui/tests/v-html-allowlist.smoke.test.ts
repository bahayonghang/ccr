import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

type VHtmlUsage = {
  file: string
  binding: string
}

const normalizePath = (value: string) => value.replace(/\\/g, '/')

const collectVueFiles = async (root: string): Promise<string[]> => {
  const entries = await readdir(root, { withFileTypes: true })
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(root, entry.name)

      if (entry.isDirectory()) {
        return collectVueFiles(fullPath)
      }

      return entry.isFile() && entry.name.endsWith('.vue') ? [fullPath] : []
    })
  )

  return files.flat()
}

const collectVHtmlUsages = async (): Promise<VHtmlUsage[]> => {
  const sourceRoot = path.resolve('src')
  const vueFiles = await collectVueFiles(sourceRoot)
  const usages: VHtmlUsage[] = []

  await Promise.all(
    vueFiles.map(async (filePath) => {
      const source = await readFile(filePath, 'utf8')
      const matches = source.matchAll(/\bv-html\s*=\s*"([^"]+)"/g)

      for (const match of matches) {
        usages.push({
          file: normalizePath(path.relative(process.cwd(), filePath)),
          binding: match[1],
        })
      }
    })
  )

  return usages.sort((left, right) =>
    `${left.file}:${left.binding}`.localeCompare(`${right.file}:${right.binding}`)
  )
}

describe('v-html safety allowlist', () => {
  it('keeps every v-html usage tied to an audited sanitizer, escaped helper, or static i18n source', async () => {
    await expect(collectVHtmlUsages()).resolves.toEqual([])
  })

  it('keeps audited v-html sources behind explicit escaping or sanitization helpers', async () => {
    const [ansiRenderer, claudeProfiles, installDialog, commandsLedger] = await Promise.all([
      readFile('src/utils/ansiRenderer.ts', 'utf8'),
      readFile('src/utils/claudeProfiles.ts', 'utf8'),
      readFile('src/features/usage/components/LlmusageInstallDialog.tsx', 'utf8'),
      readFile('src/features/commands/LedgerLine.tsx', 'utf8'),
    ])

    expect(ansiRenderer).toMatch(/sanitizeTerminal/)
    expect(claudeProfiles).toMatch(/escapeHtml/)
    expect(installDialog).toMatch(/只渲染 i18n 文案，无用户输入/)
    expect(commandsLedger).toMatch(/DOMPurify\.sanitize/)
  })
})
