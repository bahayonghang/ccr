import { defineStore } from 'pinia'
import { listConfigs } from '@/api/client'
import type { ConfigListResponse } from '@/types'

interface ConfigsState {
  items: ConfigListResponse | null
  lastFetchedAt: number
  loading: boolean
  error: string | null
}

export const useConfigsStore = defineStore('configs', {
  state: (): ConfigsState => ({
    items: null,
    lastFetchedAt: 0,
    loading: false,
    error: null
  }),

  getters: {
    /**
     * 获取配置列表
     */
    configs: (state) => state.items?.configs || [],
    
    /**
     * 获取当前配置
     */
    currentConfig: (state) => state.items?.current_config || null,
    
    /**
     * 是否有配置数据
     */
    hasConfigs: (state) => (state.items?.configs?.length || 0) > 0,
    
    /**
     * 缓存是否有效（5分钟内）
     */
    isCacheValid: (state) => {
      const now = Date.now()
      const cacheAge = now - state.lastFetchedAt
      return cacheAge < 5 * 60 * 1000 && state.items !== null
    }
  },

  actions: {
    /**
     * 获取配置列表
     * @param force 是否强制刷新（忽略缓存）
     */
    async fetch(force = false) {
      // 如果缓存有效且不是强制刷新，直接返回
      if (!force && this.isCacheValid) {
        return this.items
      }

      this.loading = true
      this.error = null

      try {
        const data = await listConfigs()
        this.items = data
        this.lastFetchedAt = Date.now()
        return data
      } catch (err: any) {
        this.error = err.message || '获取配置列表失败'
        throw err
      } finally {
        this.loading = false
      }
    },

    /**
     * 清除缓存
     */
    clearCache() {
      this.items = null
      this.lastFetchedAt = 0
      this.error = null
    },

    /**
     * 更新当前配置（本地更新，不发请求）
     */
    updateCurrent(configName: string) {
      if (this.items) {
        this.items.current_config = configName
      }
    }
  }
})
