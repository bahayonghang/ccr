import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { codexDeleteModelProvider, codexListModelProviders, codexSaveModelProvider } from '@/api'
import { codexKeys } from '@/features/codex/queries'
import type { CodexModelProviderRecord } from '@/types'
import type {
  ProviderTemplateDraftContext,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import { createTf, type TranslateFunction } from '@/utils/tf'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API。
import { useUIStore } from '@/shell/stores/ui'
import { extractErrorMessage } from '@/utils/errorHandler'
import { mapTemplateToCodexProviderPatch } from '@/utils/providerTemplates'

// Codex 模型提供商（Saved provider）CRUD 的 React 迁移（批次 5b-ii）。
// - providers 列表 → Query（codexKeys.providers.list，staleTime Infinity：原实现每次
//   显式 loadProviders 重拉，挂载拉取一次，无 TTL、无聚焦刷新）；
// - providerForm（reactive）→ react-hook-form（单表单瞬态，state-disposition.md SPLIT 判定）；
// - loadProviders → refetch；save/delete → useMutation + invalidate；
// - selectedProviderTemplate / selectedProviderEndpoint → useState；
// - codexTemplateDraft（computed）→ useMemo（来源：表单 watch 值）。
//
// 签名变化（消费方 CodexAuthView.vue 待迁移）：
// - t 参数传入（与 useCodexDashboard 同形态）；
// - deps.activeManagerTab: Ref → setActiveManagerTab 回调；
// - providerForm 返回 RHF watch 快照（普通对象）；表单方法经 providerFormApi 暴露。

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
  /** i18n 翻译函数 */
  t: TranslateFunction
  /** 共享确认弹窗：删除提供商时复用主视图的 ConfirmModal */
  openConfirmDialog: (options: ConfirmDialogOptions) => void
  /** 主视图当前 Tab 写入器；编辑提供商时切回 providers 面板 */
  setActiveManagerTab: (tab: 'accounts' | 'providers') => void
}) {
  const { t, openConfirmDialog, setActiveManagerTab } = deps

  const tf = createTf(t)
  const uiStore = useUIStore.getState()
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
  const providerLoading = providersQuery.isFetching

  const { refetch: refetchProviders } = providersQuery

  /** 写操作成功后重拉列表（原 await loadProviders 语义）。 */
  const reloadProviders = useCallback(async () => {
    await refetchProviders()
  }, [refetchProviders])

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
    [providerForm]
  )

  const formatProviderUpdatedAt = useCallback(
    (value?: string | null, detailed = false) => {
      if (!value) return t('common.notAvailable')
      const date = new Date(value)
      if (Number.isNaN(date.getTime())) return value
      return detailed
        ? date.toLocaleString()
        : new Intl.DateTimeFormat('zh-CN', {
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
          }).format(date)
    },
    [t]
  )

  // 列表加载失败并入 providerError（原 loadProviders catch 分支的赋值点）。
  const loadError = providersQuery.error
    ? extractErrorMessage(providersQuery.error) ||
      tf('codex.auth.providers.loadFailed', 'Failed to load saved providers.')
    : null

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
    [form, setActiveManagerTab]
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
    [form]
  )

  const saveMutation = useMutation({ mutationFn: codexSaveModelProvider })

  const handleSaveProvider = useCallback(async () => {
    setProviderError(null)
    const values = form.getValues()
    if (!values.name.trim()) {
      setProviderError(
        tf('codex.auth.providers.validation.nameRequired', 'Provider name is required.')
      )
      return
    }
    if (!values.baseUrl.trim()) {
      setProviderError(
        tf('codex.auth.providers.validation.baseUrlRequired', 'Base URL is required.')
      )
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
      uiStore.showSuccess(
        tf('codex.auth.providers.saveSuccess', 'Saved provider saved successfully.')
      )
    } catch (error) {
      setProviderError(
        extractErrorMessage(error) ||
          tf('codex.auth.providers.saveFailed', 'Failed to save the saved provider.')
      )
    }
  }, [
    form,
    invalidateProviders,
    reloadProviders,
    resetProviderForm,
    saveMutation,
    tf,
    uiStore,
  ])

  const deleteMutation = useMutation({ mutationFn: codexDeleteModelProvider })

  const requestDeleteProvider = useCallback(
    (provider: CodexModelProviderRecord) => {
      openConfirmDialog({
        title: tf('codex.auth.providers.deleteTitle', 'Delete saved provider'),
        message: tf(
          'codex.auth.providers.deleteMessage',
          'Delete saved provider "{name}"? Stored API keys under this saved provider will also be removed.',
          { name: provider.name }
        ),
        confirmText: t('codex.actions.delete'),
        type: 'danger',
        action: async () => {
          try {
            await deleteMutation.mutateAsync(provider.id)
            invalidateProviders()
            await reloadProviders()
            uiStore.showSuccess(
              tf('codex.auth.providers.deleteSuccess', 'Saved provider deleted successfully.')
            )
          } catch (error) {
            setProviderError(
              extractErrorMessage(error) ||
                tf('codex.auth.providers.deleteFailed', 'Failed to delete the saved provider.')
            )
          }
        },
      })
    },
    [deleteMutation, invalidateProviders, openConfirmDialog, reloadProviders, t, tf, uiStore]
  )

  return {
    providers,
    providerError: providerError ?? loadError,
    providerLoading,
    providerSaving: saveMutation.isPending,
    providerForm,
    /** react-hook-form 方法集（register/setValue/reset），供迁移后的视图绑定表单 */
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
