import { PageHeader, PageShell, SIcon } from '@/ui'
import { CommandsComposer } from './CommandsComposer'
import { CommandsLedger } from './CommandsLedger'
import { CommandsPalette } from './CommandsPalette'
import { useCommandsPage } from './useCommandsPage'
import './styles/commands-view.css'

export function CommandsView() {
  const page = useCommandsPage()
  const canRun = page.canRun
  const readinessLabel = page.runtimeUnavailable
    ? page.t('commands.runtimeWeb')
    : page.selectedClient !== 'ccr'
      ? page.t('commands.runtimeClientPreview')
      : page.isRunning
        ? page.t('commands.runtimeRunning')
        : page.t('commands.runtimeReady')
  const selectedClientLabel = page.CLI_CLIENTS.find((item) => item.id === page.selectedClient)?.name ?? page.selectedClient
  const executableCommandCount = page.commands.filter((command) => command.executable).length

  return (
    <PageShell
      className="commands-page"
      header={
        <PageHeader
          title={page.t('commands.title')}
          description={page.t('commands.description')}
          status={
            <>
              <span className={`commands-chip ${canRun ? 'commands-chip--success' : 'commands-chip--warning'}`}>
                <SIcon name={canRun ? 'CheckCircle2' : 'AlertTriangle'} size="w-3.5 h-3.5" />
                {readinessLabel}
              </span>
              <span className="commands-chip">
                <SIcon name="Cpu" size="w-3.5 h-3.5" />
                {selectedClientLabel}
              </span>
              <span className="commands-chip">
                <SIcon name="ShieldCheck" size="w-3.5 h-3.5" />
                {page.t('commands.whitelistBadge', { count: executableCommandCount })}
              </span>
              <span className="commands-chip">
                <SIcon name="Activity" size="w-3.5 h-3.5" />
                {page.currentSnapshot ? page.t(`commands.status.${page.currentSnapshot.status}`) : page.t('commands.cardJobIdle')}
              </span>
            </>
          }
        />
      }
    >
      <div className="commands-workbench">
        <CommandsPalette page={page} />
        <section className="commands-workbench__main">
          <CommandsComposer page={page} />
          <CommandsLedger page={page} />
        </section>
      </div>
    </PageShell>
  )
}
