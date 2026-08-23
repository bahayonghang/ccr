import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { createOutputStyle, deleteOutputStyle, listOutputStyles, updateOutputStyle } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { ClaudeSubnav } from '@/features/claude/ClaudeSubnav'
import { t } from '@/features/claude/locale'
import { StyleCard } from '@/features/claude/output-styles/StyleCards'
import type { OutputStyle } from '@/types'
import { BaseModal, ListSearchHeader, PageHeader, PageShell, SIcon, Spinner } from '@/ui'
import { copyText } from '@/utils/clipboard'
import { logger } from '@/utils/logger'

interface StyleForm {
  name: string
  content: string
}

export function OutputStylesView() {
  const [styles, setStyles] = useState<OutputStyle[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [editorOpen, setEditorOpen] = useState(false)
  const [viewOpen, setViewOpen] = useState(false)
  const [editing, setEditing] = useState<OutputStyle | null>(null)
  const [viewing, setViewing] = useState<OutputStyle | null>(null)
  const [copied, setCopied] = useState(false)
  const form = useForm<StyleForm>({ defaultValues: { name: '', content: '' } })
  const { register, handleSubmit, reset, formState } = form

  const loadStyles = useCallback(async () => {
    setLoading(true)
    try {
      setStyles(await listOutputStyles())
    } catch (error) {
      logger.error('Failed to load output styles:', error)
      surfaceNotify.error(t('common.loadFailed'))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadStyles()
  }, [loadStyles])

  const filtered = useMemo(() => {
    const query = searchQuery.trim().toLowerCase()
    if (!query) return styles
    return styles.filter((style) => style.name.toLowerCase().includes(query) || style.content.toLowerCase().includes(query))
  }, [searchQuery, styles])

  const closeEditor = useCallback(() => setEditorOpen(false), [])
  const closeView = useCallback(() => setViewOpen(false), [])
  const handleEditorOpen = useCallback((open: boolean) => {
    if (!open) setEditorOpen(false)
  }, [])
  const handleViewOpen = useCallback((open: boolean) => {
    if (!open) setViewOpen(false)
  }, [])
  const handleAdd = useCallback(() => {
    setEditing(null)
    reset({ name: '', content: '' })
    setEditorOpen(true)
  }, [reset])
  const handleEdit = useCallback(
    (style: OutputStyle) => {
      setEditing(style)
      reset({ name: style.name, content: style.content })
      setEditorOpen(true)
    },
    [reset],
  )
  const handleView = useCallback((style: OutputStyle) => {
    setViewing(style)
    setViewOpen(true)
  }, [])
  const handleEditFromView = useCallback(() => {
    if (!viewing) return
    setViewOpen(false)
    handleEdit(viewing)
  }, [handleEdit, viewing])
  const handleDelete = useCallback(async (name: string) => {
    const confirmed = await surfaceNotify.confirm({
      title: t('common.delete'),
      message: t('outputStyles.deleteConfirm', { name }),
      confirmText: t('common.delete'),
      cancelText: t('common.cancel'),
      type: 'danger',
    })
    if (!confirmed) return
    try {
      await deleteOutputStyle(name)
      await loadStyles()
      surfaceNotify.success(t('common.deleteSuccess'))
    } catch (error) {
      logger.error('Failed to delete output style:', error)
      surfaceNotify.error(t('common.deleteFailed'))
    }
  }, [loadStyles])
  const onValid = useCallback(
    async (values: StyleForm) => {
      if (!values.name.trim() || !values.content.trim()) {
        surfaceNotify.warning(t('outputStyles.validation.required'))
        return
      }
      try {
        if (editing) {
          await updateOutputStyle(editing.name, { content: values.content })
          surfaceNotify.success(t('common.saveSuccess'))
        } else {
          await createOutputStyle(values)
          surfaceNotify.success(t('outputStyles.createSuccess'))
        }
        setEditorOpen(false)
        await loadStyles()
      } catch (error) {
        logger.error('Failed to save output style:', error)
        surfaceNotify.error(t('common.operationFailed'))
      }
    },
    [editing, loadStyles],
  )
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])
  const copyContent = useCallback(async () => {
    if (!viewing) return
    if (await copyText(viewing.content)) {
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    }
  }, [viewing])

  const header = (
    <PageHeader
      title={t('outputStyles.pageTitle')}
      status={<span>{styles.length}</span>}
      actions={
        <button
          type="button"
          className="inline-flex min-h-11 items-center rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]"
          onClick={handleAdd}
        >
          <SIcon name="Plus" size="w-5 h-5" className="mr-2" />
          {t('outputStyles.addStyle')}
        </button>
      }
    />
  )

  return (
    <PageShell header={header} subnav={<ClaudeSubnav />}>
      <ListSearchHeader
        searchValue={searchQuery}
        onSearchValueChange={setSearchQuery}
        placeholder={t('outputStyles.searchPlaceholder')}
      />
      {loading ? (
        <div className="py-20 text-center text-text-muted" role="status">
          <Spinner size="lg" className="mx-auto mb-4 text-accent-secondary" />
          <span>{t('common.loading')}</span>
        </div>
      ) : filtered.length === 0 ? (
        <div className="py-20 text-center text-text-muted" role="status">
          <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-bg-elevated">
            <SIcon name="Palette" size="w-10 h-10" className="opacity-50" />
          </div>
          <p className="text-lg font-medium">{searchQuery ? t('outputStyles.noResults') : t('outputStyles.noStyles')}</p>
          {!searchQuery ? <p className="mt-2 text-sm text-text-muted">{t('outputStyles.noStylesHint')}</p> : null}
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3" role="list" aria-label={t('outputStyles.stylesList')}>
          {filtered.map((style) => (
            <StyleCard key={style.name} style={style} onView={handleView} onEdit={handleEdit} onDelete={handleDelete} />
          ))}
        </div>
      )}
      <BaseModal
        modelValue={editorOpen}
        title={editing ? t('common.edit') : t('outputStyles.addStyle')}
        size="xl"
        surface="solid"
        onUpdateModelValue={handleEditorOpen}
        onClose={closeEditor}
        footer={
          <div className="flex w-full gap-3">
            <button type="button" className="flex-1 rounded-lg border border-border-default px-4 py-2" onClick={closeEditor}>
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="flex-1 rounded-lg bg-accent-secondary px-4 py-2 text-[color:var(--color-accent-primary-contrast)]"
              disabled={formState.isSubmitting}
              onClick={onSubmit}
            >
              {formState.isSubmitting ? t('common.saving') : editing ? t('common.save') : t('outputStyles.create')}
            </button>
          </div>
        }
      >
        <label className="grid gap-2 text-sm font-semibold">
          <span>{t('outputStyles.name')}</span>
          <input type="text" className="rounded-lg border border-border-default bg-bg-surface px-3 py-2" {...register('name')} />
        </label>
        <label className="mt-4 grid gap-2 text-sm font-semibold">
          <span>{t('outputStyles.content')}</span>
          <textarea rows={12} className="rounded-lg border border-border-default bg-bg-surface px-3 py-2 font-mono text-sm" {...register('content')} />
        </label>
      </BaseModal>
      <BaseModal
        modelValue={viewOpen && Boolean(viewing)}
        title={viewing?.name ?? t('outputStyles.preview')}
        size="4xl"
        surface="solid"
        onUpdateModelValue={handleViewOpen}
        onClose={closeView}
        footer={
          <div className="flex w-full gap-3">
            <button type="button" className="rounded-lg border border-border-default px-4 py-2" onClick={copyContent}>
              {copied ? t('common.copied') : t('common.copy')}
            </button>
            <button type="button" className="rounded-lg bg-accent-secondary px-4 py-2 text-[color:var(--color-accent-primary-contrast)]" onClick={handleEditFromView}>
              {t('common.edit')}
            </button>
          </div>
        }
      >
        <pre className="max-h-[70vh] overflow-auto whitespace-pre-wrap break-words rounded-xl bg-bg-elevated p-4 font-mono text-sm text-text-secondary">
          {viewing?.content}
        </pre>
      </BaseModal>
    </PageShell>
  )
}
