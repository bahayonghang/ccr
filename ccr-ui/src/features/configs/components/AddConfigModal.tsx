import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { addConfig } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { mapTemplateToClaudeLegacyConfigPatch } from '@/utils/providerTemplates'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { BaseModal, SIcon } from '@/ui'
import { t } from '../locale'
import { configsNotify } from '../notify'
import {
  addConfigFormSchema,
  draftContextFromValues,
  emptyConfigForm,
  toUpdateRequest,
  type ConfigFormValues,
} from '../lib/configForm'
import { NEW_CONFIG_DRAFT_ID } from '../types'
import { useConfigsViewStore } from '../stores'
import { ProviderTemplateSelector } from '../provider-templates/ProviderTemplateSelector'
import { ConfigFormFields } from './ConfigFormFields'

interface AddConfigModalProps {
  isOpen: boolean
  onClose: () => void
  onSaved: () => void
}

export function AddConfigModal({ isOpen, onClose, onSaved }: AddConfigModalProps) {
  const formDraft = useConfigsViewStore((state) => state.formDrafts[NEW_CONFIG_DRAFT_ID])
  const setFormDraft = useConfigsViewStore((state) => state.setFormDraft)
  const clearFormDraft = useConfigsViewStore((state) => state.clearFormDraft)
  const [saving, setSaving] = useState(false)
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null)
  const [selectedEndpoint, setSelectedEndpoint] = useState('')
  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(addConfigFormSchema),
    defaultValues: emptyConfigForm(),
  })
  const { register, handleSubmit, reset, setValue, getValues } = form

  useEffect(() => {
    if (!isOpen) return
    const draft = formDraft && typeof formDraft === 'object' ? { ...emptyConfigForm(), ...(formDraft as object) } : emptyConfigForm()
    reset(draft as ConfigFormValues)
    setSelectedTemplateId(null)
    setSelectedEndpoint('')
  }, [formDraft, isOpen, reset])

  useEffect(() => {
    if (!isOpen) return
    const sub = form.watch((values) => {
      setFormDraft(NEW_CONFIG_DRAFT_ID, values)
    })
    return () => sub.unsubscribe()
  }, [form, isOpen, setFormDraft])

  const getDraftContext = useCallback(() => draftContextFromValues(getValues()), [getValues])

  const applyManual = useCallback(() => {
    setSelectedTemplateId(null)
    setSelectedEndpoint('')
    reset(emptyConfigForm())
  }, [reset])

  const applyTemplate = useCallback(
    (selection: ProviderTemplateSelection) => {
      const patch = mapTemplateToClaudeLegacyConfigPatch(selection.template, selection.endpoint)
      setSelectedTemplateId(selection.template.id)
      setSelectedEndpoint(selection.endpoint || '')
      setValue('base_url', patch.base_url || '')
      setValue('model', patch.model || '')
      setValue('small_fast_model', patch.small_fast_model || '')
      setValue('provider', patch.provider || selection.template.name)
      setValue('provider_type', patch.provider_type || '')
      if (patch.description) setValue('description', patch.description)
      if (!getValues('name')) setValue('name', patch.suggestedName || selection.template.id)
    },
    [getValues, setValue],
  )

  const onValid = useCallback(
    async (values: ConfigFormValues) => {
      setSaving(true)
      try {
        await addConfig(toUpdateRequest(values, values.name.trim()))
        configsNotify.success('Configuration added successfully')
        clearFormDraft(NEW_CONFIG_DRAFT_ID)
        onSaved()
        onClose()
      } catch (error) {
        configsNotify.error(getErrorMessage(error) || 'Failed to add configuration')
      } finally {
        setSaving(false)
      }
    },
    [clearFormDraft, onClose, onSaved],
  )

  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])
  const handleOpenChange = useCallback(
    (open: boolean) => {
      if (!open) onClose()
    },
    [onClose],
  )

  const renderHeader = useCallback(
    (scope: { titleId: string }) => (
      <div className="flex items-center gap-3">
        <div className="rounded-lg bg-accent-success/10 p-2 text-accent-success">
          <SIcon name="Plus" size="w-5 h-5" />
        </div>
        <div>
          <h2 id={scope.titleId} className="text-lg font-bold text-text-primary">
            {t('configs.addConfig.title')}
          </h2>
          <p className="text-xs text-text-secondary">{t('configs.addConfig.subtitle')}</p>
        </div>
      </div>
    ),
    [],
  )

  return (
    <BaseModal
      modelValue={isOpen}
      size="4xl"
      scrollable
      surface="solid"
      title={t('configs.addConfig.title')}
      header={renderHeader}
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <>
          <button type="button" className="rounded-lg px-4 py-2 text-sm text-text-secondary" onClick={onClose}>
            {t('common.cancel')}
          </button>
          <button
            type="button"
            className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)] disabled:opacity-50"
            disabled={saving}
            onClick={onSubmit}
          >
            {saving ? t('configs.addConfig.saving') : t('configs.addConfig.save')}
          </button>
        </>
      }
    >
      <ProviderTemplateSelector
        platform="claude"
        selectedTemplateId={selectedTemplateId}
        selectedEndpoint={selectedEndpoint}
        getDraftContext={getDraftContext}
        label="Provider template"
        helper="Search built-in and custom non-secret templates, then fill provider fields in one step."
        onSelect={applyTemplate}
        onManual={applyManual}
      />
      <div className="my-6 h-px bg-border-subtle" />
      <form className="space-y-6" onSubmit={onSubmit}>
        <ConfigFormFields register={register} showName />
      </form>
    </BaseModal>
  )
}
