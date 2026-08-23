import { useCallback, useMemo, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { deleteConfig, disableConfig, enableConfig, switchConfig } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { translateWithFallback } from '@/i18n/formatMessage'
import type { ConfigItem } from '@/types'
import { t } from '../locale'
import { configsNotify } from '../notify'
import { configsKeys, useConfigsHistory, useConfigsList, useProviderUsage } from '../queries'
import { useConfigsViewStore } from '../stores'
import { buildConfigSummary, currentConfigName, filterConfigs, quickJumpConfigs } from '../lib/configList'
import type { ConfigFilter, ConfigsTabId, ConfigSort, ProviderSortMode } from '../types'

export function useConfigsPage() {
  const queryClient = useQueryClient()
  const listQuery = useConfigsList()
  const [activeTab, setActiveTab] = useState<ConfigsTabId>('configs')
  const historyQuery = useConfigsHistory(activeTab === 'history')
  const providerQuery = useProviderUsage()
  const searchQuery = useConfigsViewStore((state) => state.searchQuery)
  const setSearchQuery = useConfigsViewStore((state) => state.setSearchQuery)
  const setCurrentConfig = useConfigsViewStore((state) => state.setCurrentConfig)
  const [currentFilter, setCurrentFilter] = useState<ConfigFilter>('all')
  const [currentSort, setCurrentSort] = useState<ConfigSort>('name')
  const [isEditOpen, setIsEditOpen] = useState(false)
  const [editingName, setEditingName] = useState('')
  const [isAddOpen, setIsAddOpen] = useState(false)
  const [showProvider, setShowProvider] = useState(false)
  const [providerSortMode, setProviderSortMode] = useState<ProviderSortMode>('count_desc')
  const [highlightedName, setHighlightedName] = useState<string | null>(null)

  const configs = useMemo(() => listQuery.data?.configs ?? [], [listQuery.data?.configs])
  const filtered = useMemo(
    () => filterConfigs({ configs, filter: currentFilter, searchQuery, sort: currentSort }),
    [configs, currentFilter, currentSort, searchQuery],
  )
  const summary = useMemo(() => buildConfigSummary(configs, t), [configs])
  const jumps = useMemo(() => quickJumpConfigs(filtered), [filtered])
  const currentName = currentConfigName(configs, t('configs.noCurrentConfig'))

  const refresh = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: configsKeys.all })
  }, [queryClient])

  const handleSwitch = useCallback(
    async (name: string) => {
      const confirmed = await configsNotify.confirm({
        title: t('configs.switchConfig'),
        message: translateWithFallback(t, 'configs.confirmSwitch', '确定切换到配置 "{name}" 吗？', { name }),
        confirmText: t('configs.switchConfig'),
        type: 'warning',
      })
      if (!confirmed) return
      try {
        await switchConfig(name)
        configsNotify.success(`Switched to configuration ${name}`)
        setCurrentConfig(name)
        await refresh()
      } catch (error) {
        configsNotify.error(error instanceof Error ? error.message : 'Failed to switch configuration')
      }
    },
    [refresh, setCurrentConfig],
  )

  const handleEdit = useCallback((name: string) => {
    setEditingName(name)
    setIsEditOpen(true)
  }, [])

  const handleDelete = useCallback(
    async (name: string) => {
      const confirmed = await configsNotify.confirm({
        title: t('common.delete'),
        message: translateWithFallback(t, 'configs.confirmDelete', '确认删除配置 "{name}" 吗？', { name }),
        confirmText: t('common.delete'),
        type: 'danger',
      })
      if (!confirmed) return
      try {
        await deleteConfig(name)
        configsNotify.success(`Configuration ${name} deleted`)
        await refresh()
      } catch (error) {
        configsNotify.error(error instanceof Error ? error.message : 'Failed to delete configuration')
      }
    },
    [refresh],
  )

  const handleEnable = useCallback(
    async (name: string) => {
      try {
        await enableConfig(name)
        configsNotify.success(`Configuration ${name} enabled`)
        await refresh()
      } catch (error) {
        configsNotify.error(error instanceof Error ? error.message : 'Failed to enable configuration')
      }
    },
    [refresh],
  )

  const handleDisable = useCallback(
    async (name: string) => {
      try {
        await disableConfig(name)
        configsNotify.success(`Configuration ${name} disabled`)
        await refresh()
      } catch (error) {
        configsNotify.error(error instanceof Error ? error.message : 'Failed to disable configuration')
      }
    },
    [refresh],
  )

  const handleJump = useCallback((name: string) => {
    setHighlightedName(name)
    window.setTimeout(() => setHighlightedName(null), 1500)
    document.querySelector(`[data-config-name="${name}"]`)?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }, [])

  const closeEdit = useCallback(() => setIsEditOpen(false), [])
  const closeAdd = useCallback(() => setIsAddOpen(false), [])
  const openAdd = useCallback(() => setIsAddOpen(true), [])
  const openProvider = useCallback(() => setShowProvider(true), [])
  const closeProvider = useCallback(() => setShowProvider(false), [])

  return {
    activeTab,
    setActiveTab,
    configs,
    filtered,
    summary,
    jumps,
    currentName,
    searchQuery,
    setSearchQuery,
    currentFilter,
    setCurrentFilter,
    currentSort,
    setCurrentSort,
    loading: listQuery.isPending,
    error: listQuery.error ? getErrorMessage(listQuery.error) : null,
    historyEntries: historyQuery.data?.entries ?? [],
    historyLoading: historyQuery.isPending,
    providerUsage: providerQuery.data ?? {},
    providerLoading: providerQuery.isPending,
    providerError: providerQuery.error ? getErrorMessage(providerQuery.error) : null,
    providerSortMode,
    setProviderSortMode,
    isEditOpen,
    editingName,
    isAddOpen,
    showProvider,
    highlightedName,
    refresh,
    handleSwitch,
    handleEdit,
    handleDelete,
    handleEnable,
    handleDisable,
    handleJump,
    closeEdit,
    closeAdd,
    openAdd,
    openProvider,
    closeProvider,
  }
}

export type ConfigsPageState = ReturnType<typeof useConfigsPage>
export type { ConfigItem }
