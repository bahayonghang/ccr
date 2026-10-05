import { useCallback, useEffect, useMemo, useState } from 'react'
import { listHooks, updateHooks } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { HookGroupCard } from '@/features/claude/hooks/HookGroupCard'
import { HookGroupModal } from '@/features/claude/hooks/HookGroupModal'
import {
  ALL_EVENT_KEY,
  applyEditedGroup,
  buildGroupFromForm,
  cloneHookMap,
  emptyGroupForm,
  groupToForm,
  type HookGroupForm,
} from '@/features/claude/hooks/hooksModel'
import { tt } from '@/features/claude/locale'
import type { HookMap } from '@/types'
import { PageHeader, PageShell, SIcon, Spinner } from '@/ui'
import { logger } from '@/utils/logger'

interface EditingTarget {
  event: string
  groupIndex: number
}

const EventTab = ({
  eventName,
  count,
  active,
  onSelect,
}: {
  eventName: string
  count: number
  active: boolean
  onSelect: (eventName: string) => void
}) => {
  const handleClick = useCallback(() => {
    onSelect(eventName)
  }, [eventName, onSelect])
  const className = active
    ? 'min-h-11 whitespace-nowrap rounded-lg bg-accent-secondary px-4 py-2 text-sm font-medium text-[color:var(--color-accent-primary-contrast)] shadow-md'
    : 'min-h-11 whitespace-nowrap rounded-lg border border-border-default bg-bg-elevated px-4 py-2 text-sm font-medium text-text-secondary hover:bg-bg-surface'
  return (
    <button type="button" className={className} onClick={handleClick}>
      {eventName === ALL_EVENT_KEY ? tt('全部', 'All') : eventName}
      <span className="ml-2 opacity-70">({count})</span>
    </button>
  )
}

