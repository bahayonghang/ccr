import { translate, useAppLocale, useAppT, useAppTt, type SupportedLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function translateCheckin(
  _locale: SupportedLocale,
  key: string,
  values?: Parameters<TranslateFunction>[1],
): string {
  return translate(key, values)
}

export function useCheckinT(): TranslateFunction {
  return useAppT()
}

export function useCheckinLocale(): SupportedLocale {
  return useAppLocale()
}

export function useTt(): (zh: string, en: string) => string {
  return useAppTt()
}
