import { useAppT } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

export function useMcpT(): TranslateFunction {
  return useAppT()
}
