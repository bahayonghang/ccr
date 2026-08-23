/**
 * usePlatformMcp - 通用平台 MCP 服务器管理 Composable
 *
 * 消除各平台 MCP 视图中的重复代码（当前复用于 Gemini MCP 视图）
 *
 * @example
 * const { servers, loading, loadServers, addServer, updateServer, deleteServer } = usePlatformMcp('gemini')
 */

import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API；
// 本文件在批次 5 转换为 React hook 时整体重写。
import { useUIStore } from '@/shell/stores/ui'
import {
  listGeminiMcpServers,
  addGeminiMcpServer,
  updateGeminiMcpServer,
  deleteGeminiMcpServer,
} from '@/api'
import { genericPlatformDescriptors, type GenericPlatformId } from '@/config/platformDescriptors'
import { logger } from '@/utils/logger'
import type { GeminiMcpServer, GeminiMcpServerRequest } from '@/types'

// ============ 类型定义 ============

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

// ============ Composable 主体 ============

export function usePlatformMcp(platform: PlatformType) {
  const { t } = useI18n()
  const uiStore = useUIStore.getState()

  // 获取平台配置
  const config = computed(() => platformConfigs[platform])

  // 响应式状态
  const servers = ref<PlatformMcpServer[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 表单状态
  const showForm = ref(false)
  const editingServer = ref<PlatformMcpServer | null>(null)
  const isHttpServer = ref(false)
  const formData = ref<UnifiedMcpServerRequest>(createEmptyFormData())
  const argInput = ref('')
  const envKey = ref('')
  const envValue = ref('')

  // ============ CRUD 操作 ============

  /** 加载服务器列表 */
  async function loadServers(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      servers.value = await config.value.listApi()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      error.value = errorMessage
      logger.error(`Failed to load ${platform} MCP servers: ${errorMessage}`, err)
      uiStore.showError(t(`${config.value.i18nPrefix}.loadFailed`, { error: errorMessage }))
    } finally {
      loading.value = false
    }
  }

  /** 添加服务器 */
  async function addServer(): Promise<boolean> {
    if (!validateForm()) return false

    const request = buildRequest()
    try {
      await config.value.addApi(request)
      uiStore.showSuccess(t(`${config.value.i18nPrefix}.addSuccess`))
      await loadServers()
      closeForm()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.value.i18nPrefix}.operationFailed`, { error: errorMessage }))
      return false
    }
  }

  /** 更新服务器 */
  async function updateServer(): Promise<boolean> {
    if (!editingServer.value || !validateForm()) return false

    const name = getServerIdentifier(editingServer.value)
    const request = buildRequest()
    try {
      await config.value.updateApi(name, request)
      uiStore.showSuccess(t(`${config.value.i18nPrefix}.updateSuccess`))
      await loadServers()
      closeForm()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.value.i18nPrefix}.operationFailed`, { error: errorMessage }))
      return false
    }
  }

  /** 删除服务器（纯执行器，确认决策上移到调用视图） */
  async function deleteServer(server: PlatformMcpServer): Promise<boolean> {
    const name = getServerIdentifier(server)
    try {
      await config.value.deleteApi(name)
      uiStore.showSuccess(t(`${config.value.i18nPrefix}.deleteSuccess`))
      await loadServers()
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      uiStore.showError(t(`${config.value.i18nPrefix}.deleteFailed`, { error: errorMessage }))
      return false
    }
  }

  // ============ 表单操作 ============

  /** 打开添加表单 */
  function openAddForm(): void {
    editingServer.value = null
    isHttpServer.value = false
    formData.value = createEmptyFormData()
    argInput.value = ''
    envKey.value = ''
    envValue.value = ''
    showForm.value = true
  }

  /** 打开编辑表单 */
  function openEditForm(server: PlatformMcpServer): void {
    editingServer.value = server
    isHttpServer.value = !!server.url
    formData.value = { ...server }
    argInput.value = server.args?.join(' ') || ''
    envKey.value = ''
    envValue.value = ''
    showForm.value = true
  }

  /** 关闭表单 */
  function closeForm(): void {
    showForm.value = false
    editingServer.value = null
  }

  /** 提交表单（添加或更新） */
  async function submitForm(): Promise<boolean> {
    if (editingServer.value) {
      return updateServer()
    } else {
      return addServer()
    }
  }

  /** 添加环境变量 */
  function addEnvVar(): void {
    if (envKey.value && envValue.value) {
      formData.value.env = { ...formData.value.env, [envKey.value]: envValue.value }
      envKey.value = ''
      envValue.value = ''
    }
  }

  /** 删除环境变量 */
  function removeEnvVar(key: string): void {
    const newEnv = { ...formData.value.env }
    delete newEnv[key]
    formData.value.env = newEnv
  }

  // ============ 辅助方法 ============

  function createEmptyFormData(): UnifiedMcpServerRequest {
    return {
      name: '',
      command: undefined,
      url: undefined,
      args: [],
      env: {},
    }
  }

  function validateForm(): boolean {
    if (!isHttpServer.value && !formData.value.command) {
      uiStore.showWarning(t(`${config.value.i18nPrefix}.validation.commandRequired`))
      return false
    }
    if (isHttpServer.value && !formData.value.url) {
      uiStore.showWarning(t(`${config.value.i18nPrefix}.validation.urlRequired`))
      return false
    }
    return true
  }

  function buildRequest(): UnifiedMcpServerRequest {
    const args = argInput.value
      .split(' ')
      .map((a) => a.trim())
      .filter(Boolean)

    const request: UnifiedMcpServerRequest = {
      ...formData.value,
      args,
    }

    // 根据服务器类型清理不需要的字段
    if (isHttpServer.value) {
      request.command = undefined
    } else {
      request.url = undefined
    }

    // 当请求未提供名称时，回退到 command/url 生成稳定标识。
    if (!request.name) {
      request.name = request.command || request.url || 'unknown'
    }

    return request
  }

  // ============ 返回值 ============

  return {
    // 平台配置
    config,
    platform,
    moduleColor: computed(() => config.value.color),
    i18nPrefix: computed(() => config.value.i18nPrefix),
    parentPath: computed(() => config.value.parentPath),

    // 数据状态
    servers,
    loading,
    error,

    // 表单状态
    showForm,
    editingServer,
    isHttpServer,
    formData,
    argInput,
    envKey,
    envValue,

    // CRUD 操作
    loadServers,
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
