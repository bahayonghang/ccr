import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { SettingsRawSource } from '@/configs/settings'
import type { EnvironmentInfo } from '@/api/runtime/environment'
import { environmentRevision, EnvironmentSessionError } from '@/configs/environmentSession'
import { ConfigSourcePanel } from '@/features/platform/editor/ConfigSourcePanel'
import { useAppT } from '@/i18n'
import { Button } from '@/ui'

export function SettingsSource({ source, environment, disabled, isCurrent, assertCurrent, onDirtyChange, onClose, onSaved }: {
  source: SettingsRawSource
  environment: EnvironmentInfo
  disabled: boolean
  isCurrent: () => boolean
  assertCurrent: () => Promise<void>
  onDirtyChange: (dirty: boolean) => void
  onClose: () => void
  onSaved: () => void
}) {
  const t = useAppT()
  const client = useQueryClient()
  const [opened, setOpened] = useState(false)
  const allowed = useRef(false)
  const isAllowed = useCallback(() => isCurrent() && allowed.current, [isCurrent])
  const guarded = useMemo(() => {
    const read = async <T,>(operation: () => Promise<T>) => {
      if (!isAllowed()) throw new EnvironmentSessionError()
      await assertCurrent()
      const revision = environmentRevision(client)
      const result = await operation()
      await assertCurrent()
      if (!isAllowed() || revision !== environmentRevision(client)) throw new EnvironmentSessionError()
      return result
    }
    return {
      getRaw: () => read(source.getRaw),
      listLayers: () => read(source.listLayers),
      saveRaw: (content: string, token: string) => read(() => source.saveRaw(content, token)),
    }
  }, [assertCurrent, client, isAllowed, source])
  const probe = useQuery({
    queryKey: ['settings-source-environment', environment.id, environment.env_type],
    queryFn: async () => {
      await assertCurrent()
      return environment.env_type === 'local' ? source.probe() : 'unsupported_environment' as const
    },
    enabled: !disabled,
    retry: false,
    staleTime: 0,
    gcTime: 0,
    refetchOnWindowFocus: false,
  })
  allowed.current = !disabled && probe.data === 'ok' && !probe.isError && !probe.isFetching
  useEffect(() => { if (probe.data === 'ok') setOpened(true) }, [probe.data])
  const retry = useCallback(async () => { await probe.refetch() }, [probe])
  if (!opened && probe.isPending) return <p role="status">{t('settingsRaw.loading')}</p>
  if (!opened && probe.data !== 'ok') {
    return (
      <div role="alert">
        <p>{probe.isError ? t('settingsRaw.loadFailed') : t('settingsRaw.unsupportedEnvironment')}</p>
        <Button disabled={disabled} onClick={retry}>{t('common.retry')}</Button>
        <Button onClick={onClose}>{t('common.back')}</Button>
      </div>
    )
  }
  return (
    <>
    {probe.isError || probe.data !== 'ok' ? <div role="alert"><p>{probe.isError ? t('settingsRaw.loadFailed') : t('settingsRaw.unsupportedEnvironment')}</p><Button disabled={disabled} onClick={retry}>{t('common.retry')}</Button></div> : null}
    <ConfigSourcePanel
      language={source.language}
      getRaw={guarded.getRaw}
      saveRaw={guarded.saveRaw}
      listLayers={guarded.listLayers}
      disabled={!allowed.current}
      isCurrent={isAllowed}
      onDirtyChange={onDirtyChange}
      backupNotice={source.backupNoticeKey ? t(source.backupNoticeKey) : undefined}
      policyNotice={source.policyNoticeKey ? t(source.policyNoticeKey) : undefined}
      policyLayerIds={source.policyLayerIds}
      onClose={onClose}
      onSaved={onSaved}
    />
    </>
  )
}
