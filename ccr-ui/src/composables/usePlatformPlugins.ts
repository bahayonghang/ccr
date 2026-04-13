/**
 * usePlatformPlugins - 通用平台插件管理 Composable
 *
 * 消除各平台 Plugins 视图中的重复代码（GeminiPluginsView）
 *
 * @example
 * const { plugins, loading, loadPlugins, addPlugin, updatePlugin, deletePlugin, togglePlugin } = usePlatformPlugins('gemini')
 */

import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUIStore } from '@/stores/ui'
import { listGeminiPlugins, addGeminiPlugin, updateGeminiPlugin, deleteGeminiPlugin, toggleGeminiPlugin } from '@/api'
import { genericPlatformDescriptors } from '@/config/platformDescriptors'
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

// ============ Composable 主体 ============

export function usePlatformPlugins(platform: PluginPlatformType) {
    const { t } = useI18n()
    const uiStore = useUIStore()

    // 获取平台配置
    const config = computed(() => platformConfigs[platform])

    // 响应式状态
    const plugins = ref<PluginType[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    // 表单状态
    const showForm = ref(false)
    const editingPlugin = ref<PluginType | null>(null)
    const formData = ref<PluginRequest>({
        id: '',
        name: '',
        version: '1.0.0',
        enabled: true,
        config: undefined,
    })
    const configJson = ref('')

    // ============ CRUD 操作 ============

    /** 加载插件列表 */
    async function loadPlugins(): Promise<void> {
        loading.value = true
        error.value = null
        try {
            plugins.value = await config.value.listApi()
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Unknown error'
            error.value = errorMessage
            logger.error(`Failed to load ${platform} plugins:`, err)
            uiStore.showError(t(`${config.value.i18nPrefix}.messages.loadFailed`))
        } finally {
            loading.value = false
        }
    }

    /** 添加插件 */
    async function addPlugin(): Promise<boolean> {
        if (!validateForm()) return false

        const request = buildRequest()
        if (!request) return false

        try {
            await config.value.addApi(request)
            uiStore.showSuccess(t(`${config.value.i18nPrefix}.messages.addSuccess`))
            await loadPlugins()
            closeForm()
            return true
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(t(`${config.value.i18nPrefix}.messages.operationFailed`, { error: errorMessage }))
            return false
        }
    }

    /** 更新插件 */
    async function updatePlugin(): Promise<boolean> {
        if (!editingPlugin.value || !validateForm()) return false

        const request = buildRequest()
        if (!request) return false

        try {
            await config.value.updateApi(editingPlugin.value.id, request)
            uiStore.showSuccess(t(`${config.value.i18nPrefix}.messages.updateSuccess`))
            await loadPlugins()
            closeForm()
            return true
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(t(`${config.value.i18nPrefix}.messages.operationFailed`, { error: errorMessage }))
            return false
        }
    }

    /** 删除插件 */
    async function deletePlugin(plugin: PluginType): Promise<boolean> {
        if (!confirm(t(`${config.value.i18nPrefix}.deleteConfirm`, { name: plugin.name || plugin.id }))) {
            return false
        }

        try {
            await config.value.deleteApi(plugin.id)
            uiStore.showSuccess(t(`${config.value.i18nPrefix}.messages.deleteSuccess`))
            await loadPlugins()
            return true
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(t(`${config.value.i18nPrefix}.messages.deleteFailed`, { error: errorMessage }))
            return false
        }
    }

    /** 切换插件启用状态 */
    async function togglePlugin(plugin: PluginType): Promise<boolean> {
        try {
            await config.value.toggleApi(plugin.id)
            await loadPlugins()
            return true
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Unknown error'
            uiStore.showError(t(`${config.value.i18nPrefix}.messages.toggleFailed`, { error: errorMessage }))
            return false
        }
    }

    // ============ 表单操作 ============

    /** 打开添加表单 */
    function openAddForm(): void {
        editingPlugin.value = null
        formData.value = {
            id: '',
            name: '',
            version: '1.0.0',
            enabled: true,
            config: undefined,
        }
        configJson.value = ''
        showForm.value = true
    }

    /** 打开编辑表单 */
    function openEditForm(plugin: PluginType): void {
        editingPlugin.value = plugin
        formData.value = {
            id: plugin.id,
            name: plugin.name,
            version: plugin.version,
            enabled: plugin.enabled,
            config: plugin.config,
        }
        configJson.value = plugin.config ? JSON.stringify(plugin.config, null, 2) : ''
        showForm.value = true
    }

    /** 关闭表单 */
    function closeForm(): void {
        showForm.value = false
        editingPlugin.value = null
    }

    /** 提交表单（添加或更新） */
    async function submitForm(): Promise<boolean> {
        if (editingPlugin.value) {
            return updatePlugin()
        } else {
            return addPlugin()
        }
    }

    // ============ 辅助方法 ============

    function validateForm(): boolean {
        if (!formData.value.id || !formData.value.name || !formData.value.version) {
            uiStore.showWarning(t(`${config.value.i18nPrefix}.validation.required`))
            return false
        }
        return true
    }

    function buildRequest(): PluginRequest | null {
        let parsedConfig = undefined

        if (configJson.value.trim()) {
            try {
                parsedConfig = JSON.parse(configJson.value)
            } catch {
                uiStore.showError(t(`${config.value.i18nPrefix}.validation.invalidJson`))
                return null
            }
        }

        return {
            ...formData.value,
            config: parsedConfig,
        }
    }

    // ============ 返回值 ============

    return {
        // 平台配置
        config,
        platform,
        moduleColor: computed(() => config.value.color),
        i18nPrefix: computed(() => config.value.i18nPrefix),
        parentPath: computed(() => config.value.parentPath),
        sidebarModule: computed(() => config.value.sidebarModule),

        // 数据状态
        plugins,
        loading,
        error,

        // 表单状态
        showForm,
        editingPlugin,
        formData,
        configJson,

        // CRUD 操作
        loadPlugins,
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
