import { describe, expect, it } from 'vitest'
import { bootLocaleMessages } from '@/i18n/bootMessages'
import zhCN from '@/i18n/locales/zh-CN'

const CJK_RE = /[一-鿿\u3000-\u303f\uff00-\uffef]/
const KICKER_SEGMENT_RE = /eyebrow|kicker/i

// 纯数字/符号样例不设中文要求
const CJK_EXEMPT_KEYS = new Set(['settings.appearance.typography.previewSampleCode'])

const collectLeaves = (node: unknown, prefix: string[], out: Map<string, string>): void => {
  if (!node || typeof node !== 'object') return
  for (const [key, value] of Object.entries(node as Record<string, unknown>)) {
    if (typeof value === 'string') {
      out.set([...prefix, key].join('.'), value)
    } else {
      collectLeaves(value, [...prefix, key], out)
    }
  }
}

const leavesOf = (root: unknown): Map<string, string> => {
  const out = new Map<string, string>()
  collectLeaves(root, [], out)
  return out
}

describe('zh-CN copy is actually Chinese', () => {
  const packs: Array<[string, unknown]> = [
    ['locales/zh-CN', zhCN],
    ['bootMessages zh-CN', bootLocaleMessages['zh-CN']],
  ]

  for (const [label, pack] of packs) {
    it(`${label}: every eyebrow/kicker value contains CJK`, () => {
      const offenders = [...leavesOf(pack).entries()].filter(
        ([key, value]) =>
          key.split('.').some((segment) => KICKER_SEGMENT_RE.test(segment)) && !CJK_RE.test(value),
      )
      expect(offenders).toEqual([])
    })

    it(`${label}: settings domain values contain CJK`, () => {
      const offenders = [...leavesOf(pack).entries()].filter(
        ([key, value]) =>
          key.startsWith('settings.') && !CJK_EXEMPT_KEYS.has(key) && !CJK_RE.test(value),
      )
      expect(offenders).toEqual([])
    })
  }
})
