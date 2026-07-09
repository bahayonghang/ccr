import { useI18n } from 'vue-i18n'
import { translateWithFallback } from '@/i18n/formatMessage'

type TranslateValues = Record<string, string | number | boolean | null | undefined>

/**
 * 带兜底的翻译 composable。
 *
 * 返回 `tf(key, fallback, values?)`：命中 key 且无残留占位符时取译文，
 * 否则回退到 `fallback`。收口此前在多个组件里重复定义的局部 `tf` 包装。
 */
export function useTf() {
  const { t } = useI18n()
  return (key: string, fallback: string, values: TranslateValues = {}) =>
    translateWithFallback(t, key, fallback, values)
}
