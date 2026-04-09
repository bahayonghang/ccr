import { describe, expect, it } from 'vitest'
import { createI18n } from 'vue-i18n'
import zhCnMessages from '@/i18n/locales/zh-CN'
import { translateWithFallback } from '@/i18n/formatMessage'

describe('translateWithFallback', () => {
  it('returns interpolated locale messages when vue-i18n resolves normally', () => {
    const i18n = createI18n({
      legacy: false,
      locale: 'zh-CN',
      fallbackLocale: 'zh-CN',
      missingWarn: false,
      fallbackWarn: false,
      messages: {
        'zh-CN': zhCnMessages,
      },
    })

    expect(
      translateWithFallback(
        i18n.global.t.bind(i18n.global),
        'codex.profiles.confirmApply',
        '确定切换到 Profile "{name}" 吗？',
        { name: 'demo' },
      ),
    ).toBe('确定切换到 Profile "demo" 吗？')
  })

  it('falls back to manual interpolation when the translator leaves placeholders unresolved', () => {
    const translate = (_key: string, _values?: Record<string, unknown>) => '确定要应用 Profile "{name}" 吗？'

    expect(
      translateWithFallback(
        translate,
        'claudeProfiles.confirmApply',
        '确定要应用 Profile "{name}" 吗？这将同步更新当前 Claude 配置。',
        { name: 'alpha' },
      ),
    ).toBe('确定要应用 Profile "alpha" 吗？')
  })
})
