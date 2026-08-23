import {
  ensureLocaleLoaded,
  readStoredLocale,
  translate,
  useAppT,
  type SupportedLocale,
} from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export type ShellTranslate = TranslateFunction

export const translateShell = (
  _locale: SupportedLocale,
  key: string,
  values?: Parameters<TranslateFunction>[1],
): string => translate(key, values)

/** 加载完整 locale 包。settings 等 defer 路由可在挂载后再调用。 */
export async function hydrateShellLocale(locale?: SupportedLocale): Promise<SupportedLocale> {
  return ensureLocaleLoaded(locale ?? readStoredLocale())
}

export function useShellT(): ShellTranslate {
  return useAppT()
}