export function HooksView() {
  const [hooksConfig, setHooksConfig] = useState<HookMap>({})
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [showModal, setShowModal] = useState(false)
  const [selectedEvent, setSelectedEvent] = useState(ALL_EVENT_KEY)
  const [editingTarget, setEditingTarget] = useState<EditingTarget | null>(null)
  const [formInitial, setFormInitial] = useState(emptyGroupForm())

  const loadHooks = useCallback(async () => {
    setLoading(true)
    try {
      const next = await listHooks()
      setHooksConfig(next)
      setSelectedEvent((current) => (current !== ALL_EVENT_KEY && !next[current] ? ALL_EVENT_KEY : current))
    } catch (error) {
      logger.error('Failed to load hooks:', error)
      surfaceNotify.error(error instanceof Error ? error.message : tt('加载 Hooks 失败', 'Failed to load hooks'))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadHooks()
  }, [loadHooks])

  const persistHooks = useCallback(async (nextHooks: HookMap, successMessage: string) => {
    setSaving(true)
    try {
      const saved = await updateHooks(nextHooks)
      setHooksConfig(saved)
      surfaceNotify.success(successMessage)
      setSelectedEvent((current) => (current !== ALL_EVENT_KEY && !saved[current] ? ALL_EVENT_KEY : current))
    } catch (error) {
      logger.error('Failed to save hooks:', error)
      surfaceNotify.error(error instanceof Error ? error.message : tt('保存 Hooks 失败', 'Failed to save hooks'))
      throw error
    } finally {
      setSaving(false)
    }
  }, [])

  const openCreate = useCallback((eventName = '') => {
    setEditingTarget(null)
    setFormInitial(emptyGroupForm(eventName))
    setShowModal(true)
  }, [])
  const openCreateBlank = useCallback(() => {
    openCreate()
  }, [openCreate])
  const closeModal = useCallback(() => setShowModal(false), [])
  const openEdit = useCallback(
    (eventName: string, groupIndex: number) => {
      const group = hooksConfig[eventName]?.[groupIndex]
      if (!group) return
      setEditingTarget({ event: eventName, groupIndex })
      setFormInitial(groupToForm(eventName, group))
      setShowModal(true)
    },
    [hooksConfig],
  )

  const saveGroup = useCallback(
    async (values: HookGroupForm) => {
      try {
        const { event, group } = buildGroupFromForm(values)
        const nextHooks = applyEditedGroup({
          source: hooksConfig,
          editing: editingTarget,
          event,
          group,
        })
        await persistHooks(
          nextHooks,
          editingTarget ? tt('Hook 组更新成功', 'Hook group updated successfully') : tt('Hook 组添加成功', 'Hook group added successfully'),
        )
        setShowModal(false)
      } catch (error) {
        surfaceNotify.error(error instanceof Error ? error.message : tt('保存 Hook 组失败', 'Failed to save hook group'))
      }
    },
    [editingTarget, hooksConfig, persistHooks],
  )

  const handleDeleteGroup = useCallback(
    async (eventName: string, groupIndex: number) => {
      const confirmed = await surfaceNotify.confirm({
        title: tt('删除 Hook 组', 'Delete hook group'),
        message: tt(`确认删除 "${eventName}" 下的第 ${groupIndex + 1} 个匹配组吗？`, `Delete matcher group ${groupIndex + 1} from "${eventName}"?`),
        confirmText: tt('删除', 'Delete'),
        cancelText: tt('取消', 'Cancel'),
        type: 'danger',
      })
      if (!confirmed) return
      const nextHooks = cloneHookMap(hooksConfig)
      nextHooks[eventName]?.splice(groupIndex, 1)
      if ((nextHooks[eventName] ?? []).length === 0) delete nextHooks[eventName]
      await persistHooks(nextHooks, tt('Hook 组删除成功', 'Hook group deleted successfully'))
    },
    [hooksConfig, persistHooks],
  )

  const handleDeleteHandler = useCallback(
    async (eventName: string, groupIndex: number, handlerIndex: number) => {
      const confirmed = await surfaceNotify.confirm({
        title: tt('删除处理器', 'Delete handler'),
        message: tt(`确认删除 "${eventName}" 下的第 ${handlerIndex + 1} 个处理器吗？`, `Delete handler ${handlerIndex + 1} from "${eventName}"?`),
        confirmText: tt('删除', 'Delete'),
        cancelText: tt('取消', 'Cancel'),
        type: 'danger',
      })
      if (!confirmed) return
      const nextHooks = cloneHookMap(hooksConfig)
      const group = nextHooks[eventName]?.[groupIndex]
      if (!group) return
      group.hooks.splice(handlerIndex, 1)
      if (group.hooks.length === 0) nextHooks[eventName]?.splice(groupIndex, 1)
      if ((nextHooks[eventName] ?? []).length === 0) delete nextHooks[eventName]
      await persistHooks(nextHooks, tt('处理器删除成功', 'Handler deleted successfully'))
    },
    [hooksConfig, persistHooks],
  )

  const sortedEventNames = useMemo(
    () => Object.keys(hooksConfig).sort((left, right) => left.localeCompare(right)),
    [hooksConfig],
  )
  const eventTabs = useMemo(() => [ALL_EVENT_KEY, ...sortedEventNames], [sortedEventNames])
  const visibleEntries = useMemo(() => {
    const entries = Object.entries(hooksConfig).sort(([left], [right]) => left.localeCompare(right))
    return selectedEvent === ALL_EVENT_KEY ? entries : entries.filter(([eventName]) => eventName === selectedEvent)
  }, [hooksConfig, selectedEvent])
  const totalHandlers = useMemo(
    () => Object.values(hooksConfig).reduce((count, groups) => count + groups.reduce((sum, group) => sum + group.hooks.length, 0), 0),
    [hooksConfig],
  )
  const eventCount = useCallback(
    (eventName: string) => {
      if (eventName === ALL_EVENT_KEY) {
        return Object.values(hooksConfig).reduce((count, groups) => count + groups.length, 0)
      }
      return hooksConfig[eventName]?.length ?? 0
    },
    [hooksConfig],
  )

  const header = (
    <PageHeader
      title={tt('Hooks 管理', 'Hooks Management')}
      status={<span>{totalHandlers}</span>}
      actions={
        <button
          type="button"
          className="inline-flex min-h-11 items-center rounded-lg bg-accent-secondary px-4 py-2 font-medium text-[color:var(--color-accent-primary-contrast)]"
          onClick={openCreateBlank}
        >
          <SIcon name="Plus" size="w-5 h-5" className="mr-2" />
          {tt('添加 Hook 组', 'Add Hook Group')}
        </button>
      }
    />
  )

  return (
    <PageShell className="hooks-view" header={header}>
      <article className="mb-6 rounded-2xl border border-border-subtle bg-bg-surface">
        <div className="space-y-3 p-5">
          <p className="text-sm text-text-secondary">
            {tt('Claude Code hooks 使用官方分组格式：', 'Claude Code hooks use the official grouped format:')}
            <code className="font-mono text-xs"> event -&gt; matcher groups -&gt; handlers</code>
            {tt('。', '.')}
          </p>
          <p className="text-xs text-text-muted">
            {tt(
              '官方 schema 不支持单独关闭某个 hook。要禁用时，请删除对应 handler 或 matcher group。',
              'Individual hooks cannot be toggled off in the official schema. Remove a handler or matcher group to disable it.',
            )}
          </p>
        </div>
      </article>
      <div className="mb-6 flex gap-2 overflow-x-auto pb-2 md:flex-wrap md:overflow-x-visible md:pb-0">
        {eventTabs.map((eventName) => (
          <EventTab
            key={eventName}
            eventName={eventName}
            count={eventCount(eventName)}
            active={selectedEvent === eventName}
            onSelect={setSelectedEvent}
          />
        ))}
      </div>
      {loading ? (
        <div className="py-20 text-center text-text-muted">
          <Spinner size="lg" className="mx-auto mb-4 text-accent-secondary" />
          <span>{tt('加载中...', 'Loading...')}</span>
        </div>
      ) : visibleEntries.length === 0 ? (
        <div className="py-20 text-center text-text-muted">
          <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-bg-elevated">
            <SIcon name="Webhook" size="w-10 h-10" className="opacity-50" />
          </div>
          <p className="text-lg font-medium">{tt('没有找到 Hook 组', 'No hook groups found')}</p>
        </div>
      ) : (
        <div className="space-y-6">
          {visibleEntries.map(([eventName, groups]) => (
            <HookGroupCard
              key={eventName}
              eventName={eventName}
              groups={groups}
              onAdd={openCreate}
              onEdit={openEdit}
              onDeleteGroup={handleDeleteGroup}
              onDeleteHandler={handleDeleteHandler}
            />
          ))}
        </div>
      )}
      <HookGroupModal
        open={showModal}
        editing={Boolean(editingTarget)}
        initial={formInitial}
        saving={saving}
        onClose={closeModal}
        onSave={saveGroup}
      />
    </PageShell>
  )
}
