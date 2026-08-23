import { useCallback, useState } from 'react'
import {
  getOpenCodeConfig,
  getOpenCodeTuiSettings,
  listOpenCodeAgents,
  listOpenCodeCommands,
  listOpenCodeLocalPlugins,
  listOpenCodeMcpServers,
  listOpenCodePlugins,
  listOpenCodeProviders,
  listOpenCodeThemes,
} from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import type {
  OpenCodeAgent,
  OpenCodeCommand,
  OpenCodeConfig,
  OpenCodeLocalPluginFile,
  OpenCodeMcpServer,
  OpenCodePluginPackage,
  OpenCodeProviderConfig,
  OpenCodeTheme,
  OpenCodeTuiConfig,
} from '@/types'

export interface OpenCodeHomeState {
  loading: boolean
  loadedOnce: boolean
  loadErrors: Record<string, string>
  config: OpenCodeConfig
  tui: OpenCodeTuiConfig
  providers: OpenCodeProviderConfig[]
  mcpServers: OpenCodeMcpServer[]
  agents: OpenCodeAgent[]
  commands: OpenCodeCommand[]
  plugins: OpenCodePluginPackage[]
  localPlugins: OpenCodeLocalPluginFile[]
  themes: OpenCodeTheme[]
}

const emptyState = (): Omit<OpenCodeHomeState, 'loading' | 'loadedOnce' | 'loadErrors'> => ({
  config: {},
  tui: {},
  providers: [],
  mcpServers: [],
  agents: [],
  commands: [],
  plugins: [],
  localPlugins: [],
  themes: [],
})

export function useOpenCodeHome() {
  const [loading, setLoading] = useState(false)
  const [loadedOnce, setLoadedOnce] = useState(false)
  const [loadErrors, setLoadErrors] = useState<Record<string, string>>({})
  const [data, setData] = useState(emptyState)

  const loadOverview = useCallback(async () => {
    setLoading(true)
    const tasks = {
      config: getOpenCodeConfig(),
      tui: getOpenCodeTuiSettings(),
      providers: listOpenCodeProviders(),
      mcp: listOpenCodeMcpServers(),
      agents: listOpenCodeAgents(),
      commands: listOpenCodeCommands(),
      plugins: listOpenCodePlugins(),
      localPlugins: listOpenCodeLocalPlugins(),
      themes: listOpenCodeThemes(),
    }
    const entries = Object.entries(tasks)
    const results = await Promise.allSettled(entries.map(([, task]) => task))
    const nextErrors: Record<string, string> = {}
    const next = emptyState()
    const asList = <T,>(value: unknown): T[] => (Array.isArray(value) ? (value as T[]) : [])
    results.forEach((result, index) => {
      const key = entries[index]?.[0]
      if (!key) return
      if (result.status === 'rejected') {
        nextErrors[key] = getErrorMessage(result.reason)
        return
      }
      if (key === 'config') next.config = result.value as OpenCodeConfig
      if (key === 'tui') next.tui = result.value as OpenCodeTuiConfig
      if (key === 'providers') next.providers = asList<OpenCodeProviderConfig>(result.value)
      if (key === 'mcp') next.mcpServers = asList<OpenCodeMcpServer>(result.value)
      if (key === 'agents') next.agents = asList<OpenCodeAgent>(result.value)
      if (key === 'commands') next.commands = asList<OpenCodeCommand>(result.value)
      if (key === 'plugins') next.plugins = asList<string>(result.value).map((name) => ({ name }))
      if (key === 'localPlugins') next.localPlugins = asList<OpenCodeLocalPluginFile>(result.value)
      if (key === 'themes') next.themes = asList<OpenCodeTheme>(result.value)
    })
    setData(next)
    setLoadErrors(nextErrors)
    setLoadedOnce(true)
    setLoading(false)
  }, [])

  return { loading, loadedOnce, loadErrors, ...data, loadOverview }
}
