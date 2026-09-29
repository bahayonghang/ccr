import { useAppT } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function useEditorT(): TranslateFunction {
  return useAppT()
}
