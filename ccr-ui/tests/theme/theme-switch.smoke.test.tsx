import { readFile } from 'node:fs/promises'
import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

// 批次 1 验证用例（08-22-design-system design.md §1 两层结构 + implement.md 批次 1）：
// 「切换 data-theme 后工具类生效的颜色随之变化」。
//
// jsdom 的 getComputedStyle 不解析普通属性内的 var() 链，但会：
//   - 按 data-* 属性轴解析 :root 上的自定义属性字面量；
//   - 返回工具类属性的原始声明（可断言其引用运行时变量而非内联字面量）。
// 因此本测试分两段验证，合起来构成完整链条：
//   1) 第 2 层工具类规则引用第 1 层变量（非字面量）—— `.bg-bg-surface` 的
//      background-color 必须形如 rgb(var(--color-bg-surface-rgb))。
//   2) 第 1 层变量随 data-theme 切换变化—— :root 计算样式中的
//      --color-bg-surface-rgb 在 light/dark 下取值不同，且各自等于
//      tokens.css 中对应选择器的字面量锚点（防漂移）。
// 再由测试内 var() 替换算出每个主题下的实际色值并断言二者不同。

const TOKENS_PATH = 'src/styles/tokens.css'
const CORE_PATH = 'src/styles/core.css'

const trim = (value: string): string => value.trim()

/** 从 tokens.css 文本抽取指定选择器块内的变量值。 */
const extractVarValue = (css: string, selectorBlock: RegExp, varName: string): string | null => {
  const match = css.match(selectorBlock)
  if (!match) return null
  const varMatch = match[0].match(new RegExp(`${varName}:\\s*([^;]+);`))
  return varMatch ? trim(varMatch[1]) : null
}

// 顶层 :root 块（颜色系统）——匹配第一个 `:root {` 到下一个 `}`。
const ROOT_BLOCK = /:root\s*\{[\s\S]*?\n\}/
// [data-theme='dark'] 块。
const DARK_BLOCK = /\[data-theme='dark'\]\s*\{[\s\S]*?\n\}/
// [data-flavor='clay'] 块。
const CLAY_BLOCK = /\[data-flavor='clay'\]\s*\{[\s\S]*?\n\}/

const injectStyle = (css: string): HTMLStyleElement => {
  const style = document.createElement('style')
  style.textContent = css
  document.head.appendChild(style)
  return style
}

describe('theme-switch（批次 1 两层结构）', () => {
  it('工具类读取运行时变量，切换 data-theme 后计算色随之变化', async () => {
    const tokensCss = await readFile(TOKENS_PATH, 'utf8')
    const coreCss = await readFile(CORE_PATH, 'utf8')

    // --- 第 2 层：@theme inline 映射存在且指向第 1 层变量 ---
    expect(coreCss).toMatch(/--color-bg-surface:\s*rgb\(var\(--color-bg-surface-rgb\)\)/)

    // --- 第 1 层锚点值（防漂移）：tokens.css 内 light/dark/clay 字面量 ---
    const lightRgb = extractVarValue(tokensCss, ROOT_BLOCK, '--color-bg-surface-rgb')
    const darkRgb = extractVarValue(tokensCss, DARK_BLOCK, '--color-bg-surface-rgb')
    const clayRgb = extractVarValue(tokensCss, CLAY_BLOCK, '--color-bg-surface-rgb')

    expect(lightRgb).toBe('250 247 236')
    expect(darkRgb).toBe('31 27 20')
    expect(clayRgb).toBe('254 250 242')
    expect(darkRgb).not.toBe(lightRgb)
    expect(clayRgb).not.toBe(lightRgb)

    // --- 注入 tokens.css + 由 @theme inline 映射产出的工具类规则 ---
    // Tailwind v4 对 --color-bg-surface: rgb(var(--color-bg-surface-rgb)) 的 bg-* 工具类
    // 内联结果为 background-color: rgb(var(--color-bg-surface-rgb))（不产生新 :root 变量）。
    injectStyle(`${tokensCss}
.bg-bg-surface { background-color: rgb(var(--color-bg-surface-rgb)); }
`)

    render(
      <div data-testid="surface-el" className="bg-bg-surface">
        surface
      </div>,
    )

    const element = document.querySelector('[data-testid="surface-el"]')
    expect(element).not.toBeNull()
    const elementStyle = window.getComputedStyle(element as HTMLElement)

    // 工具类引用第 1 层变量而非内联字面量（jsdom 返回原始声明）。
    expect(elementStyle.backgroundColor).toBe('rgb(var(--color-bg-surface-rgb))')

    const readSurfaceRgb = (): string =>
      trim(window.getComputedStyle(document.documentElement).getPropertyValue('--color-bg-surface-rgb'))

    // 默认 light（neutral flavor）
    expect(readSurfaceRgb()).toBe(lightRgb)

    // 切换 data-theme → dark
    document.documentElement.setAttribute('data-theme', 'dark')
    expect(readSurfaceRgb()).toBe(darkRgb)

    // 切换 data-flavor → clay（独立于 theme 轴）
    document.documentElement.setAttribute('data-theme', 'light')
    document.documentElement.setAttribute('data-flavor', 'clay')
    expect(readSurfaceRgb()).toBe(clayRgb)

    // var() 替换后：各主题下工具类实际色值互不相同（完整链条成立）
    const resolve = (rgbTriplet: string | null): string =>
      rgbTriplet === null ? '' : `rgb(${rgbTriplet})`

    const colors = new Set(
      [lightRgb, darkRgb, clayRgb].map((triplet) => resolve(triplet)),
    )
    expect(colors.size).toBe(3)

    document.documentElement.removeAttribute('data-theme')
    document.documentElement.removeAttribute('data-flavor')
  })
})
