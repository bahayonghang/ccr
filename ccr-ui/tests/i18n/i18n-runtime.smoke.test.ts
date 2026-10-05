import { beforeEach, describe, expect, it } from 'vitest'
import i18n, {
  DEFAULT_LOCALE,
  ensureLocaleLoaded,
  LOCALE_STORAGE_KEY,
  setLocale,
  translate,
} from '@/i18n'
import { unescapeVueI18nLiterals } from '@/i18n/formatMessage'

describe('i18n runtime', () => {
  beforeEach(async () => {
    await setLocale(DEFAULT_LOCALE)
  })

  it('interpolates {name} placeholders from boot messages', () => {
    expect(translate('common.about.title', { name: 'CCR' })).toContain('CCR')
    expect(translate('common.about.title', { name: 'CCR' })).not.toContain('{name}')
  })

  it('persists locale preference and switches without reload', async () => {
    await setLocale('en-US')
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('en-US')
    expect(i18n.language).toBe('en-US')
    expect(document.documentElement.lang).toBe('en-US')
    expect(translate('common.save')).toBe('Save')

    await setLocale('zh-CN')
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('zh-CN')
    expect(translate('common.save')).toBe('保存')
  })

  it('unescapes vue-style quoted literals then hydrates full catalog', async () => {
    expect(unescapeVueI18nLiterals("you{'@'}example.com")).toBe('you@example.com')
    expect(unescapeVueI18nLiterals("{'{'}\"session\":\"xxx\"{'}'}")).toBe('{"session":"xxx"}')
    await ensureLocaleLoaded('zh-CN')
    expect(translate('sync.account.usernamePlaceholder')).toBe('you@example.com')
    expect(translate('mcp.argsPlaceholder')).toContain('@modelcontextprotocol')
    expect(translate('mcp.argsPlaceholder')).not.toContain("{'@'}")
  })

  it('returns the key for missing messages', () => {
    expect(translate('not.a.real.key.path')).toBe('not.a.real.key.path')
  })
})
