/**
 * useUnifiedMcp - 统一 MCP 服务器管理 Hook（React）
 *
 * 对接后端统一 MCP API（/api/unified/mcp），提供跨平台 MCP 服务器的
 * 列表、添加、编辑、删除、启停等全部操作。
 *
 * React 迁移（08-22-state-logic-port 批次 5b-ii）：
 * - 列表快照（servers + capabilities + diagnostics 单次 IPC）→ Query
 *   （mcpKeys.unifiedList，staleTime Infinity：原实现为挂载显式加载 + 写操作后
 *   显式重载，无 TTL）；CRUD/toggle → useMutation + refetch；
 * - 表单（formData/argInput/env/header/includeTools 输入）→ react-hook-form +
 *   useState（state-disposition.md SPLIT 判定）；
 * - 筛选（filter*）→ useState；filteredServers/scopeCounts/platformCounts 等
 *   computed → useMemo（来源逐一登记于 classification §3）；
 * - 加载失败 toast 在 queryFn 内触发一次（Query 去重保证不重复弹）。
 *
 * 签名变化（下游批次 5c 的 useMcpManager 消费）：Ref/computed → 普通值；
 * loadServers(platform?) 收窄为 loadServers()（唯一消费方从不传参，统一视图始终
 * 全平台拉取）；formData 返回 RHF watch 快照，方法集经 formApi 暴露。
 *
 * @example
 * const { servers, loading, loadServers, openAddForm, submitForm } = useUnifiedMcp({ t })
 */

import { useMutation, useQuery } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API。
import { useUIStore } from '@/shell/stores/ui'
import {
  listUnifiedMcp,
  addUnifiedMcp,
  updateUnifiedMcp,
  deleteUnifiedMcp,
  toggleUnifiedMcp,
} from '@/api'
import { mcpKeys } from '@/features/mcp/queries'
import { logger } from '@/utils/logger'
import type {
  UnifiedMcpServer,
  UnifiedMcpRequest,
  PlatformMcpCapability,
  UnifiedMcpPlatform,
  PlatformMeta,
  UnifiedMcpListResponse,
  UnifiedMcpDiagnostic,
  McpScopeFilter,
} from '@/types/unifiedMcp'

import type { UnknownRecord } from '@/types/common'

function asRecord(value: unknown): UnknownRecord {
  return typeof value === 'object' && value !== null ? (value as UnknownRecord) : {}
}

function toSuccessMessage(raw: unknown, fallback: string): string {
  if (typeof raw === 'string' && raw) return raw
  const source = asRecord(raw)
  if (typeof source.message === 'string' && source.message) return source.message
  return fallback
}
// ============ 平台元信息 ============

export const PLATFORM_META: Record<UnifiedMcpPlatform, PlatformMeta> = {
  claude: { id: 'claude', label: 'Claude Code', color: '#d97706', icon: 'terminal' },
  codex: { id: 'codex', label: 'Codex', color: '#10b981', icon: 'code' },
  gemini: { id: 'gemini', label: 'Antigravity CLI', color: '#8b5cf6', icon: 'sparkles' },
}

export const ALL_PLATFORMS: UnifiedMcpPlatform[] = ['claude', 'codex', 'gemini']

function createEmptyForm(): UnifiedMcpRequest {
  return {
    platform: 'claude',
    scope: 'user',
    name: '',
    command: null,
    url: null,
    args: null,
    env: null,
  }
}

/** 编辑态下剥离未变更的密钥预览值（原 stripUnchangedSecretPreviews 纯函数保留）。 */
function stripUnchangedSecretPreviews(
  patch: Record<string, string> | null,
  current: Record<string, string> | null | undefined
): void {
  if (!patch) return

  for (const [key, value] of Object.entries(patch)) {
    if (value.includes('•') && value === current?.[key]) {
      delete patch[key]
    }
  }
}

// ============ Hook 主体 ============

