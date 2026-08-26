import { useCallback, useEffect, useMemo, useState } from 'react'
import type {
  ProfileEditorAdapter,
  ProfileEditorIssue,
  ProfileWriteOutcome,
} from '@/configs/profileEditorAdapter'

export interface UseProfileEditorArgs<TForm, TRecord> {
  adapter: ProfileEditorAdapter<TForm, TRecord>
  target: TRecord | null
  originalName: string | null
  existingNames: readonly string[]
  hasExistingBaseUrl: boolean
  onApply?: (name: string) => Promise<void>
  onDone: (outcome: ProfileWriteOutcome, applied: boolean) => void
}

const nameOf = (form: unknown): string => {
  if (!form || typeof form !== 'object') return ''
  const value = (form as { name?: unknown }).name
  return typeof value === 'string' ? value.trim() : ''
}

const applyWhenOk = async (input: {
  outcome: ProfileWriteOutcome
  apply: boolean
  form: unknown
  onApply?: (name: string) => Promise<void>
}): Promise<boolean> => {
  if (input.outcome.status !== 'ok' || !input.apply) return false
  const appliedName = input.outcome.appliedName ?? nameOf(input.form)
  if (input.onApply && appliedName) await input.onApply(appliedName)
  return true
}

const errorMessageOf = (outcome: ProfileWriteOutcome): string | null => {
  if (outcome.status === 'ok') return null
  return outcome.message
}

const cloneForm = <TForm>(form: TForm): TForm => {
  if (Array.isArray(form)) return [...form] as TForm
  if (form && typeof form === 'object') return { ...(form as object) } as TForm
  return form
}

/** 表单状态、校验与提交编排；外壳不解析后端 status 字符串。 */
export function useProfileEditor<TForm, TRecord>(args: UseProfileEditorArgs<TForm, TRecord>) {
  const {
    adapter,
    target,
    originalName,
    existingNames,
    hasExistingBaseUrl,
    onApply,
    onDone,
  } = args
  const isEditing = originalName != null
  const [form, setForm] = useState<TForm>(() =>
    target != null ? adapter.fromRecord(target) : adapter.createEmpty(),
  )
  const [issues, setIssues] = useState<readonly ProfileEditorIssue[]>([])
  const [saving, setSaving] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [dirtyFields, setDirtyFields] = useState<Set<string>>(() => new Set())

  useEffect(() => {
    setForm(target != null ? adapter.fromRecord(target) : adapter.createEmpty())
    setIssues([])
    setSubmitError(null)
    setDirtyFields(new Set())
  }, [adapter, originalName, target])

  const validateCtx = useMemo(
    () => ({
      isEditing,
      originalName,
      existingNames,
      hasExistingBaseUrl,
    }),
    [existingNames, hasExistingBaseUrl, isEditing, originalName],
  )

  const setField = useCallback((key: string, value: unknown) => {
    setForm((current) => {
      if (!current || typeof current !== 'object') return current
      return { ...(current as object), [key]: value } as TForm
    })
    setDirtyFields((current) => {
      const next = new Set(current)
      next.add(key)
      return next
    })
  }, [])

  const submit = useCallback(
    async (apply: boolean) => {
      if (saving) return
      const nextIssues = adapter.validate(form, validateCtx)
      if (nextIssues.length > 0) {
        setIssues(nextIssues)
        return
      }
      setIssues([])
      setSubmitError(null)
      setSaving(true)
      try {
        const outcome = await adapter.submit(cloneForm(form), {
          isEditing,
          originalName,
          apply,
          dirtyFields,
        })
        const applied = await applyWhenOk({ outcome, apply, form, onApply })
        const errorMessage = errorMessageOf(outcome)
        if (errorMessage) setSubmitError(errorMessage)
        onDone(outcome, applied)
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        setSubmitError(message)
        onDone({ status: 'error', message }, false)
      } finally {
        setSaving(false)
      }
    },
    [
      adapter,
      dirtyFields,
      form,
      isEditing,
      onApply,
      onDone,
      originalName,
      saving,
      validateCtx,
    ],
  )

  return {
    form,
    issues,
    saving,
    submitError,
    setField,
    submit,
    isEditing,
  }
}
