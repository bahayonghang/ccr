import { useAppT, useAppTt } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function useTrayT(): TranslateFunction {
  return useAppT()
}

export function useTrayTt(): (zh: string, en: string) => string {
  return useAppTt()
}
