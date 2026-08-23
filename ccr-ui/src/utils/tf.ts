import { translateWithFallback } from '@/i18n/formatMessage'

type TranslateValues = Record<string, string | number | boolean | null | undefined>

export type TranslateFunction = (
  key: string,
  values?: TranslateValues,
) => string

/**
 * 带兜底的翻译工厂（纯函数；08-22-state-logic-port 批次 5 由 useTf composable 迁入）。
 *
 * 返回 `tf(key, fallback, values?)`：命中 key 且无残留占位符时取译文，
 * 否则回退到 `fallback`。收口此前在多个组件里重复定义的局部 `tf` 包装。
 * React 侧的 t 来源为 `useAppT()` / `translate`（i18next）。
 */
export function createTf(t: TranslateFunction) {
  return (key: string, fallback: string, values: TranslateValues = {}) =>
    translateWithFallback(t, key, fallback, values)
}
