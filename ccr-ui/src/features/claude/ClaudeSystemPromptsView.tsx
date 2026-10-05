import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import {
  createSystemPrompt,
  getSystemPrompt,
  listSystemPrompts,
  saveSystemPrompt,
  type SystemPromptFile,
  type SystemPromptRule,
} from '@/api/domains/systemPrompts'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { ClaudeSubnav } from '@/features/claude/ClaudeSubnav'
import { t } from '@/features/claude/locale'
import { AsyncStatePanel, PageHeader, PageShell, SIcon } from '@/ui'
import { logger } from '@/utils/logger'

interface PromptForm {
  content: string
}

const FileRow = ({
  file,
  selected,
  onSelect,
  onCreate,
}: {
  file: SystemPromptFile
  selected: boolean
  onSelect: (file: SystemPromptFile) => void
  onCreate: (file: SystemPromptFile) => void
}) => {
  const handleSelect = useCallback(() => {
    onSelect(file)
  }, [file, onSelect])
  const handleCreate = useCallback(() => {
    onCreate(file)
  }, [file, onCreate])
  return (
    <article className={`rounded-xl border px-3 py-3 ${selected ? 'border-accent-primary/40 bg-accent-primary/10' : 'border-border-default bg-bg-surface'}`}>
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="truncate font-semibold text-text-primary">{t(file.labelKey)}</p>
          <p className="truncate font-mono text-xs text-text-muted">{file.path}</p>
        </div>
        {file.exists ? (
          <button type="button" className="rounded-lg border border-border-default px-2 py-1 text-xs" onClick={handleSelect}>
            {t('common.open')}
          </button>
        ) : (
          <button type="button" className="rounded-lg border border-border-default px-2 py-1 text-xs" onClick={handleCreate}>
            {t('common.create')}
          </button>
        )}
      </div>
    </article>
  )
}

