/* eslint-disable no-console -- Development debugging, console output acceptable */
import { ref } from 'vue'

// ===== Types =====

export type MarketItemCategory = 'skill' | 'mcp' | 'plugin' | 'command'
export type MarketItemSource = 'builtin' | 'remote' | 'local'

export interface MarketItem {
    id: string
    name: string
    description: string
    category: MarketItemCategory
    author?: string
    version?: string
    downloads?: number
    rating?: number
    installed: boolean
    source: MarketItemSource
    tags?: string[]
    homepage?: string
    requires_api_key?: boolean
    api_key_env?: string
}

export interface MarketplaceResponse {
    items: MarketItem[]
    total: number
}

export interface InstallRequest {
    item_id: string
    category: MarketItemCategory
    platforms?: string[]
    env?: Record<string, string>
}

// ===== Composable =====

export function useMarketplace() {
    const items = ref<MarketItem[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)
    const installingItems = ref<Set<string>>(new Set())

    /**
     * 获取所有市场项目
     * TODO: Marketplace 功能尚未迁移到 Tauri command，当前返回空数据
     */
    const fetchMarketItems = async (_category?: MarketItemCategory) => {
        loading.value = true
        error.value = null
        try {
            console.warn('[useMarketplace] fetchMarketItems: Marketplace 尚未迁移到 Tauri')
            items.value = []
        } catch (err: any) {
            error.value = err.message || 'Failed to load marketplace items'
            console.error('Marketplace fetch error:', err)
        } finally {
            loading.value = false
        }
    }

    /**
     * 获取已安装的项目
     * TODO: Marketplace 功能尚未迁移到 Tauri command
     */
    const fetchInstalledItems = async () => {
        loading.value = true
        error.value = null
        try {
            console.warn('[useMarketplace] fetchInstalledItems: Marketplace 尚未迁移到 Tauri')
            items.value = []
        } catch (err: any) {
            error.value = err.message || 'Failed to load installed items'
            console.error('Installed items fetch error:', err)
        } finally {
            loading.value = false
        }
    }

    /**
     * 安装市场项目
     * TODO: Marketplace 功能尚未迁移到 Tauri command
     */
    const installItem = async (item: MarketItem, _platforms?: string[], _env?: Record<string, string>) => {
        const itemId = item.id
        installingItems.value.add(itemId)
        error.value = null

        try {
            console.warn('[useMarketplace] installItem: Marketplace 尚未迁移到 Tauri')
            return false
        } catch (err: any) {
            error.value = err.message || `Failed to install ${item.name}`
            console.error('Install error:', err)
            return false
        } finally {
            installingItems.value.delete(itemId)
        }
    }

    /**
     * 卸载市场项目
     * TODO: Marketplace 功能尚未迁移到 Tauri command
     */
    const uninstallItem = async (item: MarketItem) => {
        const itemId = item.id
        installingItems.value.add(itemId) // 复用 loading 状态
        error.value = null

        try {
            console.warn('[useMarketplace] uninstallItem: Marketplace 尚未迁移到 Tauri')
            return false
        } catch (err: any) {
            error.value = err.message || `Failed to uninstall ${item.name}`
            console.error('Uninstall error:', err)
            return false
        } finally {
            installingItems.value.delete(itemId)
        }
    }

    /**
     * 检查项目是否正在安装中
     */
    const isInstalling = (itemId: string) => {
        return installingItems.value.has(itemId)
    }

    /**
     * 按分类过滤项目
     */
    const filterByCategory = (category: MarketItemCategory) => {
        return items.value.filter(item => item.category === category)
    }

    /**
     * 搜索项目
     */
    const searchItems = (query: string) => {
        const lowerQuery = query.toLowerCase()
        return items.value.filter(item =>
            item.name.toLowerCase().includes(lowerQuery) ||
            item.description.toLowerCase().includes(lowerQuery) ||
            item.tags?.some(tag => tag.toLowerCase().includes(lowerQuery))
        )
    }

    return {
        items,
        loading,
        error,
        installingItems,
        fetchMarketItems,
        fetchInstalledItems,
        installItem,
        uninstallItem,
        isInstalling,
        filterByCategory,
        searchItems,
    }
}
