import { memo, useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { installMcpPreset, listMcpPresets } from '@/api'
import type { McpPreset } from '@/types/api'
import type { SyncResult } from '@/types/sync'
import { logger } from '@/utils/logger'
import { BaseModal, SIcon, cn } from '@/ui'
import { useMcpT } from './locale'
import { mcpNotify } from './notify'

const PLATFORMS = [
  { id: 'claude', name: 'Claude', icon: 'Bot' },
  { id: 'codex', name: 'Codex', icon: 'Terminal' },
  { id: 'gemini', name: 'Antigravity CLI', icon: 'Sparkles' },
]

interface McpPresetsPanelProps {
  onInstalled: () => void
}

const PresetCard = memo(function PresetCard({
  preset,
  onOpen,
}: {
  preset: McpPreset
  onOpen: (preset: McpPreset) => void
}) {
  const t = useMcpT()
  const handleOpen = useCallback(() => {
    onOpen(preset)
  }, [onOpen, preset])
  return (
    <button type="button" className="rounded-2xl border border-border-default/15 bg-bg-elevated p-4 text-left hover:border-accent-secondary/30" onClick={handleOpen}>
      <div className="mb-3 flex flex-wrap gap-1.5">
        {preset.tags.slice(0, 2).map((tag) => (
          <span key={tag} className="rounded-full bg-accent-secondary/10 px-2 py-0.5 text-[0.625rem] font-medium text-accent-secondary">
            {tag}
          </span>
        ))}
        {preset.requires_api_key ? (
          <span className="inline-flex items-center gap-1 rounded-full bg-accent-warning/10 px-2 py-0.5 text-[0.625rem] font-medium text-accent-warning">
            <SIcon name="KeyRound" size="w-3 h-3" />
            {t('mcp.presets.apiKeyBadge')}
          </span>
        ) : null}
      </div>
      <h3 className="mb-1 truncate text-sm font-bold text-text-primary">{preset.name}</h3>
      <p className="mb-3 line-clamp-2 text-xs text-text-muted">{preset.description}</p>
      <div className="flex items-center gap-1.5 overflow-hidden rounded-lg bg-bg-base px-2 py-1.5 font-mono text-[0.625rem] text-text-muted">
        <SIcon name="Terminal" size="w-3 h-3" />
        <span className="truncate">{preset.command} {(preset.args || []).join(' ')}</span>
      </div>
    </button>
  )
})

export function McpPresetsPanel({ onInstalled }: McpPresetsPanelProps) {
  const t = useMcpT()
  const [open, setOpen] = useState(true)
  const [loading, setLoading] = useState(true)
  const [presets, setPresets] = useState<McpPreset[]>([])
  const [selected, setSelected] = useState<McpPreset | null>(null)
  const [platforms, setPlatforms] = useState<string[]>(['claude'])
  const [installing, setInstalling] = useState(false)
  const [results, setResults] = useState<SyncResult[]>([])
  const form = useForm<{ apiKey: string }>({ defaultValues: { apiKey: '' } })

  const loadPresets = useCallback(async () => {
    try {
      setLoading(true)
      setPresets(await listMcpPresets<McpPreset[]>())
    } catch (err) {
      logger.error('Failed to load MCP presets:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadPresets()
  }, [loadPresets])

  const togglePanel = useCallback(() => {
    setOpen((value) => !value)
  }, [])
  const closeModal = useCallback(() => {
    setSelected(null)
    form.reset({ apiKey: '' })
    setResults([])
  }, [form])
  const handleOpenChange = useCallback((next: boolean) => {
    if (!next) closeModal()
  }, [closeModal])
  const openPreset = useCallback((preset: McpPreset) => {
    setSelected(preset)
    setPlatforms(['claude'])
    form.reset({ apiKey: '' })
    setResults([])
  }, [form])

  const togglePlatform = useCallback((id: string) => {
    setPlatforms((current) => (current.includes(id) ? current.filter((item) => item !== id) : [...current, id]))
  }, [])

  const confirmInstall = useCallback(async () => {
    if (!selected || platforms.length === 0) return
    const apiKey = form.getValues('apiKey')
    if (selected.requires_api_key && selected.api_key_env && !apiKey) {
      mcpNotify.warning(t('mcp.presets.apiKeyRequired'))
      return
    }
    setInstalling(true)
    try {
      const env: Record<string, string> = {}
      if (selected.api_key_env && apiKey) env[selected.api_key_env] = apiKey
      const result = await installMcpPreset<{ results: SyncResult[] }>(selected.id, platforms, env)
      const next = result.results ?? []
      const failed = next.filter((item) => !item.success)
      if (failed.length > 0) {
        setResults(next)
        onInstalled()
      } else {
        mcpNotify.success(t('mcp.presets.installSuccess'))
        closeModal()
        onInstalled()
      }
    } catch (err) {
      logger.error('Failed to install preset:', err)
      mcpNotify.error(`${t('mcp.presets.installFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setInstalling(false)
    }
  }, [closeModal, form, onInstalled, platforms, selected, t])

  const handleConfirm = useCallback(() => {
    void confirmInstall()
  }, [confirmInstall])

  return (
    <div className="rounded-3xl border border-border-default/25 bg-bg-elevated p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold text-text-primary">{t('mcp.presets.title')}</h2>
          <p className="text-xs text-text-muted">{t('mcp.presets.subtitle')}</p>
        </div>
        <button type="button" className="flex items-center gap-1.5 rounded-lg bg-bg-surface px-3 py-1.5 text-xs text-text-secondary" onClick={togglePanel}>
          <SIcon name={open ? 'ChevronUp' : 'ChevronDown'} size="w-4 h-4" />
          {open ? t('mcp.presets.collapse') : t('mcp.presets.expand')}
        </button>
      </div>
      {open && loading ? <div className="flex justify-center py-8"><div className="loading-spinner h-8 w-8 border-accent-secondary/30 border-t-accent-secondary" /></div> : null}
      {open && !loading ? (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
          {presets.map((preset) => (
            <PresetCard key={preset.id} preset={preset} onOpen={openPreset} />
          ))}
        </div>
      ) : null}

      <BaseModal modelValue={Boolean(selected)} title={selected?.name ?? t('mcp.presets.install')} size="md" surface="solid" onUpdateModelValue={handleOpenChange} onClose={closeModal} footer={
        <div className="flex w-full gap-3">
          <button type="button" className="flex-1 rounded-xl border border-border-default bg-bg-elevated px-4 py-2 text-text-secondary" onClick={closeModal}>{t('common.cancel')}</button>
          <button type="button" className="flex-1 rounded-xl bg-accent-secondary px-4 py-2 text-[color:var(--color-accent-primary-contrast)] disabled:opacity-60" disabled={installing || platforms.length === 0} onClick={handleConfirm}>
            {installing ? t('mcp.presets.installing') : t('mcp.presets.confirmInstall')}
          </button>
        </div>
      }>
        <div className="grid gap-3">
          <p className="text-sm text-text-secondary">{selected?.description}</p>
          <div className="flex flex-wrap gap-2">
            {PLATFORMS.map((platform) => (
              <PlatformChip key={platform.id} id={platform.id} name={platform.name} icon={platform.icon} active={platforms.includes(platform.id)} onToggle={togglePlatform} />
            ))}
          </div>
          {selected?.requires_api_key ? (
            <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2 font-mono text-sm" placeholder={selected.api_key_env ?? 'API key'} {...form.register('apiKey')} />
          ) : null}
          {results.map((result) => (
            <p key={result.platform} className={result.success ? 'text-accent-success' : 'text-accent-danger'}>
              {result.platform}: {result.message}
            </p>
          ))}
        </div>
      </BaseModal>
    </div>
  )
}

const PlatformChip = memo(function PlatformChip({
  id,
  name,
  icon,
  active,
  onToggle,
}: {
  id: string
  name: string
  icon: string
  active: boolean
  onToggle: (id: string) => void
}) {
  const handleClick = useCallback(() => {
    onToggle(id)
  }, [id, onToggle])
  return (
    <button type="button" className={cn('inline-flex items-center gap-1.5 rounded-xl border px-3 py-2 text-xs', active ? 'border-accent-secondary/30 bg-accent-secondary/20 text-accent-secondary' : 'border-transparent bg-bg-surface text-text-muted')} onClick={handleClick}>
      <SIcon name={icon} size="w-3.5 h-3.5" />
      {name}
    </button>
  )
})
