import { codexProfilePresentation } from '@/configs/profilePresentation'
import { ProfileEditorModal } from '@/features/platform/profiles/shared'
import { ProfilesSurface } from '@/features/platform/profiles/ProfilesSurface'
import { SurfacePage, surfaceStateOf } from '@/features/platform/SurfacePage'
import { useAppT } from '@/i18n'
import { codexProfileEditorAdapter } from './codexProfileEditorAdapter'
import { useCodexProfilesPage } from './useCodexProfilesPage'

/** Codex Profiles 装配：控制器 + 统一列表 + 编辑器。 */
export function CodexProfilesScreen() {
  const t = useAppT()
  const page = useCodexProfilesPage()

  return (
    <>
      <SurfacePage
        title={t('codex.profiles.title')}
        description={t('codex.profiles.subtitle')}
        state={surfaceStateOf(page)}
        stateTitle={page.error ?? undefined}
        onRetry={page.onReload}
      >
        <ProfilesSurface
          platformKey="codex"
          presentation={codexProfilePresentation}
          records={page.records}
          current={page.current}
          environmentLabel={page.environmentLabel}
          environmentOk={page.environmentOk}
          loading={page.loading}
          canOff={page.canOff}
          commandPalette
          onAdd={page.onAdd}
          onEdit={page.onEdit}
          onApply={page.onApply}
          onOff={page.onOff}
          onReload={page.onReload}
          onExport={page.onExport}
          onToggle={page.onToggle}
          onDelete={page.onDelete}
          rawSource={page.rawSource}
        />
      </SurfacePage>
      <ProfileEditorModal
        open={page.editorOpen}
        adapter={codexProfileEditorAdapter}
        presentation={codexProfilePresentation}
        target={page.editorTarget}
        originalName={page.originalName}
        existingNames={page.existingNames}
        onClose={page.closeEditor}
        onApply={page.onApply}
        onDone={page.handleEditorDone}
      />
    </>
  )
}
