import { useAppLocale, useAppT, type SupportedLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function useMonitoringT(): TranslateFunction {
  return useAppT()
}

export function useMonitoringLocale(): SupportedLocale {
  return useAppLocale()
}
