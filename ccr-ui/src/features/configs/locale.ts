import { readStoredLocale, translate, tt as translateLiteral } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

/** configs 域默认 t：i18next 单例。组件内请用 useAppT 订阅语言切换。 */
export const t: TranslateFunction = translate

export function tt(zh: string, en: string): string {
  return translateLiteral(zh, en)
}

export function isZhLocale(): boolean {
  return readStoredLocale().startsWith('zh')
}
