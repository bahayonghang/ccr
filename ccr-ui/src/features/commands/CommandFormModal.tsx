import { useCallback, useEffect, type FormEvent } from 'react'
import { useForm } from 'react-hook-form'
import type { SlashCommand, SlashCommandRequest } from '@/types/platform'
import { BaseModal, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/ui'
import { useCommandsT } from './locale'

interface CommandFormModalProps {
  visible: boolean
  editingCommand: SlashCommand | null
  folders: string[]
  onClose: () => void
  onSubmit: (data: SlashCommandRequest) => void
}

const empty = (): SlashCommandRequest => ({ name: '', command: '', description: '', folder: '' })

export function CommandFormModal({ visible, editingCommand, folders, onClose, onSubmit }: CommandFormModalProps) {
  const t = useCommandsT()
  const form = useForm<SlashCommandRequest>({ defaultValues: empty() })
  const folder = form.watch('folder')
  const isEditing = Boolean(editingCommand)

  useEffect(() => {
    if (editingCommand) {
      form.reset({
        name: editingCommand.name,
        command: editingCommand.command,
        description: editingCommand.description,
        folder: editingCommand.folder,
      })
      return
    }
    form.reset(empty())
  }, [editingCommand, form])

  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) onClose()
  }, [onClose])
  const handleFolder = useCallback((value: string) => {
    form.setValue('folder', value)
  }, [form])
  const submit = useCallback(() => {
    onSubmit(form.getValues())
    onClose()
  }, [form, onClose, onSubmit])
  const handleSubmit = useCallback((event: FormEvent) => {
    event.preventDefault()
    submit()
  }, [submit])

  return (
    <BaseModal
      modelValue={visible}
      title={isEditing ? t('common.edit') : t('common.add')}
      size="md"
      scrollable
      surface="solid"
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <div className="flex w-full justify-end gap-3">
          <button type="button" className="rounded-lg border border-border-default px-4 py-2" onClick={onClose}>{t('common.cancel')}</button>
          <button type="button" className="rounded-lg bg-accent-primary px-4 py-2 text-[color:var(--color-accent-primary-contrast)]" onClick={submit}>
            {isEditing ? t('common.update') : t('common.create')}
          </button>
        </div>
      }
    >
      <form className="space-y-4" onSubmit={handleSubmit}>
        <label className="block text-sm font-medium text-text-primary">
          {t('common.name')}
          <input className="mt-1 w-full rounded-lg border border-border-default bg-bg-surface px-3 py-2 text-sm" required disabled={isEditing} placeholder={t('slashCommands.namePlaceholder')} {...form.register('name')} />
        </label>
        <label className="block text-sm font-medium text-text-primary">
          {t('common.command')}
          <input className="mt-1 w-full rounded-lg border border-border-default bg-bg-surface px-3 py-2 font-mono text-sm" required placeholder={t('slashCommands.commandPlaceholder')} {...form.register('command')} />
        </label>
        <label className="block text-sm font-medium text-text-primary">
          {t('common.description')}
          <textarea className="mt-1 min-h-20 w-full rounded-lg border border-border-default bg-bg-surface px-3 py-2 text-sm" required placeholder={t('slashCommands.descriptionPlaceholder')} {...form.register('description')} />
        </label>
        <label className="block text-sm font-medium text-text-primary">
          {t('common.folder')}
          <Select value={folder} onValueChange={handleFolder}>
            <SelectTrigger><SelectValue placeholder={t('slashCommands.selectFolder')} /></SelectTrigger>
            <SelectContent>
              {folders.map((item) => (
                <SelectItem key={item} value={item}>{item}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </label>
      </form>
    </BaseModal>
  )
}
