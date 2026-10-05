import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import {
  addOpenCodeProvider,
  deleteOpenCodeProvider,
  getOpenCodeConfig,
  listOpenCodeProviders,
  updateOpenCodeConfig,
} from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type {
  OpenCodeConfig,
  OpenCodeModelConfig,
  OpenCodeProviderConfig,
  OpenCodeProviderOptions,
  OpenCodeProviderRequest,
} from '@/types'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { getErrorMessage } from '@/utils/errorHandler'
import { formatJsonInput, parseJsonInput } from '@/utils/opencode'
import { mapTemplateToOpenCodeProviderPatch } from '@/utils/providerTemplates'
import { OpenCodePageShell } from './OpenCodePageShell'
import { useOpenCodeLocale } from './locale'
import { OpenCodeProviderCard } from './providers/OpenCodeProviderCard'
import { OpenCodeProviderForm } from './providers/OpenCodeProviderForm'
import { OpenCodeTemplatePicker } from './providers/OpenCodeTemplatePicker'
import { emptyProviderForm, type OpenCodeProviderFormValues } from './providers/providerForm'
import { SIcon, buttonClass } from '@/ui'

const managedRootKeys = new Set(['id', 'name', 'npm', 'options', 'models'])

export function OpenCodeProvidersView() {
  const { tt } = useOpenCodeLocale()
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [showModal, setShowModal] = useState(false)
  const [editingId, setEditingId] = useState('')
  const [providers, setProviders] = useState<OpenCodeProviderConfig[]>([])
  const [configState, setConfigState] = useState<OpenCodeConfig>({})
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null)
  const form = useForm<OpenCodeProviderFormValues>({ defaultValues: emptyProviderForm() })

  const providerEnabled = useCallback(
    (id: string) => {
      const disabledProviders = new Set(configState.disabled_providers || [])
      const enabledProviders = configState.enabled_providers || []
      if (disabledProviders.has(id)) return false
      if (enabledProviders.length > 0) return enabledProviders.includes(id)
      return true
    },
    [configState.disabled_providers, configState.enabled_providers],
  )

  const loadProviders = useCallback(async () => {
    setLoading(true)
    try {
      const [providerList, config] = await Promise.all([listOpenCodeProviders(), getOpenCodeConfig()])
      setProviders(Array.isArray(providerList) ? providerList : [])
      setConfigState(config && typeof config === 'object' ? config : {})
    } catch (error) {
      surfaceNotify.error(getErrorMessage(error))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadProviders()
  }, [loadProviders])

  const closeModal = useCallback(() => setShowModal(false), [])
  const openCreate = useCallback(() => {
    setEditingId('')
    form.reset(emptyProviderForm())
    setSelectedTemplate(null)
    setShowModal(true)
  }, [form])

  const applyTemplate = useCallback(
    (selection: ProviderTemplateSelection) => {
      const patch = mapTemplateToOpenCodeProviderPatch(selection.template, selection.endpoint)
      setSelectedTemplate(selection.template.id)
      form.reset({
        ...emptyProviderForm(),
        id: patch.id || '',
        name: patch.name || selection.template.name,
        npm: patch.npm || '',
        baseURL: patch.baseURL || '',
        modelsJson: patch.modelsJson || '{}',
        extraOptionsJson: patch.extraOptionsJson || '{}',
        rootExtraJson: patch.rootExtraJson || '{}',
      })
    },
    [form],
  )

  const handleSidebarSelect = useCallback(
    (selection: ProviderTemplateSelection) => {
      applyTemplate(selection)
      setEditingId('')
      setShowModal(true)
    },
    [applyTemplate],
  )

  const handleManual = useCallback(() => {
    setSelectedTemplate(null)
    if (!showModal) openCreate()
  }, [openCreate, showModal])

  const openEdit = useCallback(
    (provider: OpenCodeProviderConfig) => {
      const extraOptions = { ...(provider.options || {}) }
      delete extraOptions.apiKey
      delete extraOptions.baseURL
      const rootExtra = Object.fromEntries(
        Object.entries(provider).filter(([key]) => !managedRootKeys.has(key)),
      )
      setEditingId(provider.id)
      form.reset({
        id: provider.id,
        name: provider.name || '',
        npm: provider.npm || '',
        apiKey: String(provider.options?.apiKey || ''),
        baseURL: String(provider.options?.baseURL || ''),
        enabled: providerEnabled(provider.id),
        modelsJson: formatJsonInput(provider.models || {}),
        extraOptionsJson: formatJsonInput(extraOptions),
        rootExtraJson: formatJsonInput(rootExtra),
      })
      setSelectedTemplate(null)
      setShowModal(true)
    },
    [form, providerEnabled],
  )

  const syncVisibility = useCallback(
    async (id: string, enabled: boolean) => {
      const nextDisabled = new Set(configState.disabled_providers || [])
      const nextEnabled = new Set(configState.enabled_providers || [])
      if (enabled) {
        nextDisabled.delete(id)
        if (nextEnabled.size > 0) nextEnabled.add(id)
      } else {
        nextDisabled.add(id)
        nextEnabled.delete(id)
      }
      const patch: Record<string, unknown> = { disabled_providers: [...nextDisabled] }
      if ((configState.enabled_providers || []).length > 0) {
        patch.enabled_providers = [...nextEnabled]
      }
      setConfigState(await updateOpenCodeConfig(patch))
    },
    [configState],
  )

  const saveProvider = useCallback(async () => {
    const values = form.getValues()
    if (!values.id.trim()) {
      surfaceNotify.error(tt('Provider id 不能为空', 'Provider ID is required'))
      return
    }
    setSaving(true)
    try {
      const extraOptions = parseJsonInput<Record<string, unknown>>(values.extraOptionsJson, {})
      const rootExtra = parseJsonInput<Record<string, unknown>>(values.rootExtraJson, {})
      const models = parseJsonInput<Record<string, OpenCodeModelConfig>>(values.modelsJson, {})
      const options: OpenCodeProviderOptions = { ...extraOptions }
      if (values.apiKey.trim()) options.apiKey = values.apiKey.trim()
      if (values.baseURL.trim()) options.baseURL = values.baseURL.trim()
      const request: OpenCodeProviderRequest = {
        ...rootExtra,
        id: values.id.trim(),
        name: values.name.trim() || undefined,
        npm: values.npm.trim() || undefined,
        options,
        models,
      }
      const { id, ...providerConfig } = request
      await addOpenCodeProvider(id, providerConfig)
      await syncVisibility(id, values.enabled)
      surfaceNotify.success(editingId ? tt('Provider 已更新', 'Provider updated') : tt('Provider 已创建', 'Provider created'))
      setShowModal(false)
      await loadProviders()
    } catch (error) {
      surfaceNotify.error(getErrorMessage(error))
    } finally {
      setSaving(false)
    }
  }, [editingId, form, loadProviders, syncVisibility, tt])

  const toggleEnabled = useCallback(
    async (provider: OpenCodeProviderConfig) => {
      try {
        const next = !providerEnabled(provider.id)
        await syncVisibility(provider.id, next)
        surfaceNotify.success(next ? tt('Provider 已启用', 'Provider enabled') : tt('Provider 已停用', 'Provider disabled'))
        await loadProviders()
      } catch (error) {
        surfaceNotify.error(getErrorMessage(error))
      }
    },
    [loadProviders, providerEnabled, syncVisibility, tt],
  )

  const removeProvider = useCallback(
    async (provider: OpenCodeProviderConfig) => {
      try {
        await deleteOpenCodeProvider(provider.id)
        setConfigState(
          await updateOpenCodeConfig({
            disabled_providers: (configState.disabled_providers || []).filter((item) => item !== provider.id),
            enabled_providers: (configState.enabled_providers || []).filter((item) => item !== provider.id),
          }),
        )
        surfaceNotify.success(tt('Provider 已删除', 'Provider deleted'))
        await loadProviders()
      } catch (error) {
        surfaceNotify.error(getErrorMessage(error))
      }
    },
    [configState.disabled_providers, configState.enabled_providers, loadProviders, tt],
  )

  const handleSaveClick = useCallback(() => {
    void saveProvider()
  }, [saveProvider])

  return (
    <OpenCodePageShell
      title={tt('Providers', 'Providers')}
      description={tt(
        '按官方 provider schema 管理认证、baseURL、模型和启用状态。',
        'Manage auth, baseURL, models, and enabled state with the official provider schema.',
      )}
      icon="Layers"
      tone="lime"
      badge="provider"
      actions={
        <button type="button" className={buttonClass({ variant: 'primary' })} onClick={openCreate}>
          <SIcon name="Plus" size="w-4 h-4" />
          {tt('添加 Provider', 'Add provider')}
        </button>
      }
    >
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <div className="space-y-4">
          {loading ? (
            <div className="rounded-2xl border border-border-subtle p-8 text-center text-sm text-text-muted">{tt('加载中', 'Loading')}</div>
          ) : providers.length === 0 ? (
            <div className="rounded-2xl border border-border-subtle p-8 text-center">
              <h2 className="text-lg font-semibold text-text-primary">{tt('暂无 Provider', 'No providers yet')}</h2>
              <p className="mt-2 text-sm text-text-secondary">
                {tt(
                  '从 Anthropic、OpenAI、Google 或自定义 OpenAI-compatible provider 开始。',
                  'Start with Anthropic, OpenAI, Google, or a custom OpenAI-compatible provider.',
                )}
              </p>
            </div>
          ) : (
            providers.map((provider) => (
              <OpenCodeProviderCard
                key={provider.id}
                provider={provider}
                enabled={providerEnabled(provider.id)}
                onToggle={toggleEnabled}
                onEdit={openEdit}
                onRemove={removeProvider}
              />
            ))
          )}
        </div>
        <aside className="rounded-2xl border border-border-subtle bg-bg-surface p-5">
          <h2 className="text-lg font-semibold text-text-primary">{tt('Provider templates', 'Provider templates')}</h2>
          <p className="mt-2 text-sm text-text-secondary">
            {tt(
              '搜索内置或自定义的非敏感模板，一次性填写 provider id、npm、baseURL 和模型 JSON。',
              'Search built-in or custom non-secret templates, then fill provider id, npm, baseURL, and model JSON in one step.',
            )}
          </p>
          <div className="mt-4">
            <OpenCodeTemplatePicker
              selectedTemplateId={selectedTemplate}
              label="Template"
              helper="Templates never store apiKey; credentials stay in this provider form."
              manualLabel="Manual"
              onSelect={handleSidebarSelect}
              onManual={handleManual}
            />
          </div>
        </aside>
      </div>
      <OpenCodeProviderForm
        open={showModal}
        editingId={editingId}
        saving={saving}
        form={form}
        selectedTemplateId={selectedTemplate}
        onClose={closeModal}
        onSave={handleSaveClick}
        onSelectTemplate={applyTemplate}
        onManualTemplate={handleManual}
      />
    </OpenCodePageShell>
  )
}
