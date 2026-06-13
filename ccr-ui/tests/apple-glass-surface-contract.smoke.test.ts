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

// 已完成设计语言迁移并锁定的表面（随 WS6 批次扩充，防止回退）。
// 批次①（签到主题迁移 = WS1.2/1.3）：锁定 Tailwind 调色板（工具类 + raw rgb 三元组）。
// 注意：圆角收敛属于 WS6 批次④（全局），本批不锁 border-radius 字面量；
//      backdrop-filter blur / hex / `.dark ` 后代选择器由 stylelint overrides 精确锁定。
const styleLockedPaths = [
  '../src/styles/checkin-shared.css',
  '../src/views/CheckinView.vue',
  '../src/views/checkin/CheckinAccountDashboardView.vue',
  '../src/views/checkin/tabs/CheckinAccountsTab.vue',
  '../src/views/checkin/tabs/CheckinImportExportTab.vue',
  '../src/views/checkin/tabs/CheckinProvidersTab.vue',
  '../src/views/checkin/tabs/CheckinRecordsTab.vue',
  '../src/views/checkin/components/AccountActionsMenu.vue',
  '../src/views/checkin/components/AccountDashboardCalendar.vue',
  '../src/views/checkin/components/AccountDashboardTrend.vue',
  '../src/views/checkin/components/AccountFormModal.vue',
  '../src/views/checkin/components/AccountsTable.vue',
  '../src/views/checkin/components/OAuthWizardModal.vue',
  '../src/components/CheckinProgressModal.vue',
  '../src/components/MainLayout.vue',
  // WS6 批次④：收口到 BaseModal 的表单弹窗（扁平语言已锁定）。
  '../src/components/AddConfigModal.vue',
  '../src/components/EditConfigModal.vue',
  '../src/components/CommandFormModal.vue',
]

const forbiddenLegacyUtilities = /\btext-white(?:\/|\b)|\bbg-white\/|\bborder-white\//
const forbiddenLegacyBranding = /pink-|purple-|neko-|cyber-grid/
// Tailwind 默认调色板工具类（带数字档位），语义 token 工具类（surface/primary/accent…）不在其列。
const forbiddenPaletteUtilities =
  /\b(?:bg|text|border|ring|from|to|via|fill|stroke|divide|outline|decoration|caret|accent)-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|fuchsia|rose)-(?:50|100|200|300|400|500|600|700|800|900|950)\b/
// raw rgb/rgba 三元组（纯黑阴影/遮罩除外）；token 形式 rgb(var(--x-rgb) / α) 不匹配。
const forbiddenRawRgbPalette = /rgba?\(\s*(?!0[\s,]+0[\s,]+0\b)\d{1,3}[\s,]/
// 旧玻璃语言别名（tailwind.config 中已 @deprecated）；已锁定表面禁止新增使用。
const forbiddenGlassAliases =
  /\b(?:glass-effect(?:-strong)?|glass-surface|glass-elevated|glass-modal|liquid-glass)\b/
const forbiddenLegacyFontStacks =
  /JetBrains Mono|Fira Code|Maple Mono|Cascadia Code|SFMono-Regular|ui-monospace|Menlo|Monaco|Consolas|Liberation Mono|Courier New/
const mochaOverridePattern = /html:root\[data-resolved-flavor=["']mocha["']\]\s*{[\s\S]*?^}/m

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

  it.each(styleLockedPaths)(
    'keeps locked surface %s on semantic palette tokens',
    async (relativePath) => {
      const absolutePath = fileURLToPath(new URL(relativePath, import.meta.url))
      const source = await readFile(absolutePath, 'utf8')

      expect(source).not.toMatch(forbiddenLegacyUtilities)
      expect(source).not.toMatch(forbiddenLegacyBranding)
      expect(source).not.toMatch(forbiddenPaletteUtilities)
      expect(source).not.toMatch(forbiddenRawRgbPalette)
      expect(source).not.toMatch(forbiddenGlassAliases)
    }
  )

  it('maps global font tokens to MapleBright so visible copy stays consistent', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).toMatch(/--font-sans:\s*'MapleBright'/)
    expect(source).toMatch(/--font-brand:\s*'MapleBright'/)
    expect(source).toMatch(/--font-mono:\s*'MapleBright'/)
    const mochaOverride = source.match(mochaOverridePattern)?.[0] ?? ''

    expect(mochaOverride).toMatch(/--font-brand:\s*'SF Pro Display'/)
    expect(mochaOverride).toMatch(/--font-mono:\s*'Cascadia Code'/)
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