export function useUnifiedMcp() {
  const uiStore = useUIStore.getState()

  // ============ 服务端数据 ============

  const listQuery = useQuery({
    queryKey: mcpKeys.unifiedList(),
    staleTime: Infinity,
    queryFn: async () => {
      try {
        return await listUnifiedMcp<UnifiedMcpListResponse>()
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error'
        logger.error('Failed to load unified MCP servers', err)
        uiStore.showError(`加载 MCP 服务器失败: ${msg}`)
        throw err
      }
    },
  })

  // 派生数组经 useMemo 稳定引用（exhaustive-deps：下游 useMemo/useCallback 依赖）。
  const servers: UnifiedMcpServer[] = useMemo(
    () => (Array.isArray(listQuery.data?.servers) ? listQuery.data.servers : []),
    [listQuery.data]
  )
  const capabilities: PlatformMcpCapability[] = useMemo(
    () =>
      Array.isArray(listQuery.data?.capabilities) ? listQuery.data.capabilities : [],
    [listQuery.data]
  )
  const diagnostics: UnifiedMcpDiagnostic[] = useMemo(
    () =>
      Array.isArray(listQuery.data?.diagnostics) ? listQuery.data.diagnostics : [],
    [listQuery.data]
  )

  const loading = listQuery.isFetching
  const error = listQuery.error
    ? listQuery.error instanceof Error
      ? listQuery.error.message
      : 'Unknown error'
    : null

  const { refetch: refetchList } = listQuery

  /** 重拉列表（原 await loadServers 语义）。 */
  const reloadServers = useCallback(async () => {
    await refetchList()
  }, [refetchList])

  // ============ 过滤状态 ============

  const [filterPlatform, setFilterPlatform] = useState<UnifiedMcpPlatform | ''>('')
  const [filterKeyword, setFilterKeyword] = useState('')
  const [filterProtocol, setFilterProtocol] = useState<'all' | 'stdio' | 'http'>('all')
  const [filterScope, setFilterScope] = useState<McpScopeFilter>('effective')

  // ============ 表单状态 ============

  const form = useForm<UnifiedMcpRequest>({ defaultValues: createEmptyForm() })
  const formData = form.watch()

  const [showForm, setShowForm] = useState(false)
  const [editingServer, setEditingServer] = useState<UnifiedMcpServer | null>(null)
  const [isHttpMode, setIsHttpMode] = useState(false)
  const [argInput, setArgInput] = useState('')
  const [envKey, setEnvKey] = useState('')
  const [envValue, setEnvValue] = useState('')
  const [headerKey, setHeaderKey] = useState('')
  const [headerValue, setHeaderValue] = useState('')
  const [includeToolInput, setIncludeToolInput] = useState('')

  // ============ 计算属性 ============
  // 原 computed 的响应式来源：servers、filter* 五项、capabilities、formData.platform。

  /** 按条件过滤后的服务器列表 */
  const filteredServers = useMemo(() => {
    let result = servers

    // 按平台过滤
    if (filterPlatform) {
      result = result.filter((s) => s.platform === filterPlatform)
    }

    // 按协议过滤
    if (filterProtocol === 'stdio') {
      result = result.filter((s) => s.command && !s.url)
    } else if (filterProtocol === 'http') {
      result = result.filter((s) => s.url)
    }

    if (filterScope === 'effective') {
      result = result.filter((s) => s.effective !== false && !s.hidden_by)
    } else if (filterScope === 'hidden') {
      result = result.filter((s) => s.effective === false || !!s.hidden_by)
    } else {
      result = result.filter((s) => s.scope === filterScope)
    }

    // 按关键字过滤
    if (filterKeyword) {
      const kw = filterKeyword.toLowerCase()
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(kw) ||
          (s.command && s.command.toLowerCase().includes(kw)) ||
          (s.url && s.url.toLowerCase().includes(kw))
      )
    }

    return result
  }, [filterKeyword, filterPlatform, filterProtocol, filterScope, servers])

  const scopeCounts = useMemo<Record<McpScopeFilter, number>>(
    () => ({
      effective: servers.filter((s) => s.effective !== false && !s.hidden_by).length,
      local: servers.filter((s) => s.scope === 'local').length,
      project: servers.filter((s) => s.scope === 'project').length,
      user: servers.filter((s) => s.scope === 'user').length,
      hidden: servers.filter((s) => s.effective === false || !!s.hidden_by).length,
    }),
    [servers]
  )

  const sourceDiagnostics = diagnostics

  /** 各平台服务器数量统计 */
  const platformCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const p of ALL_PLATFORMS) {
      counts[p] = servers.filter((s) => s.platform === p).length
    }
    return counts
  }, [servers])

  /** 当前平台的能力矩阵 */
  const currentCapability = useMemo(() => {
    if (!formData.platform) return null
    return capabilities.find((c) => c.platform === formData.platform) ?? null
  }, [capabilities, formData.platform])

  /** 是否有活跃过滤器 */
  const hasActiveFilters = useMemo(
    () =>
      !!filterPlatform ||
      !!filterKeyword ||
      filterProtocol !== 'all' ||
      filterScope !== 'effective',
    [filterKeyword, filterPlatform, filterProtocol, filterScope]
  )

  // ============ CRUD 操作 ============

  const addMutation = useMutation({
    mutationFn: (request: UnifiedMcpRequest) => addUnifiedMcp<string | UnknownRecord>(request),
  })
  const updateMutation = useMutation({
    mutationFn: ({
      platform,
      name,
      request,
    }: {
      platform: string
      name: string
      request: UnifiedMcpRequest
    }) => updateUnifiedMcp<string | UnknownRecord>(platform, name, request),
  })
  const deleteMutation = useMutation({
    mutationFn: ({ platform, name, scope }: { platform: string; name: string; scope?: string }) =>
      deleteUnifiedMcp<string | UnknownRecord>(platform, name, scope),
  })
  const toggleMutation = useMutation({
    mutationFn: ({
      platform,
      name,
      disabled,
      scope,
    }: {
      platform: string
      name: string
      disabled: boolean
      scope?: string
    }) => toggleUnifiedMcp(platform, name, disabled, scope),
  })

  /** 关闭表单 */
  const closeForm = useCallback(() => {
    setShowForm(false)
    setEditingServer(null)
  }, [])

  /** 校验表单（原 validateForm）。 */
  const validateForm = useCallback(() => {
    const values = form.getValues()
    if (!values.name) {
      uiStore.showWarning('服务器名称不能为空')
      return false
    }
    if (!isHttpMode && !values.command) {
      uiStore.showWarning('STDIO 模式必须提供 command')
      return false
    }
    if (isHttpMode && !values.url) {
      uiStore.showWarning('HTTP 模式必须提供 url')
      return false
    }
    return true
  }, [form, isHttpMode, uiStore])

  /** 由表单值构建请求（原 buildRequest：密钥预览剥离 + 模式清理 + 编辑态空值置 null）。 */
  const buildRequest = useCallback((): UnifiedMcpRequest => {
    const values = form.getValues()
    const args = argInput
      .split(' ')
      .map((a) => a.trim())
      .filter(Boolean)

    const includeTools = includeToolInput
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean)

    const env = values.env ? { ...values.env } : null
    const headers = values.headers ? { ...values.headers } : null

    if (editingServer) {
      stripUnchangedSecretPreviews(env, editingServer.env ?? {})
      stripUnchangedSecretPreviews(headers, editingServer.headers ?? {})
    }

    const request: UnifiedMcpRequest = {
      ...values,
      args,
      include_tools: includeTools,
      env: env ?? {},
      headers: headers ?? {},
    }

    if (request.platform !== 'claude') {
      request.scope = null
      request.headers = null
      request.timeout = null
      request.cwd = null
      request.trust = null
      request.include_tools = null
      request.disabled = null
    }

    // 按模式清理字段
    if (isHttpMode) {
      request.command = null
      request.args = null
    } else {
      request.url = null
    }

    if (editingServer) {
      if (!args.length) request.args = null
      if (!includeTools.length) request.include_tools = null
      if (!Object.keys(env ?? {}).length) request.env = null
      if (!Object.keys(headers ?? {}).length) request.headers = null
    }

    return request
  }, [argInput, editingServer, form, includeToolInput, isHttpMode])


  /** 添加服务器 */
  const addServer = useCallback(async (): Promise<boolean> => {
    if (!validateForm()) return false
    const request = buildRequest()
    try {
      const message = await addMutation.mutateAsync(request)
      uiStore.showSuccess(toSuccessMessage(message, '添加成功'))
      await reloadServers()
      closeForm()
      return true
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(`添加失败: ${msg}`)
      return false
    }
  }, [
    addMutation,
    buildRequest,
    closeForm,
    reloadServers,
    uiStore,
    validateForm,
  ])

  /** 更新服务器 */
  const updateServer = useCallback(async (): Promise<boolean> => {
    if (!editingServer || !validateForm()) return false
    const { platform, name } = editingServer
    const request = buildRequest()
    if (!request.scope && typeof editingServer.scope === 'string') {
      request.scope = editingServer.scope
    }
    try {
      const message = await updateMutation.mutateAsync({ platform, name, request })
      uiStore.showSuccess(toSuccessMessage(message, '更新成功'))
      await reloadServers()
      closeForm()
      return true
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(`更新失败: ${msg}`)
      return false
    }
  }, [
    buildRequest,
    closeForm,
    editingServer,
    reloadServers,
    updateMutation,
    uiStore,
    validateForm,
  ])

  /** 删除服务器 */
  const deleteServer = useCallback(
    async (server: UnifiedMcpServer): Promise<boolean> => {
      try {
        const message = await deleteMutation.mutateAsync({
          platform: server.platform,
          name: server.name,
          scope: typeof server.scope === 'string' ? server.scope : undefined,
        })
        uiStore.showSuccess(toSuccessMessage(message, '删除成功'))
        await reloadServers()
        return true
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error'
        uiStore.showError(`删除失败: ${msg}`)
        return false
      }
    },
    [deleteMutation, reloadServers, uiStore]
  )

  /** 切换服务器启停 */
  const toggleServer = useCallback(
    async (server: UnifiedMcpServer): Promise<boolean> => {
      try {
        const result = await toggleMutation.mutateAsync({
          platform: server.platform,
          name: server.name,
          disabled: !server.disabled,
          scope: typeof server.scope === 'string' ? server.scope : undefined,
        })
        uiStore.showSuccess(toSuccessMessage(result, '状态已更新'))
        await reloadServers()
        return true
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error'
        uiStore.showError(`切换状态失败: ${msg}`)
        return false
      }
    },
    [reloadServers, toggleMutation, uiStore]
  )

  // ============ 表单操作 ============


  /** 重置输入行状态（原 resetFormInputs）。 */
  const resetFormInputs = useCallback(() => {
    setArgInput('')
    setEnvKey('')
    setEnvValue('')
    setHeaderKey('')
    setHeaderValue('')
    setIncludeToolInput('')
  }, [])

  /** 打开添加表单 */
  const openAddForm = useCallback(
    (platform?: UnifiedMcpPlatform, scope: UnifiedMcpRequest['scope'] = 'user') => {
      setEditingServer(null)
      setIsHttpMode(false)
      form.reset(createEmptyForm())
      if (platform) {
        form.setValue('platform', platform)
        form.setValue('scope', platform === 'claude' ? scope : null)
      } else {
        form.setValue('scope', form.getValues('platform') === 'claude' ? scope : null)
      }
      resetFormInputs()
      setShowForm(true)
    },
    [form, resetFormInputs]
  )

  /** 打开编辑表单 */
  const openEditForm = useCallback(
    (server: UnifiedMcpServer) => {
      setEditingServer(server)
      setIsHttpMode(!!server.url)
      form.reset({
        platform: server.platform,
        name: server.name,
        scope: server.scope ?? 'user',
        command: server.command,
        url: server.url,
        args: server.args?.length ? server.args : null,
        env: server.env && Object.keys(server.env).length > 0 ? { ...server.env } : null,
        headers: server.headers ? { ...server.headers } : null,
        timeout: server.timeout,
        disabled: server.disabled,
        cwd: server.cwd,
        trust: server.trust,
        include_tools: server.include_tools ? [...server.include_tools] : null,
      })
      setArgInput(server.args?.join(' ') ?? '')
      setIncludeToolInput(server.include_tools?.join(', ') ?? '')
      setEnvKey('')
      setEnvValue('')
      setHeaderKey('')
      setHeaderValue('')
      setShowForm(true)
    },
    [form]
  )

  /** 提交表单 */
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
      const current = form.getValues('env')
      if (current) {
        const newEnv = { ...current }
        delete newEnv[key]
        form.setValue('env', Object.keys(newEnv).length > 0 ? newEnv : null)
      }
    },
    [form]
  )

  /** 添加 Header */
  const addHeader = useCallback(() => {
    if (headerKey && headerValue) {
      form.setValue('headers', {
        ...(form.getValues('headers') ?? {}),
        [headerKey]: headerValue,
      })
      setHeaderKey('')
      setHeaderValue('')
    }
  }, [form, headerKey, headerValue])

  /** 删除 Header */
  const removeHeader = useCallback(
    (key: string) => {
      const current = form.getValues('headers')
      if (current) {
        const newHeaders = { ...current }
        delete newHeaders[key]
        form.setValue('headers', Object.keys(newHeaders).length > 0 ? newHeaders : null)
      }
    },
    [form]
  )

  /** 重置过滤器 */
  const resetFilters = useCallback(() => {
    setFilterPlatform('')
    setFilterKeyword('')
    setFilterProtocol('all')
    setFilterScope('effective')
  }, [])

  // ============ 辅助方法 ============

  /** 判断平台是否支持某能力 */
  const supportsFeature = useCallback(
    (platform: string, feature: keyof Omit<PlatformMcpCapability, 'platform'>): boolean => {
      const cap = capabilities.find((c) => c.platform === platform)
      return cap ? cap[feature] : false
    },
    [capabilities]
  )

  // ============ 返回值 ============

  return {
    // 平台元信息
    PLATFORM_META,
    ALL_PLATFORMS,

    // 数据状态
    servers,
    capabilities,
    diagnostics,
    sourceDiagnostics,
    loading,
    error,

    // 过滤
    filterPlatform,
    filterKeyword,
    filterProtocol,
    filterScope,
    setFilterPlatform,
    setFilterKeyword,
    setFilterProtocol,
    setFilterScope,
    filteredServers,
    platformCounts,
    scopeCounts,
    hasActiveFilters,
    resetFilters,

    // 表单状态
    showForm,
    editingServer,
    isHttpMode,
    formData,
    /** react-hook-form 方法集（register/setValue/reset），供迁移后的视图绑定表单 */
    formApi: form,
    argInput,
    envKey,
    envValue,
    headerKey,
    headerValue,
    includeToolInput,
    setArgInput,
    setEnvKey,
    setEnvValue,
    setHeaderKey,
    setHeaderValue,
    setIncludeToolInput,
    setShowForm,
    setIsHttpMode,
    currentCapability,

    // CRUD
    loadServers: reloadServers,
    addServer,
    updateServer,
    deleteServer,
    toggleServer,

    // 表单操作
    openAddForm,
    openEditForm,
    closeForm,
    submitForm,
    addEnvVar,
    removeEnvVar,
    addHeader,
    removeHeader,

    // 工具
    supportsFeature,
  }
}

export default useUnifiedMcp
