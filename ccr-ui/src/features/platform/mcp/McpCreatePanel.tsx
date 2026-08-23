import { useCallback, type FormEvent } from 'react'
import { useForm, type UseFormReturn } from 'react-hook-form'
import type { PlatformMeta, UnifiedMcpPlatform, UnifiedMcpRequest } from '@/types/unifiedMcp'
import type { TranslateFunction } from '@/utils/tf'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SIcon,
  cn,
} from '@/ui'
import {
  CLAUDE_USER_SCOPE_PATH,
  mcpFieldInputClass,
  mcpFieldLabelClass,
  mcpGhostBtnClass,
  mcpMonoInputClass,
  mcpPanelBodyClass,
  mcpPanelFooterClass,
  mcpPanelHeaderClass,
  mcpPanelTitleClass,
  mcpPrimaryBtnClass,
} from './mcp-classes'
import { McpKvEditor } from './McpKvEditor'

export interface McpCreatePanelProps {
  isEditing: boolean
  formApi: UseFormReturn<UnifiedMcpRequest>
  formData: UnifiedMcpRequest
  isHttpMode: boolean
  argInput: string
  envKey: string
  envValue: string
  headerKey: string
  headerValue: string
  platforms: UnifiedMcpPlatform[]
  platformMeta: Record<string, PlatformMeta>
  t: TranslateFunction
  onSubmit: () => void
  onCancel: () => void
  onIsHttpModeChange: (value: boolean) => void
  onArgInputChange: (value: string) => void
  onEnvKeyChange: (value: string) => void
  onEnvValueChange: (value: string) => void
  onHeaderKeyChange: (value: string) => void
  onHeaderValueChange: (value: string) => void
  onAddEnv: () => void
  onRemoveEnv: (key: string) => void
  onAddHeader: () => void
  onRemoveHeader: (key: string) => void
}

function McpProtocolToggle({
  isHttpMode,
  onIsHttpModeChange,
}: {
  isHttpMode: boolean
  onIsHttpModeChange: (value: boolean) => void
}) {
  const handleStdio = useCallback(() => {
    onIsHttpModeChange(false)
  }, [onIsHttpModeChange])

  const handleHttp = useCallback(() => {
    onIsHttpModeChange(true)
  }, [onIsHttpModeChange])

  return (
    <div className="flex gap-1 rounded-xl border border-border-default/35 bg-bg-base/55 p-1">
      <button
        type="button"
        className={cn(
          'inline-flex flex-1 items-center justify-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium text-text-muted transition-colors',
          !isHttpMode && 'bg-bg-surface text-text-primary shadow-sm',
        )}
        onClick={handleStdio}
      >
        <SIcon name="Terminal" size="w-4 h-4" />
        STDIO
      </button>
      <button
        type="button"
        className={cn(
          'inline-flex flex-1 items-center justify-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium text-text-muted transition-colors',
          isHttpMode && 'bg-bg-surface text-text-primary shadow-sm',
        )}
        onClick={handleHttp}
      >
        <SIcon name="Globe" size="w-4 h-4" />
        HTTP
      </button>
    </div>
  )
}

