/**
 * usePlatformMcp - 通用平台 MCP 服务器管理 Composable
 * 
 * 消除各平台 MCP 视图中的重复代码（GeminiMcpView、QwenMcpView、IflowMcpView）
 * 
 * @example
 * const { servers, loading, loadServers, addServer, updateServer, deleteServer } = usePlatformMcp('gemini')
 */

import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUIStore } from '@/stores/ui'
import {
    // Gemini APIs
    listGeminiMcpServers,
    addGeminiMcpServer,
    updateGeminiMcpServer,
    deleteGeminiMcpServer,
    // Qwen APIs
    listQwenMcpServers,
    addQwenMcpServer,
    updateQwenMcpServer,
    deleteQwenMcpServer,
    // iFlow APIs
    listIflowMcpServers,
    addIflowMcpServer,
    updateIflowMcpServer,
    deleteIflowMcpServer,
    // Droid APIs
    listDroidMcpServers,
    addDroidMcpServer,
    updateDroidMcpServer,
    deleteDroidMcpServer,
} from '@/api/client'
import type {
    GeminiMcpServer,
    GeminiMcpServerRequest,
    QwenMcpServer,
    QwenMcpServerRequest,
    IflowMcpServer,
    IflowMcpServerRequest,
    DroidMcpServer,
    DroidMcpServerRequest,
} from '@/types'

// ============ 类型定义 ============

/** 支持的平台类型 */
export type PlatformType = 'gemini' | 'qwen' | 'iflow' | 'droid'

/** 统一的 MCP 服务器类型（合并各平台差异） */
export interface UnifiedMcpServer {
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
interface PlatformConfig {
    color: string
    i18nPrefix: string
    parentPath: string
    listApi: () => Promise<UnifiedMcpServer[]>
    addApi: (req: UnifiedMcpServerRequest) => Promise<string>
    updateApi: (name: string, req: UnifiedMcpServerRequest) => Promise<string>
    deleteApi: (name: string) => Promise<string>
}

// ============ 平台 API 映射 ============

const platformConfigs: Record<PlatformType, PlatformConfig> = {
    gemini: {
        color: '#8b5cf6',
        i18nPrefix: 'gemini.mcp',
        parentPath: '/gemini-cli',
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
    qwen: {
        color: '#14b8a6',
        i18nPrefix: 'qwen.mcp',
        parentPath: '/qwen',
        listApi: async () => {
            const servers = await listQwenMcpServers()
            return servers.map(normalizeServer)
        },
        addApi: async (req) => {
            const qwenReq: QwenMcpServerRequest = {
                name: req.name,
                command: req.command,
                args: req.args,
                env: req.env,
                url: req.url,
                headers: req.headers,
                timeout: req.timeout,
            }
            return addQwenMcpServer(qwenReq)
        },
        updateApi: async (name, req) => {
            const qwenReq: QwenMcpServerRequest = {
                name: req.name,
                command: req.command,
                args: req.args,
                env: req.env,
                url: req.url,
                headers: req.headers,
                timeout: req.timeout,
            }
            return updateQwenMcpServer(name, qwenReq)
        },
        deleteApi: deleteQwenMcpServer,
    },
    iflow: {
        color: '#f97316',
        i18nPrefix: 'iflow.mcp',
        parentPath: '/iflow',
        listApi: async () => {
            const servers = await listIflowMcpServers()
            return servers.map((s: IflowMcpServer) => ({
                name: s.command || s.url || 'unknown',
                command: s.command,
                url: s.url,
                args: s.args,
                env: s.env,
            }))
        },
        addApi: async (req) => {
            const iflowReq: IflowMcpServerRequest = {
                command: req.command,
                url: req.url,
                args: req.args,
                env: req.env,
            }
            return addIflowMcpServer(iflowReq)
        },
        updateApi: async (name, req) => {
            const iflowReq: IflowMcpServerRequest = {
                command: req.command,
                url: req.url,
                args: req.args,
                env: req.env,
            }
            return updateIflowMcpServer(name, iflowReq)
        },
        deleteApi: deleteIflowMcpServer,
    },
    droid: {
        color: '#ec4899',
        i18nPrefix: 'droid.mcp',
        parentPath: '/droid',
        listApi: async () => {
            const servers = await listDroidMcpServers()
            return servers.map((s: DroidMcpServer) => ({
                name: s.name,
                command: s.command,
                url: s.url || s.httpUrl,
                args: s.args,
                env: s.env,
                headers: s.headers,
                timeout: s.timeout,
            }))
        },
        addApi: async (req) => {
            const droidReq: DroidMcpServerRequest = {
                name: req.name,
                command: req.command,
                args: req.args,
                env: req.env,
                url: req.url,
                headers: req.headers,
                timeout: req.timeout,
            }
            return addDroidMcpServer(droidReq)
        },
        updateApi: async (name, req) => {
            const droidReq: DroidMcpServerRequest = {
                name: req.name,
                command: req.command,
                args: req.args,
                env: req.env,
                url: req.url,
                headers: req.headers,
                timeout: req.timeout,
            }
            return updateDroidMcpServer(name, droidReq)
        },
        deleteApi: deleteDroidMcpServer,
    },
}

// ============ 辅助函数 ============

/** 统一各平台服务器数据结构 */
function normalizeServer(server: GeminiMcpServer | QwenMcpServer): UnifiedMcpServer {
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
        headers: 'headers' in server ? server.headers : undefined,
    }
}

/** 获取服务器标识符（用于编辑/删除） */
export function getServerIdentifier(server: UnifiedMcpServer): string {
    return server.name || server.command || server.url || ''
}

// ============ Composable 主体 ============

export function usePlatformMcp(platform: PlatformType) {
    const { t } = useI18n()
    const uiStore = useUIStore()

    // 获取平台配置
    const config = computed(() => platformConfigs[platform])

    // 响应式状态
    const servers = ref<UnifiedMcpServer[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    // 表单状态
    const showForm = ref(false)
    const editingServer = ref<UnifiedMcpServer | null>(null)
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
            console.error(`Failed to load ${platform} MCP servers:`, err)
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

    /** 删除服务器 */
    async function deleteServer(server: UnifiedMcpServer): Promise<boolean> {
        const name = getServerIdentifier(server)
        if (!confirm(t(`${config.value.i18nPrefix}.deleteConfirm`, { name }))) {
            return false
        }

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
    function openEditForm(server: UnifiedMcpServer): void {
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
            .map(a => a.trim())
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

        // 设置 name（iFlow 平台特殊处理）
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
