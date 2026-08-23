import { memo, useCallback, type ChangeEvent } from 'react'
import { useForm } from 'react-hook-form'
import { Checkbox, SIcon, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/ui'
import type { useCommandsPage } from './useCommandsPage'

type Page = ReturnType<typeof useCommandsPage>

function ComposerActions({
  page,
  selectedFavorite,
  onFavorite,
  onCancel,
  onExecute,
}: {
  page: Page
  selectedFavorite: boolean
  onFavorite: () => void
  onCancel: () => void
  onExecute: () => void
}) {
  return (
    <div className="commands-composer__actions">
      <button type="button" className="inline-flex items-center gap-2 rounded-lg border border-border-default px-3 py-2 text-sm" disabled={page.selectedClient !== 'ccr' || !page.selectedCommandInfo?.executable} onClick={onFavorite}>
        <SIcon name={selectedFavorite ? 'StarOff' : 'Star'} size="w-4 h-4" />
        {selectedFavorite ? page.t('commands.removeFavorite') : page.t('commands.addFavorite')}
      </button>
      {page.isRunning ? (
        <button type="button" className="inline-flex items-center gap-2 rounded-lg border border-accent-danger/30 px-3 py-2 text-sm text-accent-danger" disabled={!page.currentSnapshot} onClick={onCancel}>
          <SIcon name="Square" size="w-4 h-4" />
          {page.t('commands.cancelJob')}
        </button>
      ) : null}
      <button type="button" className="inline-flex items-center gap-2 rounded-lg bg-accent-primary px-3 py-2 text-sm text-[color:var(--color-accent-primary-contrast)] disabled:opacity-55" disabled={!page.canExecuteSelected} onClick={onExecute}>
        <SIcon name="Play" size="w-4 h-4" />
        {page.isRunning ? page.t('commands.executing') : page.t('commands.run')}
      </button>
    </div>
  )
}

function ArgsAndDanger({
  page,
  onSelectConfig,
  onArgsInput,
  onDanger,
}: {
  page: Page
  onSelectConfig: (value: string) => void
  onArgsInput: (event: ChangeEvent<HTMLInputElement>) => void
  onDanger: (value: boolean | 'indeterminate') => void
}) {
  const placeholder = page.selectedCommandInfo?.requiresArgs ? page.t('commands.requiredArgsPlaceholder') : page.t('commands.argsPlaceholder')
  return (
    <div className="commands-form-grid">
      <label className="commands-field">
        <span>{page.t('commands.args')}</span>
        {page.selectedCommand === 'switch' ? (
          <Select value={page.args} onValueChange={onSelectConfig} disabled={!page.canEditArgs}>
            <SelectTrigger><SelectValue placeholder={page.t('commands.selectConfig')} /></SelectTrigger>
            <SelectContent>
              {page.configs.map((config) => (
                <SelectItem key={config.name} value={config.name}>{config.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <input key={page.selectedCommand} type="text" defaultValue={page.args} disabled={!page.canEditArgs} placeholder={placeholder} onChange={onArgsInput} />
        )}
      </label>
      {page.selectedCommandInfo?.dangerous ? (
        <label className="commands-danger-confirm">
          <Checkbox checked={page.dangerAccepted} disabled={page.runtimeUnavailable || page.isRunning} onCheckedChange={onDanger} />
          <span>
            <strong>{page.t('commands.dangerConfirmTitle')}</strong>
            {page.t('commands.dangerConfirmDescription')}
          </span>
        </label>
      ) : null}
    </div>
  )
}

function ComposerNotices({ page }: { page: Page }) {
  if (page.runtimeUnavailable) {
    return (
      <div className="commands-notice commands-notice--neutral">
        <SIcon name="MonitorOff" size="w-5 h-5" />
        <div>
          <strong>{page.runtimeCopy.title}</strong>
          <p>{page.t('commands.webUnavailableDetail')}</p>
        </div>
      </div>
    )
  }
  if (page.selectedClient !== 'ccr') {
    const clientName = page.CLI_CLIENTS.find((item) => item.id === page.selectedClient)?.name ?? page.selectedClient
    return (
      <div className="commands-notice">
        <SIcon name="Lock" size="w-5 h-5" />
        <div>
          <strong>{page.t('commands.clientUnavailableTitle')}</strong>
          <p>{page.t('commands.clientUnavailableDescription', { client: clientName })}</p>
        </div>
      </div>
    )
  }
  return null
}

export const CommandsComposer = memo(function CommandsComposer({ page }: { page: Page }) {
  const form = useForm({ values: { args: page.args } })
  const handleArgs = useCallback((value: string) => {
    page.setArgs(value)
    form.setValue('args', value)
  }, [form, page])
  const onArgsInput = useCallback((event: ChangeEvent<HTMLInputElement>) => {
    handleArgs(event.currentTarget.value)
  }, [handleArgs])
  const onSelectConfig = useCallback((value: string) => {
    handleArgs(value)
  }, [handleArgs])
  const onDanger = useCallback((value: boolean | 'indeterminate') => {
    page.setDangerAccepted(value === true)
  }, [page])
  const onFavorite = useCallback(() => {
    void page.handleToggleFavorite()
  }, [page])
  const onCancel = useCallback(() => {
    void page.handleCancel()
  }, [page])
  const onExecute = useCallback(() => {
    void page.handleExecute()
  }, [page])
  const selectedFavorite = page.favorites.find((item) => item.command === page.selectedCommand && JSON.stringify(item.args) === JSON.stringify(page.args.split(' ').filter(Boolean)))
  const preview = page.selectedCommandInfo?.usage || `${page.selectedClient} ${page.selectedCommandInfo?.name ?? '<command>'}`

  return (
    <section className="commands-panel commands-composer">
      <div className="commands-panel__header commands-panel__header--wide">
        <div>
          <p className="commands-panel__eyebrow">{page.t('commands.composerEyebrow')}</p>
          <h2 className="commands-panel__title commands-panel__title--large">{page.selectedCommandInfo?.name || page.t('commands.selectCommand')}</h2>
          <p className="commands-panel__subtitle">{page.selectedCommandInfo?.description || page.t('commands.selectCommandHint')}</p>
        </div>
        <ComposerActions page={page} selectedFavorite={Boolean(selectedFavorite)} onFavorite={onFavorite} onCancel={onCancel} onExecute={onExecute} />
      </div>
      <ComposerNotices page={page} />
      <div className="command-strip">
        <div className="command-strip__label">{page.t('commands.previewLabel')}</div>
        <div className="command-strip__body">
          <span className="command-strip__prompt">➜</span>
          <span className="command-strip__binary">{preview}</span>
          {page.args.trim() ? <span className="command-strip__args">{page.args}</span> : null}
        </div>
      </div>
      <ArgsAndDanger page={page} onSelectConfig={onSelectConfig} onArgsInput={onArgsInput} onDanger={onDanger} />
    </section>
  )
})
