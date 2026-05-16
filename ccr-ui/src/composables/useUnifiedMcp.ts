/**
 * useUnifiedMcp - 统一 MCP 服务器管理 Composable
 *
 * 对接后端统一 MCP API（/api/unified/mcp），提供跨平台 MCP 服务器的
 * 列表、添加、编辑、删除、启停等全部操作。
 *
 * @example
 * const { servers, loading, loadServers, openAddForm, submitForm } = useUnifiedMcp()
 */

import { ref, computed } from 'vue'
import { useUIStore } from '@/stores/ui'
import {
    listUnifiedMcp,
    addUnifiedMcp,
    updateUnifiedMcp,
    deleteUnifiedMcp,
    toggleUnifiedMcp,
} from '@/api'
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

type UnknownRecord = Record<string, unknown>

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
    gemini: { id: 'gemini', label: 'Gemini CLI', color: '#8b5cf6', icon: 'sparkles' },
}

export const ALL_PLATFORMS: UnifiedMcpPlatform[] = ['claude', 'codex', 'gemini']

// ============ Composable ============

export function useUnifiedMcp() {
    const uiStore = useUIStore()

    // 响应式状态
    const servers = ref<UnifiedMcpServer[]>([])
    const capabilities = ref<PlatformMcpCapability[]>([])
    const diagnostics = ref<UnifiedMcpDiagnostic[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    // 过滤状态
    const filterPlatform = ref<UnifiedMcpPlatform | ''>('')
    const filterKeyword = ref('')
    const filterProtocol = ref<'all' | 'stdio' | 'http'>('all')
    const filterScope = ref<McpScopeFilter>('effective')

    // 表单状态
    const showForm = ref(false)
    const editingServer = ref<UnifiedMcpServer | null>(null)
    const isHttpMode = ref(false)
    const formData = ref<UnifiedMcpRequest>(createEmptyForm())
    const argInput = ref('')
    const envKey = ref('')
    const envValue = ref('')
    const headerKey = ref('')
    const headerValue = ref('')
    const includeToolInput = ref('')

    // ============ 计算属性 ============

    /** 按条件过滤后的服务器列表 */
    const filteredServers = computed(() => {
        let result = servers.value

        // 按平台过滤
        if (filterPlatform.value) {
            result = result.filter(s => s.platform === filterPlatform.value)
        }

        // 按协议过滤
        if (filterProtocol.value === 'stdio') {
            result = result.filter(s => s.command && !s.url)
        } else if (filterProtocol.value === 'http') {
            result = result.filter(s => s.url)
        }

        if (filterScope.value === 'effective') {
            result = result.filter(s => s.effective !== false && !s.hidden_by)
        } else if (filterScope.value === 'hidden') {
            result = result.filter(s => s.effective === false || !!s.hidden_by)
        } else {
            result = result.filter(s => s.scope === filterScope.value)
        }

        // 按关键字过滤
        if (filterKeyword.value) {
            const kw = filterKeyword.value.toLowerCase()
            result = result.filter(s =>
                s.name.toLowerCase().includes(kw) ||
                (s.command && s.command.toLowerCase().includes(kw)) ||
                (s.url && s.url.toLowerCase().includes(kw))
            )
        }

        return result
    })

    const scopeCounts = computed<Record<McpScopeFilter, number>>(() => ({
        effective: servers.value.filter(s => s.effective !== false && !s.hidden_by).length,
        local: servers.value.filter(s => s.scope === 'local').length,
        project: servers.value.filter(s => s.scope === 'project').length,
        user: servers.value.filter(s => s.scope === 'user').length,
        hidden: servers.value.filter(s => s.effective === false || !!s.hidden_by).length,
    }))

    const sourceDiagnostics = computed(() => diagnostics.value)

    /** 各平台服务器数量统计 */
    const platformCounts = computed(() => {
        const counts: Record<string, number> = {}
        for (const p of ALL_PLATFORMS) {
            counts[p] = servers.value.filter(s => s.platform === p).length
        }
        return counts
    })

    /** 当前平台的能力矩阵 */
    const currentCapability = computed(() => {
        if (!formData.value.platform) return null
        return capabilities.value.find(c => c.platform === formData.value.platform) ?? null
    })

    /** 是否有活跃过滤器 */
    const hasActiveFilters = computed(() =>
        !!filterPlatform.value || !!filterKeyword.value || filterProtocol.value !== 'all' || filterScope.value !== 'effective'
    )

    // ============ CRUD 操作 ============

    /** 加载所有平台的 MCP 服务器 */
    async function loadServers(platform?: string): Promise<void> {
        loading.value = true
        error.value = null
        try {
            const resp = await listUnifiedMcp<UnifiedMcpListResponse>(platform)
            servers.value = Array.isArray(resp.servers) ? resp.servers : []
            capabilities.value = Array.isArray(resp.capabilities) ? resp.capabilities : []
            diagnostics.value = Array.isArray(resp.diagnostics) ? resp.diagnostics : []
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Unknown error'
            error.value = msg
            logger.error('Failed to load unified MCP servers', err)
            uiStore.showError(`加载 MCP 服务器失败: ${msg}`)
        } finally {
            loading.value = false
        }
    }

    /** 添加服务器 */
    async function addServer(): Promise<boolean> {
        if (!validateForm()) return false
        const request = buildRequest()
        try {
            const message = await addUnifiedMcp<string | UnknownRecord>(request)
            uiStore.showSuccess(toSuccessMessage(message, '添加成功'))
            await loadServers()
            closeForm()
            return true
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(`添加失败: ${msg}`)
            return false
        }
    }

    /** 更新服务器 */
    async function updateServer(): Promise<boolean> {
        if (!editingServer.value || !validateForm()) return false
        const { platform, name } = editingServer.value
        const request = buildRequest()
        if (!request.scope && typeof editingServer.value.scope === 'string') {
            request.scope = editingServer.value.scope
        }
        try {
            const message = await updateUnifiedMcp<string | UnknownRecord>(platform, name, request)
            uiStore.showSuccess(toSuccessMessage(message, '更新成功'))
            await loadServers()
            closeForm()
            return true
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(`更新失败: ${msg}`)
            return false
        }
    }

    /** 删除服务器 */
    async function deleteServer(server: UnifiedMcpServer): Promise<boolean> {
        try {
            const message = await deleteUnifiedMcp<string | UnknownRecord>(
                server.platform,
                server.name,
                typeof server.scope === 'string' ? server.scope : undefined,
            )
            uiStore.showSuccess(toSuccessMessage(message, '删除成功'))
            await loadServers()
            return true
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(`删除失败: ${msg}`)
            return false
        }
    }

    /** 切换服务器启停 */
    async function toggleServer(server: UnifiedMcpServer): Promise<boolean> {
        try {
            const result = await toggleUnifiedMcp<string | UnknownRecord>(
                server.platform,
                server.name,
                !server.disabled,
                typeof server.scope === 'string' ? server.scope : undefined,
            )
            uiStore.showSuccess(toSuccessMessage(result, '状态已更新'))
            await loadServers()
            return true
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(`切换状态失败: ${msg}`)
            return false
        }
    }

    // ============ 表单操作 ============

    /** 打开添加表单 */
    function openAddForm(platform?: UnifiedMcpPlatform, scope: UnifiedMcpRequest['scope'] = 'user'): void {
        editingServer.value = null
        isHttpMode.value = false
        formData.value = createEmptyForm()
        if (platform) formData.value.platform = platform
        formData.value.scope = formData.value.platform === 'claude' ? scope : null
        resetFormInputs()
        showForm.value = true
    }

    /** 打开编辑表单 */
    function openEditForm(server: UnifiedMcpServer): void {
        editingServer.value = server
        isHttpMode.value = !!server.url
        formData.value = {
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
        }
        argInput.value = server.args?.join(' ') ?? ''
        includeToolInput.value = server.include_tools?.join(', ') ?? ''
        envKey.value = ''
        envValue.value = ''
        headerKey.value = ''
        headerValue.value = ''
        showForm.value = true
    }

    /** 关闭表单 */
    function closeForm(): void {
        showForm.value = false
        editingServer.value = null
    }

    /** 提交表单 */
    async function submitForm(): Promise<boolean> {
        return editingServer.value ? updateServer() : addServer()
    }

    /** 添加环境变量 */
    function addEnvVar(): void {
        if (envKey.value && envValue.value) {
            formData.value.env = { ...(formData.value.env ?? {}), [envKey.value]: envValue.value }
            envKey.value = ''
            envValue.value = ''
        }
    }

    /** 删除环境变量 */
    function removeEnvVar(key: string): void {
        if (formData.value.env) {
            const newEnv = { ...formData.value.env }
            delete newEnv[key]
            formData.value.env = Object.keys(newEnv).length > 0 ? newEnv : null
        }
    }

    /** 添加 Header */
    function addHeader(): void {
        if (headerKey.value && headerValue.value) {
            formData.value.headers = {
                ...(formData.value.headers ?? {}),
                [headerKey.value]: headerValue.value,
            }
            headerKey.value = ''
            headerValue.value = ''
        }
    }

    /** 删除 Header */
    function removeHeader(key: string): void {
        if (formData.value.headers) {
            const newHeaders = { ...formData.value.headers }
            delete newHeaders[key]
            formData.value.headers = Object.keys(newHeaders).length > 0 ? newHeaders : null
        }
    }

    /** 重置过滤器 */
    function resetFilters(): void {
        filterPlatform.value = ''
        filterKeyword.value = ''
        filterProtocol.value = 'all'
        filterScope.value = 'effective'
    }

    // ============ 辅助方法 ============

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

    function resetFormInputs(): void {
        argInput.value = ''
        envKey.value = ''
        envValue.value = ''
        headerKey.value = ''
        headerValue.value = ''
        includeToolInput.value = ''
    }

    function validateForm(): boolean {
        if (!formData.value.name) {
            uiStore.showWarning('服务器名称不能为空')
            return false
        }
        if (!isHttpMode.value && !formData.value.command) {
            uiStore.showWarning('STDIO 模式必须提供 command')
            return false
        }
        if (isHttpMode.value && !formData.value.url) {
            uiStore.showWarning('HTTP 模式必须提供 url')
            return false
        }
        return true
    }

    function buildRequest(): UnifiedMcpRequest {
        const args = argInput.value
            .split(' ')
            .map(a => a.trim())
            .filter(Boolean)

        const includeTools = includeToolInput.value
            .split(',')
            .map(t => t.trim())
            .filter(Boolean)

        const env = formData.value.env ? { ...formData.value.env } : null
        const headers = formData.value.headers ? { ...formData.value.headers } : null

        if (editingServer.value) {
            stripUnchangedSecretPreviews(env, editingServer.value.env ?? {})
            stripUnchangedSecretPreviews(headers, editingServer.value.headers ?? {})
        }

        const request: UnifiedMcpRequest = {
            ...formData.value,
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
        if (isHttpMode.value) {
            request.command = null
            request.args = null
        } else {
            request.url = null
        }

        if (editingServer.value) {
            if (!args.length) request.args = null
            if (!includeTools.length) request.include_tools = null
            if (!Object.keys(env ?? {}).length) request.env = null
            if (!Object.keys(headers ?? {}).length) request.headers = null
        }

        return request
    }

    function stripUnchangedSecretPreviews(
        patch: Record<string, string> | null,
        current: Record<string, string> | null | undefined,
    ): void {
        if (!patch) return

        for (const [key, value] of Object.entries(patch)) {
            if (value.includes('•') && value === current?.[key]) {
                delete patch[key]
            }
        }
    }

    /** 判断平台是否支持某能力 */
    function supportsFeature(
        platform: string,
        feature: keyof Omit<PlatformMcpCapability, 'platform'>
    ): boolean {
        const cap = capabilities.value.find(c => c.platform === platform)
        return cap ? cap[feature] : false
    }

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
        argInput,
        envKey,
        envValue,
        headerKey,
        headerValue,
        includeToolInput,
        currentCapability,

        // CRUD
        loadServers,
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
