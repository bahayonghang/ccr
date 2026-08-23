import { ensureLocaleLoaded, readStoredLocale, translate, useAppT, type SupportedLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export const translateUsage = (
  _locale: SupportedLocale,
  key: string,
  values?: Parameters<TranslateFunction>[1],
): string => translate(key, values)

export async function hydrateUsageLocale(locale?: SupportedLocale): Promise<SupportedLocale> {
  return ensureLocaleLoaded(locale ?? readStoredLocale())
}

export function useUsageT(): TranslateFunction {
  return useAppT()
}
