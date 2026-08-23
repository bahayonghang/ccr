import { useCallback, useEffect, useMemo, useState } from 'react'
import { opencodeCapabilityCards } from '@/config/opencodeMeta'
import { PageHeader, PageShell, SIcon, StatTile } from '@/ui'
import { CapabilityCard } from './home/OpenCodeHomeCards'
import { OpenCodeInspector } from './home/OpenCodeInspector'
import { OpenCodeOpsBoard } from './home/OpenCodeOpsBoard'
import { OpenCodeUsageStrip } from './home/OpenCodeUsageStrip'
import { useOpenCodeHome } from './home/useOpenCodeHome'
import { useOpenCodeLocale } from './locale'
import { ghostBtnClass } from './ui-classes'

type InspectorTab = 'runtime' | 'tools' | 'topology' | 'discovery' | 'themes'

function statusLabelOf(loading: boolean, loadedOnce: boolean, warningCount: number) {
  if (loading && !loadedOnce) return 'Loading local OpenCode surfaces…'
  if (warningCount > 0) return `${warningCount} degraded source(s), usable data kept visible.`
  if (loadedOnce) return 'All local surfaces are available.'
  return 'Ready to read local settings.'
}

export function OpenCodeView() {
  const { tt } = useOpenCodeLocale()
  const home = useOpenCodeHome()
  const [activeInspector, setActiveInspector] = useState<InspectorTab>('runtime')
  const {
    loading,
    loadedOnce,
    loadErrors,
    config,
    tui,
    providers,
    mcpServers,
    agents,
    commands,
    plugins,
    localPlugins,
    themes,
    loadOverview,
  } = home

  useEffect(() => {
    void loadOverview()
  }, [loadOverview])

  const handleRefresh = useCallback(() => {
    void loadOverview()
  }, [loadOverview])

  const themeLabel = tui.theme || 'system'
  const shareLabel = config.share || 'manual'
  const serverLabel = `${config.server?.hostname || 'localhost'}:${config.server?.port ?? 4096}`
  const warningItems = Object.keys(loadErrors)
  const overviewStatusLabel = statusLabelOf(loading, loadedOnce, warningItems.length)

  const capabilityDeck = useMemo(() => {
    const counts = {
      providers: providers.length,
      mcp: mcpServers.length,
      agents: agents.length,
      commands: commands.length,
      plugins: plugins.length + localPlugins.length,
      settings: Object.keys(config).length + Object.keys(tui).length,
    }
    const failed = {
      providers: Boolean(loadErrors.providers || loadErrors.config),
      mcp: Boolean(loadErrors.mcp || loadErrors.config),
      agents: Boolean(loadErrors.agents),
      commands: Boolean(loadErrors.commands),
      plugins: Boolean(loadErrors.plugins || loadErrors.localPlugins || loadErrors.config),
      settings: Boolean(loadErrors.config || loadErrors.tui),
    }
    return opencodeCapabilityCards.map((card) => {
      const count = counts[card.id as keyof typeof counts] ?? 0
      const isFailed = failed[card.id as keyof typeof failed]
      return {
        ...card,
        badge: isFailed ? 'warning' : `${count} live`,
        cta: isFailed ? 'Retry or inspect' : 'Open surface',
        status: isFailed ? 'warning' : 'ok',
      }
    })
  }, [
    agents.length,
    commands.length,
    config,
    loadErrors,
    localPlugins.length,
    mcpServers.length,
    plugins.length,
    providers.length,
    tui,
  ])

  return (
    <PageShell
      className="opencode-view bg-bg-elevated"
      header={
        <PageHeader
          title={tt('操作总台', 'Operational console')}
          eyebrow="OpenCode operator deck"
          eyebrowLang="en"
          description={tt(
            '高密度收敛 provider、MCP、agents、commands、plugins 与 runtime 配置；首屏直接进入可操作状态。',
            'Bring providers, MCP, agents, commands, plugins, and runtime config into one dense surface that is actionable on first load.',
          )}
          actions={
            <button type="button" className={ghostBtnClass} disabled={loading} onClick={handleRefresh}>
              <SIcon name="RefreshCw" size="w-4 h-4" className={loading ? 'animate-spin' : undefined} />
              <span>{loading ? tt('加载中', 'Loading') : tt('刷新', 'Refresh')}</span>
            </button>
          }
        />
      }
    >
      <div className="mb-4 grid grid-cols-[repeat(auto-fit,minmax(10rem,1fr))] gap-4 rounded-xl border border-border-subtle bg-bg-surface p-4">
        <StatTile label="Providers" value={providers.length} hint={config.model || 'not configured'} />
        <StatTile label="MCP" value={mcpServers.length} hint={`${agents.length} agents`} />
        <StatTile label="Commands" value={commands.length} hint={`${plugins.length + localPlugins.length} plugins`} />
        <StatTile label="Theme" value={themeLabel} hint={`share ${shareLabel}`} />
      </div>

      <OpenCodeOpsBoard
        configPath="~/.config/opencode/opencode.json"
        tuiPath="~/.config/opencode/tui.json"
        defaultAgent={config.default_agent || 'build'}
        serverLabel={serverLabel}
        webLabel={config.server?.cors?.length ? 'cors configured' : 'same host'}
        shareLabel={shareLabel}
        configWarn={Boolean(loadErrors.config)}
        nextLabel={tt('下一步动作', 'Next actions')}
        statusLabel={overviewStatusLabel}
        providerCount={providers.length}
        mcpCount={mcpServers.length}
        themeLabel={themeLabel}
      />

      <OpenCodeUsageStrip />

      <section className="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-3" aria-label="OpenCode capability entries">
        {capabilityDeck.map((card) => (
          <CapabilityCard key={card.id} item={card} />
        ))}
      </section>

      <div className="mt-6">
        <OpenCodeInspector
          active={activeInspector}
          onSelect={setActiveInspector}
          localPlugins={localPlugins}
          agents={agents}
          commands={commands}
          themes={themes}
        />
      </div>
    </PageShell>
  )
}


