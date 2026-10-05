import { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { claudeCodeConfig } from '@/configs/slashCommands'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { ClaudeSubnav } from '@/features/claude/ClaudeSubnav'
import { t } from '@/features/claude/locale'
import { NamedItemCard } from '@/features/platform/NamedItemCard'
import type { SlashCommandRequest } from '@/types/platform'
import { BaseModal, EmptyState, ListSearchHeader, PageHeader, PageShell, Spinner } from '@/ui'

interface SlashForm {
  name: string
  command: string
  description: string
  folder: string
}

const emptyForm = (): SlashForm => ({ name: '', command: '', description: '', folder: '' })

export function ClaudeSlashCommandsView() {
  const [search, setSearch] = useState('')
  const [showForm, setShowForm] = useState(false)
  const [editingName, setEditingName] = useState<string | null>(null)
  const query = useQuery({
    queryKey: ['claude-slash-commands'],
    queryFn: claudeCodeConfig.api.list,
  })
  const form = useForm<SlashForm>({ defaultValues: emptyForm() })
  const { register, handleSubmit, reset } = form
  const commands = useMemo(() => query.data?.commands ?? [], [query.data?.commands])
  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase()
    if (!q) return commands
    return commands.filter(
      (item) =>
        item.name.toLowerCase().includes(q) ||
        item.command.toLowerCase().includes(q) ||
        item.description.toLowerCase().includes(q),
    )
  }, [commands, search])

  const closeForm = useCallback(() => setShowForm(false), [])
  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) setShowForm(false)
  }, [])
  const openCreate = useCallback(() => {
    setEditingName(null)
    reset(emptyForm())
    setShowForm(true)
  }, [reset])
  const handleEdit = useCallback(
    (id: string) => {
      const current = commands.find((item) => item.name === id)
      if (!current) return
      setEditingName(current.name)
      reset({
        name: current.name,
        command: current.command,
        description: current.description,
        folder: current.folder,
      })
      setShowForm(true)
    },
    [commands, reset],
  )
  const handleDelete = useCallback(
    async (id: string) => {
      const confirmed = await surfaceNotify.confirm({
        title: t('common.delete'),
        message: t('slashCommands.deleteConfirm', { name: id }),
        confirmText: t('common.delete'),
        cancelText: t('common.cancel'),
        type: 'danger',
      })
      if (!confirmed) return
      await claudeCodeConfig.api.delete(id)
      surfaceNotify.success(t('common.deleteSuccess'))
      await query.refetch()
    },
    [query],
  )
  const handleToggle = useCallback(
    async (id: string) => {
      await claudeCodeConfig.api.toggle(id)
      await query.refetch()
    },
    [query],
  )
  const onValid = useCallback(
    async (values: SlashForm) => {
      const payload: SlashCommandRequest = {
        name: values.name.trim(),
        command: values.command.trim(),
        description: values.description.trim(),
        folder: values.folder.trim(),
      }
      if (editingName) await claudeCodeConfig.api.update(editingName, payload)
      else await claudeCodeConfig.api.add(payload)
      surfaceNotify.success(t('common.saveSuccess'))
      setShowForm(false)
      reset(emptyForm())
      await query.refetch()
    },
    [editingName, query, reset],
  )
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])

  const header = (
    <PageHeader
      title={t('slashCommands.title')}
      description={t('slashCommands.subtitle')}
      status={<span>{filtered.length}/{commands.length}</span>}
      actions={
        <button
          type="button"
          className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]"
          onClick={openCreate}
        >
          {t('common.add')}
        </button>
      }
    />
  )

  if (query.isPending) {
    return (
      <PageShell header={header} subnav={<ClaudeSubnav />}>
        <Spinner size="lg" className="mx-auto text-accent-primary" />
      </PageShell>
    )
  }

  return (
    <PageShell header={header} subnav={<ClaudeSubnav />}>
      <ListSearchHeader searchValue={search} onSearchValueChange={setSearch} placeholder={t('common.search')} />
      {filtered.length === 0 ? (
        <EmptyState title={t('slashCommands.emptyTitle')} />
      ) : (
        <div className="grid gap-3">
          {filtered.map((item) => (
            <NamedItemCard
              key={item.name}
              item={{
                id: item.name,
                name: item.name,
                description: item.description || item.command,
                enabled: item.enabled,
                badge: item.folder || undefined,
              }}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onToggle={handleToggle}
              editLabel={t('common.edit')}
              deleteLabel={t('common.delete')}
              toggleLabel={item.enabled ? t('common.disable') : t('common.enable')}
            />
          ))}
        </div>
      )}
      <BaseModal
        modelValue={showForm}
        title={editingName ? t('common.edit') : t('common.add')}
        size="lg"
        surface="solid"
        onUpdateModelValue={handleOpenChange}
        onClose={closeForm}
        footer={
          <div className="flex w-full gap-3">
            <button type="button" className="flex-1 rounded-lg border border-border-default px-4 py-2" onClick={closeForm}>
              {t('common.cancel')}
            </button>
            <button type="button" className="flex-1 rounded-lg bg-accent-primary px-4 py-2 text-[color:var(--color-accent-primary-contrast)]" onClick={onSubmit}>
              {t('common.save')}
            </button>
          </div>
        }
      >
        <div className="grid gap-3">
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.name')} {...register('name')} />
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2 font-mono" placeholder={t('common.command')} {...register('command')} />
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.description')} {...register('description')} />
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder="folder" {...register('folder')} />
        </div>
      </BaseModal>
    </PageShell>
  )
}
