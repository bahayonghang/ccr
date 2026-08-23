import { useCallback } from 'react'
import { ProfilesHeader } from '@/features/platform/profiles/shared'
import { AsyncStatePanel, EmptyState, PageShell } from '@/ui'
import { GrokSubnav } from '../GrokSubnav'
import { t } from '../locale'
import { ghostBtnClass } from '../ui-classes'
import { GrokProfileCard } from './GrokProfileCard'
import { GrokProfileEditorModal } from './GrokProfileEditorModal'
import { useGrokProfilesPage } from './useGrokProfilesPage'

export function GrokProfilesPage() {
  const page = useGrokProfilesPage()
  const {
    localOnly,
    localOnlyEnvType,
    loading,
    profiles,
    currentProfile,
    activation,
    saving,
    saveError,
    showForm,
    editingName,
    editingProfile,
    recovery,
    form,
    handleAdd,
    handleEdit,
    handleSave,
    handleApply,
    handleOff,
    handleDelete,
    handleToggle,
    handleExport,
    closeForm,
    runRecovery,
    reload,
  } = page

  const onReload = useCallback(() => {
    void reload()
  }, [reload])
  const onSave = useCallback(() => {
    void handleSave()
  }, [handleSave])
  const onOff = useCallback(() => {
    void handleOff()
  }, [handleOff])
  const onRecover = useCallback(() => {
    void runRecovery()
  }, [runRecovery])

  if (localOnly) {
    return (
      <PageShell subnav={<GrokSubnav />}>
        <AsyncStatePanel
          state="runtime-unavailable"
          title={t('settingsRaw.unsupportedEnvironment')}
          description={localOnlyEnvType ?? undefined}
        />
      </PageShell>
    )
  }

  return (
    <PageShell className="grok-profiles-view" subnav={<GrokSubnav />}>
      <ProfilesHeader
        icon="Layers"
        backTo="/grok"
        labels={{
          title: t('grok.profiles.title'),
          subtitle: t('grok.profiles.subtitle'),
          back: t('common.back'),
          reload: t('common.refresh'),
          export: t('common.export'),
          add: t('grok.profiles.actions.add'),
        }}
        loading={loading}
        onAdd={handleAdd}
        onExport={handleExport}
        onReload={onReload}
        onOpenPalette={handleAdd}
        onEditSource={handleAdd}
      />
      {activation !== 'inactive' ? (
        <button type="button" className={`${ghostBtnClass} mb-4`} onClick={onOff}>
          {t('grok.profiles.actions.off')}
        </button>
      ) : null}
      {recovery ? (
        <div className="mb-4 flex items-center justify-between gap-3 rounded-md border border-accent-warning/30 bg-bg-elevated p-3">
          <p className="text-sm text-text-secondary">{recovery.message}</p>
          <button type="button" className={ghostBtnClass} onClick={onRecover}>
            {t('grok.profiles.messages.recoverySuccess')}
          </button>
        </div>
      ) : null}
      {loading ? (
        <AsyncStatePanel state="loading" title={t('common.loading')} />
      ) : profiles.length === 0 ? (
        <EmptyState title={t('grok.profiles.emptyTitle')} />
      ) : (
        <div className="grid gap-3 md:grid-cols-2">
          {profiles.map((profile) => (
            <GrokProfileCard
              key={profile.name}
              profile={profile}
              isCurrent={profile.name === currentProfile}
              onApply={handleApply}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onToggle={handleToggle}
            />
          ))}
        </div>
      )}
      <GrokProfileEditorModal
        open={showForm}
        editingName={editingName}
        saving={saving}
        error={saveError}
        form={form}
        baseUrlDisplay={editingProfile?.base_url_display}
        hasExistingBaseUrl={editingProfile?.has_base_url}
        currentAuthMode={editingProfile?.auth_mode}
        currentEnvKey={editingProfile?.env_key}
        onClose={closeForm}
        onSave={onSave}
      />
    </PageShell>
  )
}
