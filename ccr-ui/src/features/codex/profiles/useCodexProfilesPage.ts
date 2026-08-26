import { useCallback, useMemo, useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import {
  applyCodexProfile,
  deleteCodexProfile,
  exportCodexProfiles,
  getCurrentEnvironment,
  listCodexProfiles,
  updateCodexProfile,
} from '@/api'
import {
  codexProfileOff,
  getCodexProfilesRaw,
  saveCodexProfilesRaw,
} from '@/api/domains/codex'
import { CODEX_SECRET_KEYS, stripCredentials } from '@/configs/profileCredentials'
import { codexProfilePresentation } from '@/configs/profilePresentation'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { translate } from '@/i18n'
import type { CodexProfile } from '@/types'
import { getErrorMessage } from '@/types/api'
import { downloadTextFile } from '@/utils/download'

const EMPTY_PROFILES: CodexProfile[] = []
const QUERY_KEY = ['platform-profiles', 'profiles-codex'] as const

const t = translate

type ConfirmType = 'warning' | 'danger' | 'info'

const confirmRun = async (input: {
  title: string
  message: string
  confirmText?: string
  type: ConfirmType
  run: () => Promise<void>
}): Promise<void> => {
  const ok = await surfaceNotify.confirm({
    title: input.title,
    message: input.message,
    confirmText: input.confirmText,
    type: input.type,
  })
  if (!ok) return
  await input.run()
}

const loadCodexProfiles = async () => {
  const payload = await listCodexProfiles()
  return {
    profiles: payload.profiles.map((item) => stripCredentials(item, CODEX_SECRET_KEYS)),
    current_profile: payload.current_profile ?? null,
    can_off: payload.can_off === true,
  }
}

/** Codex Profiles 列表控制器：剥离 → 投影，不含呈现层。 */
export function useCodexProfilesPage() {
  const queryClient = useQueryClient()
  const envQuery = useQuery({
    queryKey: ['current-environment'],
    queryFn: getCurrentEnvironment,
  })
  const listQuery = useQuery({
    queryKey: QUERY_KEY,
    queryFn: loadCodexProfiles,
  })
  const [editorOpen, setEditorOpen] = useState(false)
  const [editorTarget, setEditorTarget] = useState<CodexProfile | null>(null)
  const [originalName, setOriginalName] = useState<string | null>(null)

  const current = listQuery.data?.current_profile ?? null
  const profiles = listQuery.data?.profiles ?? EMPTY_PROFILES
  const records = useMemo(
    () => profiles.map((item) => codexProfilePresentation.project(item, { current })),
    [current, profiles],
  )
  const existingNames = useMemo(() => records.map((item) => item.name), [records])
  const environmentOk = !envQuery.data || envQuery.data.env_type === 'local'
  const environmentLabel = t(
    environmentOk ? 'profilesSurface.environmentLocal' : 'profilesSurface.environmentRemote',
  )
  const error = listQuery.error
    ? getErrorMessage(listQuery.error, t('codex.profiles.messages.loadFailed'))
    : null

  const onReload = useCallback(() => {
    void listQuery.refetch()
  }, [listQuery])

  const refreshAll = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: QUERY_KEY })
  }, [queryClient])

  const closeEditor = useCallback(() => {
    setEditorOpen(false)
    setEditorTarget(null)
    setOriginalName(null)
  }, [])

  const onAdd = useCallback(() => {
    setEditorTarget(null)
    setOriginalName(null)
    setEditorOpen(true)
  }, [])

  const onEdit = useCallback(
    (name: string) => {
      const target = profiles.find((item) => item.name === name) ?? null
      setEditorTarget(target)
      setOriginalName(target ? name : null)
      setEditorOpen(Boolean(target))
    },
    [profiles],
  )

  const onApply = useCallback(
    async (name: string) => {
      if (name === current) return
      try {
        await confirmRun({
          title: t('codex.profiles.apply'),
          message: t('codex.profiles.confirmApply', { name }),
          type: 'warning',
          run: async () => {
            await applyCodexProfile(name)
            surfaceNotify.success(t('common.success'))
            await listQuery.refetch()
          },
        })
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('codex.profiles.messages.operationFailed')))
      }
    },
    [current, listQuery],
  )

  const onDelete = useCallback(
    async (name: string) => {
      try {
        await confirmRun({
          title: t('codex.profiles.deleteConfirmShort', { name }),
          message: t('codex.profiles.deleteConfirm', { name }),
          confirmText: t('profilesSurface.delete'),
          type: 'danger',
          run: async () => {
            await deleteCodexProfile(name)
            surfaceNotify.success(t('codex.profiles.messages.deleteSuccess'))
            await listQuery.refetch()
          },
        })
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('codex.profiles.messages.deleteFailed')))
      }
    },
    [listQuery],
  )

  const onToggle = useCallback(
    async (name: string, enabled: boolean) => {
      try {
        await confirmRun({
          title: t(enabled ? 'profilesSurface.enable' : 'profilesSurface.stop'),
          message: name,
          confirmText: t(enabled ? 'profilesSurface.enable' : 'profilesSurface.stop'),
          type: enabled ? 'info' : 'warning',
          run: async () => {
            await updateCodexProfile(name, { enabled })
            await listQuery.refetch()
          },
        })
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('codex.profiles.messages.operationFailed')))
      }
    },
    [listQuery],
  )

  const onOff = useCallback(async () => {
    try {
      await confirmRun({
        title: t('codex.profiles.confirm.offTitle'),
        message: t('codex.profiles.confirm.offMessage'),
        confirmText: t('codex.profiles.actions.off'),
        type: 'warning',
        run: async () => {
          await codexProfileOff()
          surfaceNotify.success(t('codex.profiles.messages.offSuccess'))
          await listQuery.refetch()
        },
      })
    } catch (caught) {
      surfaceNotify.error(getErrorMessage(caught, t('codex.profiles.messages.offFailed')))
    }
  }, [listQuery])

  const onExport = useCallback(() => {
    void (async () => {
      try {
        const exported = await exportCodexProfiles(false)
        downloadTextFile(exported.filename, exported.content)
        surfaceNotify.success(t('codex.profiles.messages.exportSuccess'))
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('codex.profiles.messages.exportFailed')))
      }
    })()
  }, [])

  const handleEditorDone = useCallback(
    (outcome: { status: string }) => {
      if (outcome.status === 'ok') void listQuery.refetch()
    },
    [listQuery],
  )

  const rawSource = useMemo(
    () => ({
      getRaw: getCodexProfilesRaw,
      saveRaw: saveCodexProfilesRaw,
      refreshAll,
    }),
    [refreshAll],
  )

  return {
    records,
    current,
    canOff: listQuery.data?.can_off === true,
    loading: listQuery.isPending,
    error,
    unavailable: false,
    environmentLabel,
    environmentOk,
    rawSource,
    editorOpen,
    editorTarget,
    originalName,
    existingNames,
    onAdd,
    onEdit,
    onApply,
    onDelete,
    onToggle,
    onOff,
    onExport,
    onReload,
    closeEditor,
    handleEditorDone,
  }
}
