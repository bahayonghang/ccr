import { EmptyState, SIcon, Spinner, buttonClass } from '@/ui'
import type { CodexSessionDetailResponse, CodexSessionSummary } from '@/types'
import { panelCardClass } from '../ui-classes'
import { formatSessionAbsolute } from './session-format'

interface SessionDetailPanelProps {
  tt: (zh: string, en: string) => string
  unknownTime: string
  selectedPath: string
  selectedSession: CodexSessionSummary | null
  detail: CodexSessionDetailResponse | null
  detailPending: boolean
  actionLoading: boolean
  onCopyPath: () => void
  onExport: () => void
  onClone: () => void
  onDelete: () => void
  onCopyCwd: () => void
}

export function SessionDetailPanel({
  tt,
  unknownTime,
  selectedPath,
  selectedSession,
  detail,
  detailPending,
  actionLoading,
  onCopyPath,
  onExport,
  onClone,
  onDelete,
  onCopyCwd,
}: SessionDetailPanelProps) {
  return (
    <section className={`${panelCardClass} min-h-[45rem]`}>
      <div className="mb-4 flex flex-col gap-4">
        <div>
          <h2 className="text-base font-semibold text-text-primary">{tt('会话详情', 'Session details')}</h2>
          <p className="text-sm text-text-muted">
            {tt('当前默认只展示用户与助手消息，避免把系统提示词刷满工作台', 'By default, only user and assistant messages are shown so system prompts do not flood the workspace')}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={!selectedSession || actionLoading} onClick={onCopyPath}>
            <SIcon name="Copy" size="w-4 h-4" />{tt('复制路径', 'Copy path')}
          </button>
          <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={!selectedSession || actionLoading} onClick={onExport}>
            <SIcon name="Download" size="w-4 h-4" />{tt('导出', 'Export')}
          </button>
          <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={!selectedSession || actionLoading} onClick={onClone}>
            <SIcon name="CopyPlus" size="w-4 h-4" />{tt('克隆', 'Clone')}
          </button>
          <button type="button" className={buttonClass({ variant: 'danger' })} disabled={!selectedSession || actionLoading} onClick={onDelete}>
            <SIcon name="Trash2" size="w-4 h-4" />{tt('删除', 'Delete')}
          </button>
        </div>
      </div>
      <SessionDetailBody
        tt={tt}
        unknownTime={unknownTime}
        selectedPath={selectedPath}
        selectedSession={selectedSession}
        detail={detail}
        detailPending={detailPending}
        onCopyCwd={onCopyCwd}
      />
    </section>
  )
}

function SessionDetailBody({
  tt,
  unknownTime,
  selectedPath,
  selectedSession,
  detail,
  detailPending,
  onCopyCwd,
}: Omit<SessionDetailPanelProps, 'actionLoading' | 'onCopyPath' | 'onExport' | 'onClone' | 'onDelete'>) {
  if (detailPending && selectedPath) {
    return (
      <div className="flex min-h-[20rem] flex-col items-center justify-center gap-3 text-sm text-text-muted">
        <Spinner />
        <span>{tt('正在读取会话详情…', 'Loading session details...')}</span>
      </div>
    )
  }
  if (!detail) {
    return (
      <EmptyState
        icon="MessagesSquare"
        title={tt('还没有选中会话', 'No session selected yet')}
        description={tt('从左侧选择一个最近会话，就可以在这里查看详细上下文。', 'Select a recent session on the left to inspect its full context here.')}
      />
    )
  }
  return (
    <div className="flex h-full flex-col gap-4">
      <div className="rounded-2xl border border-border-default/15 bg-bg-elevated p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h3 className="font-mono text-lg font-semibold text-text-primary">{selectedSession?.session_id}</h3>
            <p className="mt-1 text-sm text-text-muted">
              {selectedSession?.model || tt('未知模型', 'Unknown model')} · {formatSessionAbsolute(selectedSession?.updated_at, unknownTime)}
            </p>
          </div>
          <span className="inline-flex items-center rounded-md border border-platform-codex/20 bg-platform-codex/10 px-3 py-1 text-xs font-medium text-platform-codex">
            {tt(`${selectedSession?.total_requests ?? 0} 次请求`, `${selectedSession?.total_requests ?? 0} req`)}
          </span>
        </div>
        <div className="mt-4 grid gap-3 md:grid-cols-2">
          <div className="rounded-2xl border border-border-default/15 bg-bg-base px-3 py-3">
            <span className="text-xs font-medium text-text-ghost">{tt('工作目录', 'Working directory')}</span>
            <button type="button" className="mt-1 block break-all text-left text-sm text-text-primary hover:text-platform-codex" onClick={onCopyCwd}>
              {selectedSession?.cwd || 'N/A'}
            </button>
          </div>
          <div className="rounded-2xl border border-border-default/15 bg-bg-base px-3 py-3">
            <span className="text-xs font-medium text-text-ghost">{tt('相对路径', 'Relative path')}</span>
            <span className="mt-1 block break-all text-sm text-text-primary">{selectedSession?.relative_path}</span>
          </div>
        </div>
        {detail.clipped ? (
          <div className="mt-4 rounded-xl border border-accent-warning/20 bg-accent-warning/10 px-3 py-2 text-sm text-accent-warning">
            {tt(
              `详情面板只展示最近 ${detail.message_limit} 条消息，导出会沿用同样的窗口上限。`,
              `The detail panel only shows the most recent ${detail.message_limit} messages, and export uses the same window limit.`,
            )}
          </div>
        ) : null}
      </div>
      <div className="max-h-[32.5rem] flex-1 space-y-3 overflow-y-auto pr-1">
        {detail.messages.map((message) => (
          <article
            key={`${message.timestamp || 'none'}-${message.role}-${message.text.slice(0, 24)}`}
            className={message.role === 'assistant' ? 'rounded-2xl border border-accent-primary/20 bg-accent-primary/10 p-4' : 'rounded-2xl border border-border-default/15 bg-bg-elevated p-4'}
          >
            <div className="flex items-center justify-between gap-3 text-xs">
              <span className="font-semibold uppercase tracking-[0.18em] text-text-muted">
                {message.role === 'assistant' ? tt('助手', 'Assistant') : tt('用户', 'User')}
              </span>
              <span className="text-text-ghost">{formatSessionAbsolute(message.timestamp, unknownTime)}</span>
            </div>
            <pre className="mt-3 overflow-x-auto font-mono text-sm leading-7 break-words whitespace-pre-wrap text-text-primary">
              <code>{message.text}</code>
            </pre>
          </article>
        ))}
      </div>
    </div>
  )
}
