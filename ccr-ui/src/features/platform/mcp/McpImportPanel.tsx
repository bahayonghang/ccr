import { useCallback } from 'react'
import { useForm } from 'react-hook-form'
import type { PlatformMeta, UnifiedMcpPlatform } from '@/types/unifiedMcp'
import type { TranslateFunction } from '@/utils/tf'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SIcon,
} from '@/ui'
import {
  CLAUDE_USER_SCOPE_PATH,
  mcpFieldInputClass,
  mcpFieldLabelClass,
  mcpGhostBtnClass,
  mcpIconBtnClass,
  mcpPanelBodyClass,
  mcpPanelFooterClass,
  mcpPanelHeaderClass,
  mcpPanelTitleClass,
  mcpPrimaryBtnClass,
} from './mcp-classes'
import { parseMcpImportJson, type ParsedMcpServer } from './parse-mcp-import'

interface ImportFormValues {
  jsonInput: string
  targetPlatform: UnifiedMcpPlatform
  targetScope: string
}

export interface McpImportPanelProps {
  platforms: UnifiedMcpPlatform[]
  platformMeta: Record<string, PlatformMeta>
  t: TranslateFunction
  onCancel: () => void
  onImport: (servers: ParsedMcpServer[], platform: string, scope?: string) => void
}

export function McpImportPanel({
  platforms,
  platformMeta,
  t,
  onCancel,
  onImport,
}: McpImportPanelProps) {
  const form = useForm<ImportFormValues>({
    defaultValues: {
      jsonInput: '',
      targetPlatform: platforms[0] ?? 'claude',
      targetScope: 'user',
    },
  })

  const jsonInput = form.watch('jsonInput')
  const targetPlatform = form.watch('targetPlatform')
  const parsed = parseMcpImportJson(jsonInput, t)

  const handlePlatformChange = useCallback(
    (value: string) => {
      form.setValue('targetPlatform', value as UnifiedMcpPlatform)
    },
    [form],
  )

  const handleScopeChange = useCallback(
    (value: string) => {
      form.setValue('targetScope', value)
    },
    [form],
  )

  const handleImport = useCallback(() => {
    if (parsed.servers.length === 0) return
    const platform = form.getValues('targetPlatform')
    const scope = platform === 'claude' ? form.getValues('targetScope') : undefined
    onImport(parsed.servers, platform, scope)
  }, [form, onImport, parsed.servers])

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className={mcpPanelHeaderClass}>
        <h2 className={mcpPanelTitleClass}>{t('mcp.manager.import.title')}</h2>
        <button type="button" className={mcpIconBtnClass} onClick={onCancel}>
          <SIcon name="X" size="w-4 h-4" />
        </button>
      </div>

      <div className={mcpPanelBodyClass}>
        <p className="text-sm leading-relaxed text-text-secondary">
          {t('mcp.manager.import.hintPrefix')}
          <code className="mx-1 rounded bg-bg-base/55 px-1.5 py-0.5 font-mono text-xs">mcpServers</code>
          {t('mcp.manager.import.hintSuffix')}
        </p>

        <div className="flex flex-col gap-1.5">
          <label className={mcpFieldLabelClass}>{t('mcp.manager.form.targetPlatform')}</label>
          <Select value={targetPlatform} onValueChange={handlePlatformChange}>
            <SelectTrigger className={mcpFieldInputClass}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {platforms.map((platform) => (
                <SelectItem key={platform} value={platform}>
                  {platformMeta[platform]?.label ?? platform}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {targetPlatform === 'claude' ? (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>{t('mcp.manager.form.claudeScope')}</label>
            <Select value={form.watch('targetScope')} onValueChange={handleScopeChange}>
              <SelectTrigger className={mcpFieldInputClass}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="user">
                  {t('mcp.manager.scopes.user')} — {CLAUDE_USER_SCOPE_PATH}
                </SelectItem>
                <SelectItem value="local">
                  {t('mcp.manager.scopes.local')} — {t('mcp.manager.form.currentProjectEntry')}
                </SelectItem>
                <SelectItem value="project">
                  {t('mcp.manager.scopes.project')} — {t('mcp.manager.form.repositoryMcpJson')}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        ) : null}

        <div className="flex flex-col gap-1.5">
          <label className={mcpFieldLabelClass}>{t('mcp.manager.import.jsonLabel')}</label>
          <textarea
            {...form.register('jsonInput')}
            className="w-full resize-y rounded-xl border border-border-default/55 bg-bg-elevated p-3 font-mono text-xs text-text-primary outline-none focus:border-accent-primary/40 focus:shadow-md"
            placeholder='{ "mcpServers": { "my-server": { "command": "npx", "args": ["-y", "my-mcp"] } } }'
            rows={12}
          />
        </div>

        {parsed.error ? (
          <div className="flex items-center gap-2 rounded-xl border border-danger/20 bg-danger/8 px-3 py-2.5 text-sm text-danger">
            <SIcon name="AlertCircle" size="w-4 h-4" />
            <span>{parsed.error}</span>
          </div>
        ) : null}

        {parsed.servers.length > 0 ? (
          <div className="flex flex-col gap-1.5">
            <h3 className={mcpFieldLabelClass}>
              {t('mcp.manager.import.previewTitle', { count: parsed.servers.length })}
            </h3>
            {parsed.servers.map((server) => (
              <div
                key={server.name}
                className="flex items-center gap-2 rounded-lg bg-bg-base/42 px-2.5 py-2"
              >
                <SIcon
                  name={server.type === 'stdio' ? 'Terminal' : 'Globe'}
                  size="w-4 h-4"
                  className="text-text-muted"
                />
                <span className="flex-1 text-sm font-medium text-text-primary">{server.name}</span>
                <span className="text-xs uppercase text-text-muted">{server.type}</span>
              </div>
            ))}
          </div>
        ) : null}
      </div>

      <div className={mcpPanelFooterClass}>
        <button type="button" className={mcpGhostBtnClass} onClick={onCancel}>
          {t('common.cancel')}
        </button>
        <button
          type="button"
          className={mcpPrimaryBtnClass}
          disabled={parsed.servers.length === 0}
          onClick={handleImport}
        >
          <SIcon name="Download" size="w-4 h-4" />
          <span>{t('mcp.manager.import.submit', { count: parsed.servers.length })}</span>
        </button>
      </div>
    </div>
  )
}
