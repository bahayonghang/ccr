import { useCallback, useMemo, useState, type SyntheticEvent } from 'react'
import { importUnifiedMcpServers } from '@/api'
import {
  McpCreatePanel,
  McpDetailPanel,
  McpImportPanel,
  McpListPanel,
} from '@/features/platform'
import type { McpGroup } from '@/types/mcpManager'
import type { McpScopeFilter } from '@/types/unifiedMcp'
import { getErrorMessage } from '@/utils/errorHandler'
import {
  BulkDeleteDialog,
  MasterDetailLayout,
  PageHeader,
  PageShell,
  PillToggleGroup,
  SIcon,
  type BulkDeleteItem,
} from '@/ui'
import { useMcpT } from './locale'
import { McpPresetsPanel } from './McpPresetsPanel'
import { McpSyncPanel } from './McpSyncPanel'
import { mcpNotify } from './notify'
import { useMcpManager } from './useMcpManager'
import './styles/mcp-manager.css'

const SCOPE_OPTIONS: Array<{ value: McpScopeFilter; labelKey: string }> = [
  { value: 'effective', labelKey: 'mcp.manager.scopes.effective' },
  { value: 'local', labelKey: 'mcp.manager.scopes.local' },
  { value: 'project', labelKey: 'mcp.manager.scopes.project' },
  { value: 'user', labelKey: 'mcp.manager.scopes.user' },
  { value: 'hidden', labelKey: 'mcp.manager.scopes.hidden' },
]

