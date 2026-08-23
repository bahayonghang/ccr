import { memo, useCallback } from 'react'
import {
  opencodeBuiltInTools,
  opencodeCliCommands,
  opencodeConfigTopology,
} from '@/config/opencodeMeta'
import type { OpenCodeAgent, OpenCodeCommand, OpenCodeLocalPluginFile, OpenCodeTheme } from '@/types'
import { SIcon } from '@/ui'
import { OC_THEME_VAR_PREFIX } from '../theme/ocThemeVars'
import { useOpenCodeLocale } from '../locale'

type InspectorTab = 'runtime' | 'tools' | 'topology' | 'discovery' | 'themes'

interface OpenCodeInspectorProps {
  active: InspectorTab
  onSelect: (id: InspectorTab) => void
  localPlugins: OpenCodeLocalPluginFile[]
  agents: OpenCodeAgent[]
  commands: OpenCodeCommand[]
  themes: OpenCodeTheme[]
}

const TABS: { id: InspectorTab; label: string }[] = [
  { id: 'runtime', label: 'CLI runtime' },
  { id: 'tools', label: 'Built-in tools' },
  { id: 'topology', label: 'Config topology' },
  { id: 'discovery', label: 'Local discovery' },
  { id: 'themes', label: 'Themes' },
]

export function OpenCodeInspector({
  active,
  onSelect,
  localPlugins,
  agents,
  commands,
  themes,
}: OpenCodeInspectorProps) {
  const { tt } = useOpenCodeLocale()
  const localPreview = localPlugins.slice(0, 3).map((item) => item.name).join(', ') || 'No plugin files detected'
  const primary = agents.filter((agent) => agent.mode === 'primary').length
  const subagent = agents.filter((agent) => agent.mode === 'subagent').length
  const project = commands.filter((command) => command.scope === 'project').length

  return (
    <section className={['rounded-2xl border border-border-subtle bg-bg-surface p-5'].join(' ')}>
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-[0.6875rem] font-semibold uppercase tracking-wide text-text-muted">
            {tt('精简巡检', 'Compact inspector')}
          </p>
          <h2 className="text-lg font-semibold text-text-primary">{tt('运行态情报', 'Runtime intelligence')}</h2>
        </div>
        <div className="flex flex-wrap gap-2" role="tablist" aria-label="OpenCode inspector sections">
          {TABS.map((tab) => (
            <InspectorTabButton key={tab.id} tab={tab} active={active === tab.id} onSelect={onSelect} />
          ))}
        </div>
      </div>
      {active === 'runtime' ? (
        <div className="grid gap-2">
          {opencodeCliCommands.map((item) => (
            <div key={item.command} className="rounded-xl border border-border-subtle px-3 py-2">
              <code className="font-mono text-sm text-text-primary">{item.command}</code>
              <span className="ml-2 text-sm text-text-secondary">{item.description}</span>
              {item.note ? <strong className="ml-2 text-xs text-text-muted">{item.note}</strong> : null}
            </div>
          ))}
        </div>
      ) : null}
      {active === 'tools' ? (
        <div className="grid gap-2 md:grid-cols-2">
          {opencodeBuiltInTools.map((tool) => (
            <div key={tool.id} className="rounded-xl border border-border-subtle p-3">
              <div className="flex items-center justify-between">
                <strong>{tool.id}</strong>
                <span className="text-xs text-text-muted">{tool.permissionKey}</span>
              </div>
              <p className="mt-1 text-sm text-text-secondary">{tool.description}</p>
            </div>
          ))}
        </div>
      ) : null}
      {active === 'topology' ? (
        <div className="grid gap-2">
          {opencodeConfigTopology.map((item) => (
            <div key={item.path} className="rounded-xl border border-border-subtle p-3">
              <span className="text-xs uppercase tracking-wide text-text-muted">{item.title}</span>
              <code className="mt-1 block font-mono text-sm">{item.path}</code>
              <p className="mt-1 text-sm text-text-secondary">{item.description}</p>
            </div>
          ))}
        </div>
      ) : null}
      {active === 'discovery' ? (
        <div className="grid gap-3 md:grid-cols-3">
          <DiscoveryCard label={tt('本地插件', 'Local plugins')} value={String(localPlugins.length)} detail={localPreview} />
          <DiscoveryCard
            label="Agents"
            value={String(agents.length)}
            detail={`${primary} primary · ${subagent} subagent · ${Math.max(agents.length - primary - subagent, 0)} mixed`}
          />
          <DiscoveryCard
            label="Commands"
            value={String(commands.length)}
            detail={`${project} project · ${commands.length - project} global/builtin`}
          />
        </div>
      ) : null}
      {active === 'themes' ? (
        <div className="grid gap-2 md:grid-cols-2">
          {themes.map((theme) => (
            <ThemeChip key={theme.id} id={theme.id} name={theme.name} themeType={theme.themeType} />
          ))}
        </div>
      ) : null}
    </section>
  )
}

const InspectorTabButton = memo(function InspectorTabButton({
  tab,
  active,
  onSelect,
}: {
  tab: { id: InspectorTab; label: string }
  active: boolean
  onSelect: (id: InspectorTab) => void
}) {
  const handleClick = useCallback(() => onSelect(tab.id), [onSelect, tab.id])
  return (
    <button
      type="button"
      role="tab"
      aria-selected={active}
      className={
        active
          ? 'rounded-full border border-accent-primary/40 bg-accent-primary/10 px-3 py-1.5 text-xs'
          : 'rounded-full border border-border-default px-3 py-1.5 text-xs'
      }
      onClick={handleClick}
    >
      {tab.label}
    </button>
  )
})

function DiscoveryCard({ label, value, detail }: { label: string; value: string; detail: string }) {
  return (
    <div className="rounded-2xl border border-border-default/50 bg-bg-base p-3">
      <span className="text-[0.6875rem] font-semibold uppercase tracking-wide text-text-muted">{label}</span>
      <strong className="mt-1 block text-lg text-text-primary">{value}</strong>
      <p className="mt-1 text-sm text-text-secondary">{detail}</p>
    </div>
  )
}

function ThemeChip({ id, name, themeType }: { id: string; name: string; themeType: string }) {
  const swatch = themeType === 'dark' ? 'var(--color-bg-elevated)' : 'var(--color-bg-surface)'
  return (
    <article
      className="flex items-center gap-3 rounded-xl border border-border-subtle p-3"
      style={{
        [OC_THEME_VAR_PREFIX + 'theme-id']: id,
        [OC_THEME_VAR_PREFIX + 'theme-name']: name,
        [OC_THEME_VAR_PREFIX + 'theme-type']: themeType,
        [OC_THEME_VAR_PREFIX + 'theme-swatch']: swatch,
      }}
    >
      <span
        className="h-6 w-6 rounded-full border border-border-default"
        style={{ background: `var(${OC_THEME_VAR_PREFIX}theme-swatch)` }}
        aria-hidden="true"
      />
      <div>
        <strong className="block text-sm text-text-primary">{name}</strong>
        <span className="text-xs text-text-muted">{id} · {themeType}</span>
      </div>
      <SIcon name="Palette" size="w-4 h-4" className="ml-auto text-text-muted" />
    </article>
  )
}