export function McpCreatePanel({
  isEditing,
  formApi,
  formData,
  isHttpMode,
  argInput,
  envKey,
  envValue,
  headerKey,
  headerValue,
  platforms,
  platformMeta,
  t,
  onSubmit,
  onCancel,
  onIsHttpModeChange,
  onArgInputChange,
  onEnvKeyChange,
  onEnvValueChange,
  onHeaderKeyChange,
  onHeaderValueChange,
  onAddEnv,
  onRemoveEnv,
  onAddHeader,
  onRemoveHeader,
}: McpCreatePanelProps) {
  const handleSubmit = useCallback(
    (event: FormEvent) => {
      event.preventDefault()
      onSubmit()
    },
    [onSubmit],
  )

  const handlePlatformChange = useCallback(
    (value: string) => {
      formApi.setValue('platform', value)
    },
    [formApi],
  )

  const handleScopeChange = useCallback(
    (value: string) => {
      formApi.setValue('scope', value)
    },
    [formApi],
  )

  const extra = useForm({ values: { argInput } })

  const handleArgInputChange = useCallback(
    (event: { target: EventTarget | null }) => {
      const target = event.target as HTMLInputElement
      onArgInputChange(target.value)
    },
    [onArgInputChange],
  )

  const envEntries = formData.env ?? {}
  const headerEntries = formData.headers ?? {}

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className={mcpPanelHeaderClass}>
        <h2 className={mcpPanelTitleClass}>
          {isEditing ? t('mcp.manager.form.editTitle') : t('mcp.manager.form.addTitle')}
        </h2>
        <button type="button" className={mcpGhostBtnClass} onClick={onCancel}>
          <SIcon name="X" size="w-4 h-4" />
        </button>
      </div>

      <form className={mcpPanelBodyClass} onSubmit={handleSubmit}>
        <div className="flex flex-col gap-1.5">
          <label className={mcpFieldLabelClass}>{t('mcp.manager.form.targetPlatform')}</label>
          <Select value={formData.platform} onValueChange={handlePlatformChange} disabled={isEditing}>
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

        {formData.platform === 'claude' ? (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>{t('mcp.manager.form.claudeScope')}</label>
            <Select value={formData.scope ?? 'user'} onValueChange={handleScopeChange}>
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
            {formData.scope === 'project' ? (
              <p className="text-xs leading-snug text-warning/92">
                {t('mcp.manager.form.projectScopeWarningPrefix')}
                <code className="font-mono"> .mcp.json </code>
                {t('mcp.manager.form.projectScopeWarningSuffix')}
              </p>
            ) : null}
          </div>
        ) : null}

        <div className="flex flex-col gap-1.5">
          <label className={mcpFieldLabelClass}>
            {t('mcp.manager.form.nameLabel')} <span className="text-danger">*</span>
          </label>
          <input
            {...formApi.register('name')}
            type="text"
            className={mcpFieldInputClass}
            placeholder="my-mcp-server"
            disabled={isEditing}
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <label className={mcpFieldLabelClass}>{t('mcp.manager.form.protocolLabel')}</label>
          <McpProtocolToggle isHttpMode={isHttpMode} onIsHttpModeChange={onIsHttpModeChange} />
        </div>

        {isHttpMode ? (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>
              {t('mcp.manager.detail.urlLabel')} <span className="text-danger">*</span>
            </label>
            <input
              {...formApi.register('url')}
              type="text"
              className={mcpMonoInputClass}
              placeholder="http://localhost:3000/mcp"
            />
          </div>
        ) : (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>
              {t('mcp.manager.detail.commandLabel')} <span className="text-danger">*</span>
            </label>
            <input
              {...formApi.register('command')}
              type="text"
              className={mcpMonoInputClass}
              placeholder="npx -y @example/mcp-server"
            />
          </div>
        )}

        {isHttpMode ? null : (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>{t('mcp.manager.form.argsLabel')}</label>
            <input
              {...extra.register('argInput', { onChange: handleArgInputChange })}
              type="text"
              className={mcpMonoInputClass}
              placeholder="--port 3000 --verbose"
            />
          </div>
        )}

        {isHttpMode ? (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>{t('mcp.manager.form.headersLabel')}</label>
            <McpKvEditor
              entries={headerEntries}
              keyValue={headerKey}
              valueValue={headerValue}
              onKeyChange={onHeaderKeyChange}
              onValueChange={onHeaderValueChange}
              onAdd={onAddHeader}
              onRemove={onRemoveHeader}
              keyPlaceholder="Header-Name"
              valuePlaceholder="header-value"
            />
          </div>
        ) : (
          <div className="flex flex-col gap-1.5">
            <label className={mcpFieldLabelClass}>{t('mcp.manager.form.envLabel')}</label>
            <McpKvEditor
              entries={envEntries}
              keyValue={envKey}
              valueValue={envValue}
              onKeyChange={onEnvKeyChange}
              onValueChange={onEnvValueChange}
              onAdd={onAddEnv}
              onRemove={onRemoveEnv}
              keyPlaceholder={t('mcp.manager.form.envKeyPlaceholder')}
              valuePlaceholder={t('mcp.manager.form.envValuePlaceholder')}
            />
          </div>
        )}

        <div className={mcpPanelFooterClass}>
          <button type="button" className={mcpGhostBtnClass} onClick={onCancel}>
            {t('common.cancel')}
          </button>
          <button type="submit" className={mcpPrimaryBtnClass}>
            <SIcon name="Check" size="w-4 h-4" />
            <span>{isEditing ? t('common.save') : t('mcp.manager.form.create')}</span>
          </button>
        </div>
      </form>
    </div>
  )
}
