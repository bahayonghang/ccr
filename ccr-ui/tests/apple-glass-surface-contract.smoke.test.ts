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
// 受控例外（精确到块）：tokens.css “字体系统” :root 块是唯一允许出现真等宽字体栈字面量的位置。
const fontTrackBlockPattern =
  /\/\* ========== 字体系统 ========== \*\/\s*:root:where\(:root\):where\(:root\) {[\s\S]*?^}/m

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

  it('keeps sans on MapleBright and splits brand/mono into dedicated global tracks', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')
    const fontTrackBlock = source.match(fontTrackBlockPattern)?.[0] ?? ''

    // 字体三轨分离（R5）：正文保留 MapleBright，标题走比例显示字体，数值/代码走真等宽。
    expect(fontTrackBlock).toMatch(/--font-sans:\s*'MapleBright'/)
    expect(fontTrackBlock).toMatch(/--font-brand:\s*'SF Pro Display'/)
    expect(fontTrackBlock).toMatch(/--font-mono:\s*'Cascadia Code'/)

    // mocha 覆盖块的字体分离已上移为全局默认，不得再在覆盖块内重复声明。
    const mochaOverride = source.match(mochaOverridePattern)?.[0] ?? ''

    expect(mochaOverride).toContain('--color-bg-base: var(--ctp-crust)')
    expect(mochaOverride).not.toMatch(/--font-brand|--font-mono/)
    expect(source).not.toMatch(/#0071E3|#2997FF/)
  })

  it('locks the three-tier material glass tokens with budget note and reduced-transparency fallback', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    for (const tier of ['floating', 'chrome', 'inline']) {
      for (const part of ['bg', 'blur', 'border', 'highlight', 'shadow']) {
        expect(source, `--material-glass-${tier}-${part}`).toMatch(
          new RegExp(`--material-glass-${tier}-${part}:`)
        )
      }
    }

    // 语义重映射：modal→floating / shell→chrome / status→inline；card/workspace 不透明。
    expect(source).toMatch(/--surface-modal-bg:\s*var\(--material-glass-floating-bg\)/)
    expect(source).toMatch(/--surface-shell-bg:\s*var\(--material-glass-chrome-bg\)/)
    expect(source).toMatch(/--surface-status-bg:\s*var\(--material-glass-inline-bg\)/)
    expect(source).toMatch(/--surface-card-blur:\s*none/)
    expect(source).toMatch(/--surface-workspace-blur:\s*none/)

    // mocha 覆盖块必须以 html:root 高优先级选择器携带 material 覆盖（specificity 契约）。
    const mochaOverride = source.match(mochaOverridePattern)?.[0] ?? ''

    expect(mochaOverride).toMatch(/--material-glass-floating-bg:/)
    expect(mochaOverride).toMatch(/--material-glass-chrome-bg:/)

    // 玻璃预算注释存在（review 依据）。
    expect(source).toContain('同屏 backdrop-filter 元素 ≤ 3')
    expect(source).toContain('禁止嵌套玻璃')

    // reduced-transparency 下三档 material 全部回退为不透明并关闭模糊。
    const reducedBlocks =
      source.match(/@media \(prefers-reduced-transparency: reduce\) {[\s\S]*?^}/gm)?.join('\n') ??
      ''

    expect(reducedBlocks).toMatch(/--material-glass-floating-blur:\s*none/)
    expect(reducedBlocks).toMatch(/--material-glass-chrome-blur:\s*none/)
    expect(reducedBlocks).toMatch(/--material-glass-inline-blur:\s*none/)

    // mocha 主覆盖块 (0,2,1) 压得过查询内的 html:root (0,1,1)：
    // 降级查询内必须携带 mocha 同级作用域的 bg/blur 重置，否则 mocha 下玻璃残留半透明（PRD R3）。
    const reducedMochaReset =
      reducedBlocks.match(/html:root\[data-resolved-flavor=["']mocha["']\] {[\s\S]*?}/)?.[0] ?? ''

    expect(reducedMochaReset).toMatch(/--material-glass-floating-bg:\s*var\(--color-bg-elevated\)/)
    expect(reducedMochaReset).toMatch(/--material-glass-chrome-bg:\s*var\(--color-bg-elevated\)/)
    expect(reducedMochaReset).toMatch(/--material-glass-inline-bg:\s*var\(--color-bg-elevated\)/)
    expect(reducedMochaReset).toMatch(/--material-glass-floating-blur:\s*none/)
  })

  it('ships glass utility classes with paint containment and reduced-transparency fallback', async () => {
    const source = await readFile('src/styles/utilities.css', 'utf8')

    for (const cls of ['glass-floating', 'glass-chrome', 'glass-inline']) {
      expect(source, `.${cls}`).toMatch(new RegExp(`\\.${cls} {`))
    }
    expect(source).toContain('contain: paint')
    expect(source).toContain('-webkit-backdrop-filter')
    expect(source).toContain('玻璃预算')
    expect(source).toContain('禁止嵌套玻璃')

    const reduced =
      source.match(/@media \(prefers-reduced-transparency: reduce\) {[\s\S]*$/)?.[0] ?? ''

    expect(reduced).toMatch(/\.glass-floating[\s\S]*?backdrop-filter: none/)
  })

  it('drops deferred neko decorations from the runtime decoration layer', async () => {
    const source = await readFile('src/styles/deferred-decorations.css', 'utf8')

    expect(source).not.toMatch(/neko-decorations/)
  })

  it('keeps source files free of legacy monospace font stacks outside the global font-track block', async () => {
    const testFilePath = fileURLToPath(import.meta.url)
    const testDir = path.dirname(testFilePath)
    const sourceRoot = path.resolve(testDir, '../src')
    const projectRoot = path.resolve(testDir, '..')
    const sourceFiles = await collectSourceFiles(sourceRoot)
    const filesToCheck = [...sourceFiles, path.join(projectRoot, 'index.html')]

    await Promise.all(
      filesToCheck.map(async (filePath) => {
        // 受控例外仅剥离 tokens.css 的“字体系统” :root 块（--font-mono 真等宽栈的唯一驻地）。
        const source = (await readFile(filePath, 'utf8')).replace(fontTrackBlockPattern, '')
        expect(source, filePath).not.toMatch(forbiddenLegacyFontStacks)
      })
    )
  })
})
