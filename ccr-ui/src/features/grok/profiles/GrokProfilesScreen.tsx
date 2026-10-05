import { grokProfilePresentation } from '@/configs/profilePresentation'
import { ProfileEditorModal, ProfilesNotice } from '@/features/platform/profiles/shared'
import { ProfilesSurface } from '@/features/platform/profiles/ProfilesSurface'
import { SurfacePage, surfaceStateOf } from '@/features/platform/SurfacePage'
import { GrokSubnav } from '../GrokSubnav'
import { t } from '../locale'
import { grokProfileEditorAdapter } from './grokProfileEditorAdapter'
import { useGrokProfilesPage } from './useGrokProfilesPage'

/** Grok Profiles 装配：subnav + 统一列表 + recovery notice + 编辑器。 */
export function GrokProfilesScreen() {
  const page = useGrokProfilesPage()
  const notice = page.recovery ? (
    <ProfilesNotice
      tone="warning"
      message={page.recovery.message}
      actionLabel={t('grok.profiles.messages.recoverySuccess')}
      onAction={page.runRecovery}
    />
  ) : undefined

  return (
    <>
      <SurfacePage
        title={t('grok.profiles.title')}
        description={t('grok.profiles.subtitle')}
        subnav={<GrokSubnav />}
        state={surfaceStateOf(page)}
        stateTitle={
          page.unavailable ? t('settingsRaw.unsupportedEnvironment') : page.error ?? undefined
        }
        stateDescription={page.unavailable ? page.localOnlyEnvType ?? undefined : undefined}
        onRetry={page.onReload}
      >
        <ProfilesSurface
          platformKey="grok"
          presentation={grokProfilePresentation}
          records={page.records}
          current={page.current}
          environmentLabel={page.environmentLabel}
          environmentOk={page.environmentOk}
          loading={page.loading}
          canOff={page.canOff}
          commandPalette
          notice={notice}
          onAdd={page.onAdd}
          onEdit={page.onEdit}
          onApply={page.onApply}
          onOff={page.onOff}
          onReload={page.onReload}
          onExport={page.onExport}
          onToggle={page.onToggle}
          onDelete={page.onDelete}
        />
      </SurfacePage>
      <ProfileEditorModal
        open={page.editorOpen}
        adapter={grokProfileEditorAdapter}
        presentation={grokProfilePresentation}
        target={page.editorTarget}
        originalName={page.originalName}
        existingNames={page.existingNames}
        hasExistingBaseUrl={Boolean(page.editorTarget?.has_base_url)}
        onClose={page.closeEditor}
        onApply={page.onApply}
        onDone={page.handleEditorDone}
      />
    </>
  )
}
