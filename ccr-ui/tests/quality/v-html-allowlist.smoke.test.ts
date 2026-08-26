import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

type HtmlInjection = {
  file: string
  expression: string
}

const normalizePath = (value: string) => value.replace(/\\/g, '/')

const SOURCE_FILE_RE = /\.(ts|tsx)$/

const collectSourceFiles = async (root: string): Promise<string[]> => {
  const entries = await readdir(root, { withFileTypes: true })
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(root, entry.name)

      if (entry.isDirectory()) {
        return collectSourceFiles(fullPath)
      }

      return entry.isFile() && SOURCE_FILE_RE.test(entry.name) ? [fullPath] : []
    }),
  )

  return files.flat()
}

const collectHtmlInjections = async (): Promise<HtmlInjection[]> => {
  const sourceRoot = path.resolve('src')
  const sourceFiles = await collectSourceFiles(sourceRoot)
  const usages: HtmlInjection[] = []

  await Promise.all(
    sourceFiles.map(async (filePath) => {
      const source = await readFile(filePath, 'utf8')
      const matches = source.matchAll(/dangerouslySetInnerHTML=\{\{\s*__html:\s*([^}]+?)\s*\}\}/g)

      for (const match of matches) {
        usages.push({
          file: normalizePath(path.relative(process.cwd(), filePath)),
          expression: match[1].trim(),
        })
      }
    }),
  )

  return usages.sort((left, right) =>
    `${left.file}:${left.expression}`.localeCompare(`${right.file}:${right.expression}`),
  )
}

describe('HTML injection safety allowlist', () => {
  it('keeps every dangerouslySetInnerHTML usage on an audited sanitizer or static i18n source', async () => {
    await expect(collectHtmlInjections()).resolves.toEqual([
      {
        file: 'src/features/commands/LedgerLine.tsx',
        expression: 'DOMPurify.sanitize(sanitized)',
      },
      {
        file: 'src/features/monitoring/MonitoringLogRow.tsx',
        expression: 'DOMPurify.sanitize(html)',
      },
      {
        file: 'src/features/usage/components/LlmusageInstallDialog.tsx',
        expression: 'descriptionHtml',
      },
    ])
  })

  it('keeps audited HTML sources behind explicit escaping or sanitization helpers', async () => {
    const [ansiRenderer, claudeProfiles, installDialog, commandsLedger, monitoringRow] = await Promise.all([
      readFile('src/utils/ansiRenderer.ts', 'utf8'),
      readFile('src/utils/claudeProfiles.ts', 'utf8'),
      readFile('src/features/usage/components/LlmusageInstallDialog.tsx', 'utf8'),
      readFile('src/features/commands/LedgerLine.tsx', 'utf8'),
      readFile('src/features/monitoring/MonitoringLogRow.tsx', 'utf8'),
    ])

    expect(ansiRenderer).toMatch(/sanitizeTerminal/)
    expect(claudeProfiles).toMatch(/escapeHtml/)
    expect(installDialog).toMatch(/只渲染 i18n 文案，无用户输入/)
    expect(commandsLedger).toMatch(/DOMPurify\.sanitize/)
    expect(monitoringRow).toMatch(/DOMPurify\.sanitize/)
  })
})