export function ClaudeSystemPromptsView() {
  const [loading, setLoading] = useState(true)
  const [unsupported, setUnsupported] = useState(false)
  const [files, setFiles] = useState<SystemPromptFile[]>([])
  const [rules, setRules] = useState<SystemPromptRule[]>([])
  const [selected, setSelected] = useState<SystemPromptFile | null>(null)
  const [token, setToken] = useState('')
  const [baseline, setBaseline] = useState('')
  const [conflict, setConflict] = useState(false)
  const form = useForm<PromptForm>({ defaultValues: { content: '' } })
  const { register, handleSubmit, reset, watch } = form
  const content = watch('content')
  const dirty = content !== baseline

  const loadList = useCallback(async () => {
    const result = await listSystemPrompts('claude')
    if (result.status === 'unsupported_environment') {
      setUnsupported(true)
      setFiles([])
      setRules([])
      return
    }
    setUnsupported(false)
    setFiles(result.files)
    setRules(result.rules)
  }, [])

  const loadSelected = useCallback(async (file: SystemPromptFile) => {
    const result = await getSystemPrompt('claude', file.id)
    if (result.status === 'unsupported_environment') {
      setUnsupported(true)
      return
    }
    setSelected({ ...file, exists: result.exists, path: result.path })
    reset({ content: result.content })
    setBaseline(result.content)
    setToken(result.token)
    setConflict(false)
  }, [reset])

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      await loadList()
    } catch (error) {
      logger.error('Failed to load system prompts', error)
      surfaceNotify.error(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
    } finally {
      setLoading(false)
    }
  }, [loadList])

  useEffect(() => {
    void refresh()
  }, [refresh])

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
  }, [dirty])

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
    [confirmDiscard, loadSelected, selected?.id],
  )

  const createFile = useCallback(
    async (file: SystemPromptFile) => {
      try {
        const result = await createSystemPrompt('claude', file.id)
        if (result.status === 'unsupported_environment') {
          setUnsupported(true)
          return
        }
        surfaceNotify.success(t('systemPrompts.createSuccess'))
        await loadList()
        const created = await listSystemPrompts('claude')
        const next = created.status === 'ok' ? created.files.find((item) => item.id === file.id) : undefined
        if (next) await loadSelected(next)
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.createFailed')}: ${String(error)}`)
      }
    },
    [loadList, loadSelected],
  )

  const onValid = useCallback(
    async (values: PromptForm) => {
      if (!selected || !dirty) return
      try {
        const result = await saveSystemPrompt('claude', selected.id, values.content, token)
        if (result.status === 'unsupported_environment') {
          setUnsupported(true)
          return
        }
        if (result.status === 'conflict') {
          setConflict(true)
          return
        }
        setToken(result.token)
        setBaseline(values.content)
        surfaceNotify.success(t('systemPrompts.saveSuccess'))
        await loadList()
      } catch (error) {
        surfaceNotify.error(`${t('systemPrompts.saveFailed')}: ${String(error)}`)
      }
    },
    [dirty, loadList, selected, token],
  )
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])

  const header = (
    <PageHeader
      title={t('systemPrompts.title')}
      eyebrow={t('systemPrompts.platforms.claude')}
      description={t('systemPrompts.description')}
      leading={<SIcon name="ScrollText" size="w-8 h-8" />}
    />
  )

  if (unsupported) {
    return (
      <PageShell header={header} subnav={<ClaudeSubnav />}>
        <AsyncStatePanel state="runtime-unavailable" title={t('settingsRaw.unsupportedEnvironment')} />
      </PageShell>
    )
  }

  return (
    <PageShell header={header} subnav={<ClaudeSubnav />}>
      <section className="mb-4 flex items-start gap-3 rounded-xl border border-border-default bg-bg-surface p-4">
        <SIcon name="Layers" size="w-5 h-5" />
        <div>
          <strong>{t('systemPrompts.claudeHierarchyTitle')}</strong>
          <p className="text-sm text-text-secondary">{t('systemPrompts.claudeHierarchyDescription')}</p>
        </div>
      </section>
      {loading ? (
        <p className="text-sm text-text-muted">{t('systemPrompts.loading')}</p>
      ) : (
        <div className="grid gap-4 lg:grid-cols-[20rem_minmax(0,1fr)]">
          <aside className="space-y-3">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="font-semibold text-text-primary">{t('systemPrompts.filesTitle')}</h2>
                <p className="text-xs text-text-muted">{t('systemPrompts.filesDescription')}</p>
              </div>
              <span>{files.length}</span>
            </div>
            {files.map((file) => (
              <FileRow
                key={file.id}
                file={file}
                selected={selected?.id === file.id}
                onSelect={selectFile}
                onCreate={createFile}
              />
            ))}
            {rules.length > 0 ? (
              <div className="rounded-xl border border-border-default/50 p-3">
                <p className="mb-2 text-xs font-semibold text-text-muted">{t('systemPrompts.rulesTitle')}</p>
                <ul className="space-y-1 text-xs text-text-secondary">
                  {rules.map((rule) => (
                    <li key={rule.path}>
                      {rule.name} · {rule.path}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </aside>
          <section className="min-w-0">
            {selected ? (
              <div className="space-y-3">
                {conflict ? (
                  <p className="rounded-xl border border-accent-warning/30 bg-accent-warning/10 px-3 py-2 text-sm text-accent-warning">
                    {t('systemPrompts.conflict')}
                  </p>
                ) : null}
                <textarea
                  className="min-h-80 w-full rounded-xl border border-border-default bg-bg-base p-3 font-mono text-sm"
                  {...register('content')}
                />
                <div className="flex justify-end">
                  <button
                    type="button"
                    className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)] disabled:opacity-50"
                    disabled={!dirty}
                    onClick={onSubmit}
                  >
                    {t('common.save')}
                  </button>
                </div>
              </div>
            ) : (
              <p className="text-sm text-text-muted">{t('systemPrompts.selectFile')}</p>
            )}
          </section>
        </div>
      )}
    </PageShell>
  )
}
