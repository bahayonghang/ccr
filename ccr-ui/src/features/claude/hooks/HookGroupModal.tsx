import { useCallback, useMemo } from 'react'
import { useFieldArray, useForm, useWatch, type UseFormRegister } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { tt } from '@/features/claude/locale'
import {
  emptyGroupForm,
  emptyHandlerForm,
  hookGroupSchema,
  KNOWN_HANDLER_TYPES,
  KNOWN_HOOK_EVENTS,
  type HookGroupForm,
  type HookHandlerForm,
} from '@/features/claude/hooks/hooksModel'
import { BaseModal, SIcon } from '@/ui'

interface HookGroupModalProps {
  open: boolean
  editing: boolean
  initial: HookGroupForm
  saving: boolean
  onClose: () => void
  onSave: (values: HookGroupForm) => void
}

const inputClass =
  'w-full rounded-lg border border-border-default bg-bg-surface px-4 py-2.5 outline-none focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary'
const labelClass = 'mb-1.5 block text-sm font-semibold text-text-secondary'

function HandlerFields({
  index,
  register,
  type,
  canRemove,
  onRemove,
}: {
  index: number
  register: UseFormRegister<HookGroupForm>
  type: string
  canRemove: boolean
  onRemove: (index: number) => void
}) {
  const handleRemove = useCallback(() => {
    onRemove(index)
  }, [index, onRemove])
  return (
    <div className="space-y-4 rounded-2xl border border-border-default/60 bg-bg-elevated p-4">
      <div className="flex items-center justify-between">
        <h5 className="text-sm font-semibold tracking-wide text-text-muted uppercase">
          {tt(`处理器 ${index + 1}`, `Handler ${index + 1}`)}
        </h5>
        <button
          type="button"
          className="flex min-h-11 min-w-11 items-center justify-center rounded-md text-accent-danger hover:bg-accent-danger/10 disabled:opacity-40"
          disabled={!canRemove}
          onClick={handleRemove}
        >
          <SIcon name="Trash2" size="w-4 h-4" />
        </button>
      </div>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className={labelClass}>{tt('类型', 'Type')}</label>
          <input list="known-handler-types" className={inputClass} placeholder="command" {...register(`handlers.${index}.type`)} />
        </div>
        <div>
          <label className={labelClass}>{tt('超时（秒）', 'Timeout (seconds)')}</label>
          <input className={inputClass} placeholder="30" {...register(`handlers.${index}.timeout`)} />
        </div>
      </div>
      {type === 'command' ? (
        <div className="space-y-4">
          <div>
            <label className={labelClass}>{tt('命令', 'Command')}</label>
            <input className={`${inputClass} font-mono text-sm`} placeholder="./scripts/check-style.sh" {...register(`handlers.${index}.command`)} />
          </div>
          <label className="flex cursor-pointer items-center gap-3">
            <input type="checkbox" className="h-4 w-4 rounded border-border-default" {...register(`handlers.${index}.asyncEnabled`)} />
            <span className="text-sm font-semibold text-text-secondary">{tt('异步运行', 'Run asynchronously')}</span>
          </label>
        </div>
      ) : null}
      {type === 'http' ? (
        <div className="space-y-4">
          <div>
            <label className={labelClass}>{tt('URL', 'URL')}</label>
            <input className={`${inputClass} font-mono text-sm`} placeholder="https://example.com/hooks" {...register(`handlers.${index}.url`)} />
          </div>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className={labelClass}>{tt('请求头 JSON', 'Headers JSON')}</label>
              <textarea rows={4} className={`${inputClass} font-mono text-sm`} {...register(`handlers.${index}.headersJson`)} />
            </div>
            <div>
              <label className={labelClass}>{tt('允许的环境变量', 'Allowed Env Vars')}</label>
              <textarea rows={4} className={`${inputClass} font-mono text-sm`} {...register(`handlers.${index}.allowedEnvVarsText`)} />
            </div>
          </div>
          <label className="flex cursor-pointer items-center gap-3">
            <input type="checkbox" className="h-4 w-4 rounded border-border-default" {...register(`handlers.${index}.asyncEnabled`)} />
            <span className="text-sm font-semibold text-text-secondary">{tt('异步运行', 'Run asynchronously')}</span>
          </label>
        </div>
      ) : null}
      {type !== 'command' && type !== 'http' ? (
        <div className="space-y-4">
          <div>
            <label className={labelClass}>{tt('提示词', 'Prompt')}</label>
            <textarea rows={4} className={`${inputClass} font-mono text-sm`} {...register(`handlers.${index}.prompt`)} />
          </div>
          <div>
            <label className={labelClass}>{tt('模型', 'Model')}</label>
            <input className={inputClass} placeholder="claude-haiku-4-5" {...register(`handlers.${index}.model`)} />
          </div>
        </div>
      ) : null}
      <div>
        <label className={labelClass}>{tt('状态消息', 'Status Message')}</label>
        <input className={inputClass} placeholder="Checking style..." {...register(`handlers.${index}.statusMessage`)} />
      </div>
      <div>
        <label className={labelClass}>{tt('处理器高级 JSON', 'Handler Advanced JSON')}</label>
        <textarea rows={4} className={`${inputClass} font-mono text-sm`} {...register(`handlers.${index}.extraJson`)} />
      </div>
    </div>
  )
}

