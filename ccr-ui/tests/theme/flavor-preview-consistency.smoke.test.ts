import { readFile } from 'node:fs/promises'

import { describe, expect, it } from 'vitest'

/**
 * 守护 `FLAVOR_PREVIEW_TOKENS` 与 `tokens.css` 四面作用域取值一致。
 * 解析规则：块内有定义则取块内值，否则继承上层（:root → dark → clay → dark+clay）。
 */

const TOKENS_PATH = 'src/styles/tokens.css'
const PREVIEW_PATH = 'src/features/configs/lib/flavorPreview.ts'

const FLAVORS = ['neutral', 'clay'] as const
const THEMES = ['light', 'dark'] as const
const SURFACE_KEYS = ['base', 'elevated', 'surface', 'text', 'muted'] as const

const TOKEN_BY_SURFACE = {
  base: '--color-bg-base',
  elevated: '--color-bg-elevated',
  surface: '--color-bg-surface',
  text: '--color-text-primary',
  muted: '--color-text-muted',
} as const

const SCOPE_SELECTORS = {
  root: ':root',
  dark: "[data-theme='dark']",
  clay: "[data-flavor='clay']",
  darkClay: "[data-theme='dark'][data-flavor='clay']",
} as const

type SurfaceKey = (typeof SURFACE_KEYS)[number]
type FlavorName = (typeof FLAVORS)[number]
type ThemeName = (typeof THEMES)[number]
type TokenMap = Record<string, string>
type PreviewTable = Record<FlavorName, Record<ThemeName, Record<SurfaceKey, string>>>

const escapeRegExp = (value: string): string => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

const normalizeHex = (value: string): string => value.trim().toLowerCase()

const stripComments = (source: string): string => source.replace(/\/\*[\s\S]*?\*\//g, '')

const extractDeclarations = (body: string): TokenMap => {
  const declarations: TokenMap = {}
  for (const piece of body.split(';')) {
    const match = piece.match(/^\s*(--[\w-]+)\s*:\s*([\s\S]+?)\s*$/)
    if (match) {
      declarations[match[1]] = match[2].trim()
    }
  }
  return declarations
}

const extractScopeBlock = (source: string, selector: string): TokenMap => {
  const pattern = new RegExp(`(?:^|\\n)\\s*${escapeRegExp(selector)}\\s*\\{`)
  const match = pattern.exec(source)
  if (!match) {
    throw new Error(`tokens.css missing scope selector ${selector}`)
  }

  const openBrace = source.indexOf('{', match.index)
  let depth = 0
  let closeBrace = -1
  for (let index = openBrace; index < source.length; index += 1) {
    const char = source[index]
    if (char === '{') depth += 1
    if (char === '}') {
      depth -= 1
      if (depth === 0) {
        closeBrace = index
        break
      }
    }
  }
  if (closeBrace < 0) {
    throw new Error(`tokens.css unclosed block for ${selector}`)
  }

  return extractDeclarations(source.slice(openBrace + 1, closeBrace))
}

const resolvePreviewTokens = (
  scopes: Record<keyof typeof SCOPE_SELECTORS, TokenMap>,
  flavor: FlavorName,
  theme: ThemeName,
): TokenMap => {
  const merged: TokenMap = { ...scopes.root }
  if (theme === 'dark') {
    Object.assign(merged, scopes.dark)
  }
  if (flavor === 'clay') {
    Object.assign(merged, scopes.clay)
  }
  if (theme === 'dark' && flavor === 'clay') {
    Object.assign(merged, scopes.darkClay)
  }
  return merged
}

const parseFlavorPreviewTokens = (source: string): PreviewTable => {
  const table = {
    neutral: { light: {}, dark: {} },
    clay: { light: {}, dark: {} },
  } as PreviewTable

  for (const flavor of FLAVORS) {
    for (const theme of THEMES) {
      const blockMatch = source.match(
        new RegExp(`${flavor}:\\s*\\{[\\s\\S]*?${theme}:\\s*\\{([^}]+)\\}`),
      )
      if (!blockMatch) {
        throw new Error(`FLAVOR_PREVIEW_TOKENS missing ${flavor}/${theme}`)
      }
      for (const key of SURFACE_KEYS) {
        const valueMatch = blockMatch[1].match(new RegExp(`${key}:\\s*['"](#[0-9a-fA-F]{3,8})['"]`))
        if (!valueMatch) {
          throw new Error(`FLAVOR_PREVIEW_TOKENS missing ${flavor}/${theme}/${key}`)
        }
        table[flavor][theme][key] = valueMatch[1]
      }
    }
  }

  return table
}

describe('flavor preview tokens stay aligned with tokens.css', () => {
  it('matches inherited --color-bg-* / --color-text-* across four theme×flavor scopes', async () => {
    const [tokensSource, previewSource] = await Promise.all([
      readFile(TOKENS_PATH, 'utf8'),
      readFile(PREVIEW_PATH, 'utf8'),
    ])
    const tokensCss = stripComments(tokensSource)

    const scopes = {
      root: extractScopeBlock(tokensCss, SCOPE_SELECTORS.root),
      dark: extractScopeBlock(tokensCss, SCOPE_SELECTORS.dark),
      clay: extractScopeBlock(tokensCss, SCOPE_SELECTORS.clay),
      darkClay: extractScopeBlock(tokensCss, SCOPE_SELECTORS.darkClay),
    }
    const previewTable = parseFlavorPreviewTokens(previewSource)

    for (const flavor of FLAVORS) {
      for (const theme of THEMES) {
        const resolved = resolvePreviewTokens(scopes, flavor, theme)
        for (const key of SURFACE_KEYS) {
          const tokenName = TOKEN_BY_SURFACE[key]
          const cssValue = resolved[tokenName]
          const previewValue = previewTable[flavor][theme][key]
          expect(cssValue, `flavor=${flavor} theme=${theme} token=${tokenName} missing in tokens.css`).toBeTruthy()
          expect(
            normalizeHex(previewValue),
            `flavor=${flavor} theme=${theme} token=${tokenName}`,
          ).toBe(normalizeHex(cssValue))
        }
      }
    }
  })
})
