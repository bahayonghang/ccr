import { useCallback, useMemo, useState } from 'react'
import type { ProfileEditorAdapter, ProfileWriteOutcome } from '@/configs/profileEditorAdapter'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import { useProfileEditor } from '@/features/platform/profiles/useProfileEditor'
import { useAppT } from '@/i18n'
import { BaseModal, Spinner } from '@/ui'
import { ProfileEditorFields } from './ProfileEditorFields'
import './profile-editor-shell.css'
import './profiles-shared.css'

export interface ProfileEditorModalProps<TForm, TRecord> {
  open: boolean
  adapter: ProfileEditorAdapter<TForm, TRecord>
  presentation: ProfilePresentationView
  target: TRecord | null
  originalName: string | null
  existingNames: readonly string[]
  hasExistingBaseUrl?: boolean
  onClose: () => void
  onApply?: (name: string) => Promise<void>
  onDone?: (outcome: ProfileWriteOutcome, applied: boolean) => void
}

const sectionClassOf = (layout: 'grid' | 'row' | 'group') => {
  if (layout === 'grid') return 'pe-section pe-section--grid'
  if (layout === 'group') return 'pe-section pe-section--group'
  return 'pe-section pe-section--row'
}

/** 统一 Profile 新建/编辑外壳：分区由 adapter.sections 声明。 */
export function ProfileEditorModal<TForm, TRecord>(props: ProfileEditorModalProps<TForm, TRecord>) {
  const {
    open,
    adapter,
    presentation,
    target,
    originalName,
    existingNames,
    hasExistingBaseUrl = false,
    onClose,
    onApply,
    onDone,
  } = props
  const t = useAppT()
  const [advancedOpen, setAdvancedOpen] = useState(false)

  const handleDone = useCallback(
    (outcome: ProfileWriteOutcome, applied: boolean) => {
      if (outcome.status === 'ok') onClose()
      onDone?.(outcome, applied)
    },
    [onClose, onDone],
  )

  const editor = useProfileEditor({
    adapter,
    target,
    originalName,
    existingNames,
    hasExistingBaseUrl,
    onApply,
    onDone: handleDone,
  })

  const coreSections = useMemo(
    () => adapter.sections.filter((section) => !section.advanced),
    [adapter.sections],
  )
  const advancedSections = useMemo(
    () => adapter.sections.filter((section) => section.advanced),
    [adapter.sections],
  )
  const title = editor.isEditing
    ? t('profileEditor.editTitle', { name: originalName ?? '' })
    : t('profileEditor.createTitle', { platform: t(presentation.nameKey) })
  const hint = editor.isEditing
    ? t('profileEditor.overwriteHint', {
        file: presentation.configFile,
        name: originalName ?? '',
      })
    : t('profileEditor.appendHint', { file: presentation.configFile })

  const jumpTo = (sectionId: string) => {
    document.getElementById(`pe-section-${sectionId}`)?.scrollIntoView({ block: 'start' })
  }

  const onSave = () => {
    void editor.submit(false)
  }
  const onSaveApply = () => {
    void editor.submit(true)
  }

  return (
    <BaseModal
      modelValue={open}
      title={title}
      size="3xl"
      surface="solid"
      persistent={editor.saving}
      contentClass="pe-modal"
      onUpdateModelValue={(value) => {
        if (!value && !editor.saving) onClose()
      }}
      footer={
        <div className="pe-footer">
          <p className="pe-footer__hint" data-testid="profile-editor-hint">
            {hint}
          </p>
          <div className="pe-footer__actions">
            <button
              type="button"
              className="cp-btn cp-btn--ghost"
              disabled={editor.saving}
              data-testid="profile-editor-cancel"
              onClick={onClose}
            >
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="cp-btn cp-btn--ghost"
              disabled={editor.saving}
              data-testid="profile-editor-save"
              onClick={onSave}
            >
              {t('profileEditor.save')}
            </button>
            <button
              type="button"
              className="cp-btn cp-btn--primary"
              disabled={editor.saving}
              data-testid="profile-editor-save-apply"
              onClick={onSaveApply}
            >
              {editor.saving ? <Spinner size="sm" /> : null}
              {editor.saving ? t('profileEditor.saving') : t('profileEditor.saveAndApply')}
            </button>
          </div>
        </div>
      }
    >
      <div
        className="pe-shell max-h-[calc(88vh-9rem)] overflow-hidden"
        data-testid="profile-editor-shell"
        data-mode={editor.isEditing ? 'edit' : 'create'}
      >
        {editor.issues.length > 0 ? (
          <div className="pe-summary" data-testid="profile-editor-summary">
            <strong>{t('profileEditor.issuesTitle')}</strong>
            <ul>
              {editor.issues.map((issue) => (
                <li key={`${issue.section}:${issue.field ?? issue.message}`}>
                  <button
                    type="button"
                    className="pe-summary__jump"
                    data-testid={`profile-editor-jump-${issue.section}`}
                    onClick={() => jumpTo(issue.section)}
                  >
                    {issue.message}
                    <span>{t('profileEditor.jump')}</span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ) : null}
        {editor.submitError ? (
          <p className="pe-summary" data-testid="profile-editor-error">
            {editor.submitError}
          </p>
        ) : null}
        <div className="pe-scroll">
          {coreSections.map((section) => (
            <section
              key={section.id}
              id={`pe-section-${section.id}`}
              className={sectionClassOf(section.layout)}
              data-section={section.id}
              data-testid={`profile-editor-section-${section.id}`}
            >
              {section.titleKey ? <h3 className="pe-section__title">{t(section.titleKey)}</h3> : null}
              {section.fields.map((field) => (
                <ProfileEditorFields
                  key={field.key}
                  field={field}
                  form={editor.form}
                  onChange={editor.setField}
                />
              ))}
            </section>
          ))}
          {advancedSections.length > 0 ? (
            <div className="pe-advanced" data-testid="profile-editor-advanced">
              <button
                type="button"
                className="cp-btn cp-btn--ghost"
                aria-expanded={advancedOpen}
                data-testid="profile-editor-advanced-toggle"
                onClick={() => setAdvancedOpen((expanded) => !expanded)}
              >
                {t('profileEditor.advanced')}
              </button>
              {advancedOpen
                ? advancedSections.map((section) => (
                    <section
                      key={section.id}
                      id={`pe-section-${section.id}`}
                      className={sectionClassOf(section.layout)}
                      data-section={section.id}
                      data-testid={`profile-editor-section-${section.id}`}
                    >
                      {section.titleKey ? (
                        <h3 className="pe-section__title">{t(section.titleKey)}</h3>
                      ) : null}
                      {section.fields.map((field) => (
                        <ProfileEditorFields
                          key={field.key}
                          field={field}
                          form={editor.form}
                          onChange={editor.setField}
                        />
                      ))}
                    </section>
                  ))
                : null}
            </div>
          ) : null}
        </div>
      </div>
    </BaseModal>
  )
}
