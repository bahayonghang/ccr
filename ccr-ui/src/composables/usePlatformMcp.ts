/**
 * usePlatformMcp - 通用平台 MCP 服务器管理 Hook（React）
 *
 * 消除各平台 MCP 视图中的重复代码（当前复用于 Gemini MCP 视图）
 *
 * React 迁移（08-22-state-logic-port 批次 5b-ii）：
 * - 服务器列表 → Query（mcpKeys.platformServers，staleTime Infinity：原实现为挂载
 *   显式加载 + 写操作后显式重载，无 TTL）；CRUD → useMutation + refetch；
 * - 表单（formData/argInput/env 输入）→ react-hook-form + useState
 *   （state-disposition.md SPLIT 判定：服务器数据 Query、表单瞬态 RHF）；
 * - 加载失败 toast 在 queryFn 内触发一次（Query 去重保证不重复弹）。
 *
 * 签名变化（消费方 PlatformMcpView.vue 待迁移）：useI18n → t 参数传入；
 * Ref<T>/computed → 普通值；formData 返回 RHF watch 快照，方法集经 formApi 暴露。
 *
 * @example
 * const { servers, loading, loadServers, addServer, updateServer, deleteServer } =
 *   usePlatformMcp('gemini', { t })
 */

import { useMutation, useQuery } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API。
import { useUIStore } from '@/shell/stores/ui'
import {
  listGeminiMcpServers,
  addGeminiMcpServer,
  updateGeminiMcpServer,
  deleteGeminiMcpServer,
} from '@/api'
import { genericPlatformDescriptors, type GenericPlatformId } from '@/config/platformDescriptors'
import { mcpKeys } from '@/features/mcp/queries'
import type { TranslateFunction } from '@/utils/tf'
import type { GeminiMcpServer, GeminiMcpServerRequest } from '@/types'
import { logger } from '@/utils/logger'


/** 支持的平台类型 */
export type PlatformType = GenericPlatformId

/** 统一的 MCP 服务器类型（合并各平台差异） */
export interface PlatformMcpServer {
  name: string
  command?: string
  url?: string
  args?: string[]
  env?: Record<string, string>
  cwd?: string
  timeout?: number
  trust?: boolean
  includeTools?: string[]
  headers?: Record<string, string>
}

/** 统一的 MCP 服务器请求类型 */
export interface UnifiedMcpServerRequest {
  name: string
  command?: string
  url?: string
  args?: string[]
  env?: Record<string, string>
  cwd?: string
  timeout?: number
  trust?: boolean
  includeTools?: string[]
  headers?: Record<string, string>
}

/** 平台配置 */
interface PlatformMcpConfig {
  color: string
  i18nPrefix: string
  parentPath: string
  listApi: () => Promise<PlatformMcpServer[]>
  addApi: (req: UnifiedMcpServerRequest) => Promise<string>
  updateApi: (name: string, req: UnifiedMcpServerRequest) => Promise<string>
  deleteApi: (name: string) => Promise<string>
}

// ============ 平台 API 映射 ============

const platformApiMap: Record<
  PlatformType,
  Pick<PlatformMcpConfig, 'listApi' | 'addApi' | 'updateApi' | 'deleteApi'>
> = {
  gemini: {
    listApi: async () => {
      const servers = await listGeminiMcpServers()
      return servers.map(normalizeServer)
    },
    addApi: async (req) => {
      const geminiReq: GeminiMcpServerRequest = {
        name: req.name,
        command: req.command,
        args: req.args,
        env: req.env,
        cwd: req.cwd,
        timeout: req.timeout,
        trust: req.trust,
        includeTools: req.includeTools,
        url: req.url,
      }
      return addGeminiMcpServer(geminiReq)
    },
    updateApi: async (name, req) => {
      const geminiReq: GeminiMcpServerRequest = {
        name: req.name,
        command: req.command,
        args: req.args,
        env: req.env,
        cwd: req.cwd,
        timeout: req.timeout,
        trust: req.trust,
        includeTools: req.includeTools,
        url: req.url,
      }
      return updateGeminiMcpServer(name, geminiReq)
    },
    deleteApi: deleteGeminiMcpServer,
  },
}
const platformConfigs: Record<PlatformType, PlatformMcpConfig> = {
  gemini: {
    color: genericPlatformDescriptors.gemini.color,
    i18nPrefix: genericPlatformDescriptors.gemini.mcp.i18nPrefix,
    parentPath: `/${genericPlatformDescriptors.gemini.rootPath}`,
    ...platformApiMap.gemini,
  },
}

