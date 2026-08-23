import { defaultSurfaceT } from '@/features/platform'
import { readStoredLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

/** configs 域默认 t：boot catalog，不经过 shell。 */
export const t: TranslateFunction = defaultSurfaceT

/** 双语字面量（原 Vue `tt`）。读存储 locale，不订阅 shell store。 */
export function tt(zh: string, en: string): string {
  return readStoredLocale().startsWith('zh') ? zh : en
}

export function isZhLocale(): boolean {
  return readStoredLocale().startsWith('zh')
}
