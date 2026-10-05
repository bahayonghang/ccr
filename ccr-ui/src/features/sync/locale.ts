import { useAppT, useAppTt } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function useSyncT(): TranslateFunction {
  return useAppT()
}

export function useSyncTt(): (zh: string, en: string) => string {
  return useAppTt()
}