// ============ 辅助函数 ============

/** 统一各平台服务器数据结构 */
function normalizeServer(server: GeminiMcpServer): PlatformMcpServer {
  return {
    name: server.name,
    command: server.command,
    url: server.url,
    args: server.args,
    env: server.env,
    cwd: 'cwd' in server ? server.cwd : undefined,
    timeout: server.timeout,
    trust: 'trust' in server ? server.trust : undefined,
    includeTools: 'includeTools' in server ? server.includeTools : undefined,
    headers: undefined,
  }
}

/** 获取服务器标识符（用于编辑/删除） */
export function getServerIdentifier(server: PlatformMcpServer): string {
  return server.name || server.command || server.url || ''
}

function createEmptyFormData(): UnifiedMcpServerRequest {
  return {
    name: '',
    command: undefined,
    url: undefined,
    args: [],
    env: {},
  }
}

// ============ Hook 主体 ============

export function usePlatformMcp(platform: PlatformType, deps: { t: TranslateFunction }) {
  const { t } = deps

  // 获取平台配置（platform 为挂载期常量，配置直接查表，无响应性需求）
  const config = platformConfigs[platform]
  const uiStore = useUIStore.getState()

  // ============ 服务端数据 ============

  const serversQuery = useQuery({
    queryKey: mcpKeys.platformServers(platform),
    staleTime: Infinity,
    queryFn: async () => {
      try {
        return await config.listApi()
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error'
        logger.error(`Failed to load ${platform} MCP servers: ${errorMessage}`, err)
        uiStore.showError(t(`${config.i18nPrefix}.loadFailed`, { error: errorMessage }))
        throw err
      }
    },
  })

  const servers = serversQuery.data ?? []
  const loading = serversQuery.isFetching
  const error = serversQuery.error
    ? serversQuery.error instanceof Error
      ? serversQuery.error.message
      : 'Unknown error'
    : null

  const { refetch: refetchServers } = serversQuery

  /** 重拉列表（原 await loadServers 语义）。 */
  const reloadServers = useCallback(async () => {
    await refetchServers()
  }, [refetchServers])

  // ============ 表单状态 ============

  const form = useForm<UnifiedMcpServerRequest>({ defaultValues: createEmptyFormData() })
  const formData = form.watch()

  const [showForm, setShowForm] = useState(false)
  const [editingServer, setEditingServer] = useState<PlatformMcpServer | null>(null)
  const [isHttpServer, setIsHttpServer] = useState(false)
  const [argInput, setArgInput] = useState('')
  const [envKey, setEnvKey] = useState('')
  const [envValue, setEnvValue] = useState('')

  // ============ CRUD 操作 ============

  const addMutation = useMutation({ mutationFn: config.addApi })
  const updateMutation = useMutation({
    mutationFn: ({ name, request }: { name: string; request: UnifiedMcpServerRequest }) =>
      config.updateApi(name, request),
  })
  const deleteMutation = useMutation({ mutationFn: config.deleteApi })

  /** 关闭表单 */
  const closeForm = useCallback(() => {
    setShowForm(false)
    setEditingServer(null)
  }, [])

  /** 校验表单（原 validateForm；读 RHF 当前值与 isHttpServer 状态）。 */
  const validateForm = useCallback(() => {
    const values = form.getValues()
    if (!isHttpServer && !values.command) {
      uiStore.showWarning(t(`${config.i18nPrefix}.validation.commandRequired`))
      return false
    }
    if (isHttpServer && !values.url) {
      uiStore.showWarning(t(`${config.i18nPrefix}.validation.urlRequired`))
      return false
    }
    return true
  }, [config, form, isHttpServer, t, uiStore])

  /** 由表单值构建请求（原 buildRequest：args 拆分 + 模式清理 + 名称回退）。 */
  const buildRequest = useCallback(() => {
    const values = form.getValues()
    const args = argInput
      .split(' ')
      .map((a) => a.trim())
      .filter(Boolean)

    const request: UnifiedMcpServerRequest = {
      ...values,
      args,
    }

    // 根据服务器类型清理不需要的字段
    if (isHttpServer) {
      request.command = undefined
    } else {
      request.url = undefined
    }

    // 当请求未提供名称时，回退到 command/url 生成稳定标识。
    if (!request.name) {
      request.name = request.command || request.url || 'unknown'
    }

    return request
  }, [argInput, form, isHttpServer])

  /** 添加服务器 */
  const addServer = useCallback(async (): Promise<boolean> => {
    if (!validateForm()) return false

    const request = buildRequest()
    try {
      await addMutation.mutateAsync(request)
      uiStore.showSuccess(t(`${config.i18nPrefix}.addSuccess`))
      await reloadServers()
      closeForm()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.i18nPrefix}.operationFailed`, { error: errorMessage }))
      return false
    }
  }, [
    addMutation,
    buildRequest,
    closeForm,
    config,
    reloadServers,
    t,
    uiStore,
    validateForm,
  ])

  /** 更新服务器 */
  const updateServer = useCallback(async (): Promise<boolean> => {
    if (!editingServer || !validateForm()) return false

    const name = getServerIdentifier(editingServer)
    const request = buildRequest()
    try {
      await updateMutation.mutateAsync({ name, request })
      uiStore.showSuccess(t(`${config.i18nPrefix}.updateSuccess`))
      await reloadServers()
      closeForm()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.i18nPrefix}.operationFailed`, { error: errorMessage }))
      return false
    }
  }, [
    buildRequest,
    closeForm,
    config,
    editingServer,
    reloadServers,
    t,
    uiStore,
    updateMutation,
    validateForm,
  ])

  /** 删除服务器（纯执行器，确认决策上移到调用视图） */
  const deleteServer = useCallback(
    async (server: PlatformMcpServer): Promise<boolean> => {
      const name = getServerIdentifier(server)
      try {
        await deleteMutation.mutateAsync(name)
        uiStore.showSuccess(t(`${config.i18nPrefix}.deleteSuccess`))
        await reloadServers()
        return true
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error'
        uiStore.showError(t(`${config.i18nPrefix}.deleteFailed`, { error: errorMessage }))
        return false
      }
    },
    [config, deleteMutation, reloadServers, t, uiStore]
  )

  // ============ 表单操作 ============

  /** 打开添加表单 */
  const openAddForm = useCallback(() => {
    setEditingServer(null)
    setIsHttpServer(false)
    form.reset(createEmptyFormData())
    setArgInput('')
    setEnvKey('')
    setEnvValue('')
    setShowForm(true)
  }, [form])

  /** 打开编辑表单 */
  const openEditForm = useCallback(
    (server: PlatformMcpServer) => {
      setEditingServer(server)
      setIsHttpServer(!!server.url)
      form.reset({ ...server, args: server.args?.length ? server.args : [] })
      setArgInput(server.args?.join(' ') || '')
      setEnvKey('')
      setEnvValue('')
      setShowForm(true)
    },
    [form]
  )

  /** 提交表单（添加或更新） */
  const submitForm = useCallback((): Promise<boolean> => {
    return editingServer ? updateServer() : addServer()
  }, [addServer, editingServer, updateServer])

  /** 添加环境变量 */
  const addEnvVar = useCallback(() => {
    if (envKey && envValue) {
      form.setValue('env', { ...(form.getValues('env') ?? {}), [envKey]: envValue })
      setEnvKey('')
      setEnvValue('')
    }
  }, [envKey, envValue, form])

  /** 删除环境变量 */
  const removeEnvVar = useCallback(
    (key: string) => {
      const newEnv = { ...(form.getValues('env') ?? {}) }
      delete newEnv[key]
      form.setValue('env', newEnv)
    },
    [form]
  )

  const moduleColor = config.color
  const i18nPrefix = useMemo(() => config.i18nPrefix, [config])
  const parentPath = config.parentPath

  // ============ 返回值 ============

  return {
    // 平台配置
    config,
    platform,
    moduleColor,
    i18nPrefix,
    parentPath,

    // 数据状态
    servers,
    loading,
    error,

    // 表单状态
    showForm,
    editingServer,
    isHttpServer,
    formData,
    /** react-hook-form 方法集（register/setValue/reset），供迁移后的视图绑定表单 */
    formApi: form,
    argInput,
    envKey,
    envValue,
    setArgInput,
    setEnvKey,
    setEnvValue,
    setShowForm,
    setIsHttpServer,

    // CRUD 操作
    loadServers: reloadServers,
    addServer,
    updateServer,
    deleteServer,

    // 表单操作
    openAddForm,
    openEditForm,
    closeForm,
    submitForm,
    addEnvVar,
    removeEnvVar,

    // 工具函数
    getServerIdentifier,
  }
}

export default usePlatformMcp
