/**
 * usePlatformPlugins - 通用平台插件管理 Hook（React）
 *
 * 消除各平台 Plugins 视图中的重复代码（GeminiPluginsView）
 *
 * React 迁移（08-22-state-logic-port 批次 5b-ii）：
 * - 插件列表 → Query（mcpKeys.plugins，staleTime Infinity：原实现为挂载显式加载 +
 *   写操作后显式重载，无 TTL）；CRUD/toggle → useMutation + refetch；
 * - 表单（formData/configJson）→ react-hook-form + useState
 *   （state-disposition.md SPLIT 判定）；加载失败 toast 在 queryFn 内触发一次。
 *
 * 签名变化（消费方 PlatformPluginsView.vue 待迁移）：useI18n → t 参数传入；
 * Ref<T>/computed → 普通值；formData 返回 RHF watch 快照，方法集经 formApi 暴露。
 *
 * @example
 * const { plugins, loading, loadPlugins, addPlugin, updatePlugin, deletePlugin, togglePlugin } =
 *   usePlatformPlugins('gemini', { t })
 */

import { useMutation, useQuery } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API。
import { useUIStore } from '@/shell/stores/ui'
import {
  listGeminiPlugins,
  addGeminiPlugin,
  updateGeminiPlugin,
  deleteGeminiPlugin,
  toggleGeminiPlugin,
} from '@/api'
import { genericPlatformDescriptors } from '@/config/platformDescriptors'
import { mcpKeys } from '@/features/mcp/queries'
import type { TranslateFunction } from '@/utils/tf'
import type { Plugin as PluginType, PluginRequest } from '@/types'
import { logger } from '@/utils/logger'

// ============ 类型定义 ============

/** 支持的平台类型 */
export type PluginPlatformType = 'gemini'

/** 平台配置 */
interface PlatformPluginConfig {
  color: string
  i18nPrefix: string
  parentPath: string
  sidebarModule: string
  listApi: () => Promise<PluginType[]>
  addApi: (req: PluginRequest) => Promise<string>
  updateApi: (id: string, req: PluginRequest) => Promise<string>
  deleteApi: (id: string) => Promise<string>
  toggleApi: (id: string) => Promise<string>
}

// ============ 平台 API 映射 ============

const platformConfigs: Record<PluginPlatformType, PlatformPluginConfig> = {
  gemini: {
    color: genericPlatformDescriptors.gemini.color,
    i18nPrefix: genericPlatformDescriptors.gemini.plugins!.i18nPrefix,
    parentPath: `/${genericPlatformDescriptors.gemini.rootPath}`,
    sidebarModule: genericPlatformDescriptors.gemini.plugins!.sidebarModule,
    listApi: listGeminiPlugins,
    addApi: addGeminiPlugin,
    updateApi: updateGeminiPlugin,
    deleteApi: deleteGeminiPlugin,
    toggleApi: toggleGeminiPlugin,
  },
}

const PLUGIN_FORM_DEFAULTS: PluginRequest = {
  id: '',
  name: '',
  version: '1.0.0',
  enabled: true,
  config: undefined,
}

// ============ Hook 主体 ============

