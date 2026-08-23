import { memo, useCallback } from 'react'
import { useForm } from 'react-hook-form'
import { BaseModal, SIcon } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'
import { defaultAgentModelOptions } from './agentModels'

export interface AgentEditForm {
  model: string
  systemPrompt: string
  toolDraft: string
  toolsText: string
}

interface AgentEditModalProps {
  open: boolean
  name: string
  saving: boolean
  t: TranslateFunction
  form: ReturnType<typeof useForm<AgentEditForm>>
  onClose: () => void
  onSave: () => void
}

const splitTools = (value: string): string[] =>
  value
    .split('\n')
    .map((item) => item.trim())
    .filter(Boolean)

export const AgentEditModal = memo(function AgentEditModal({
  open,
  name,
  saving,
  t,
  form,
  onClose,
  onSave,
}: AgentEditModalProps) {
  const { register, getValues, setValue, watch } = form
  const toolsText = watch('toolsText')
  const tools = splitTools(toolsText)

  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) onClose()
    },
    [onClose],
  )

  const addTool = useCallback(() => {
    const draft = getValues('toolDraft').trim()
    if (!draft) return
    const current = splitTools(getValues('toolsText'))
    if (current.includes(draft)) {
      setValue('toolDraft', '')
      return
    }
    setValue('toolsText', [...current, draft].join('\n'))
    setValue('toolDraft', '')
  }, [getValues, setValue])

  const removeTool = useCallback(
    (tool: string) => {
      const next = splitTools(getValues('toolsText')).filter((item) => item !== tool)
      setValue('toolsText', next.join('\n'))
    },
    [getValues, setValue],
  )

  return (
    <BaseModal
      modelValue={open}
      title={t('agents.editAgent')}
      size="lg"
      surface="solid"
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <div className="flex w-full gap-3">
          <button type="button" className="flex-1 rounded-xl border border-border-default bg-bg-elevated px-4 py-3 text-text-secondary" onClick={onClose}>
            {t('common.cancel')}
          </button>
          <button
            type="button"
            className="flex-1 rounded-lg bg-accent-secondary px-4 py-3 text-[color:var(--color-accent-primary-contrast)] disabled:opacity-50"
            disabled={saving}
            onClick={onSave}
          >
            {saving ? t('common.saving') : t('common.save')}
          </button>
        </div>
      }
    >
      <div className="space-y-6">
        <div className="grid gap-6 md:grid-cols-2">
          <label className="block text-xs font-medium text-text-secondary">
            {t('agents.nameLabel')}
            <input
              defaultValue={name}
              disabled
              className="mt-2 w-full cursor-not-allowed rounded-xl border border-border-default bg-bg-elevated px-4 py-3 opacity-60"
            />
          </label>
          <label className="block text-xs font-medium text-text-secondary">
            {t('agents.modelLabel')}
            <select className="mt-2 w-full rounded-xl border border-border-default bg-bg-surface px-4 py-3" {...register('model')}>
              {defaultAgentModelOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        </div>
        <div>
          <p className="mb-2 text-xs font-medium text-text-secondary">{t('agents.toolsLabel')}</p>
          <div className="mb-3 flex gap-2">
            <input
              className="flex-1 rounded-xl border border-border-default bg-bg-surface px-4 py-3"
              placeholder={t('agents.toolPlaceholder')}
              {...register('toolDraft')}
            />
            <button type="button" className="rounded-lg bg-accent-secondary px-6 py-3 text-[color:var(--color-accent-primary-contrast)]" onClick={addTool}>
              {t('agents.addTool')}
            </button>
          </div>
          <div className="flex min-h-12 flex-wrap gap-2 rounded-xl border border-dashed border-border-default/50 bg-bg-base p-4">
            {tools.length === 0 ? (
              <span className="w-full py-2 text-center text-sm text-text-muted italic">{t('agents.noTools')}</span>
            ) : (
              tools.map((tool) => (
                <ToolChip key={tool} tool={tool} onRemove={removeTool} />
              ))
            )}
          </div>
        </div>
        <label className="block text-xs font-medium text-text-secondary">
          {t('agents.systemPromptLabel')}
          <textarea
            rows={8}
            className="mt-2 w-full resize-y rounded-xl border border-border-default bg-bg-elevated px-4 py-3 font-mono text-sm"
            placeholder={t('agents.systemPromptPlaceholder')}
            {...register('systemPrompt')}
          />
        </label>
      </div>
    </BaseModal>
  )
})

const ToolChip = memo(function ToolChip({
  tool,
  onRemove,
}: {
  tool: string
  onRemove: (tool: string) => void
}) {
  const handleRemove = useCallback(() => {
    onRemove(tool)
  }, [onRemove, tool])
  return (
    <span className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-bg-elevated px-3 py-1.5 text-sm text-text-primary">
      {tool}
      <button type="button" className="text-text-muted" onClick={handleRemove}>
        <SIcon name="X" size="w-3.5 h-3.5" />
      </button>
    </span>
  )
})
