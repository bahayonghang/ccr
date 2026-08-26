import { readFile } from 'node:fs/promises'

import { describe, expect, it } from 'vitest'

const TOKENS_PATH = 'src/styles/tokens.css'

const ROOT_BLOCK = /:root\s*\{[\s\S]*?\n\}/
const DARK_BLOCK = /\[data-theme='dark'\]\s*\{[\s\S]*?\n\}/
const CLAY_BLOCK = /\[data-flavor='clay'\]\s*\{[\s\S]*?\n\}/
const CLAY_DARK_BLOCK = /\[data-theme='dark'\]\[data-flavor='clay'\]\s*\{[\s\S]*?\n\}/

const SOLID_BORDERS = [
  '--color-border-subtle',
  '--color-border-default',
  '--color-border-strong',
] as const

const TINTS = [
  '--color-success-tint',
  '--color-warning-tint',
  '--color-danger-tint',
  '--color-info-tint',
] as const

const HEX = /^#[0-9a-f]{6}$/i
const RGB_TRIPLET = /^\d{1,3} \d{1,3} \d{1,3}$/

const extractVar = (block: string, name: string): string | null => {
  const match = block.match(new RegExp(`${name}:\\s*([^;]+);`))
  return match ? match[1].trim() : null
}

const hexToRgbTriplet = (hex: string): string => {
  const digits = hex.slice(1)
  return [
    parseInt(digits.slice(0, 2), 16),
    parseInt(digits.slice(2, 4), 16),
    parseInt(digits.slice(4, 6), 16),
  ].join(' ')
}

describe('token consolidation（实色边框 / 四档圆角 / 名称增量）', () => {
  it('四作用域边框为实色十六进制，且 -rgb 与该实色一致', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    const blocks = [ROOT_BLOCK, DARK_BLOCK, CLAY_BLOCK, CLAY_DARK_BLOCK].map((pattern) => {
      const match = source.match(pattern)
      expect(match, String(pattern)).not.toBeNull()
      return match?.[0] ?? ''
    })

    for (const block of blocks) {
      for (const name of SOLID_BORDERS) {
        const value = extractVar(block, name)
        expect(value, name).toMatch(HEX)
        expect(value, name).not.toMatch(/rgb\(/i)

        const rgbName = `${name}-rgb`
        const rgb = extractVar(block, rgbName)
        expect(rgb, rgbName).toMatch(RGB_TRIPLET)
        expect(rgb, rgbName).toBe(hexToRgbTriplet(value ?? ''))
      }
    }
  })

  it('圆角取值收敛为 {0, 6px, 8px, 12px, 9999px}', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    const expected: Record<string, string> = {
      '--radius-none': '0',
      '--radius-sm': '6px',
      '--radius-md': '6px',
      '--radius-lg': '8px',
      '--radius-xl': '12px',
      '--radius-2xl': '12px',
      '--radius-3xl': '12px',
      '--radius-full': '9999px',
    }

    for (const [name, value] of Object.entries(expected)) {
      expect(extractVar(source, name), name).toBe(value)
    }

    const radiusDecls = [...source.matchAll(/--radius-[a-z0-9]+:\s*([^;]+);/g)].map(
      (match) => match[1].trim(),
    )
    expect(new Set(radiusDecls)).toEqual(new Set(['0', '6px', '8px', '12px', '9999px']))
  })

  it('不引入 chrome / 圆角角色新名称', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    expect(source).not.toMatch(/--color-bg-chrome\s*:/)
    expect(source).not.toMatch(/--radius-(chip|control|card|pill)\s*:/)
  })

  it('四个 tint 在四作用域可解析为实色', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    const blocks = [ROOT_BLOCK, DARK_BLOCK, CLAY_BLOCK, CLAY_DARK_BLOCK].map((pattern) => {
      const match = source.match(pattern)
      expect(match).not.toBeNull()
      return match?.[0] ?? ''
    })

    for (const block of blocks) {
      for (const name of TINTS) {
        expect(extractVar(block, name), name).toMatch(HEX)
      }
    }
  })

  it('opencode 平台色在 :root 定义且 -rgb 配对', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    const root = source.match(ROOT_BLOCK)?.[0] ?? ''
    const hex = extractVar(root, '--color-platform-opencode')
    const rgb = extractVar(root, '--color-platform-opencode-rgb')

    expect(hex).toBe('#735f52')
    expect(rgb).toBe('115 95 82')
    expect(rgb).toBe(hexToRgbTriplet(hex ?? ''))
  })
})
