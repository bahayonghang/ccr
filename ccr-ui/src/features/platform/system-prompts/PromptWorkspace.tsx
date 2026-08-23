import type { UseFormRegister } from 'react-hook-form'
import type { SystemPromptFile, SystemPromptRule } from '@/api/domains/systemPrompts'
import { SIcon } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'
import { PromptFileRow } from './PromptFileRow'

interface PromptForm {
  content: string
}

interface PromptWorkspaceProps {
  loading: boolean
  files: SystemPromptFile[]
  rules: SystemPromptRule[]
  selected: SystemPromptFile | null
  busy: boolean
  creatingId: string | null
  dirty: boolean
  saving: boolean
  conflict: boolean
  sizeWarning: boolean
  showLimitHint: boolean
  showRules: boolean
  t: TranslateFunction
  formatTime: (timestamp: number) => string
  register: UseFormRegister<PromptForm>
  onSelect: (file: SystemPromptFile) => void
  onCreate: (file: SystemPromptFile) => void
  onReload: () => void
  onSave: () => void
}

export function PromptWorkspace({
  loading,
  files,
  rules,
  selected,
  busy,
  creatingId,
  dirty,
  saving,
  conflict,
  sizeWarning,
  showLimitHint,
  showRules,
  t,
  formatTime,
  register,
  onSelect,
  onCreate,
  onReload,
  onSave,
}: PromptWorkspaceProps) {
  if (loading) {
    return <p className="grid min-h-80 place-items-center text-sm text-text-muted">{t('systemPrompts.loading')}</p>
  }
  return (
    <div className="grid min-h-96 gap-4 border-t border-border-subtle pt-4 xl:grid-cols-[minmax(18rem,24rem)_minmax(0,1fr)]">
      <aside className="min-w-0 xl:border-r xl:border-border-subtle xl:pr-4">
        <div className="mb-3 flex items-end justify-between gap-4">
          <div>
            <h2 className="m-0 text-sm font-semibold text-text-primary">{t('systemPrompts.filesTitle')}</h2>
            <p className="m-0 text-xs text-text-muted">{t('systemPrompts.filesDescription')}</p>
          </div>
          <span className="tabular-nums text-text-muted">{files.length}</span>
        </div>
        {files.map((file) => (
          <PromptFileRow
            key={file.id}
            file={file}
            active={selected?.id === file.id}
            busy={busy}
            creating={creatingId === file.id}
            t={t}
            formatTime={formatTime}
            onSelect={onSelect}
            onCreate={onCreate}
          />
        ))}
        {showRules ? <RulesList rules={rules} t={t} /> : null}
      </aside>
      <PromptEditor
        selected={selected}
        dirty={dirty}
        busy={busy}
        saving={saving}
        conflict={conflict}
        sizeWarning={sizeWarning}
        showLimitHint={showLimitHint}
        t={t}
        register={register}
        onReload={onReload}
        onSave={onSave}
      />
    </div>
  )
}

function RulesList({ rules, t }: { rules: SystemPromptRule[]; t: TranslateFunction }) {
  return (
    <section className="mt-5">
      <h3 className="text-sm font-semibold text-text-primary">{t('systemPrompts.rulesTitle')}</h3>
      {rules.length === 0 ? (
        <p className="text-xs text-text-muted">{t('systemPrompts.rulesEmpty')}</p>
      ) : (
        rules.map((rule) => (
          <div key={rule.path} className="flex items-center gap-2 border-b border-border-subtle py-2 text-xs">
            <SIcon name="FileText" size="w-4 h-4" />
            <span className="min-w-0">
              <strong className="block">{rule.name}</strong>
              <code className="block truncate text-text-muted">{rule.path}</code>
            </span>
          </div>
        ))
      )}
    </section>
  )
}

function PromptEditor({
  selected,
  dirty,
  busy,
  saving,
  conflict,
  sizeWarning,
  showLimitHint,
  t,
  register,
  onReload,
  onSave,
}: {
  selected: SystemPromptFile | null
  dirty: boolean
  busy: boolean
  saving: boolean
  conflict: boolean
  sizeWarning: boolean
  showLimitHint: boolean
  t: TranslateFunction
  register: UseFormRegister<PromptForm>
  onReload: () => void
  onSave: () => void
}) {
  if (!selected) {
    return (
      <section className="min-w-0">
        <div className="grid min-h-80 place-items-center gap-2 text-text-muted">
          <SIcon name="FileText" size="w-8 h-8" />
          {t('systemPrompts.emptySelection')}
        </div>
      </section>
    )
  }
  return (
    <section className="min-w-0 space-y-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <strong>{t(selected.labelKey)}</strong>
          <code className="ml-2 font-mono text-xs text-text-muted">{selected.path}</code>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {dirty ? <span className="text-xs text-accent-warning">{t('systemPrompts.unsaved')}</span> : null}
          <button
            type="button"
            className="inline-flex min-h-9 items-center gap-1 rounded-md border border-border-default bg-bg-elevated px-3 text-sm text-text-secondary disabled:opacity-50"
            disabled={busy}
            onClick={onReload}
          >
            <SIcon name="RefreshCw" size="w-4 h-4" />
            {t('systemPrompts.reload')}
          </button>
          <button
            type="button"
            className="inline-flex min-h-9 items-center gap-1 rounded-md bg-accent-primary px-3 text-sm text-[color:var(--color-accent-primary-contrast)] disabled:opacity-50"
            disabled={busy || !dirty}
            onClick={onSave}
          >
            <SIcon name="Save" size="w-4 h-4" />
            {saving ? t('systemPrompts.saving') : t('systemPrompts.save')}
          </button>
        </div>
      </div>
      {conflict ? (
        <div className="flex items-center justify-between gap-3 rounded-md border border-accent-warning/30 bg-bg-elevated px-3 py-2 text-sm" role="alert">
          <div>
            <strong>{t('systemPrompts.conflictTitle')}</strong>
            <p className="m-0">{t('systemPrompts.conflictMessage')}</p>
          </div>
          <button type="button" className="rounded-md border border-border-default px-3 py-1" onClick={onReload}>
            {t('systemPrompts.reload')}
          </button>
        </div>
      ) : null}
      {sizeWarning ? (
        <p className="rounded-md border border-accent-warning/30 bg-bg-elevated px-3 py-2 text-sm" role="status">
          {t('systemPrompts.sizeWarning')}
        </p>
      ) : null}
      {showLimitHint && selected.limitHint ? (
        <p className="rounded-md border border-border-subtle bg-bg-elevated px-3 py-2 text-sm" role="note">
          {t('systemPrompts.codexLimit')}
        </p>
      ) : null}
      <textarea rows={24} className="min-h-80 w-full rounded-xl border border-border-default bg-bg-base p-3 font-mono text-sm" {...register('content')} />
    </section>
  )
}
