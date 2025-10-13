'use client';

import type { HistoryEntry } from '@/lib/types';

interface HistoryListProps {
  entries: HistoryEntry[];
  loading?: boolean;
}

export default function HistoryList({ entries, loading }: HistoryListProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center py-20" role="status">
        <div
          className="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
          style={{
            borderTopColor: 'var(--accent-primary)',
            borderRightColor: 'var(--accent-secondary)',
          }}
          aria-label="加载中"
        />
        <span className="sr-only">加载历史记录中...</span>
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
        暂无历史记录
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {entries.map((entry) => (
        <HistoryItem key={entry.id} entry={entry} />
      ))}
    </div>
  );
}

function HistoryItem({ entry }: { entry: HistoryEntry }) {
  const formatDate = (timestamp: string) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  return (
    <article
      className="rounded-lg p-3.5 transition-all hover:translate-y-[-2px]"
      style={{
        background: 'var(--bg-tertiary)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >
      {/* 头部 */}
      <header className="flex items-center justify-between mb-2.5">
        <h3
          className="text-sm font-semibold"
          style={{ color: 'var(--accent-primary)' }}
        >
          {entry.operation}
        </h3>
        <time
          className="text-xs"
          style={{ color: 'var(--text-muted)' }}
          dateTime={entry.timestamp}
        >
          {formatDate(entry.timestamp)}
        </time>
      </header>

      {/* 详情 */}
      <div className="text-xs space-y-1" style={{ color: 'var(--text-secondary)' }}>
        <div>
          <span style={{ color: 'var(--text-muted)' }}>操作者:</span>{' '}
          <span className="font-mono">{entry.actor}</span>
        </div>

        {entry.from_config && entry.to_config && (
          <div>
            <span style={{ color: 'var(--text-muted)' }}>切换:</span>{' '}
            <span className="font-mono font-semibold" style={{ color: 'var(--accent-warning)' }}>
              {entry.from_config}
            </span>
            {' → '}
            <span className="font-mono font-semibold" style={{ color: 'var(--accent-success)' }}>
              {entry.to_config}
            </span>
          </div>
        )}

        {entry.changes && entry.changes.length > 0 && (
          <details className="mt-2">
            <summary
              className="cursor-pointer font-medium hover:underline"
              style={{ color: 'var(--text-muted)' }}
            >
              环境变量变化 ({entry.changes.length})
            </summary>
            <div
              className="mt-2 p-2 rounded text-xs font-mono space-y-1"
              style={{ background: 'var(--bg-secondary)' }}
            >
              {entry.changes.map((change, idx) => (
                <div key={idx}>
                  <span style={{ color: 'var(--text-muted)' }}>{change.key}:</span>{' '}
                  <span style={{ color: 'var(--accent-warning)', opacity: 0.7 }}>
                    {change.old_value || '(无)'}
                  </span>
                  {' → '}
                  <span style={{ color: 'var(--accent-success)' }}>
                    {change.new_value || '(无)'}
                  </span>
                </div>
              ))}
            </div>
          </details>
        )}
      </div>
    </article>
  );
}

