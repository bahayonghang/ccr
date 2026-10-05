import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { getCurrentEnvironment } from '@/api'
import { createSystemPrompt, getSystemPrompt, listSystemPrompts, saveSystemPrompt, type SystemPromptFile } from '@/api/domains/systemPrompts'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { AsyncStatePanel, PageHeader, PageShell, SIcon, buttonClass } from '@/ui'
import { CodexSubnav } from './CodexSubnav'
import { fieldInputClass, panelCardClass } from './ui-classes'
import { useCodexLocale } from './useCodexLocale'

export function CodexSystemPromptsView() {
  const { t } = useCodexLocale()
  const [loading, setLoading] = useState(true)
  const [unsupported, setUnsupported] = useState(false)
  const [files, setFiles] = useState<SystemPromptFile[]>([])
  const [selected, setSelected] = useState<SystemPromptFile | null>(null)
  const [token, setToken] = useState('')
  const [baseline, setBaseline] = useState('')
  const [conflict, setConflict] = useState(false)
  const form = useForm({ defaultValues: { content: '' } })
  const content = form.watch('content')
  const dirty = content !== baseline

  const loadList = useCallback(async () => {
    const result = await listSystemPrompts('codex')
    if (result.status === 'unsupported_environment') {
      setUnsupported(true)
      setFiles([])
      return
    }
    setUnsupported(false)
    setFiles(result.files)
  }, [])

  const loadSelected = useCallback(async (file: SystemPromptFile) => {
    const result = await getSystemPrompt('codex', file.id)
    if (result.status === 'unsupported_environment') {
      setUnsupported(true)
      return
    }
    setSelected({ ...file, exists: result.exists, path: result.path })
    form.reset({ content: result.content })
    setBaseline(result.content)
    setToken(result.token)
    setConflict(false)
  }, [form])

  useEffect(() => {
    void (async () => {
      try {
        const environment = await getCurrentEnvironment()
        if (environment && environment.env_type !== 'local') {
          setUnsupported(true)
          return
        }
        await loadList()
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
      } finally {
        setLoading(false)
      }
    })()
  }, [loadList, t])

  const handleSelect = useCallback(
    async (file: SystemPromptFile) => {
      if (!file.exists || file.id === selected?.id) return
      if (dirty) {
        const ok = await surfaceNotify.confirm({
          title: t('systemPrompts.discardTitle'),
          message: t('systemPrompts.discardMessage'),
          confirmText: t('systemPrompts.discard'),
          cancelText: t('common.cancel'),
          type: 'warning',
        })
        if (!ok) return
      }
      try {
        await loadSelected(file)
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
      }
    },
    [dirty, loadSelected, selected?.id, t],
  )

  const handleCreate = useCallback(
    async (file: SystemPromptFile) => {
      try {
        const result = await createSystemPrompt('codex', file.id)
        if (result.status === 'unsupported_environment') {
          setUnsupported(true)
          return
        }
        surfaceNotify.success(t('systemPrompts.createSuccess'))
        await loadList()
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.createFailed')}: ${String(error)}`)
      }
    },
    [loadList, t],
  )

  const handleSave = useCallback(async () => {
    if (!selected || !dirty) return
    try {
      const result = await saveSystemPrompt('codex', selected.id, content, token)
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
      surfaceNotify.success(t('systemPrompts.saveSuccess'))
      await loadList()
    } catch (error) {
      surfaceNotify.error(`${t('systemPrompts.saveFailed')}: ${String(error)}`)
    }
  }, [content, dirty, loadList, selected, t, token])

  if (loading) {
    return <PageShell header={<PageHeader title={t('systemPrompts.title')} />} subnav={<CodexSubnav />}><AsyncStatePanel state="loading" title={t('systemPrompts.loading')} /></PageShell>
  }
  if (unsupported) {
    return <PageShell header={<PageHeader title={t('systemPrompts.title')} />} subnav={<CodexSubnav />}><AsyncStatePanel state="runtime-unavailable" title={t('settingsRaw.unsupportedEnvironment')} /></PageShell>
  }

  return (
    <PageShell
      header={<PageHeader title={t('systemPrompts.title')} eyebrow={t('systemPrompts.platforms.codex')} description={t('systemPrompts.description')} leading={<SIcon name="ScrollText" size="w-8 h-8" />} />}
      subnav={<CodexSubnav />}
    >
      <div className="grid gap-4 xl:grid-cols-[20rem_minmax(0,1fr)]">
        <aside className={panelCardClass}>
          <h2 className="mb-3 text-base font-semibold">{t('systemPrompts.filesTitle')}</h2>
          <div className="space-y-2">
            {files.map((file) => (
              <PromptFileRow key={file.id} file={file} active={selected?.id === file.id} t={t} onSelect={handleSelect} onCreate={handleCreate} />
            ))}
          </div>
        </aside>
        <section className={panelCardClass}>
          {!selected ? (
            <p className="text-sm text-text-muted">{t('systemPrompts.emptySelection')}</p>
          ) : (
            <div className="space-y-3">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <strong>{t(selected.labelKey)}</strong>
                  <code className="ml-2 text-xs text-text-muted">{selected.path}</code>
                </div>
                <button type="button" className={buttonClass({ variant: 'primary' })} disabled={!dirty} onClick={handleSave}>
                  {t('systemPrompts.save')}
                </button>
              </div>
              {conflict ? <p className="text-sm text-accent-warning">{t('systemPrompts.conflictMessage')}</p> : null}
              {selected.limitHint ? <p className="text-sm text-text-muted">{t('systemPrompts.codexLimit')}</p> : null}
              <textarea rows={24} className={`${fieldInputClass} font-mono`} {...form.register('content')} />
            </div>
          )}
        </section>
      </div>
    </PageShell>
  )
}

function PromptFileRow({
  file,
  active,
  t,
  onSelect,
  onCreate,
}: {
  file: SystemPromptFile
  active: boolean
  t: (key: string) => string
  onSelect: (file: SystemPromptFile) => void
  onCreate: (file: SystemPromptFile) => void
}) {
  const handleSelect = useCallback(() => onSelect(file), [file, onSelect])
  const handleCreate = useCallback(() => onCreate(file), [file, onCreate])
  return (
    <article className={active ? 'rounded-xl border border-accent-primary/30 bg-bg-elevated p-3' : 'rounded-xl border border-border-default/15 p-3'}>
      <button type="button" className={buttonClass({ variant: 'ghost', className: 'w-full justify-start' })} disabled={!file.exists} onClick={handleSelect}>
        <SIcon name={file.exists ? 'FileCheck2' : 'FileQuestion'} size="w-4 h-4" />
        <span>{t(file.labelKey)}</span>
      </button>
      {file.exists ? null : (
        <button type="button" className={buttonClass({ variant: 'ghost', className: 'mt-2' })} onClick={handleCreate}>
          <SIcon name="FilePlus2" size="w-4 h-4" />
          {t('systemPrompts.create')}
        </button>
      )}
    </article>
  )
}
