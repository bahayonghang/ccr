import { describe, expect, it } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import zhCN from '@/i18n/locales/zh-CN'

const getPathValue = (source: Record<string, unknown>, path: string): unknown => {
  return path.split('.').reduce<unknown>((current, segment) => {
    if (!current || typeof current !== 'object') return undefined
    return (current as Record<string, unknown>)[segment]
  }, source)
}

describe('settings i18n smoke', () => {
  it('keeps required settings strings in both locales', () => {
    const requiredPaths = [
      'nav.settings',
      'theme.system',
      'settings.title',
      'settings.dock.summary',
      'settings.appearance.systemSummary',
      'settings.shell.resetLayoutAction',
      'settings.diagnostics.restartNote',
    ]

    for (const path of requiredPaths) {
      expect(getPathValue(zhCN as Record<string, unknown>, path)).toBeTruthy()
      expect(getPathValue(enUS as Record<string, unknown>, path)).toBeTruthy()
    }
  })
})
