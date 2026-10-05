import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
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
  readConfigEditDraft,
  toConfigPatch,
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
  const baseline = useRef<{ values: ConfigFormValues; version: string } | null>(null)
  const setFormDraft = useConfigsViewStore((state) => state.setFormDraft)
  const clearFormDraft = useConfigsViewStore((state) => state.clearFormDraft)
  const [loading, setLoading] = useState(false)
  const [loadedName, setLoadedName] = useState<string | null>(null)
  const [reloadVersion, setReloadVersion] = useState(0)
  const [saving, setSaving] = useState(false)
  const [showToken, setShowToken] = useState(false)
  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configFormSchema),
    defaultValues: emptyConfigForm(),
  })
  const { register, handleSubmit, reset, watch } = form

  useEffect(() => {
    baseline.current = null
    setLoadedName(null)
    if (!isOpen || !configName) return
    let cancelled = false
    const load = async () => {
      setLoading(true)
      setShowToken(false)
      try {
        const data = await getConfig('claude', configName)
        if (!data) throw new Error(`Configuration not found: ${configName}`)
        if (cancelled) return
        const loaded = valuesFromConfig(data)
        const stored = useConfigsViewStore.getState().formDrafts[configName]
        const draft = readConfigEditDraft(stored, configName)
        baseline.current = draft
          ? { values: draft.baseline, version: draft.version }
          : { values: loaded, version: data.version }
        reset(draft?.values ?? loaded)
        setLoadedName(configName)
      } catch (error) {
        if (cancelled) return
        baseline.current = null
        setLoadedName(null)
        configsNotify.error(getErrorMessage(error) || 'Failed to load configuration')
      } finally {
        if (!cancelled) setLoading(false)
      }
    }
    void load()
    return () => {
      cancelled = true
    }
  }, [configName, isOpen, reloadVersion, reset])

  useEffect(() => {
    if (!isOpen || !configName) return
    const sub = watch((values) => {
      const snapshot = baseline.current
      if (!snapshot || snapshot.values.name !== configName) return
      setFormDraft(configName, {
        values: { ...values, auth_token: '' },
        baseline: snapshot.values,
        version: snapshot.version,
      })
    })
    return () => sub.unsubscribe()
  }, [configName, isOpen, setFormDraft, watch])

  const toggleToken = useCallback(() => {
    setShowToken((value) => !value)
  }, [])

  const reload = useCallback(async () => {
    const confirmed = await configsNotify.confirm({
      title: tt('重新载入配置', 'Reload configuration'),
      message: tt('丢弃未保存的草稿并读取当前配置？', 'Discard the unsaved draft and read the current configuration?'),
      confirmText: tt('重新载入', 'Reload'),
      type: 'warning',
    })
    if (!confirmed) return
    clearFormDraft(configName)
    baseline.current = null
    setLoadedName(null)
    setReloadVersion((version) => version + 1)
  }, [clearFormDraft, configName])

  const onValid = useCallback(
    async (values: ConfigFormValues) => {
      setSaving(true)
      try {
        const snapshot = baseline.current
        if (!snapshot || snapshot.values.name !== configName || loadedName !== configName) {
          throw new Error('Configuration snapshot is unavailable')
        }
        await updateConfig({
          platform: 'claude', name: configName,
          data: toConfigPatch(values, snapshot.values), expectedVersion: snapshot.version,
        })
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
    [clearFormDraft, configName, loadedName, onClose, onSaved],
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
          <button type="button" className="flex-1 rounded-lg px-4 py-2 text-sm text-text-secondary" disabled={saving || loading} onClick={reload}>
            {tt('重新载入', 'Reload')}
          </button>
          <button
            type="button"
            className="flex-1 rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]"
            disabled={saving || loading || loadedName !== configName}
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
