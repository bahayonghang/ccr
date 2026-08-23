import { useCallback, useMemo, useState } from 'react'
import type { UseFormRegister, UseFormReturn } from 'react-hook-form'
import type { GrokAuthModeDto, GrokCredentialAction, GrokProfileKindDto } from '@/types'
import {
  GROK_API_BACKEND_OPTIONS,
  GROK_REASONING_EFFORT_OPTIONS,
  type GrokProfileEditorForm,
} from '@/utils/grokProfileEditor'
import { BaseModal, SIcon } from '@/ui'
import { t } from '../locale'
import { fieldInputClass } from '../ui-classes'
import { validateGrokEditor, type GrokEditorSectionId } from './grokEditorValidation'

interface GrokProfileEditorModalProps {
  open: boolean
  editingName: string | null
  saving: boolean
  error: string | null
  form: UseFormReturn<GrokProfileEditorForm>
  baseUrlDisplay?: string | null
  hasExistingBaseUrl?: boolean
  currentAuthMode?: GrokAuthModeDto | null
  currentEnvKey?: string | null
  onClose: () => void
  onSave: () => void
}

const profileKinds: GrokProfileKindDto[] = ['official', 'third_party']
const credentialActions: GrokCredentialAction[] = [
  'preserve',
  'replace_api_key',
  'replace_env_key',
  'clear',
]

export function GrokProfileEditorModal({
  open,
  editingName,
  saving,
  error,
  form,
  baseUrlDisplay = null,
  hasExistingBaseUrl = false,
  currentAuthMode = null,
  currentEnvKey = null,
  onClose,
  onSave,
}: GrokProfileEditorModalProps) {
  const { register, watch, setValue } = form
  const [section, setSection] = useState<GrokEditorSectionId>('identity')
  const [showValidation, setShowValidation] = useState(false)
  const values = watch()
  const isThirdParty = values.profileKind === 'third_party'
  const sections: GrokEditorSectionId[] = isThirdParty
    ? ['identity', 'connection', 'runtime', 'status']
    : ['identity', 'runtime', 'status']
  const issues = useMemo(
    () => validateGrokEditor({ form: values, editingName, hasExistingBaseUrl, t }),
    [editingName, hasExistingBaseUrl, values],
  )

  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next && !saving) onClose()
    },
    [onClose, saving],
  )

  const handleSave = useCallback(() => {
    if (issues.length > 0) {
      setShowValidation(true)
      setSection(issues[0].section)
      return
    }
    setShowValidation(false)
    onSave()
  }, [issues, onSave])

  const setKind = useCallback(
    (kind: GrokProfileKindDto) => {
      setValue('profileKind', kind, { shouldDirty: true })
    },
    [setValue],
  )

  const title = values.name.trim() || editingName || t('grok.profiles.editor.createTitle')
  const placeholder = editingName && baseUrlDisplay ? baseUrlDisplay : 'https://api.example.com/v1'
  const currentAuthLabel = editingName
    ? t(`grok.profiles.authModes.${currentAuthMode ?? 'session'}`)
    : t('grok.states.notSet')

  const renderHeader = useCallback(
    ({ titleId }: { titleId: string }) => (
      <EditorHeader titleId={titleId} title={title} editing={Boolean(editingName)} saving={saving} onClose={onClose} />
    ),
    [editingName, onClose, saving, title],
  )

  return (
    <BaseModal
      modelValue={open}
      persistent={saving}
      showClose={false}
      size="3xl"
      closeOnBackdrop={false}
      contentClass="grok-profile-editor"
      onUpdateModelValue={handleOpenChange}
      header={renderHeader}
    >
      <EditorAlert error={error} issue={showValidation ? issues[0]?.message ?? null : null} />
      <div className="mb-3 flex flex-wrap gap-2" role="tablist">
        {sections.map((id) => (
          <SectionTab key={id} id={id} active={section === id} onSelect={setSection} />
        ))}
      </div>
      <EditorFields
        section={section}
        register={register}
        values={values}
        isThirdParty={isThirdParty}
        kindLocked={Boolean(editingName)}
        placeholder={placeholder}
        currentAuthLabel={currentAuthLabel}
        currentEnvKey={currentEnvKey}
        onKind={setKind}
      />
      <div className="mt-6 flex justify-end gap-2 border-t border-border-subtle pt-4">
        <button type="button" className="rounded-lg border border-border-default px-4 py-2 text-sm" disabled={saving} onClick={onClose}>
          {t('common.cancel')}
        </button>
        <button
          type="button"
          className="inline-flex items-center gap-2 rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)] disabled:opacity-50"
          disabled={saving}
          onClick={handleSave}
        >
          {t('grok.profiles.actions.save')}
        </button>
      </div>
    </BaseModal>
  )
}