export function McpManagerView() {
  const t = useMcpT()
  const mcp = useMcpManager()
  const [showPresetsDrawer, setShowPresetsDrawer] = useState(false)
  const [showBulkDeleteDialog, setShowBulkDeleteDialog] = useState(false)
  const [bulkDeleting, setBulkDeleting] = useState(false)

  const scopeToggleOptions = useMemo(
    () => SCOPE_OPTIONS.map((scope) => ({ value: scope.value, label: `${t(scope.labelKey)} ${mcp.scopeCounts[scope.value]}` })),
    [mcp.scopeCounts, t],
  )

  const bulkDeleteItems = useMemo<BulkDeleteItem[]>(
    () => mcp.selectedGroups.map((g) => ({ key: g.name, label: g.name, badge: t('mcp.manager.bulkDelete.badge', { count: g.platforms.length }) })),
    [mcp.selectedGroups, t],
  )

  const handleRefresh = useCallback(() => {
    void mcp.refresh()
  }, [mcp])
  const handleInstalled = useCallback(() => {
    void mcp.refresh()
  }, [mcp])
  const handleSubmit = useCallback(async () => {
    if (mcp.formData.scope === 'project') {
      const confirmed = await mcpNotify.confirm({
        title: t('common.warning'),
        message: t('mcp.manager.confirm.projectScopeWrite'),
        confirmText: t('common.confirm'),
        cancelText: t('common.cancel'),
        type: 'warning',
      })
      if (!confirmed) return
    }
    const success = await mcp.submitForm()
    if (success) mcp.closePanel()
  }, [mcp, t])
  const onSubmit = useCallback(() => {
    void handleSubmit()
  }, [handleSubmit])
  const handleBulkDelete = useCallback(() => {
    setShowBulkDeleteDialog(true)
  }, [])
  const closeBulkDelete = useCallback(() => {
    setShowBulkDeleteDialog(false)
  }, [])
  const confirmBulkDelete = useCallback(async () => {
    setBulkDeleting(true)
    try {
      await mcp.bulkDelete()
      setShowBulkDeleteDialog(false)
      mcpNotify.success(t('mcp.manager.messages.deletedSelected'))
    } catch (err) {
      mcpNotify.error(getErrorMessage(err))
    } finally {
      setBulkDeleting(false)
    }
  }, [mcp, t])
  const onConfirmBulk = useCallback(() => {
    void confirmBulkDelete()
  }, [confirmBulkDelete])

  const handleDeleteGroup = useCallback(async (group: McpGroup) => {
    const confirmed = await mcpNotify.confirm({
      title: t('common.delete'),
      message: t('mcp.manager.confirm.deleteGroup', { count: group.items.length, name: group.name }),
      confirmText: t('common.delete'),
      cancelText: t('common.cancel'),
      type: 'danger',
    })
    if (!confirmed) return
    try {
      await mcp.deleteGroup(group)
      mcpNotify.success(t('mcp.manager.messages.deletedServer', { name: group.name }))
    } catch (err) {
      mcpNotify.error(getErrorMessage(err))
    }
  }, [mcp, t])

  const handleImportServers = useCallback(async (
    servers: Array<{ name: string; type: string; command?: string; args?: string[]; url?: string; env?: Record<string, string>; headers?: Record<string, string> }>,
    platform: string,
    scope?: string,
  ) => {
    if (scope === 'project') {
      const confirmed = await mcpNotify.confirm({
        title: t('common.warning'),
        message: t('mcp.manager.confirm.projectScopeImport'),
        confirmText: t('common.confirm'),
        cancelText: t('common.cancel'),
        type: 'warning',
      })
      if (!confirmed) return
    }
    const requests = servers.map((server) => ({
      platform,
      scope,
      name: server.name,
      command: server.command ?? null,
      args: server.args ?? [],
      url: server.url ?? null,
      env: server.env ?? {},
      headers: server.headers ?? {},
      disabled: false,
    }))
    const results = await importUnifiedMcpServers(requests)
    const failed = results.filter((result) => !result.ok)
    if (failed.length > 0) {
      mcpNotify.error(t('mcp.manager.messages.importPartialFailed', {
        success: results.length - failed.length,
        total: results.length,
        failures: failed.map((item) => `${item.name}: ${item.error ?? t('mcp.manager.messages.unknownError')}`).join('; '),
      }))
    } else {
      mcpNotify.success(t('mcp.manager.messages.importSuccess', {
        count: results.length,
        platform: mcp.PLATFORM_META[platform as keyof typeof mcp.PLATFORM_META]?.label ?? platform,
      }))
    }
    mcp.closePanel()
    await mcp.refresh()
  }, [mcp, t])

  const onToggleDrawer = useCallback((event: SyntheticEvent<HTMLDetailsElement>) => {
    setShowPresetsDrawer(event.currentTarget.open)
  }, [])

  const createPanel = mcp.panelMode.type === 'create' || mcp.panelMode.type === 'edit'

  return (
    <PageShell
      className="mcp-manager-view"
      header={
        <PageHeader
          title={t('mcp.manager.hero.title')}
          description={t('mcp.manager.hero.subtitle')}
          actions={
            <>
              <button type="button" className="mcp-action mcp-action--ghost" onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={mcp.loading ? 'animate-spin' : ''} />
                {t('common.refresh')}
              </button>
              <button type="button" className="mcp-action mcp-action--ghost" onClick={mcp.openImport}>
                <SIcon name="Download" size="w-4 h-4" />
                {t('common.import')}
              </button>
              <button type="button" className="mcp-action mcp-action--primary" onClick={mcp.openCreate}>
                <SIcon name="Plus" size="w-4 h-4" />
                {t('mcp.manager.actions.addServer')}
              </button>
            </>
          }
        />
      }
    >
      <details className="mcp-presets-drawer" onToggle={onToggleDrawer}>
        <summary>
          <span>{t('mcp.manager.presetsDrawer.title')}</span>
          <em>{t('mcp.manager.presetsDrawer.subtitle')}</em>
        </summary>
        {showPresetsDrawer ? (
          <div className="mcp-presets-drawer__content">
            <McpPresetsPanel onInstalled={handleInstalled} />
            <McpSyncPanel onSynced={handleInstalled} />
          </div>
        ) : null}
      </details>

      <PillToggleGroup className="mcp-scope-rail" options={scopeToggleOptions} value={mcp.filterScope} onValueChange={mcp.setFilterScope} />

      {mcp.error ? (
        <div className="mcp-alert mcp-alert--error">
          <SIcon name="AlertTriangle" size="w-4 h-4" />
          {mcp.error}
        </div>
      ) : null}

      <MasterDetailLayout
        listWidth="23rem"
        list={
          <McpListPanel
            groups={mcp.filteredGroups}
            searchQuery={mcp.searchQuery}
            selectedKeys={mcp.effectiveSelectedKeys}
            isMultiSelectMode={mcp.isMultiSelectMode}
            loading={mcp.loading}
            t={t}
            onSearchQueryChange={mcp.setSearchQuery}
            onSelect={mcp.selectGroup}
            onCreate={mcp.openCreate}
            onImport={mcp.openImport}
            onRefresh={handleRefresh}
            onToggleMultiSelect={mcp.toggleMultiSelect}
            onBulkDelete={handleBulkDelete}
          />
        }
        detail={
          createPanel ? (
            <McpCreatePanel
              isEditing={mcp.panelMode.type === 'edit'}
              formApi={mcp.formApi}
              formData={mcp.formData}
              isHttpMode={mcp.isHttpMode}
              argInput={mcp.argInput}
              envKey={mcp.envKey}
              envValue={mcp.envValue}
              headerKey={mcp.headerKey}
              headerValue={mcp.headerValue}
              platforms={mcp.ALL_PLATFORMS}
              platformMeta={mcp.PLATFORM_META}
              t={t}
              onSubmit={onSubmit}
              onCancel={mcp.closePanel}
              onIsHttpModeChange={mcp.setIsHttpMode}
              onArgInputChange={mcp.setArgInput}
              onEnvKeyChange={mcp.setEnvKey}
              onEnvValueChange={mcp.setEnvValue}
              onHeaderKeyChange={mcp.setHeaderKey}
              onHeaderValueChange={mcp.setHeaderValue}
              onAddEnv={mcp.addEnvVar}
              onRemoveEnv={mcp.removeEnvVar}
              onAddHeader={mcp.addHeader}
              onRemoveHeader={mcp.removeHeader}
            />
          ) : mcp.panelMode.type === 'import' ? (
            <McpImportPanel platforms={mcp.ALL_PLATFORMS} platformMeta={mcp.PLATFORM_META} t={t} onCancel={mcp.closePanel} onImport={handleImportServers} />
          ) : (
            <McpDetailPanel group={mcp.activeGroup} diagnostics={mcp.sourceDiagnostics} t={t} onEdit={mcp.openEdit} onDelete={handleDeleteGroup} onToggle={mcp.toggleServer} />
          )
        }
      />

      <BulkDeleteDialog
        isOpen={showBulkDeleteDialog}
        items={bulkDeleteItems}
        title={t('mcp.manager.bulkDelete.title')}
        message={t('mcp.manager.bulkDelete.message', { count: bulkDeleteItems.length })}
        overflowMessage={t('mcp.manager.bulkDelete.overflow', { count: Math.max(bulkDeleteItems.length - 10, 0) })}
        cancelLabel={t('common.cancel')}
        confirmLabel={t('mcp.manager.bulkDelete.confirm', { count: bulkDeleteItems.length })}
        loading={bulkDeleting}
        onConfirm={onConfirmBulk}
        onCancel={closeBulkDelete}
      />
    </PageShell>
  )
}
