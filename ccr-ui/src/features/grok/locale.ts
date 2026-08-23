import { defaultSurfaceT } from '@/features/platform'
import { readStoredLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export const t: TranslateFunction = defaultSurfaceT

export function tt(zh: string, en: string): string {
  return readStoredLocale().startsWith('zh') ? zh : en
}
