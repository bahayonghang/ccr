import { useCallback, useEffect, useRef, useState } from 'react'
import { replaceEqualDeep, useQuery, useQueryClient } from '@tanstack/react-query'
import type { UseFormReturn } from 'react-hook-form'
import { getCurrentEnvironment, switchEnvironment, type EnvironmentInfo } from '@/api/runtime/environment'
import type { SettingsConfig, SettingsSaveResult, SettingsSnapshot, SettingsValues } from '@/configs/settings'
import { SettingsValidationError } from '@/configs/settings-types'
import { currentEnvironmentKey, environmentIsReady, environmentRevision, EnvironmentSessionError, sameEnvironment, verifyEnvironment } from '@/configs/environmentSession'
import { saveSettingsValues } from '@/features/platform/settings-model'
import type { TranslateFunction } from '@/utils/tf'

interface EditSession {
  environment: EnvironmentInfo
  snapshot: SettingsSnapshot
}

const probeKey = (config: SettingsConfig, env: EnvironmentInfo | undefined) =>
  ['platform-settings-probe', config.cacheKey, env?.id, env?.env_type] as const
const valuesKey = (config: SettingsConfig, env: EnvironmentInfo | undefined) =>
  ['platform-settings', config.cacheKey, env?.id, env?.env_type] as const

async function probeSettings(config: SettingsConfig, environment: EnvironmentInfo) {
  if (config.features.localOnly && environment.env_type !== 'local') return 'unsupported_environment' as const
  return config.probe ? config.probe() : 'ok' as const
}

