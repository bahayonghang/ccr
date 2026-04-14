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

  it('interpolates Claude profile count labels even when the translator leaves template placeholders intact', () => {
    const translate = (key: string, _values?: Record<string, unknown>) => {
      if (key === 'claudeProfiles.providerSectionsCount') {
        return 'Provider 分组 {count}'
      }

      return key
    }

    expect(
      translateWithFallback(
        translate,
        'claudeProfiles.providerSectionsCount',
        'Provider 分组 {count}',
        { count: 3 },
      ),
    ).toBe('Provider 分组 3')
  })

  it('interpolates Claude provider summaries when the translator returns an unresolved template', () => {
    const translate = (key: string, _values?: Record<string, unknown>) => {
      if (key === 'claudeProfiles.providerSectionSummary') {
        return '共 {count} 个 Profile，其中 {enabled} 个处于启用状态。'
      }

      return key
    }

    expect(
      translateWithFallback(
        translate,
        'claudeProfiles.providerSectionSummary',
        '共 {count} 个 Profile，其中 {enabled} 个处于启用状态。',
        { count: 12, enabled: 7 },
      ),
    ).toBe('共 12 个 Profile，其中 7 个处于启用状态。')
  })

  it('interpolates Codex auth login state labels when the translator leaves templates unresolved', () => {
    const translate = (key: string, _values?: Record<string, unknown>) => {
      if (key === 'codex.auth.loginState.loggedInSaved') {
        return '已登录 ({name})'
      }

      return key
    }

    expect(
      translateWithFallback(
        translate,
        'codex.auth.loginState.loggedInSaved',
        '已登录 ({name})',
        { name: 'qq_pro' },
      ),
    ).toBe('已登录 (qq_pro)')
  })
})
