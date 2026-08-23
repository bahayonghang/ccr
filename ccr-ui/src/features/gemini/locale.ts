import { translate, tt as translateLiteral } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export const t: TranslateFunction = translate

export function tt(zh: string, en: string): string {
  return translateLiteral(zh, en)
}