export function useSettingsSession(config: SettingsConfig, form: UseFormReturn<SettingsValues>, t: TranslateFunction) {
  const client = useQueryClient()
  const [session, setSession] = useState<EditSession | null>(null)
  const sessionRef = useRef(session)
  const [saveResult, setSaveResult] = useState<SettingsSaveResult | null>(null)
  const [busy, setBusy] = useState(false)
  const busyRef = useRef(false)
  const [reloadRequired, setReloadRequired] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const generation = useRef(0)
  const mounted = useRef(true)
  const { reset } = form

  useEffect(() => {
    mounted.current = true
    return () => { mounted.current = false }
  }, [])

  const environmentQuery = useQuery({
    queryKey: currentEnvironmentKey, queryFn: getCurrentEnvironment, retry: false,
  })
  const environment = environmentQuery.data
  const ready = !environmentQuery.isFetching && !environmentQuery.isError && environmentIsReady(client, environment)
  const probeQuery = useQuery({
    queryKey: probeKey(config, environment),
    queryFn: () => probeSettings(config, environment!),
    enabled: ready, retry: false,
  })
  const readSnapshot = useCallback(async (expected: EnvironmentInfo, signal?: AbortSignal) => {
    if (!environmentIsReady(client, expected) || signal?.aborted) throw new EnvironmentSessionError()
    const revision = environmentRevision(client)
    const snapshot = await config.load({ environmentId: expected.id })
    await verifyEnvironment(client, expected)
    if (signal?.aborted || revision !== environmentRevision(client)) throw new EnvironmentSessionError()
    return snapshot
  }, [client, config])
  const valuesQuery = useQuery({
    queryKey: valuesKey(config, environment),
    queryFn: ({ signal }) => readSnapshot(environment!, signal),
    enabled: ready && probeQuery.data === 'ok' && !probeQuery.isError,
    retry: false, gcTime: 0,
  })
  const snapshotReady = valuesQuery.isSuccess && !valuesQuery.isFetching && probeQuery.isSuccess && probeQuery.data === 'ok'

  const bind = useCallback((expected: EnvironmentInfo, snapshot: SettingsSnapshot) => {
    const next = { environment: expected, snapshot }
    sessionRef.current = next
    setSession(next)
    reset(snapshot.values)
    setSaveResult(null)
    setReloadRequired(false)
    setActionError(null)
    generation.current++
  }, [reset])

  useEffect(() => {
    if (!sessionRef.current && environment && ready && valuesQuery.data && !valuesQuery.isError) {
      bind(environment, valuesQuery.data)
    }
  }, [bind, environment, ready, valuesQuery.data, valuesQuery.isError])

  const isCurrent = useCallback(() => {
    return mounted.current && environmentIsReady(client, sessionRef.current?.environment)
  }, [client])
  const assertCurrent = useCallback(async () => {
    const origin = sessionRef.current
    if (!origin || !isCurrent()) throw new EnvironmentSessionError()
    await verifyEnvironment(client, origin.environment)
    if (origin !== sessionRef.current || !isCurrent()) throw new EnvironmentSessionError()
  }, [client, isCurrent])

  const reportError = useCallback((error: unknown) => {
    const message = error instanceof SettingsValidationError ? t(error.messageKey)
      : error instanceof EnvironmentSessionError ? t(error.message)
      : error instanceof Error ? error.message : String(error)
    setActionError(error instanceof SettingsValidationError ? null : message)
    config.notify.error(message)
  }, [config.notify, t])

  const reload = useCallback(async () => {
    if (busyRef.current) return false
    busyRef.current = true
    setBusy(true)
    const ticket = generation.current
    try {
      const detected = await environmentQuery.refetch()
      if (detected.isError || !detected.data) throw detected.error
      const expected = detected.data
      const probe = await client.fetchQuery({ queryKey: probeKey(config, expected), queryFn: () => probeSettings(config, expected), staleTime: 0 })
      if (probe !== 'ok') throw new EnvironmentSessionError()
      const snapshot = await client.fetchQuery({ queryKey: valuesKey(config, expected), queryFn: ({ signal }) => readSnapshot(expected, signal), staleTime: 0 })
      if (!mounted.current || ticket !== generation.current || !environmentIsReady(client, expected)) return false
      bind(expected, snapshot)
      return true
    } catch (error) {
      if (mounted.current && ticket === generation.current) reportError(error)
      return false
    } finally {
      busyRef.current = false
      if (mounted.current) setBusy(false)
    }
  }, [bind, client, config, environmentQuery, readSnapshot, reportError])

  const save = useCallback(async (values: SettingsValues) => {
    const origin = sessionRef.current
    if (!origin || saveBlocked({ pending: busyRef.current, current: isCurrent(), reloadRequired, result: saveResult, loading: !snapshotReady })) return
    if (snapshotChanged(origin.snapshot, valuesQuery.data)) return
    const dirtyKeys = Object.keys(form.formState.dirtyFields)
    if (dirtyKeys.length === 0) return
    busyRef.current = true
    setBusy(true)
    const ticket = generation.current
    try {
      await assertCurrent()
      const revision = environmentRevision(client)
      const result = await saveSettingsValues(config, { values, dirtyKeys, snapshot: origin.snapshot, environmentId: origin.environment.id })
      if (revision !== environmentRevision(client)) return
      if (!isCurrent() || ticket !== generation.current) return
      setSaveResult(result)
      if (result.status === 'unsupported_environment') setActionError(t('settingsRaw.unsupportedEnvironment'))
      if (result.status !== 'saved') return
      setReloadRequired(true)
      config.notify.success(t(config.i18nPrefix + '.messages.saveSuccess'))
      const snapshot = await readSnapshot(origin.environment)
      if (!isCurrent() || ticket !== generation.current) return
      client.setQueryData<SettingsSnapshot>(valuesKey(config, origin.environment), snapshot)
      bind(origin.environment, snapshot)
    } catch (error) {
      if (mounted.current && ticket === generation.current) reportError(error)
    } finally {
      busyRef.current = false
      if (mounted.current) setBusy(false)
    }
  }, [assertCurrent, bind, client, config, form.formState.dirtyFields, isCurrent, readSnapshot, reloadRequired, reportError, saveResult, snapshotReady, t, valuesQuery.data])

  const returnToEnvironment = useCallback(async () => {
    const origin = sessionRef.current
    if (!origin || busyRef.current) return
    busyRef.current = true
    setBusy(true)
    try {
      await switchEnvironment(origin.environment.id)
      const result = await environmentQuery.refetch()
      if (result.isError) throw result.error
    } catch (error) { reportError(error) } finally {
      busyRef.current = false
      if (mounted.current) setBusy(false)
    }
  }, [environmentQuery, reportError])

  const frozen = Boolean(session && (!ready || !sameEnvironment(environment, session.environment)))
  const serverChanged = Boolean(session && !frozen && snapshotChanged(session.snapshot, valuesQuery.data))
  return {
    session, saveResult, busy, frozen, serverChanged, reloadRequired, actionError, snapshotReady,
    environmentQuery, probeQuery, valuesQuery, save, reload, returnToEnvironment, isCurrent, assertCurrent,
  }
}

function saveBlocked(state: { pending: boolean; current: boolean; reloadRequired: boolean; result: SettingsSaveResult | null; loading: boolean }) {
  return state.pending || !state.current || state.reloadRequired || state.loading
    || state.result?.status === 'conflict' || state.result?.status === 'unsupported_environment'
}

function snapshotChanged(baseline: SettingsSnapshot, latest: SettingsSnapshot | undefined) {
  return Boolean(latest && replaceEqualDeep(baseline, latest) !== baseline)
}
