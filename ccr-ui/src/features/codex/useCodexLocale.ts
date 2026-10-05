import { useMemo } from 'react'
import { useAppLocale, useAppT, useAppTt } from '@/i18n'
import { createTf } from '@/utils/tf'

export type CodexTf = ReturnType<typeof createTf>

export function useCodexLocale() {
  const locale = useAppLocale()
  const isZh = locale.startsWith('zh')
  const t = useAppT()
  const tf = useMemo(() => createTf(t), [t])
  const tt = useAppTt()
  return { locale, isZh, t, tf, tt }
}
