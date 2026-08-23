import { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { grokApi } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { GrokActivationDto, GrokProfileActionResponse, GrokProfileDto } from '@/types'
import { getErrorMessage } from '@/types/api'
import { downloadTextFile } from '@/utils/download'
import {
  buildGrokCreateRequest,
  buildGrokPatch,
  createEmptyGrokForm,
  fillGrokForm,
  type GrokProfileDirtyField,
  type GrokProfileEditorForm,
} from '@/utils/grokProfileEditor'
import { fetchGrokEnvironment, grokKeys } from '../queries'
import { t } from '../locale'

const dirtyKeysOf = (fields: object): Set<GrokProfileDirtyField> =>
  new Set(Object.keys(fields) as GrokProfileDirtyField[])

function readProfilesSnapshot(list: Awaited<ReturnType<typeof grokApi.listGrokProfiles>> | undefined) {
  if (!list) {
    return {
      unsupported: false,
      profiles: [] as GrokProfileDto[],
      currentProfile: null as string | null,
      activation: 'inactive' as GrokActivationDto,
    }
  }
  if (list.status === 'unsupported_environment') {
    return { unsupported: true, profiles: [] as GrokProfileDto[], currentProfile: null, activation: 'inactive' as GrokActivationDto }
  }
  return {
    unsupported: false,
    profiles: list.profiles,
    currentProfile: list.current_profile,
    activation: list.activation,
  }
}

async function runProfileRecovery(pending: {
  status: 'rename_apply_failed' | 'rename_cleanup_failed'
  oldName: string
  newName: string
}) {
  if (pending.status === 'rename_apply_failed') {
    const response = await grokApi.applyGrokProfile(pending.newName)
    if (response.status === 'unsupported_environment') return
    if (response.status !== 'applied') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
    return
  }
  const response = await grokApi.deleteGrokProfile(pending.oldName)
  if (response.status === 'unsupported_environment') return
  if (response.status !== 'deleted') throw new Error(t('grok.profiles.messages.deleteFailed'))
}

export function useGrokProfilesPage() {
  const probeQuery = useQuery({
    queryKey: grokKeys.environment(),
    queryFn: fetchGrokEnvironment,
    staleTime: 0,
  })
  const localOnly = Boolean(probeQuery.data && probeQuery.data.env_type !== 'local')
  const localOnlyEnvType = localOnly ? probeQuery.data?.env_type ?? null : null
  const listQuery = useQuery({
    queryKey: grokKeys.profiles(),
    queryFn: grokApi.listGrokProfiles,
    enabled: probeQuery.isSuccess && !localOnly,
  })

  const [saving, setSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [editingName, setEditingName] = useState<string | null>(null)
  const [editingProfile, setEditingProfile] = useState<GrokProfileDto | null>(null)
  const [recovery, setRecovery] = useState<{
    status: 'rename_apply_failed' | 'rename_cleanup_failed'
    oldName: string
    newName: string
    message: string
  } | null>(null)
  const form = useForm<GrokProfileEditorForm>({ defaultValues: createEmptyGrokForm() })

  const list = listQuery.data
  const snapshot = useMemo(() => readProfilesSnapshot(list), [list])
  const { unsupported, profiles, currentProfile, activation } = snapshot

  const actionUnsupported = useCallback((response: GrokProfileActionResponse) => {
    return response.status === 'unsupported_environment'
  }, [])

  const reload = useCallback(async () => {
    await listQuery.refetch()
  }, [listQuery])

  const closeForm = useCallback(() => {
    setShowForm(false)
    setEditingName(null)
    setEditingProfile(null)
    setSaveError(null)
    form.reset(createEmptyGrokForm())
  }, [form])

  const handleAdd = useCallback(() => {
    if (localOnly) return
    form.reset(createEmptyGrokForm())
    setEditingName(null)
    setEditingProfile(null)
    setSaveError(null)
    setShowForm(true)
  }, [form, localOnly])

  const handleEdit = useCallback(
    (name: string) => {
      if (localOnly) return
      const profile = profiles.find((item) => item.name === name)
      if (!profile) return
      form.reset(fillGrokForm(profile))
      setEditingName(name)
      setEditingProfile(profile)
      setSaveError(null)
      setShowForm(true)
    },
    [form, localOnly, profiles],
  )

  const handleSave = useCallback(async () => {
    setSaving(true)
    setSaveError(null)
    const values = form.getValues()
    const previousName = editingName
    try {
      const response = previousName
        ? await grokApi.updateGrokProfile(previousName, buildGrokPatch(values, dirtyKeysOf(form.formState.dirtyFields)))
        : await grokApi.addGrokProfile(buildGrokCreateRequest(values))
      if (actionUnsupported(response)) return
      if (response.status === 'rename_apply_failed' || response.status === 'rename_cleanup_failed') {
        setRecovery({
          status: response.status,
          oldName: response.old_name,
          newName: response.new_name,
          message: response.message,
        })
        surfaceNotify.warning(response.message)
      } else if (response.status !== 'created' && response.status !== 'updated' && response.status !== 'renamed') {
        throw new Error(t('grok.profiles.messages.unexpectedResponse'))
      } else {
        setRecovery(null)
        surfaceNotify.success(
          previousName ? t('grok.profiles.messages.updateSuccess') : t('grok.profiles.messages.createSuccess'),
        )
      }
      closeForm()
      await reload()
    } catch (error) {
      setSaveError(getErrorMessage(error, t('grok.profiles.messages.saveFailed')))
    } finally {
      setSaving(false)
    }
  }, [actionUnsupported, closeForm, editingName, form, reload])

  const handleApply = useCallback(
    async (name: string) => {
      if (localOnly) return
      const target = profiles.find((profile) => profile.name === name)
      if (!target || !target.enabled || target.name === currentProfile) return
      const ok = await surfaceNotify.confirm({
        title: t('grok.profiles.confirm.applyTitle'),
        message: t('grok.profiles.confirm.applyMessage', { name }),
        confirmText: t('grok.profiles.actions.apply'),
        type: 'warning',
      })
      if (!ok) return
      try {
        const response = await grokApi.applyGrokProfile(name)
        if (actionUnsupported(response)) return
        if (response.status !== 'applied') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
        surfaceNotify.success(t('grok.profiles.messages.applySuccess', { name }))
        await reload()
      } catch (error) {
        surfaceNotify.error(getErrorMessage(error, t('grok.profiles.messages.applyFailed')))
      }
    },
    [actionUnsupported, currentProfile, localOnly, profiles, reload],
  )

  const handleOff = useCallback(async () => {
    if (localOnly) return
    const ok = await surfaceNotify.confirm({
      title: t('grok.profiles.confirm.offTitle'),
      message:
        activation === 'drifted'
          ? t('grok.profiles.confirm.offDriftedMessage')
          : t('grok.profiles.confirm.offMessage'),
      confirmText: t('grok.profiles.actions.off'),
      type: 'warning',
    })
    if (!ok) return
    try {
      const response = await grokApi.grokProfileOff()
      if (actionUnsupported(response)) return
      if (response.status !== 'off') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
      surfaceNotify.success(t('grok.profiles.messages.offSuccess'))
      await reload()
    } catch (error) {
      surfaceNotify.error(getErrorMessage(error, t('grok.profiles.messages.offFailed')))
    }
  }, [actionUnsupported, activation, localOnly, reload])

  const deleteProfile = useCallback(
    async (name: string, force = false) => {
      const response = await grokApi.deleteGrokProfile(name, { force })
      if (actionUnsupported(response)) return
      if (response.status === 'deleted') {
        surfaceNotify.success(t('grok.profiles.messages.deleteSuccess', { name }))
        await reload()
        return
      }
      if (response.status !== 'blocked') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
      if (force) throw new Error(response.message)
      const confirmed = await surfaceNotify.confirm({
        title: t('grok.profiles.confirm.forceDeleteTitle'),
        message: t('grok.profiles.confirm.forceDeleteMessage', { name }),
        confirmText: t('grok.profiles.confirm.forceDeleteAction'),
        type: 'danger',
      })
      if (confirmed) await deleteProfile(name, true)
    },
    [actionUnsupported, reload],
  )

  const handleDelete = useCallback(
    async (name: string) => {
      if (localOnly) return
      const ok = await surfaceNotify.confirm({
        title: t('grok.profiles.confirm.deleteTitle'),
        message: t('grok.profiles.confirm.deleteMessage', { name }),
        confirmText: t('grok.profiles.actions.delete'),
        type: 'danger',
      })
      if (!ok) return
      try {
        await deleteProfile(name)
      } catch (error) {
        surfaceNotify.error(getErrorMessage(error, t('grok.profiles.messages.deleteFailed')))
      }
    },
    [deleteProfile, localOnly],
  )

  const handleToggle = useCallback(
    async (name: string, enabled: boolean) => {
      if (localOnly) return
      const ok = await surfaceNotify.confirm({
        title: t(enabled ? 'grok.profiles.confirm.enableTitle' : 'grok.profiles.confirm.disableTitle'),
        message: t(enabled ? 'grok.profiles.confirm.enableMessage' : 'grok.profiles.confirm.disableMessage', { name }),
        confirmText: t(enabled ? 'grok.profiles.actions.enable' : 'grok.profiles.actions.disable'),
        type: enabled ? 'info' : 'warning',
      })
      if (!ok) return
      try {
        const response = await grokApi.updateGrokProfile(name, { enabled })
        if (actionUnsupported(response)) return
        if (response.status !== 'updated') throw new Error(t('grok.profiles.messages.unexpectedResponse'))
        await reload()
      } catch (error) {
        surfaceNotify.error(getErrorMessage(error, t('grok.profiles.messages.saveFailed')))
      }
    },
    [actionUnsupported, localOnly, reload],
  )

  const handleExport = useCallback(() => {
    if (localOnly) return
    downloadTextFile(
      'grok-profiles-summary.json',
      `${JSON.stringify({ activation, current_profile: currentProfile, profiles }, null, 2)}\n`,
      'application/json;charset=utf-8',
    )
    surfaceNotify.success(t('grok.profiles.messages.exportSuccess'))
  }, [activation, currentProfile, localOnly, profiles])

  const runRecovery = useCallback(async () => {
    if (!recovery) return
    try {
      await runProfileRecovery(recovery)
      setRecovery(null)
      surfaceNotify.success(t('grok.profiles.messages.recoverySuccess'))
      await reload()
    } catch (error) {
      surfaceNotify.error(getErrorMessage(error, t('grok.profiles.messages.recoveryFailed')))
    }
  }, [recovery, reload])

  return {
    localOnly: localOnly || Boolean(unsupported),
    localOnlyEnvType: unsupported && list && 'env_type' in list ? list.env_type : localOnlyEnvType,
    loading: probeQuery.isPending || listQuery.isPending,
    profiles,
    currentProfile,
    activation,
    saving,
    saveError,
    showForm,
    editingName,
    editingProfile,
    recovery,
    form,
    handleAdd,
    handleEdit,
    handleSave,
    handleApply,
    handleOff,
    handleDelete,
    handleToggle,
    handleExport,
    closeForm,
    runRecovery,
    reload,
  }
}
