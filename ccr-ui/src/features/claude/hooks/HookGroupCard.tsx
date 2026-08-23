import { memo, useCallback } from 'react'
import { tt } from '@/features/claude/locale'
import {
  getEventColor,
  getHandlerSummary,
  groupExtraKeys,
  handlerExtraKeys,
} from '@/features/claude/hooks/hooksModel'
import type { Hook, HookMatcherGroup } from '@/types'
import { SIcon } from '@/ui'

interface HookGroupCardProps {
  eventName: string
  groups: HookMatcherGroup[]
  onAdd: (eventName: string) => void
  onEdit: (eventName: string, groupIndex: number) => void
  onDeleteGroup: (eventName: string, groupIndex: number) => void
  onDeleteHandler: (eventName: string, groupIndex: number, handlerIndex: number) => void
}

const iconBtn = 'flex min-h-11 min-w-11 items-center justify-center rounded-md'
const HandlerRow = memo(function HandlerRow({
  handler,
  onDelete,
  canDelete,
}: {
  handler: Hook
  onDelete: () => void
  canDelete: boolean
}) {
  return (
    <div className="rounded-xl border border-border-default/50 bg-bg-base p-3">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-md border border-accent-secondary/20 bg-accent-secondary/10 px-2 py-1 text-xs font-semibold tracking-wide text-accent-secondary uppercase">
              {handler.type}
            </span>
            {handler.model ? (
              <span className="rounded-md border border-border-default px-2 py-1 text-xs text-text-secondary">
                {tt('模型：', 'model:')} {handler.model}
              </span>
            ) : null}
            {typeof handler.timeout === 'number' ? (
              <span className="rounded-md border border-border-default px-2 py-1 text-xs text-text-secondary">
                {tt('超时：', 'timeout:')} {handler.timeout}s
              </span>
            ) : null}
            {handler.async === true ? (
              <span className="rounded-md border border-border-default px-2 py-1 text-xs text-text-secondary">
                {tt('异步', 'async')}
              </span>
            ) : null}
          </div>
          <code className="block break-all font-mono text-xs text-text-primary">{getHandlerSummary(handler)}</code>
          {handlerExtraKeys(handler).length > 0 ? (
            <p className="text-xs text-text-muted">
              {tt('高级处理器字段：', 'Advanced handler fields:')} {handlerExtraKeys(handler).join(', ')}
            </p>
          ) : null}
        </div>
        {canDelete ? (
          <button type="button" className={`${iconBtn} text-accent-danger hover:bg-accent-danger/10`} onClick={onDelete}>
            <SIcon name="Trash2" size="w-4 h-4" />
          </button>
        ) : null}
      </div>
    </div>
  )
})

const MatcherGroup = memo(function MatcherGroup({
  eventName,
  group,
  groupIndex,
  onEdit,
  onDeleteGroup,
  onDeleteHandler,
}: {
  eventName: string
  group: HookMatcherGroup
  groupIndex: number
  onEdit: HookGroupCardProps['onEdit']
  onDeleteGroup: HookGroupCardProps['onDeleteGroup']
  onDeleteHandler: HookGroupCardProps['onDeleteHandler']
}) {
  const handleEdit = useCallback(() => {
    onEdit(eventName, groupIndex)
  }, [eventName, groupIndex, onEdit])
  const handleDeleteGroup = useCallback(() => {
    onDeleteGroup(eventName, groupIndex)
  }, [eventName, groupIndex, onDeleteGroup])
  return (
    <div className="rounded-2xl border border-border-default/60 bg-bg-elevated p-4">
      <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-md bg-bg-elevated px-2 py-1 text-xs font-semibold tracking-wide text-text-muted uppercase">
              {tt('匹配器', 'Matcher')}
            </span>
            <code className="break-all rounded-md border border-border-default bg-bg-elevated px-2 py-1 font-mono text-xs text-text-primary">
              {group.matcher || tt('全部匹配', 'All matches')}
            </code>
          </div>
          {groupExtraKeys(group).length > 0 ? (
            <p className="text-xs text-text-muted">
              {tt('高级分组字段：', 'Advanced group fields:')} {groupExtraKeys(group).join(', ')}
            </p>
          ) : null}
        </div>
        <div className="flex items-center gap-2">
          <button type="button" className={`${iconBtn} text-accent-secondary hover:bg-accent-secondary/10`} onClick={handleEdit}>
            <SIcon name="Edit2" size="w-4 h-4" />
          </button>
          <button type="button" className={`${iconBtn} text-accent-danger hover:bg-accent-danger/10`} onClick={handleDeleteGroup}>
            <SIcon name="Trash2" size="w-4 h-4" />
          </button>
        </div>
      </div>
      <div className="space-y-3">
        {group.hooks.map((handler, handlerIndex) => (
          <HandlerDeleteWrap
            key={`${getHandlerSummary(handler)}-${handler.type}-${handler.model ?? ''}-${handler.timeout ?? 'none'}-${handler.command ?? handler.url ?? handler.prompt ?? ''}`}
            eventName={eventName}
            groupIndex={groupIndex}
            handler={handler}
            handlerIndex={handlerIndex}
            onDeleteHandler={onDeleteHandler}
          />
        ))}
      </div>
    </div>
  )
})

const HandlerDeleteWrap = memo(function HandlerDeleteWrap({
  eventName,
  groupIndex,
  handler,
  handlerIndex,
  onDeleteHandler,
}: {
  eventName: string
  groupIndex: number
  handler: Hook
  handlerIndex: number
  onDeleteHandler: HookGroupCardProps['onDeleteHandler']
}) {
  const handleDelete = useCallback(() => {
    onDeleteHandler(eventName, groupIndex, handlerIndex)
  }, [eventName, groupIndex, handlerIndex, onDeleteHandler])
  return <HandlerRow handler={handler} onDelete={handleDelete} canDelete />
})

export function HookGroupCard({ eventName, groups, onAdd, onEdit, onDeleteGroup, onDeleteHandler }: HookGroupCardProps) {
  const handleAdd = useCallback(() => {
    onAdd(eventName)
  }, [eventName, onAdd])
  const handlerCount = groups.reduce((sum, group) => sum + group.hooks.length, 0)
  return (
    <article className="space-y-5 rounded-2xl border border-border-subtle bg-bg-surface p-5">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-lg font-bold text-text-primary">{eventName}</h3>
            <span className={`rounded-md border px-2 py-0.5 text-xs font-medium ${getEventColor(eventName)}`}>
              {tt(`${groups.length} 组`, `${groups.length} group${groups.length === 1 ? '' : 's'}`)}
            </span>
          </div>
          <p className="text-xs text-text-muted">
            {tt(`${handlerCount} 个处理器`, `${handlerCount} handler${handlerCount === 1 ? '' : 's'}`)}
          </p>
        </div>
        <button
          type="button"
          className="inline-flex min-h-11 items-center justify-center rounded-lg border border-accent-secondary/20 bg-accent-secondary/10 px-3 py-2 text-sm font-medium text-accent-secondary"
          onClick={handleAdd}
        >
          <SIcon name="Plus" size="w-4 h-4" className="mr-2" />
          {tt('添加分组', 'Add Group')}
        </button>
      </div>
      <div className="space-y-4">
        {groups.map((group, groupIndex) => (
          <MatcherGroup
            key={`${eventName}-${JSON.stringify(group)}`}
            eventName={eventName}
            group={group}
            groupIndex={groupIndex}
            onEdit={onEdit}
            onDeleteGroup={onDeleteGroup}
            onDeleteHandler={onDeleteHandler}
          />
        ))}
      </div>
    </article>
  )
}