function EditorFields({
  section,
  register,
  values,
  isThirdParty,
  kindLocked,
  placeholder,
  currentAuthLabel,
  currentEnvKey,
  onKind,
}: {
  section: GrokEditorSectionId
  register: UseFormRegister<GrokProfileEditorForm>
  values: GrokProfileEditorForm
  isThirdParty: boolean
  kindLocked: boolean
  placeholder: string
  currentAuthLabel: string
  currentEnvKey: string | null
  onKind: (kind: GrokProfileKindDto) => void
}) {
  if (section === 'identity') {
    return (
      <div className="space-y-4">
        <div className="flex gap-1 rounded-md border border-border-default bg-bg-base p-1">
          {profileKinds.map((kind) => (
            <KindButton key={kind} kind={kind} active={values.profileKind === kind} disabled={kindLocked} onSelect={onKind} />
          ))}
        </div>
        <label className="block text-xs font-medium text-text-secondary">
          {t('grok.profiles.fields.name')}
          <input className={`${fieldInputClass} mt-2 font-mono`} autoComplete="off" {...register('name')} />
        </label>
        {isThirdParty ? (
          <label className="block text-xs font-medium text-text-secondary">
            {t('grok.profiles.fields.provider')}
            <input className={`${fieldInputClass} mt-2`} autoComplete="off" {...register('provider')} />
          </label>
        ) : null}
        <label className="block text-xs font-medium text-text-secondary">
          {t('grok.profiles.fields.description')}
          <textarea className={`${fieldInputClass} mt-2 min-h-20`} {...register('description')} />
        </label>
      </div>
    )
  }
  if (section === 'connection') {
    if (!isThirdParty) return null
    return (
      <div className="space-y-4">
        <label className="block text-xs font-medium text-text-secondary">
          {t('grok.profiles.fields.baseUrl')}
          <input className={`${fieldInputClass} mt-2 font-mono`} placeholder={placeholder} autoComplete="off" {...register('baseUrl')} />
        </label>
        <div className="rounded-xl border border-border-subtle bg-bg-elevated p-3 text-sm">
          <span className="text-text-muted">{t('grok.profiles.editor.currentCredential')}</span>
          <strong className="ml-2">{currentAuthLabel}</strong>
          {currentEnvKey ? <code className="ml-2 text-xs">{currentEnvKey}</code> : null}
        </div>
        <label className="block text-xs font-medium text-text-secondary">
          {t('grok.profiles.fields.credentialAction')}
          <select className={`${fieldInputClass} mt-2`} {...register('credentialAction')}>
            {credentialActions.map((action) => (
              <option key={action} value={action}>{t(`grok.profiles.credentialActions.${action}`)}</option>
            ))}
          </select>
        </label>
        {values.credentialAction === 'replace_api_key' ? (
          <label className="block text-xs font-medium text-text-secondary">
            {t('grok.profiles.fields.apiKey')}
            <input type="password" className={`${fieldInputClass} mt-2 font-mono`} autoComplete="new-password" {...register('apiKey')} />
          </label>
        ) : null}
        {values.credentialAction === 'replace_env_key' ? (
          <label className="block text-xs font-medium text-text-secondary">
            {t('grok.profiles.fields.envKey')}
            <input className={`${fieldInputClass} mt-2 font-mono`} placeholder="GROK_API_KEY" autoComplete="off" {...register('envKey')} />
          </label>
        ) : null}
      </div>
    )
  }
  if (section === 'runtime') {
    return (
      <div className="grid gap-4 md:grid-cols-2">
        <label className="block text-xs font-medium text-text-secondary">
          {t('grok.profiles.fields.model')}
          <input className={`${fieldInputClass} mt-2 font-mono`} autoComplete="off" {...register('model')} />
        </label>
        <label className="block text-xs font-medium text-text-secondary">
          {t('grok.profiles.fields.reasoningEffort')}
          <select className={`${fieldInputClass} mt-2`} {...register('reasoningEffort')}>
            <option value="">{t('grok.profiles.editor.notSet')}</option>
            {GROK_REASONING_EFFORT_OPTIONS.map((effort) => (
              <option key={effort} value={effort}>{effort}</option>
            ))}
          </select>
        </label>
        {isThirdParty ? (
          <>
            <label className="block text-xs font-medium text-text-secondary">
              {t('grok.profiles.fields.apiBackend')}
              <select className={`${fieldInputClass} mt-2`} {...register('apiBackend')}>
                <option value="">{t('grok.profiles.editor.notSet')}</option>
                {GROK_API_BACKEND_OPTIONS.map((backend) => (
                  <option key={backend} value={backend}>{backend}</option>
                ))}
              </select>
            </label>
            <label className="block text-xs font-medium text-text-secondary">
              {t('grok.profiles.fields.contextWindow')}
              <input type="number" min={1} step={1} className={`${fieldInputClass} mt-2 font-mono`} {...register('contextWindow')} />
            </label>
            <label className="flex items-center gap-2 text-sm md:col-span-2">
              <input type="checkbox" {...register('supportsBackendSearch')} />
              {t('grok.profiles.fields.supportsBackendSearch')}
            </label>
          </>
        ) : null}
      </div>
    )
  }
  return (
    <div className="space-y-4">
      <label className="block text-xs font-medium text-text-secondary">
        {t('grok.profiles.fields.tags')}
        <input className={`${fieldInputClass} mt-2`} placeholder={t('grok.profiles.editor.tagsPlaceholder')} {...register('tagsInput')} />
      </label>
      <label className="flex items-center gap-2 text-sm">
        <input type="checkbox" {...register('enabled')} />
        {t('grok.profiles.fields.enabled')}
      </label>
    </div>
  )
}