export function usePlatformPlugins(
  platform: PluginPlatformType,
  deps: { t: TranslateFunction }
) {
  const { t } = deps

  // 获取平台配置（platform 为挂载期常量，配置直接查表，无响应性需求）
  const config = platformConfigs[platform]
  const uiStore = useUIStore.getState()

  // ============ 服务端数据 ============

  const pluginsQuery = useQuery({
    queryKey: mcpKeys.plugins(platform),
    staleTime: Infinity,
    queryFn: async () => {
      try {
        return await config.listApi()
      } catch (err) {
        logger.error(`Failed to load ${platform} plugins:`, err)
        uiStore.showError(t(`${config.i18nPrefix}.messages.loadFailed`))
        throw err
      }
    },
  })

  const plugins = pluginsQuery.data ?? []
  const loading = pluginsQuery.isFetching
  const error = pluginsQuery.error
    ? pluginsQuery.error instanceof Error
      ? pluginsQuery.error.message
      : 'Unknown error'
    : null

  const { refetch: refetchPlugins } = pluginsQuery

  /** 重拉列表（原 await loadPlugins 语义）。 */
  const reloadPlugins = useCallback(async () => {
    await refetchPlugins()
  }, [refetchPlugins])

  // ============ 表单状态 ============

  const form = useForm<PluginRequest>({ defaultValues: PLUGIN_FORM_DEFAULTS })
  const formData = form.watch()

  const [showForm, setShowForm] = useState(false)
  const [editingPlugin, setEditingPlugin] = useState<PluginType | null>(null)
  const [configJson, setConfigJson] = useState('')

  // ============ CRUD 操作 ============

  const addMutation = useMutation({ mutationFn: config.addApi })
  const updateMutation = useMutation({
    mutationFn: ({ id, request }: { id: string; request: PluginRequest }) =>
      config.updateApi(id, request),
  })
  const deleteMutation = useMutation({ mutationFn: config.deleteApi })
  const toggleMutation = useMutation({ mutationFn: config.toggleApi })

  /** 关闭表单 */
  const closeForm = useCallback(() => {
    setShowForm(false)
    setEditingPlugin(null)
  }, [])

  /** 校验表单（原 validateForm）。 */
  const validateForm = useCallback(() => {
    const values = form.getValues()
    if (!values.id || !values.name || !values.version) {
      uiStore.showWarning(t(`${config.i18nPrefix}.validation.required`))
      return false
    }
    return true
  }, [config.i18nPrefix, form, t, uiStore])

  /** 由 configJson 构建请求（原 buildRequest：JSON 解析失败时提示并返回 null）。 */
  const buildRequest = useCallback((): PluginRequest | null => {
    let parsedConfig: Record<string, unknown> | undefined = undefined

    if (configJson.trim()) {
      try {
        parsedConfig = JSON.parse(configJson) as Record<string, unknown>
      } catch {
        uiStore.showError(t(`${config.i18nPrefix}.validation.invalidJson`))
        return null
      }
    }

    return {
      ...form.getValues(),
      config: parsedConfig,
    }
  }, [config.i18nPrefix, configJson, form, t, uiStore])

  /** 添加插件 */
  const addPlugin = useCallback(async (): Promise<boolean> => {
    if (!validateForm()) return false

    const request = buildRequest()
    if (!request) return false

    try {
      await addMutation.mutateAsync(request)
      uiStore.showSuccess(t(`${config.i18nPrefix}.messages.addSuccess`))
      await reloadPlugins()
      closeForm()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.i18nPrefix}.messages.operationFailed`, { error: errorMessage }))
      return false
    }
  }, [
    addMutation,
    buildRequest,
    closeForm,
    config,
    reloadPlugins,
    t,
    uiStore,
    validateForm,
  ])

  /** 更新插件 */
  const updatePlugin = useCallback(async (): Promise<boolean> => {
    if (!editingPlugin || !validateForm()) return false

    const request = buildRequest()
    if (!request) return false

    try {
      await updateMutation.mutateAsync({ id: editingPlugin.id, request })
      uiStore.showSuccess(t(`${config.i18nPrefix}.messages.updateSuccess`))
      await reloadPlugins()
      closeForm()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.i18nPrefix}.messages.operationFailed`, { error: errorMessage }))
      return false
    }
  }, [
    buildRequest,
    closeForm,
    config,
    editingPlugin,
    reloadPlugins,
    t,
    uiStore,
    updateMutation,
    validateForm,
  ])

  /** 删除插件（纯执行器，确认决策上移到调用视图） */
  const deletePlugin = useCallback(
    async (plugin: PluginType): Promise<boolean> => {
      try {
        await deleteMutation.mutateAsync(plugin.id)
        uiStore.showSuccess(t(`${config.i18nPrefix}.messages.deleteSuccess`))
        await reloadPlugins()
        return true
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error'
        uiStore.showError(t(`${config.i18nPrefix}.messages.deleteFailed`, { error: errorMessage }))
        return false
      }
    },
    [config, deleteMutation, reloadPlugins, t, uiStore]
  )

  /** 切换插件启用状态 */
  const togglePlugin = useCallback(
    async (plugin: PluginType): Promise<boolean> => {
      try {
        await toggleMutation.mutateAsync(plugin.id)
        await reloadPlugins()
        return true
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error'
        uiStore.showError(t(`${config.i18nPrefix}.messages.toggleFailed`, { error: errorMessage }))
        return false
      }
    },
    [config, reloadPlugins, t, uiStore, toggleMutation]
  )

  // ============ 表单操作 ============

  /** 打开添加表单 */
  const openAddForm = useCallback(() => {
    setEditingPlugin(null)
    form.reset(PLUGIN_FORM_DEFAULTS)
    setConfigJson('')
    setShowForm(true)
  }, [form])

  /** 打开编辑表单 */
  const openEditForm = useCallback(
    (plugin: PluginType) => {
      setEditingPlugin(plugin)
      form.reset({
        id: plugin.id,
        name: plugin.name,
        version: plugin.version,
        enabled: plugin.enabled,
        config: plugin.config,
      })
      setConfigJson(plugin.config ? JSON.stringify(plugin.config, null, 2) : '')
      setShowForm(true)
    },
    [form]
  )

  /** 提交表单（添加或更新） */
  const submitForm = useCallback((): Promise<boolean> => {
    return editingPlugin ? updatePlugin() : addPlugin()
  }, [addPlugin, editingPlugin, updatePlugin])

  const moduleColor = config.color
  const i18nPrefix = useMemo(() => config.i18nPrefix, [config])
  const parentPath = config.parentPath
  const sidebarModule = config.sidebarModule

  // ============ 返回值 ============

  return {
    // 平台配置
    config,
    platform,
    moduleColor,
    i18nPrefix,
    parentPath,
    sidebarModule,

    // 数据状态
    plugins,
    loading,
    error,

    // 表单状态
    showForm,
    editingPlugin,
    formData,
    /** react-hook-form 方法集（register/setValue/reset），供迁移后的视图绑定表单 */
    formApi: form,
    configJson,
    setConfigJson,
    setShowForm,

    // CRUD 操作
    loadPlugins: reloadPlugins,
    addPlugin,
    updatePlugin,
    deletePlugin,
    togglePlugin,

    // 表单操作
    openAddForm,
    openEditForm,
    closeForm,
    submitForm,
  }
}

export default usePlatformPlugins
