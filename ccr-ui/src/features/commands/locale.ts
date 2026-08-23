import { useAppT } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function useCommandsT(): TranslateFunction {
  return useAppT()
}
