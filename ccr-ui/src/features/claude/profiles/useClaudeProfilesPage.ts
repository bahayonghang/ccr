import { useCallback, useMemo, useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import {
  applyClaudeProfile,
  deleteClaudeProfile,
  exportClaudeProfiles,
  getCurrentEnvironment,
  listClaudeProfiles,
  updateClaudeProfile,
} from '@/api'
import {
  claudeProfileOff,
  getClaudeProfilesRaw,
  saveClaudeProfilesRaw,
} from '@/api/domains/claude'
import { CLAUDE_SECRET_KEYS, stripCredentials } from '@/configs/profileCredentials'
import { claudeProfilePresentation } from '@/configs/profilePresentation'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { translate } from '@/i18n'
import type { ClaudeProfile } from '@/types'
import { getErrorMessage } from '@/types/api'
import { downloadTextFile } from '@/utils/download'

const EMPTY_PROFILES: ClaudeProfile[] = []
const QUERY_KEY = ['platform-profiles', 'profiles-claude'] as const

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

const loadClaudeProfiles = async () => {
  const payload = await listClaudeProfiles()
  return {
    profiles: payload.profiles.map((item) => stripCredentials(item, CLAUDE_SECRET_KEYS)),
    current_profile: payload.current_profile,
    can_off: payload.can_off === true,
  }
}

/** Claude Profiles 列表控制器：剥离 → 投影，不含呈现层。 */
export function useClaudeProfilesPage() {
  const queryClient = useQueryClient()
  const envQuery = useQuery({
    queryKey: ['current-environment'],
    queryFn: getCurrentEnvironment,
  })
  const listQuery = useQuery({
    queryKey: QUERY_KEY,
    queryFn: loadClaudeProfiles,
  })
  const [editorOpen, setEditorOpen] = useState(false)
  const [editorTarget, setEditorTarget] = useState<ClaudeProfile | null>(null)
  const [originalName, setOriginalName] = useState<string | null>(null)

  const current = listQuery.data?.current_profile ?? null
  const profiles = listQuery.data?.profiles ?? EMPTY_PROFILES
  const records = useMemo(
    () => profiles.map((item) => claudeProfilePresentation.project(item, { current })),
    [current, profiles],
  )
  const existingNames = useMemo(() => records.map((item) => item.name), [records])
  const environmentOk = !envQuery.data || envQuery.data.env_type === 'local'
  const environmentLabel = t(
    environmentOk ? 'profilesSurface.environmentLocal' : 'profilesSurface.environmentRemote',
  )
  const error = listQuery.error
    ? getErrorMessage(listQuery.error, t('claudeProfiles.loadFailed'))
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
      const currentName = listQuery.data?.current_profile
      if (name === currentName) return
      try {
        await confirmRun({
          title: t('claudeProfiles.applyProfile'),
          message: t('claudeProfiles.confirmApply', { name }),
          type: 'warning',
          run: async () => {
            await applyClaudeProfile(name)
            surfaceNotify.success(t('common.success'))
            await listQuery.refetch()
          },
        })
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('claudeProfiles.applyFailed')))
      }
    },
    [listQuery],
  )

  const onDelete = useCallback(
    async (name: string) => {
      try {
        await confirmRun({
          title: t('claudeProfiles.deleteTooltip'),
          message: t('claudeProfiles.deleteConfirm', { name }),
          confirmText: t('profilesSurface.delete'),
          type: 'danger',
          run: async () => {
            await deleteClaudeProfile(name)
            await listQuery.refetch()
          },
        })
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('claudeProfiles.deleteFailed')))
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
            await updateClaudeProfile(name, { enabled })
            await listQuery.refetch()
          },
        })
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('claudeProfiles.operationFailed')))
      }
    },
    [listQuery],
  )

  const onOff = useCallback(async () => {
    try {
      await confirmRun({
        title: t('claudeProfiles.confirm.offTitle'),
        message: t('claudeProfiles.confirm.offMessage'),
        confirmText: t('claudeProfiles.actions.off'),
        type: 'warning',
        run: async () => {
          await claudeProfileOff()
          surfaceNotify.success(t('claudeProfiles.messages.offSuccess'))
          await listQuery.refetch()
        },
      })
    } catch (caught) {
      surfaceNotify.error(getErrorMessage(caught, t('claudeProfiles.messages.offFailed')))
    }
  }, [listQuery])

  const onExport = useCallback(() => {
    void (async () => {
      try {
        const exported = await exportClaudeProfiles(false)
        downloadTextFile(exported.filename, exported.content)
        surfaceNotify.success(t('claudeProfiles.exportSuccess'))
      } catch (caught) {
        surfaceNotify.error(getErrorMessage(caught, t('claudeProfiles.exportFailed')))
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
      getRaw: getClaudeProfilesRaw,
      saveRaw: saveClaudeProfilesRaw,
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
