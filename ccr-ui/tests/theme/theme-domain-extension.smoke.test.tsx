import { readFile, readdir } from 'node:fs/promises'
import { join } from 'node:path'
import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
  applyAccentToDocument,
  applyCustomAccent,
  applyFlavorToDocument,
  clearCustomAccent,
  CUSTOM_ACCENT_MODE,
  CUSTOM_ACCENT_VARIABLE_FAMILY,
  type AccentMode,
  type FlavorMode,
} from '@/utils/themeBootstrap'

// 主题配置域可扩展（08-22-design-system 批次 5 / design.md §10 / AC7）：
// 新增一个 flavor 或 accent 只需「类型联合 + FLAVOR_MODES/ACCENT_MODES 加一个
// 成员 + 第 1 层变量加一组定义」，不需要改任何组件。本测试用注入的测试值
// （ink-test flavor、sage-test accent、custom accent）模拟扩展后的形态并验证：
//   1) 属性轴切换后第 1 层变量重新解析（界面消费的颜色随之变化）；
//   2) applyFlavorToDocument / applyAccentToDocument 对新成员无成员级代码依赖；
//   3) 三个 data-* 属性的写入点在 src 内只有 themeBootstrap.ts（结构保证）；
//   4) applyCustomAccent 的变量族结构与明暗两套主题行为。
//
// 测试内对联合类型的 cast 是刻意的：模拟「联合已扩展、调用点照旧」的状态。

const injectStyle = (css: string): HTMLStyleElement => {
  const style = document.createElement('style')
  style.textContent = css
  document.head.appendChild(style)
  return style
}

const readRootVar = (name: string): string =>
  window.getComputedStyle(document.documentElement).getPropertyValue(name).trim()

describe('theme domain extension（08-22-design-system 批次 5 / AC7）', () => {
  it('新增 flavor：data-flavor 切到新值后第 1 层变量与工具类随之重解析', () => {
    // 模拟新增成员时的第 1 层定义块（真实新增落在 styles 层的 [data-flavor] 块）。
    const style = injectStyle(`:root { --color-bg-surface-rgb: 251 252 253; }
[data-flavor='ink-test'] { --color-bg-surface-rgb: 9 30 22; }
.bg-bg-surface { background-color: rgb(var(--color-bg-surface-rgb)); }`)

    const { container } = render(<div className="bg-bg-surface">surface</div>)

    try {
      applyFlavorToDocument('neutral')
      expect(readRootVar('--color-bg-surface-rgb')).toBe('251 252 253')

      applyFlavorToDocument('ink-test' as FlavorMode)
      expect(document.documentElement.getAttribute('data-flavor')).toBe('ink-test')
      // resolveFlavorMode 对新成员直通，无成员级映射代码。
      expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('ink-test')
      expect(readRootVar('--color-bg-surface-rgb')).toBe('9 30 22')

      // 工具类仍引用运行时变量（链条完整，界面消费的颜色随变量重解析）。
      const elementStyle = window.getComputedStyle(
        container.firstElementChild as HTMLElement,
      )
      expect(elementStyle.backgroundColor).toBe('rgb(var(--color-bg-surface-rgb))')
    } finally {
      style.remove()
      applyFlavorToDocument('neutral')
    }
  })

  it('新增 accent：data-accent 切到新值后第 1 层 accent 变量重新解析', () => {
    const style = injectStyle(`:root { --color-accent-primary: #cf6239; }
[data-accent='sage-test'] { --color-accent-primary: #2f7d5b; }`)

    try {
      applyAccentToDocument('clay')
      expect(readRootVar('--color-accent-primary')).toBe('#cf6239')

      applyAccentToDocument('sage-test' as AccentMode)
      expect(document.documentElement.getAttribute('data-accent')).toBe('sage-test')
      expect(readRootVar('--color-accent-primary')).toBe('#2f7d5b')
    } finally {
      style.remove()
      applyAccentToDocument('clay')
    }
  })

  it('data-theme / data-flavor / data-accent 的写入点在 src 内只有 themeBootstrap', async () => {
    const srcDir = 'src'
    const offenders: string[] = []
    const walk = async (dir: string): Promise<string[]> => {
      const entries = await readdir(dir, { withFileTypes: true })
      const files: string[] = []
      for (const entry of entries) {
        const fullPath = join(dir, entry.name)
        if (entry.isDirectory()) {
          files.push(...(await walk(fullPath)))
        } else if (/\.(ts|tsx)$/.test(entry.name)) {
          files.push(fullPath)
        }
      }
      return files
    }

    for (const file of await walk(srcDir)) {
      if (file === join('src', 'utils', 'themeBootstrap.ts')) continue
      const source = await readFile(file, 'utf8')
      if (/setAttribute\(\s*['"]data-(theme|flavor|accent)['"]/.test(source)) {
        offenders.push(file)
      }
    }
    // 值域扩展不需要改任何组件的结构性保证：写属性的唯一入口是 themeBootstrap。
    expect(offenders).toEqual([])
  })

  it(`applyCustomAccent：整族写入 ${CUSTOM_ACCENT_MODE} 变量并在明暗两套主题下生效`, () => {
    // 非法输入被拒绝且不改 DOM。
    expect(applyCustomAccent({ light: 'not-a-color' })).toBe(false)
    expect(document.getElementById('ccr-custom-accent')).toBeNull()
    expect(applyCustomAccent({ light: '#2f6fe' })).toBe(false)

    expect(applyCustomAccent({ light: '#2f6fed', dark: '#7aa5f8' })).toBe(true)
    expect(document.documentElement.getAttribute('data-accent')).toBe(CUSTOM_ACCENT_MODE)

    const styleText = document.getElementById('ccr-custom-accent')?.textContent ?? ''
    // 两个主题块各覆盖完整变量族（按声明形态计数，避免前缀串名重复计数）。
    for (const varName of CUSTOM_ACCENT_VARIABLE_FAMILY) {
      const occurrences = styleText.split(`${varName}:`).length - 1
      expect(occurrences, `${varName} 应出现在明暗两块中`).toBe(2)
    }

    try {
      document.documentElement.setAttribute('data-theme', 'light')
      expect(readRootVar('--color-accent-primary')).toBe('#2f6fed')
      expect(readRootVar('--color-accent-primary-rgb')).toBe('47 111 237')

      document.documentElement.setAttribute('data-theme', 'dark')
      expect(readRootVar('--color-accent-primary')).toBe('#7aa5f8')
    } finally {
      document.documentElement.removeAttribute('data-theme')
    }

    // 清除后恢复枚举 accent。
    expect(clearCustomAccent()).toBe('clay')
    expect(document.getElementById('ccr-custom-accent')).toBeNull()
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
  })
})
