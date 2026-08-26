import { useCallback } from 'react'
import type { UseFormReturn } from 'react-hook-form'
import { BaseModal, buttonClass } from '@/ui'
import { fieldInputClass } from '../ui-classes'
import { useOpenCodeLocale } from '../locale'
import { OpenCodeTemplatePicker } from './OpenCodeTemplatePicker'
import type { OpenCodeProviderFormValues } from './providerForm'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'

interface OpenCodeProviderFormProps {
  open: boolean
  editingId: string
  saving: boolean
  form: UseFormReturn<OpenCodeProviderFormValues>
  selectedTemplateId: string | null
  onClose: () => void
  onSave: () => void
  onSelectTemplate: (selection: ProviderTemplateSelection) => void
  onManualTemplate: () => void
}

export function OpenCodeProviderForm({
  open,
  editingId,
  saving,
  form,
  selectedTemplateId,
  onClose,
  onSave,
  onSelectTemplate,
  onManualTemplate,
}: OpenCodeProviderFormProps) {
  const { tt } = useOpenCodeLocale()
  const { register } = form
  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) onClose()
    },
    [onClose],
  )

  return (
    <BaseModal
      modelValue={open}
      title={editingId ? tt('编辑 Provider', 'Edit provider') : tt('添加 Provider', 'Add provider')}
      description={tt('直接编辑 OpenCode provider 配置。', 'Edit the OpenCode provider config directly.')}
      size="lg"
      contentClass="max-w-2xl overflow-y-auto"
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
    >
      <div className="space-y-4">
        {editingId ? null : (
          <OpenCodeTemplatePicker
            selectedTemplateId={selectedTemplateId}
            label="Provider template"
            helper="Apply another non-secret template before saving this provider."
            manualLabel="Manual"
            onSelect={onSelectTemplate}
            onManual={onManualTemplate}
          />
        )}
        <div className="grid gap-4 md:grid-cols-2">
          <label className="block text-xs font-semibold text-text-muted">
            {tt('provider id *', 'provider id *')}
            <input className={`${fieldInputClass} mt-2`} placeholder="anthropic" disabled={Boolean(editingId)} {...register('id')} />
          </label>
          <label className="block text-xs font-semibold text-text-muted">
            {tt('display name', 'display name')}
            <input className={`${fieldInputClass} mt-2`} placeholder="Anthropic" {...register('name')} />
          </label>
          <label className="block text-xs font-semibold text-text-muted md:col-span-2">
            {tt('npm package', 'npm package')}
            <input className={`${fieldInputClass} mt-2`} placeholder="@ai-sdk/openai-compatible" {...register('npm')} />
          </label>
          <label className="block text-xs font-semibold text-text-muted">
            {tt('api key', 'api key')}
            <input className={`${fieldInputClass} mt-2`} placeholder="{env:ANTHROPIC_API_KEY}" {...register('apiKey')} />
          </label>
          <label className="block text-xs font-semibold text-text-muted">
            {tt('baseURL', 'baseURL')}
            <input className={`${fieldInputClass} mt-2`} placeholder="https://api.example.com" {...register('baseURL')} />
          </label>
        </div>
        <label className="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm">
          <input type="checkbox" {...register('enabled')} />
          {tt('该 provider 默认启用', 'Enable this provider by default')}
        </label>
        <label className="block text-xs font-semibold text-text-muted">
          {tt('models JSON', 'models JSON')}
          <textarea rows={8} className={`${fieldInputClass} mt-2 font-mono`} {...register('modelsJson')} />
        </label>
        <label className="block text-xs font-semibold text-text-muted">
          {tt('extra options JSON', 'extra options JSON')}
          <textarea rows={6} className={`${fieldInputClass} mt-2 font-mono`} {...register('extraOptionsJson')} />
        </label>
        <label className="block text-xs font-semibold text-text-muted">
          {tt('root extra JSON', 'root extra JSON')}
          <textarea rows={5} className={`${fieldInputClass} mt-2 font-mono`} {...register('rootExtraJson')} />
        </label>
        <div className="flex justify-end gap-3 border-t border-border-default/55 pt-4">
          <button type="button" className={buttonClass({ variant: 'ghost' })} onClick={onClose}>
            {tt('取消', 'Cancel')}
          </button>
          <button type="button" className={buttonClass({ variant: 'primary' })} disabled={saving} onClick={onSave}>
            {tt('保存', 'Save')}
          </button>
        </div>
      </div>
    </BaseModal>
  )
}
