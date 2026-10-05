import type { FlavorMode, ResolvedThemeMode } from '@/utils/themeBootstrap'

interface SurfacePreviewTokens {
  base: string
  elevated: string
  surface: string
  text: string
  muted: string
}

const FLAVOR_PREVIEW_TOKENS: Record<FlavorMode, { light: SurfacePreviewTokens; dark: SurfacePreviewTokens }> = {
  neutral: {
    light: { base: '#e9e4d8', elevated: '#f2eee3', surface: '#faf7ec', text: '#211c12', muted: '#6b6150' },
    dark: { base: '#100f0c', elevated: '#171410', surface: '#1f1b14', text: '#e9e1d1', muted: '#a1937c' },
  },
  clay: {
    light: { base: '#ebe1d0', elevated: '#f5eee1', surface: '#fefaf2', text: '#31241c', muted: '#715d4c' },
    dark: { base: '#17120f', elevated: '#221b18', surface: '#2a221e', text: '#f3eadf', muted: '#b9a695' },
  },
}

export const PREVIEW_GLYPH_SAMPLE = 'Aa'

export function flavorPreviewStyle(
  flavorValue: FlavorMode,
  effectiveTheme: ResolvedThemeMode,
): Record<string, string> {
  const tokens = FLAVOR_PREVIEW_TOKENS[flavorValue][effectiveTheme]
  return {
    '--fp-bg-base': tokens.base,
    '--fp-bg-elevated': tokens.elevated,
    '--fp-bg-surface': tokens.surface,
    '--fp-text-primary': tokens.text,
    '--fp-text-muted': tokens.muted,
  }
}
