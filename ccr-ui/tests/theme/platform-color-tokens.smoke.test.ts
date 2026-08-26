import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'

const TOKENS_PATH = 'src/styles/tokens.css'
const ROOT_BLOCK = /:root\s*\{[\s\S]*?\n\}/
const DARK_BLOCK = /\[data-theme='dark'\]\s*\{[\s\S]*?\n\}/
const HEX = /^#[0-9a-f]{6}$/i

const PLATFORMS = [
  'claude',
  'codex',
  'grok',
  'gemini',
  'opencode',
  'antigravity',
] as const

const ROLES = ['', '-rgb', '-surface', '-border', '-text'] as const

const extractVar = (block: string, name: string): string | null => {
  const match = block.match(new RegExp(`${name}:\\s*([^;]+);`))
  return match ? match[1].trim() : null
}

const hexToRgb = (hex: string): [number, number, number] => {
  const digits = hex.slice(1)
  return [
    parseInt(digits.slice(0, 2), 16),
    parseInt(digits.slice(2, 4), 16),
    parseInt(digits.slice(4, 6), 16),
  ]
}

const linearize = (channel: number): number => {
  const c = channel / 255
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
}

const luminance = (hex: string): number => {
  const [r, g, b] = hexToRgb(hex)
  return 0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

const contrastRatio = (foreground: string, background: string): number => {
  const lighter = Math.max(luminance(foreground), luminance(background))
  const darker = Math.min(luminance(foreground), luminance(background))
  return (lighter + 0.05) / (darker + 0.05)
}

describe('平台色 token 四角色', () => {
  it('六平台 dot / rgb / surface / border / text 在明暗两套主题下都有取值', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    const root = source.match(ROOT_BLOCK)?.[0] ?? ''
    const dark = source.match(DARK_BLOCK)?.[0] ?? ''
    expect(root.length).toBeGreaterThan(0)
    expect(dark.length).toBeGreaterThan(0)

    for (const block of [root, dark]) {
      for (const platform of PLATFORMS) {
        for (const role of ROLES) {
          const name = `--color-platform-${platform}${role}`
          const value = extractVar(block, name)
          expect(value, name).toBeTruthy()
          expect(value, name).not.toBe('undefined')
        }
        const hex = extractVar(block, `--color-platform-${platform}`)
        expect(hex, platform).toMatch(HEX)
        const surface = extractVar(block, `--color-platform-${platform}-surface`)
        const border = extractVar(block, `--color-platform-${platform}-border`)
        const text = extractVar(block, `--color-platform-${platform}-text`)
        expect(surface).toMatch(HEX)
        expect(border).toMatch(HEX)
        expect(text).toMatch(HEX)
      }
    }
  })

  it('明色主题 -text 对 -surface 对比度不低于 4.5:1，且不使用 color-mix()', async () => {
    const source = await readFile(TOKENS_PATH, 'utf8')
    const root = source.match(ROOT_BLOCK)?.[0] ?? ''
    const platformDecls = [...root.matchAll(/--color-platform-[a-z-]+:\s*([^;]+);/g)]

    for (const match of platformDecls) {
      expect(match[1]).not.toMatch(/color-mix\(/i)
    }

    for (const platform of PLATFORMS) {
      const surface = extractVar(root, `--color-platform-${platform}-surface`)
      const text = extractVar(root, `--color-platform-${platform}-text`)
      expect(surface).toMatch(HEX)
      expect(text).toMatch(HEX)
      expect(contrastRatio(text ?? '', surface ?? '')).toBeGreaterThanOrEqual(4.5)
    }
  })
})
