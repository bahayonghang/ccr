import { useCallback, useMemo } from 'react'
import { defaultSurfaceT } from '@/features/platform'
import { readStoredLocale } from '@/i18n'
import { createTf } from '@/utils/tf'

export type CodexTf = ReturnType<typeof createTf>

export function useCodexLocale() {
  const locale = readStoredLocale()
  const isZh = locale.startsWith('zh')
  const t = defaultSurfaceT
  const tf = useMemo(() => createTf(t), [t])
  const tt = useCallback((zh: string, en: string) => (isZh ? zh : en), [isZh])
  return { locale, isZh, t, tf, tt }
}
