import { useMemo } from 'react'
import { useAppLocale, useAppT, useAppTt } from '@/i18n'
import { createTf } from '@/utils/tf'

export function useOpenCodeLocale() {
  const locale = useAppLocale()
  const isZh = locale.startsWith('zh')
  const t = useAppT()
  const tf = useMemo(() => createTf(t), [t])
  const tt = useAppTt()
  return { locale, isZh, t, tf, tt }
}
