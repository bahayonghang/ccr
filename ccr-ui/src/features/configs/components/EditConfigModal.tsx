import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { getConfig, updateConfig } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { BaseModal, SIcon, Spinner } from '@/ui'
import { tt } from '../locale'
import { configsNotify } from '../notify'
import {
  configFormSchema,
  emptyConfigForm,
  isConfigFormDraft,
  toUpdateRequest,
  valuesFromConfig,
  type ConfigFormValues,
} from '../lib/configForm'
import { useConfigsViewStore } from '../stores'
import { ConfigFormFields } from './ConfigFormFields'

interface EditConfigModalProps {
  isOpen: boolean
  configName: string
  onClose: () => void
  onSaved: () => void
}

export function EditConfigModal({ isOpen, configName, onClose, onSaved }: EditConfigModalProps) {
  const formDraft = useConfigsViewStore((state) => state.formDrafts[configName])
  const setFormDraft = useConfigsViewStore((state) => state.setFormDraft)
  const clearFormDraft = useConfigsViewStore((state) => state.clearFormDraft)
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [showToken, setShowToken] = useState(false)
  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configFormSchema),
    defaultValues: emptyConfigForm(),
  })
  const { register, handleSubmit, reset, watch } = form

  useEffect(() => {
    if (!isOpen || !configName) return
    let cancelled = false
    const load = async () => {
      setLoading(true)
      setShowToken(false)
      try {
        const data = await getConfig(configName)
        if (!data) throw new Error(`Configuration not found: ${configName}`)
        if (cancelled) return
        const loaded = valuesFromConfig(data)
        reset(isConfigFormDraft(formDraft) ? formDraft : loaded)
      } catch (error) {
        configsNotify.error(getErrorMessage(error) || 'Failed to load configuration')
      } finally {
        if (!cancelled) setLoading(false)
      }
    }
    void load()
    return () => {
      cancelled = true
    }
  }, [configName, formDraft, isOpen, reset])

  useEffect(() => {
    if (!isOpen || !configName) return
    const sub = watch((values) => {
      setFormDraft(configName, values)
    })
    return () => sub.unsubscribe()
  }, [configName, isOpen, setFormDraft, watch])

  const toggleToken = useCallback(() => {
    setShowToken((value) => !value)
  }, [])

  const onValid = useCallback(
    async (values: ConfigFormValues) => {
      setSaving(true)
      try {
        await updateConfig(configName, toUpdateRequest(values, configName))
        configsNotify.success('Configuration saved successfully')
        clearFormDraft(configName)
        onSaved()
        onClose()
      } catch (error) {
        configsNotify.error(getErrorMessage(error) || 'Failed to save configuration')
      } finally {
        setSaving(false)
      }
    },
    [clearFormDraft, configName, onClose, onSaved],
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
      <div className="flex items-center gap-4">
        <div className="rounded-xl bg-accent-primary/10 p-3 text-accent-primary">
          <SIcon name="Settings" size="w-6 h-6" />
        </div>
        <div>
          <h2 id={scope.titleId} className="text-xl font-bold text-text-primary">
            {tt('编辑配置', 'Edit Configuration')}
          </h2>
          <p className="flex items-center gap-1 font-mono text-xs text-text-secondary">
            <span>ID:</span> {configName}
          </p>
        </div>
      </div>
    ),
    [configName],
  )

  return (
    <BaseModal
      modelValue={isOpen}
      size="4xl"
      scrollable
      surface="solid"
      title={tt('编辑配置', 'Edit Configuration')}
      header={renderHeader}
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <>
          <button type="button" className="flex-1 rounded-lg px-4 py-2 text-sm text-text-secondary" onClick={onClose}>
            {tt('取消', 'Cancel')}
          </button>
          <button
            type="button"
            className="flex-1 rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]"
            disabled={saving || loading}
            onClick={onSubmit}
          >
            {tt('保存更改', 'Save Changes')}
          </button>
        </>
      }
    >
      {loading ? (
        <div className="flex justify-center py-20">
          <Spinner size="lg" className="text-accent-primary" />
        </div>
      ) : (
        <form className="space-y-8" onSubmit={onSubmit}>
          <ConfigFormFields register={register} showToken={showToken} onToggleToken={toggleToken} />
        </form>
      )}
    </BaseModal>
  )
}
