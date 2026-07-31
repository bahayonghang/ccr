import { readFile } from 'node:fs/promises'

import { beforeAll, describe, expect, it } from 'vitest'

/**
 * 配色对比度守卫（配色系统重构 design.md §7 静态解析方案，不依赖 jsdom 的 CSS var 支持）。
 *
 * 方案：读 tokens.css 文本 → 按已知选择器清单抽取定义块 → 微型级联解析
 * （specificity + 源序；var() 递归；hex/rgb 归一）→ 对 6 组 theme × resolved-flavor
 * 组合（light/dark × neutral/clay + light-latte + dark-mocha）计算 WCAG 相对亮度对比度。
 *
 * 阈值即契约：primary ≥12:1、secondary ≥7:1、muted ≥4.5:1（对 bg-surface）；
 * accent vs accent-contrast ≥3.5:1；border-default 混合后对 bg-surface ≥1.2:1。
 * 不达标时只允许在 tokens.css 侧微调锚点值，禁止降低本文件阈值。
 */

interface CssBlock {
  selectors: string[]
  declarations: Map<string, string>
  order: number
}

interface ThemeCombo {
  name: string
  theme: 'light' | 'dark'
  flavorAttr: 'neutral' | 'clay' | 'catppuccin'
  resolved: 'neutral' | 'clay' | 'latte' | 'mocha'
}

interface Rgba {
  r: number
  g: number
  b: number
  a: number
}

type AccentMode = 'clay' | 'sage' | 'sky' | 'mauve'
type Specificity = readonly [number, number, number]

const ACCENTS: readonly AccentMode[] = ['clay', 'sage', 'sky', 'mauve']

// 6 组有效组合：latte 只存在于 light、mocha 只存在于 dark（resolveFlavorMode 保证）。
const COMBOS: readonly ThemeCombo[] = [
  { name: 'light + neutral', theme: 'light', flavorAttr: 'neutral', resolved: 'neutral' },
  { name: 'dark + neutral', theme: 'dark', flavorAttr: 'neutral', resolved: 'neutral' },
  { name: 'light + clay', theme: 'light', flavorAttr: 'clay', resolved: 'clay' },
  { name: 'dark + clay', theme: 'dark', flavorAttr: 'clay', resolved: 'clay' },
  { name: 'light + latte', theme: 'light', flavorAttr: 'catppuccin', resolved: 'latte' },
  { name: 'dark + mocha', theme: 'dark', flavorAttr: 'catppuccin', resolved: 'mocha' },
]

