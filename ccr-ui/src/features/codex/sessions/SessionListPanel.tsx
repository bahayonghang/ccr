import type { Virtualizer } from '@tanstack/react-virtual'
import type { RefObject } from 'react'
import { EmptyState, Spinner } from '@/ui'
import type { CodexSessionSummary } from '@/types'
import { SessionListItem } from './SessionListItem'

interface SessionListPanelProps {
  tt: (zh: string, en: string) => string
  unknownTime: string
  listRef: RefObject<HTMLDivElement | null>
  virtualizer: Virtualizer<HTMLDivElement, Element>
  filteredSessions: CodexSessionSummary[]
  selectedPath: string
  pending: boolean
  onOpen: (filePath: string) => void
  onClearSearch: () => void
}

export function SessionListPanel({
  tt,
  unknownTime,
  listRef,
  virtualizer,
  filteredSessions,
  selectedPath,
  pending,
  onOpen,
  onClearSearch,
}: SessionListPanelProps) {
  return (
    <>
      {pending ? (
        <div className="flex min-h-[20rem] flex-col items-center justify-center gap-3 text-sm text-text-muted">
          <Spinner />
          <span>{tt('正在读取本地会话记录…', 'Loading local session records...')}</span>
        </div>
      ) : filteredSessions.length === 0 ? (
        <EmptyState
          icon="Inbox"
          title={tt('没有匹配的会话', 'No matching sessions')}
          description={tt('当前过滤条件下没有找到会话，试试清空搜索或刷新列表。', 'No sessions match the current filters. Try clearing the search or refreshing the list.')}
          actionText={tt('清空搜索', 'Clear search')}
          actionIcon="RotateCcw"
          onAction={onClearSearch}
        />
      ) : (
        <div ref={listRef} className="max-h-[38.75rem] space-y-3 overflow-y-auto pr-1">
          <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const session = filteredSessions[virtualRow.index]
              if (!session) return null
              return (
                <div key={session.file_path} className="absolute top-0 left-0 w-full pb-3" style={{ transform: `translateY(${virtualRow.start}px)` }}>
                  <SessionListItem
                    session={session}
                    active={session.file_path === selectedPath}
                    unknownModel={tt('未知模型', 'Unknown model')}
                    unknownTime={unknownTime}
                    messageLabel={tt(`${session.message_count} 条消息`, `${session.message_count} msg`)}
                    onOpen={onOpen}
                  />
                </div>
              )
            })}
          </div>
        </div>
      )}
    </>
  )
}
