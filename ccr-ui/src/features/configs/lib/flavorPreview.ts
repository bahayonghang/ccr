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
    light: { base: '#e7f1ee', elevated: '#f2f7f5', surface: '#fff', text: '#1e2932', muted: '#5c6b76' },
    dark: { base: '#10191b', elevated: '#172225', surface: '#1d292c', text: '#f0f8f5', muted: '#9fafa9' },
  },
  clay: {
    light: { base: '#eae6f2', elevated: '#f5f2fa', surface: '#fefcff', text: '#222630', muted: '#686477' },
    dark: { base: '#171722', elevated: '#20212e', surface: '#282a38', text: '#f5f4fc', muted: '#b0adbf' },
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