export function HookGroupModal({ open, editing, initial, saving, onClose, onSave }: HookGroupModalProps) {
  const form = useForm<HookGroupForm>({
    resolver: zodResolver(hookGroupSchema),
    defaultValues: emptyGroupForm(),
    values: initial,
  })
  const { register, control, handleSubmit } = form
  const { fields, append, remove } = useFieldArray({ control, name: 'handlers' })
  const handlers = useWatch({ control, name: 'handlers' }) as HookHandlerForm[] | undefined
  const addHandler = useCallback(() => {
    append(emptyHandlerForm())
  }, [append])
  const onValid = useCallback(
    (values: HookGroupForm) => {
      onSave(values)
    },
    [onSave],
  )
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])
  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) onClose()
    },
    [onClose],
  )

  return (
    <BaseModal
      modelValue={open}
      title={editing ? tt('编辑 Hook 组', 'Edit Hook Group') : tt('添加 Hook 组', 'Add Hook Group')}
      size="4xl"
      surface="solid"
      scrollable
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <div className="flex w-full gap-4">
          <button type="button" className="flex-1 rounded-lg border border-border-default bg-bg-surface px-6 py-3 font-medium text-text-secondary" onClick={onClose}>
            {tt('取消', 'Cancel')}
          </button>
          <button
            type="button"
            className="flex-1 rounded-lg bg-accent-secondary px-6 py-3 font-medium text-[color:var(--color-accent-primary-contrast)] disabled:opacity-60"
            disabled={saving}
            onClick={onSubmit}
          >
            {saving ? tt('保存中...', 'Saving...') : editing ? tt('保存分组', 'Save Group') : tt('添加分组', 'Add Group')}
          </button>
        </div>
      }
    >
      <div className="space-y-6">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <label className={labelClass}>{tt('事件', 'Event')}</label>
            <input list="known-hook-events" className={inputClass} placeholder="PreToolUse" {...register('event')} />
          </div>
          <div>
            <label className={labelClass}>{tt('匹配器', 'Matcher')}</label>
            <input className={inputClass} placeholder="Write|Edit" {...register('matcher')} />
          </div>
        </div>
        <div>
          <label className={labelClass}>{tt('分组高级 JSON', 'Group Advanced JSON')}</label>
          <textarea rows={4} className={`${inputClass} font-mono text-sm`} {...register('groupExtraJson')} />
        </div>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h4 className="text-lg font-semibold text-text-primary">{tt('处理器', 'Handlers')}</h4>
            <button
              type="button"
              className="inline-flex min-h-11 items-center rounded-lg border border-accent-secondary/20 bg-accent-secondary/10 px-3 py-2 text-sm font-medium text-accent-secondary"
              onClick={addHandler}
            >
              <SIcon name="Plus" size="w-4 h-4" className="mr-2" />
              {tt('添加处理器', 'Add Handler')}
            </button>
          </div>
          {fields.map((field, index) => (
            <HandlerFields
              key={field.id}
              index={index}
              register={register}
              type={handlers?.[index]?.type ?? 'command'}
              canRemove={fields.length > 1}
              onRemove={remove}
            />
          ))}
        </div>
      </div>
      <datalist id="known-hook-events">
        {KNOWN_HOOK_EVENTS.map((eventName) => (
          <option key={eventName} value={eventName} />
        ))}
      </datalist>
      <datalist id="known-handler-types">
        {KNOWN_HANDLER_TYPES.map((handlerType) => (
          <option key={handlerType} value={handlerType} />
        ))}
      </datalist>
    </BaseModal>
  )
}
