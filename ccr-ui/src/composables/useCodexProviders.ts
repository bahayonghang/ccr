import { computed, reactive, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { codexDeleteModelProvider, codexListModelProviders, codexSaveModelProvider } from '@/api'
import type { CodexModelProviderRecord } from '@/types'
import type {
  ProviderTemplateDraftContext,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import { useTf } from '@/composables/useTf'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import { extractErrorMessage } from '@/utils/errorHandler'
import { mapTemplateToCodexProviderPatch } from '@/utils/providerTemplates'

export interface CodexProviderForm {
  id: string
  name: string
  baseUrl: string
  websiteUrl: string
  apiKeyUrl: string
  apiKeyName: string
  apiKey: string
}

type ConfirmDialogOptions = {
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
  action: () => Promise<void>
}

/**
 * Codex 模型提供商（Saved provider）CRUD 子系统：列表加载、表单状态、模板套用，
 * 以及新建/更新/删除与更新时间格式化。删除复用主视图的共享 ConfirmModal，编辑时
 * 切回 providers 面板，故由主视图注入 openConfirmDialog 与 activeManagerTab。
 */
export function useCodexProviders(deps: {
  /** 共享确认弹窗：删除提供商时复用主视图的 ConfirmModal */
  openConfirmDialog: (options: ConfirmDialogOptions) => void
  /** 主视图当前 Tab；编辑提供商时切回 providers 面板 */
  activeManagerTab: Ref<'accounts' | 'providers'>
}) {
  const { openConfirmDialog, activeManagerTab } = deps

  const { t } = useI18n()
  const tf = useTf()
  const uiStore = useUIStore()

  const providers = ref<CodexModelProviderRecord[]>([])
  const providerError = ref<string | null>(null)
  const providerLoading = ref(false)
  const providerSaving = ref(false)

  const providerForm = reactive<CodexProviderForm>({
    id: '',
    name: '',
    baseUrl: '',
    websiteUrl: '',
    apiKeyUrl: '',
    apiKeyName: 'API Key',
    apiKey: '',
  })
  const selectedProviderTemplate = ref<string | null>(null)
  const selectedProviderEndpoint = ref('')

  const codexTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
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
  }))

  const formatProviderUpdatedAt = (value?: string | null, detailed = false) => {
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
  }

  const loadProviders = async () => {
    try {
      providerLoading.value = true
      providerError.value = null
      const data = await codexListModelProviders()
      providers.value = data.providers || []
    } catch (error) {
      logger.error('Failed to load codex providers:', error)
      providerError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.providers.loadFailed', 'Failed to load saved providers.')
    } finally {
      providerLoading.value = false
    }
  }

  const resetProviderForm = () => {
    providerForm.id = ''
    providerForm.name = ''
    providerForm.baseUrl = ''
    providerForm.websiteUrl = ''
    providerForm.apiKeyUrl = ''
    providerForm.apiKeyName = 'API Key'
    providerForm.apiKey = ''
    providerError.value = null
    selectedProviderTemplate.value = null
    selectedProviderEndpoint.value = ''
  }

  const editProvider = (provider: CodexModelProviderRecord) => {
    providerForm.id = provider.id
    providerForm.name = provider.name
    providerForm.baseUrl = provider.base_url
    providerForm.websiteUrl = provider.website_url || ''
    providerForm.apiKeyUrl = provider.api_key_url || ''
    providerForm.apiKeyName = provider.api_keys[0]?.name || 'API Key'
    providerForm.apiKey = provider.api_keys[0]?.api_key || ''
    selectedProviderTemplate.value = null
    selectedProviderEndpoint.value = ''
    activeManagerTab.value = 'providers'
  }

  const useManualProviderTemplate = () => {
    selectedProviderTemplate.value = null
    selectedProviderEndpoint.value = ''
  }

  const applyCodexProviderTemplate = (selection: ProviderTemplateSelection) => {
    const patch = mapTemplateToCodexProviderPatch(selection.template, selection.endpoint)

    selectedProviderTemplate.value = selection.template.id
    selectedProviderEndpoint.value = selection.endpoint || ''
    providerForm.name = patch.name || selection.template.name
    providerForm.baseUrl = patch.baseUrl || ''
    providerForm.websiteUrl = patch.websiteUrl || ''
    providerForm.apiKeyUrl = patch.apiKeyUrl || ''
    providerError.value = null
  }

  const handleSaveProvider = async () => {
    providerError.value = null
    if (!providerForm.name.trim()) {
      providerError.value = tf(
        'codex.auth.providers.validation.nameRequired',
        'Provider name is required.'
      )
      return
    }
    if (!providerForm.baseUrl.trim()) {
      providerError.value = tf(
        'codex.auth.providers.validation.baseUrlRequired',
        'Base URL is required.'
      )
      return
    }

    try {
      providerSaving.value = true
      await codexSaveModelProvider({
        id: providerForm.id || undefined,
        name: providerForm.name.trim(),
        baseUrl: providerForm.baseUrl.trim(),
        websiteUrl: providerForm.websiteUrl.trim() || undefined,
        apiKeyUrl: providerForm.apiKeyUrl.trim() || undefined,
        apiKeyName: providerForm.apiKeyName.trim() || undefined,
        apiKey: providerForm.apiKey.trim() || undefined,
      })
      await loadProviders()
      resetProviderForm()
      uiStore.showSuccess(
        tf('codex.auth.providers.saveSuccess', 'Saved provider saved successfully.')
      )
    } catch (error) {
      providerError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.providers.saveFailed', 'Failed to save the saved provider.')
    } finally {
      providerSaving.value = false
    }
  }

  const requestDeleteProvider = (provider: CodexModelProviderRecord) => {
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
          await codexDeleteModelProvider(provider.id)
          await loadProviders()
          uiStore.showSuccess(
            tf('codex.auth.providers.deleteSuccess', 'Saved provider deleted successfully.')
          )
        } catch (error) {
          providerError.value =
            extractErrorMessage(error) ||
            tf('codex.auth.providers.deleteFailed', 'Failed to delete the saved provider.')
        }
      },
    })
  }

  return {
    providers,
    providerError,
    providerLoading,
    providerSaving,
    providerForm,
    selectedProviderTemplate,
    selectedProviderEndpoint,
    codexTemplateDraft,
    formatProviderUpdatedAt,
    loadProviders,
    resetProviderForm,
    editProvider,
    useManualProviderTemplate,
    applyCodexProviderTemplate,
    handleSaveProvider,
    requestDeleteProvider,
  }
}
