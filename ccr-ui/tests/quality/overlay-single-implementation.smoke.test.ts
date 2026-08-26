import { readFile, readdir } from 'node:fs/promises'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

// 弹层四项行为（焦点陷阱、Esc 关闭、滚动锁定、层级）只有一处实现（AC5，
// 08-22-design-system 批次 4）。唯一实现点是 Radix（@radix-ui/react-dialog +
// react-remove-scroll），经 src/ui/dialog.tsx 封装、base-modal.tsx 适配。
// 本测试扫描 src/ui 源码，断言除委托调用外无第二处实现。

const UI_DIR = 'src/ui'
const STYLES_DIR = 'src/styles'

// 行为委托点：Radix 封装与适配器允许出现的关键字。
const DIALOG_FILE = join(UI_DIR, 'dialog.tsx')
const BASE_MODAL_FILE = join(UI_DIR, 'base-modal.tsx')

// 自实现特征：出现即视为在 Radix 之外重复实现某项弹层行为。
const FORBIDDEN_PATTERNS: { pattern: RegExp; behavior: string }[] = [
  { pattern: /document\.body\.style\.overflow|body\.style\.overflow/, behavior: '滚动锁定' },
  { pattern: /addEventListener\(\s*['"]keydown['"]/, behavior: 'Esc 关闭' },
  { pattern: /document\.activeElement/, behavior: '焦点陷阱' },
]

const stripTypeScriptComments = (source: string): string => {
  return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1')
}

const walkFiles = async (dir: string, test: (name: string) => boolean): Promise<string[]> => {
  const entries = await readdir(dir, { withFileTypes: true })
  const files: string[] = []
  for (const entry of entries) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await walkFiles(fullPath, test)))
    } else if (test(entry.name)) {
      files.push(fullPath)
    }
  }
  return files
}

const readSource = async (path: string): Promise<string> =>
  readFile(path, 'utf8').then(stripTypeScriptComments)

describe('overlay single implementation（08-22-design-system 批次 4 / AC5）', () => {
  it('src/ui 内除 Radix 委托外无滚动锁定 / Esc / 焦点陷阱的自实现', async () => {
    const files = await walkFiles(UI_DIR, (name) => /\.(ts|tsx)$/.test(name))
    expect(files.length).toBeGreaterThan(0)
    const violations: string[] = []
    for (const file of files) {
      const source = await readSource(file)
      for (const { pattern, behavior } of FORBIDDEN_PATTERNS) {
        if (pattern.test(source)) {
          violations.push(`${file} 含 ${behavior} 自实现（${pattern}）`)
        }
      }
    }
    expect(violations).toEqual([])
  })

  it('base-modal 适配器不含无效的 onPointerUpOutside（防回归）', async () => {
    const source = await readSource(BASE_MODAL_FILE)
    expect(source).not.toContain('onPointerUpOutside')
  })

  it('层级由 token 表达且 token 在 styles 层有定义', async () => {
    const dialog = await readSource(DIALOG_FILE)
    const baseModal = await readSource(BASE_MODAL_FILE)
    expect(dialog).toContain('z-[var(--layer-modal-backdrop)]')
    expect(baseModal).toContain('z-[var(--layer-modal)]')

    const cssFiles = await walkFiles(STYLES_DIR, (name) => name.endsWith('.css'))
    expect(cssFiles.length).toBeGreaterThan(0)
    const css = (
      await Promise.all(cssFiles.map((path) => readFile(path, 'utf8')))
    ).join('\n')
    expect(css).toMatch(/--layer-modal-backdrop\s*:/)
    expect(css).toMatch(/--layer-modal\s*:/)
  })
})
