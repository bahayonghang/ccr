import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { codexDeleteModelProvider, codexListModelProviders, codexSaveModelProvider } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { codexKeys } from '@/features/codex/queries'
import type { CodexModelProviderRecord } from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { extractErrorMessage } from '@/utils/errorHandler'
import { mapTemplateToCodexProviderPatch } from '@/utils/providerTemplates'
import { createTf, type TranslateFunction } from '@/utils/tf'

export interface CodexProviderForm {
  id: string
  name: string
  baseUrl: string
  websiteUrl: string
  apiKeyUrl: string
  apiKeyName: string
  apiKey: string
}

const PROVIDER_FORM_DEFAULTS: CodexProviderForm = {
  id: '',
  name: '',
  baseUrl: '',
  websiteUrl: '',
  apiKeyUrl: '',
  apiKeyName: 'API Key',
  apiKey: '',
}

type ConfirmDialogOptions = {
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
  action: () => Promise<void>
}

export function useCodexProviders(deps: {
  t: TranslateFunction
  openConfirmDialog: (options: ConfirmDialogOptions) => void
  setActiveManagerTab: (tab: 'accounts' | 'providers') => void
}) {
  const { t, openConfirmDialog, setActiveManagerTab } = deps
  const tf = useMemo(() => createTf(t), [t])
  const queryClient = useQueryClient()
  const form = useForm<CodexProviderForm>({ defaultValues: PROVIDER_FORM_DEFAULTS })
  const providerForm = form.watch()
  const [providerError, setProviderError] = useState<string | null>(null)
  const [selectedProviderTemplate, setSelectedProviderTemplate] = useState<string | null>(null)
  const [selectedProviderEndpoint, setSelectedProviderEndpoint] = useState('')

  const providersQuery = useQuery({
    queryKey: codexKeys.providers.list(),
    staleTime: Infinity,
    queryFn: codexListModelProviders,
  })
  const providers: CodexModelProviderRecord[] = providersQuery.data?.providers ?? []
  const reloadProviders = useCallback(async () => {
    await providersQuery.refetch()
  }, [providersQuery])
  const invalidateProviders = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: codexKeys.providers.all })
  }, [queryClient])

  const codexTemplateDraft = useMemo<ProviderTemplateDraftContext>(
    () => ({
      platform: 'codex',
      defaultName: providerForm.name || 'Codex provider',
      name: providerForm.name,
      websiteUrl: providerForm.websiteUrl,
      apiKeyUrl: providerForm.apiKeyUrl,
      category: 'third_party',
      baseUrls: providerForm.baseUrl.trim() ? [providerForm.baseUrl.trim()] : [],
      platformOverride: {
        baseUrl: providerForm.baseUrl,
        websiteUrl: providerForm.websiteUrl,
        apiKeyUrl: providerForm.apiKeyUrl,
      },
    }),
    [providerForm],
  )

  const formatProviderUpdatedAt = useCallback(
    (value?: string | null, detailed = false) => {
      if (!value) return t('common.notAvailable')
      const date = new Date(value)
      if (Number.isNaN(date.getTime())) return value
      return detailed
        ? date.toLocaleString()
        : new Intl.DateTimeFormat('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(date)
    },
    [t],
  )

  const resetProviderForm = useCallback(() => {
    form.reset(PROVIDER_FORM_DEFAULTS)
    setProviderError(null)
    setSelectedProviderTemplate(null)
    setSelectedProviderEndpoint('')
  }, [form])

  const editProvider = useCallback(
    (provider: CodexModelProviderRecord) => {
      form.reset({
        id: provider.id,
        name: provider.name,
        baseUrl: provider.base_url,
        websiteUrl: provider.website_url || '',
        apiKeyUrl: provider.api_key_url || '',
        apiKeyName: provider.api_keys[0]?.name || 'API Key',
        apiKey: provider.api_keys[0]?.api_key || '',
      })
      setSelectedProviderTemplate(null)
      setSelectedProviderEndpoint('')
      setActiveManagerTab('providers')
    },
    [form, setActiveManagerTab],
  )

  const useManualProviderTemplate = useCallback(() => {
    setSelectedProviderTemplate(null)
    setSelectedProviderEndpoint('')
  }, [])

  const applyCodexProviderTemplate = useCallback(
    (selection: ProviderTemplateSelection) => {
      const patch = mapTemplateToCodexProviderPatch(selection.template, selection.endpoint)
      setSelectedProviderTemplate(selection.template.id)
      setSelectedProviderEndpoint(selection.endpoint || '')
      form.setValue('name', patch.name || selection.template.name)
      form.setValue('baseUrl', patch.baseUrl || '')
      form.setValue('websiteUrl', patch.websiteUrl || '')
      form.setValue('apiKeyUrl', patch.apiKeyUrl || '')
      setProviderError(null)
    },
    [form],
  )

  const saveMutation = useMutation({ mutationFn: codexSaveModelProvider })
  const handleSaveProvider = useCallback(async () => {
    setProviderError(null)
    const values = form.getValues()
    if (!values.name.trim()) {
      setProviderError(tf('codex.auth.providers.validation.nameRequired', 'Provider name is required.'))
      return
    }
    if (!values.baseUrl.trim()) {
      setProviderError(tf('codex.auth.providers.validation.baseUrlRequired', 'Base URL is required.'))
      return
    }
    try {
      await saveMutation.mutateAsync({
        id: values.id || undefined,
        name: values.name.trim(),
        baseUrl: values.baseUrl.trim(),
        websiteUrl: values.websiteUrl.trim() || undefined,
        apiKeyUrl: values.apiKeyUrl.trim() || undefined,
        apiKeyName: values.apiKeyName.trim() || undefined,
        apiKey: values.apiKey.trim() || undefined,
      })
      invalidateProviders()
      await reloadProviders()
      resetProviderForm()
      surfaceNotify.success(tf('codex.auth.providers.saveSuccess', 'Saved provider saved successfully.'))
    } catch (error) {
      setProviderError(extractErrorMessage(error) || tf('codex.auth.providers.saveFailed', 'Failed to save the saved provider.'))
    }
  }, [form, invalidateProviders, reloadProviders, resetProviderForm, saveMutation, tf])

  const deleteMutation = useMutation({ mutationFn: codexDeleteModelProvider })
  const requestDeleteProvider = useCallback(
    (provider: CodexModelProviderRecord) => {
      openConfirmDialog({
        title: tf('codex.auth.providers.deleteTitle', 'Delete saved provider'),
        message: tf(
          'codex.auth.providers.deleteMessage',
          'Delete saved provider "{name}"? Stored API keys under this saved provider will also be removed.',
          { name: provider.name },
        ),
        confirmText: t('codex.actions.delete'),
        type: 'danger',
        action: async () => {
          try {
            await deleteMutation.mutateAsync(provider.id)
            invalidateProviders()
            await reloadProviders()
            surfaceNotify.success(tf('codex.auth.providers.deleteSuccess', 'Saved provider deleted successfully.'))
          } catch (error) {
            setProviderError(extractErrorMessage(error) || tf('codex.auth.providers.deleteFailed', 'Failed to delete the saved provider.'))
          }
        },
      })
    },
    [deleteMutation, invalidateProviders, openConfirmDialog, reloadProviders, t, tf],
  )

  return {
    providers,
    providerError:
      providerError ??
      (providersQuery.error
        ? extractErrorMessage(providersQuery.error) || tf('codex.auth.providers.loadFailed', 'Failed to load saved providers.')
        : null),
    providerLoading: providersQuery.isFetching,
    providerSaving: saveMutation.isPending,
    providerForm,
    providerFormApi: form,
    selectedProviderTemplate,
    selectedProviderEndpoint,
    codexTemplateDraft,
    formatProviderUpdatedAt,
    loadProviders: reloadProviders,
    resetProviderForm,
    editProvider,
    useManualProviderTemplate,
    applyCodexProviderTemplate,
    handleSaveProvider,
    requestDeleteProvider,
  }
}
