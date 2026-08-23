import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { getCurrentEnvironment } from '@/api'
import {
  createSystemPrompt,
  getSystemPrompt,
  listSystemPrompts,
  saveSystemPrompt,
  type SystemPromptFile,
  type SystemPromptRule,
} from '@/api/domains/systemPrompts'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { PlatformSubnav } from '@/features/platform/PlatformSubnav'
import { useAppLocale, useResolvedT } from '@/i18n'
import { AsyncStatePanel, PageHeader, PageShell, SIcon } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'
import { PromptWorkspace } from './PromptWorkspace'
import { systemPromptsConfigs, type SystemPromptsConfig } from './systemPromptsConfig'

interface PromptForm {
  content: string
}

interface SystemPromptsViewProps {
  config: SystemPromptsConfig
  t?: TranslateFunction
}

const SIZE_WARN = 64 * 1024

export function SystemPromptsView({ config, t: tProp }: SystemPromptsViewProps) {
  const t = useResolvedT(tProp)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [creatingId, setCreatingId] = useState<string | null>(null)
  const [unsupported, setUnsupported] = useState(false)
  const [files, setFiles] = useState<SystemPromptFile[]>([])
  const [rules, setRules] = useState<SystemPromptRule[]>([])
  const [selected, setSelected] = useState<SystemPromptFile | null>(null)
  const [token, setToken] = useState('')
  const [baseline, setBaseline] = useState('')
  const [conflict, setConflict] = useState(false)
  const [sizeWarning, setSizeWarning] = useState(false)
  const form = useForm<PromptForm>({ defaultValues: { content: '' } })
  const { register, reset, watch } = form
  const content = watch('content')
  const dirty = content !== baseline
  const busy = loading || saving || creatingId !== null
  const locale = useAppLocale()

  const formatTime = useCallback(
    (timestamp: number) =>
      new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(
        new Date(timestamp),
      ),
    [locale],
  )

  const loadList = useCallback(async () => {
    const result = await listSystemPrompts(config.platform)
    if (result.status === 'unsupported_environment') {
      setUnsupported(true)
      setFiles([])
      setRules([])
      return
    }
    setUnsupported(false)
    setFiles(result.files)
    setRules(result.rules)
    setSelected((current) => current ? result.files.find((file) => file.id === current.id) ?? null : current)
  }, [config.platform])

  const loadSelected = useCallback(
    async (file: SystemPromptFile) => {
      const result = await getSystemPrompt(config.platform, file.id)
      if (result.status === 'unsupported_environment') {
        setUnsupported(true)
        return
      }
      setSelected({ ...file, exists: result.exists, path: result.path })
      reset({ content: result.content })
      setBaseline(result.content)
      setToken(result.token)
      setConflict(false)
      setSizeWarning(result.content.length > SIZE_WARN)
    },
    [config.platform, reset],
  )

  const confirmDiscard = useCallback(async () => {
    if (!dirty) return true
    return surfaceNotify.confirm({
      title: t('systemPrompts.discardTitle'),
      message: t('systemPrompts.discardMessage'),
      confirmText: t('systemPrompts.discard'),
      cancelText: t('common.cancel'),
      type: 'warning',
      surface: 'solid',
    })
  }, [dirty, t])

  const selectFile = useCallback(
    async (file: SystemPromptFile) => {
      if (!file.exists || file.id === selected?.id) return
      if (!(await confirmDiscard())) return
      try {
        await loadSelected(file)
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
      }
    },
    [confirmDiscard, loadSelected, selected?.id, t],
  )

  const createFile = useCallback(
    async (file: SystemPromptFile) => {
      if (creatingId) return
      setCreatingId(file.id)
      try {
        const result = await createSystemPrompt(config.platform, file.id)
        if (result.status === 'unsupported_environment') {
          setUnsupported(true)
          return
        }
        await loadList()
        const created = (await listSystemPrompts(config.platform))
        const next = created.status === 'ok' ? created.files.find((item) => item.id === file.id) : undefined
        if (next) await loadSelected(next)
        if (result.status !== 'conflict') surfaceNotify.success(t('systemPrompts.createSuccess'))
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.createFailed')}: ${String(error)}`)
      } finally {
        setCreatingId(null)
      }
    },
    [config.platform, creatingId, loadList, loadSelected, t],
  )

  const reloadSelected = useCallback(async () => {
    if (!selected || !(await confirmDiscard())) return
    try {
      await loadSelected(selected)
    } catch (error) {
      surfaceNotify.error(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
    }
  }, [confirmDiscard, loadSelected, selected, t])

  const handleSave = useCallback(async () => {
    if (!selected || !dirty || saving) return
    setSaving(true)
    setConflict(false)
    try {
      const result = await saveSystemPrompt(config.platform, selected.id, content, token)
      if (result.status === 'unsupported_environment') {
        setUnsupported(true)
        return
      }
      if (result.status === 'conflict') {
        setConflict(true)
        return
      }
      setToken(result.token)
      setBaseline(content)
      setSizeWarning(result.warning === 'size')
      surfaceNotify.success(t('systemPrompts.saveSuccess'))
      await loadList()
    } catch (error) {
      surfaceNotify.error(`${t('systemPrompts.saveFailed')}: ${String(error)}`)
    } finally {
      setSaving(false)
    }
  }, [config.platform, content, dirty, loadList, saving, selected, t, token])

  useEffect(() => {
    void (async () => {
      try {
        const environment = await getCurrentEnvironment()
        if (environment && environment.env_type !== 'local') {
          setUnsupported(true)
          return
        }
        await loadList()
        const result = await listSystemPrompts(config.platform)
        if (result.status !== 'ok') return
        const first = result.files.find((file) => file.exists)
        if (first) await loadSelected(first)
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
      } finally {
        setLoading(false)
      }
    })()
  }, [config.platform, loadList, loadSelected, t])

  const header = (
    <PageHeader
      title={t('systemPrompts.title')}
      eyebrow={t(`systemPrompts.platforms.${config.platform}`)}
      description={t('systemPrompts.description')}
      leading={<SIcon name="ScrollText" size="w-8 h-8" />}
    />
  )

  const note = useMemo(() => {
    if (config.features.hierarchyNote) {
      return { icon: 'Layers', title: t('systemPrompts.claudeHierarchyTitle'), body: t('systemPrompts.claudeHierarchyDescription') }
    }
    if (config.features.geminiNote) {
      return { icon: 'CircleAlert', title: null, body: t('systemPrompts.antigravityNote') }
    }
    return null
  }, [config.features.geminiNote, config.features.hierarchyNote, t])

  if (unsupported) {
    return (
      <PageShell header={header} subnav={<PlatformSubnav module={config.module} />}>
        <AsyncStatePanel state="runtime-unavailable" title={t('settingsRaw.unsupportedEnvironment')} />
      </PageShell>
    )
  }

  return (
    <PageShell header={header} subnav={<PlatformSubnav module={config.module} />}>
      {note ? (
        <section className="mb-4 flex items-center gap-3 rounded-md border border-border-subtle bg-bg-elevated px-4 py-3 text-text-secondary">
          <SIcon name={note.icon} size="w-5 h-5" />
          <div>
            {note.title ? <strong className="block text-text-primary">{note.title}</strong> : null}
            <p className="m-0 text-sm text-text-muted">{note.body}</p>
          </div>
        </section>
      ) : null}
      <PromptWorkspace
        loading={loading}
        files={files}
        rules={rules}
        selected={selected}
        busy={busy}
        creatingId={creatingId}
        dirty={dirty}
        saving={saving}
        conflict={conflict}
        sizeWarning={sizeWarning}
        showLimitHint={Boolean(config.features.limitHint)}
        showRules={Boolean(config.features.showRules)}
        t={t}
        formatTime={formatTime}
        register={register}
        onSelect={selectFile}
        onCreate={createFile}
        onReload={reloadSelected}
        onSave={handleSave}
      />
    </PageShell>
  )
}

export { systemPromptsConfigs }
export type { SystemPromptsConfig, SystemPromptPlatform } from './systemPromptsConfig'