function EditorAlert({ error, issue }: { error: string | null; issue: string | null }) {
  const message = error || issue
  if (!message) return null
  return (
    <p className="mb-3 rounded-md border border-accent-danger/30 bg-accent-danger/10 px-3 py-2 text-sm text-accent-danger" role="alert">
      {message}
    </p>
  )
}

function EditorHeader({
  titleId,
  title,
  editing,
  saving,
  onClose,
}: {
  titleId: string
  title: string
  editing: boolean
  saving: boolean
  onClose: () => void
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <p className="text-xs font-semibold uppercase tracking-wide text-text-muted">
          {editing ? t('grok.profiles.editor.editTitle') : t('grok.profiles.editor.createTitle')}
        </p>
        <h2 id={titleId} className="mt-1 text-xl font-semibold text-text-primary">{title}</h2>
      </div>
      <button type="button" className="rounded-md p-2 text-text-muted" disabled={saving} onClick={onClose} aria-label={t('common.close')}>
        <SIcon name="X" size="w-4 h-4" />
      </button>
    </div>
  )
}

function SectionTab({
  id,
  active,
  onSelect,
}: {
  id: GrokEditorSectionId
  active: boolean
  onSelect: (id: GrokEditorSectionId) => void
}) {
  const handleClick = useCallback(() => onSelect(id), [id, onSelect])
  return (
    <button
      type="button"
      role="tab"
      className={active ? 'rounded-xl border border-accent-primary/40 bg-accent-primary/10 px-3 py-2 text-sm' : 'rounded-xl border border-border-default px-3 py-2 text-sm'}
      onClick={handleClick}
    >
      {t(`grok.profiles.editor.${id}`)}
    </button>
  )
}

function KindButton({
  kind,
  active,
  disabled,
  onSelect,
}: {
  kind: GrokProfileKindDto
  active: boolean
  disabled: boolean
  onSelect: (kind: GrokProfileKindDto) => void
}) {
  const handleClick = useCallback(() => onSelect(kind), [kind, onSelect])
  return (
    <button
      type="button"
      className={active ? 'rounded-md bg-accent-primary px-3 py-1.5 text-sm text-[color:var(--color-accent-primary-contrast)]' : 'rounded-md px-3 py-1.5 text-sm'}
      disabled={disabled}
      onClick={handleClick}
    >
      {t(`grok.profiles.profileKinds.${kind}`)}
    </button>
  )
}