// 已知选择器清单：root 伪类与 :where 链（历史 specificity 技巧）、html:root 前缀、data-* 属性轴，
// 段序任意（如 [data-theme='dark']:where([data-theme='dark']) 也合法）。
// 不在此清单内的选择器不参与本测试的级联（tokens.css 顶层实际只有这些形态）。
const KNOWN_SELECTOR_PATTERN =
  /^(?:html)?(?:(?::root)|(?::where\([^()]*\))|(?:\[data-(?:theme|flavor|resolved-flavor|accent)=['"][\w-]+['"]\]))*$/

const stripComments = (source: string): string => source.replace(/\/\*[\s\S]*?\*\//g, '')

const stripWhere = (selector: string): string => {
  let result = selector
  while (result.includes(':where(')) {
    result = result.replace(/:where\([^()]*\)/, '')
  }
  return result
}

/** 抽取顶层规则块（跳过 @media 等嵌套规则：reduced-transparency 由专项 smoke 锁定）。 */
const parseBlocks = (source: string): CssBlock[] => {
  const clean = stripComments(source)
  const blocks: CssBlock[] = []
  let index = 0
  let order = 0

  while (index < clean.length) {
    const openBrace = clean.indexOf('{', index)
    if (openBrace === -1) break

    const selectorText = clean.slice(index, openBrace).trim()

    let depth = 0
    let closeBrace = clean.length - 1
    for (let i = openBrace; i < clean.length; i += 1) {
      if (clean[i] === '{') depth += 1
      if (clean[i] === '}') {
        depth -= 1
        if (depth === 0) {
          closeBrace = i
          break
        }
      }
    }

    const body = clean.slice(openBrace + 1, closeBrace)
    index = closeBrace + 1

    if (selectorText.startsWith('@')) continue

    const declarations = new Map<string, string>()
    for (const piece of body.split(';')) {
      const match = piece.match(/^\s*(--[\w-]+)\s*:\s*([\s\S]+?)\s*$/)
      if (match) {
        declarations.set(match[1], match[2])
      }
    }

    const selectors = selectorText
      .split(',')
      .map((selector) => selector.trim())
      .filter((selector) => selector.length > 0 && KNOWN_SELECTOR_PATTERN.test(selector))

    if (selectors.length > 0 && declarations.size > 0) {
      blocks.push({ selectors, declarations, order })
      order += 1
    }
  }

  return blocks
}

const selectorSpecificity = (selector: string): Specificity => {
  let rest = stripWhere(selector)
  const attrCount = (rest.match(/\[[^\]]*\]/g) ?? []).length
  rest = rest.replace(/\[[^\]]*\]/g, ' ')
  const pseudoCount = (rest.match(/::?[a-zA-Z-]+/g) ?? []).length
  rest = rest.replace(/::?[a-zA-Z-]+/g, ' ')
  const elementCount = rest
    .split(/[\s>+~]+/)
    .filter((token) => /^[a-zA-Z][\w-]*$/.test(token)).length
  return [0, attrCount + pseudoCount, elementCount]
}

const compareSpecificity = (a: Specificity, b: Specificity): number => {
  if (a[0] !== b[0]) return a[0] - b[0]
  if (a[1] !== b[1]) return a[1] - b[1]
  return a[2] - b[2]
}

const attrValue = (selector: string, attr: string): string | null => {
  const match = selector.match(new RegExp(`\\[${attr}=['"]([\\w-]+)['"]\\]`))
  return match ? match[1] : null
}

const matchesCombo = (selector: string, combo: ThemeCombo, accent: AccentMode): boolean => {
  const normalized = stripWhere(selector)
  const theme = attrValue(normalized, 'data-theme')
  if (theme !== null && theme !== combo.theme) return false
  const flavor = attrValue(normalized, 'data-flavor')
  if (flavor !== null && flavor !== combo.flavorAttr) return false
  const resolved = attrValue(normalized, 'data-resolved-flavor')
  if (resolved !== null && resolved !== combo.resolved) return false
  const accentAttr = attrValue(normalized, 'data-accent')
  if (accentAttr !== null && accentAttr !== accent) return false
  return true
}

/** 按（specificity, 源序）升序应用匹配块，后者覆盖前者，得到组合下的最终令牌表。 */
const resolveTokens = (
  blocks: readonly CssBlock[],
  combo: ThemeCombo,
  accent: AccentMode
): Map<string, string> => {
  const applicable = blocks
    .flatMap((block) =>
      block.selectors
        .filter((selector) => matchesCombo(selector, combo, accent))
        .map((selector) => ({ block, specificity: selectorSpecificity(selector) }))
    )
    .sort(
      (a, b) => compareSpecificity(a.specificity, b.specificity) || a.block.order - b.block.order
    )

  const tokens = new Map<string, string>()
  for (const { block } of applicable) {
    for (const [name, value] of block.declarations) {
      tokens.set(name, value)
    }
  }
  return tokens
}

const VAR_PATTERN = /var\(\s*(--[\w-]+)\s*(?:,\s*([^)]*))?\)/

const resolveRawValue = (
  tokens: Map<string, string>,
  name: string,
  seen: ReadonlySet<string> = new Set()
): string | null => {
  const raw = tokens.get(name)
  if (raw === undefined || seen.has(name)) return null
  const nextSeen = new Set(seen).add(name)

  let resolved = raw
  for (let i = 0; i < 10; i += 1) {
    const match = resolved.match(VAR_PATTERN)
    if (!match || match.index === undefined) break
    const inner = tokens.has(match[1]) ? resolveRawValue(tokens, match[1], nextSeen) : null
    const replacement = inner ?? match[2]?.trim() ?? ''
    resolved = resolved.slice(0, match.index) + replacement + resolved.slice(match.index + match[0].length)
  }
  return resolved
}

const parseAlpha = (raw: string): number =>
  raw.endsWith('%') ? parseFloat(raw) / 100 : parseFloat(raw)

/** 把令牌定义归一为 RGBA（支持 #hex、rgb()/rgba() 现代与逗号语法；非颜色返回 null）。 */
const parseColor = (tokens: Map<string, string>, name: string): Rgba | null => {
  const value = resolveRawValue(tokens, name)
  if (value === null) return null

  const hex = value.trim().match(/^#([\da-f]{3,8})$/i)
  if (hex) {
    let digits = hex[1]
    if (digits.length === 3 || digits.length === 4) {
      digits = digits
        .split('')
        .map((char) => char + char)
        .join('')
    }
    return {
      r: parseInt(digits.slice(0, 2), 16),
      g: parseInt(digits.slice(2, 4), 16),
      b: parseInt(digits.slice(4, 6), 16),
      a: digits.length === 8 ? parseInt(digits.slice(6, 8), 16) / 255 : 1,
    }
  }

  const fn = value.trim().match(/^rgba?\(([\s\S]+)\)$/i)
  if (fn) {
    const slashSplit = fn[1].split('/')
    const channelsPart = slashSplit[0].trim()
    const alphaPart = slashSplit[1]?.trim()
    const channels = channelsPart.includes(',')
      ? channelsPart.split(',').map((channel) => channel.trim())
      : channelsPart.split(/\s+/)
    if (channels.length < 3) return null

    const channel = (rawValue: string): number =>
      rawValue.endsWith('%') ? (parseFloat(rawValue) / 100) * 255 : parseFloat(rawValue)
    let alpha = 1
    if (alphaPart) {
      alpha = parseAlpha(alphaPart)
    } else if (channels.length >= 4) {
      alpha = parseAlpha(channels[3])
    }

    return {
      r: channel(channels[0]),
      g: channel(channels[1]),
      b: channel(channels[2]),
      a: alpha,
    }
  }

  return null
}

const linearize = (channel: number): number => {
  const c = channel / 255
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
}

const luminance = (color: Rgba): number =>
  0.2126 * linearize(color.r) + 0.7152 * linearize(color.g) + 0.0722 * linearize(color.b)

const contrastRatio = (first: Rgba, second: Rgba): number => {
  const [lighter, darker] = [luminance(first), luminance(second)].sort((a, b) => b - a)
  return (lighter + 0.05) / (darker + 0.05)
}

/** 边框等半透明颜色按 alpha 混合到底色上，再与底色算对比。 */
const blendOver = (foreground: Rgba, background: Rgba): Rgba => ({
  r: foreground.r * foreground.a + background.r * (1 - foreground.a),
  g: foreground.g * foreground.a + background.g * (1 - foreground.a),
  b: foreground.b * foreground.a + background.b * (1 - foreground.a),
  a: 1,
})

const readToken = (tokens: Map<string, string>, name: string): Rgba => {
  const color = parseColor(tokens, name)
  expect(color, `${name} 应解析为颜色`).not.toBeNull()
  if (color === null) throw new Error(`${name} 无法解析为颜色`)
  return color
}

let blocks: CssBlock[] = []

beforeAll(async () => {
  const source = await readFile('src/styles/tokens.css', 'utf8')
  blocks = parseBlocks(source)
})

describe('theme contrast contract', () => {
  it.each(COMBOS)('$name：背景/文本/stage/卡片令牌解析后 100% 不透明', (combo) => {
    const tokens = resolveTokens(blocks, combo, 'clay')

    const opaqueTokens = [
      '--color-bg-base',
      '--color-bg-elevated',
      '--color-bg-surface',
      '--color-bg-overlay',
      '--color-text-primary',
      '--color-text-secondary',
      '--color-text-muted',
      '--color-text-ghost',
      '--color-text-disabled',
      '--color-text-inverted',
      '--color-stage-text-primary',
      '--color-stage-text-secondary',
      '--color-stage-text-muted',
      '--color-stage-text-quiet',
      '--color-stage-surface-soft',
      '--color-stage-surface-medium',
      '--color-stage-surface-strong',
      '--color-stage-chip-neutral-bg',
      '--surface-card-bg',
      '--surface-workspace-bg',
    ]

    for (const name of opaqueTokens) {
      expect(readToken(tokens, name).a, name).toBe(1)
    }
  })

  it.each(COMBOS)('$name：文本对比度达标（primary ≥12、secondary ≥7、muted ≥4.5）', (combo) => {
    const tokens = resolveTokens(blocks, combo, 'clay')
    const surface = readToken(tokens, '--color-bg-surface')

    const floors: Array<readonly [string, number]> = [
      ['--color-text-primary', 12],
      ['--color-text-secondary', 7],
      ['--color-text-muted', 4.5],
    ]

    for (const [name, min] of floors) {
      const ratio = contrastRatio(readToken(tokens, name), surface)
      expect(ratio, `${name} vs --color-bg-surface = ${ratio.toFixed(2)}:1`).toBeGreaterThanOrEqual(
        min
      )
    }
  })

  it.each(COMBOS)('$name：elevation 几何正确（暗色逐级提亮、亮色卡片最亮）', (combo) => {
    const tokens = resolveTokens(blocks, combo, 'clay')
    const base = luminance(readToken(tokens, '--color-bg-base'))
    const elevated = luminance(readToken(tokens, '--color-bg-elevated'))
    const surface = luminance(readToken(tokens, '--color-bg-surface'))
    const overlay = luminance(readToken(tokens, '--color-bg-overlay'))

    if (combo.theme === 'dark') {
      expect(base, 'base < elevated').toBeLessThan(elevated)
      expect(elevated, 'elevated < surface').toBeLessThan(surface)
      expect(surface, 'surface < overlay').toBeLessThan(overlay)
    } else {
      // 亮色：桌面压暗、卡片最亮；overlay 为下沉/chip 层（暗于 surface）
      expect(base, 'base < elevated').toBeLessThan(elevated)
      expect(elevated, 'elevated < surface').toBeLessThan(surface)
      expect(overlay, 'overlay 暗于 surface').toBeLessThan(surface)
    }
  })

  it.each(COMBOS)('$name：border-default 对 bg-surface 肉眼可辨且 alpha 达标', (combo) => {
    const tokens = resolveTokens(blocks, combo, 'clay')
    const surface = readToken(tokens, '--color-bg-surface')
    const borderDefault = readToken(tokens, '--color-border-default')

    const ratio = contrastRatio(blendOver(borderDefault, surface), surface)
    expect(ratio, `border-default 混合后 vs bg-surface = ${ratio.toFixed(2)}:1`).toBeGreaterThanOrEqual(
      1.2
    )

    // PRD R1：暗色 border alpha ≥ 14/22/34%；亮色锚点 ≥ 12/19/30%。
    const floors: Array<readonly [string, number]> =
      combo.theme === 'dark'
        ? [
            ['--color-border-subtle', 0.14],
            ['--color-border-default', 0.22],
            ['--color-border-strong', 0.34],
          ]
        : [
            ['--color-border-subtle', 0.12],
            ['--color-border-default', 0.19],
            ['--color-border-strong', 0.3],
          ]

    for (const [name, minAlpha] of floors) {
      expect(readToken(tokens, name).a, name).toBeGreaterThanOrEqual(minAlpha)
    }
  })

  it.each(COMBOS)('$name：accent vs accent-contrast ≥ 3.5（4 个 accent 全量）', (combo) => {
    for (const accent of ACCENTS) {
      const tokens = resolveTokens(blocks, combo, accent)
      const primary = readToken(tokens, '--color-accent-primary')
      const contrast = readToken(tokens, '--color-accent-primary-contrast')
      const ratio = contrastRatio(primary, contrast)
      expect(
        ratio,
        `[data-accent='${accent}'] accent vs accent-contrast = ${ratio.toFixed(2)}:1`
      ).toBeGreaterThanOrEqual(3.5)
    }
  })

  it('文本/表面令牌定义不携带 <100% alpha（正则扫全部定义行）', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')
    const linePattern =
      /^\s*(--color-(?:text|bg|stage-text|stage-surface)-[\w-]+|--surface-(?:card|workspace)-bg)\s*:\s*([^;]+);/gm

    const offenders: string[] = []
    for (const match of source.matchAll(linePattern)) {
      const [, name, value] = match
      const alpha = value.match(/\/\s*([\d.]+)\s*(%?)\s*\)?\s*$/)
      if (alpha) {
        const normalized = parseAlpha(alpha[1] + (alpha[2] ?? ''))
        if (normalized < 1) {
          offenders.push(`${name}: ${value.trim()}`)
        }
      }
    }

    expect(offenders, `以下令牌定义仍带半透明 alpha：\n${offenders.join('\n')}`).toEqual([])
  })

  it('死令牌与旧 flavor 结构无残留', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).not.toMatch(/--stage-bg-(?:mesh|aurora|orb|grid|noise)/)
    expect(source).not.toMatch(/--color-premium-(?:pink|blue)/)
    expect(source).not.toMatch(/data-resolved-flavor=["'](?:frappe|macchiato)["']/)
    expect(source).not.toMatch(/\[data-flavor=["'](?:paper|graphite)["']\]/)
  })
})
