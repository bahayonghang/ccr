import { SIcon } from '@/ui'
import { useShellT } from '@/shell/i18n'
import './profiles-shared.css'

export interface ProfilesStatStripLabels {
  total: string
  vendors: string
  running: string
  runningHint: string
  notApplied: string
  tags: string
  auth: string
}

export interface ProfilesStatStripStats {
  total: number
  vendorCount: number
  tagCounts: Record<string, number>
  authCounts: Record<string, number>
}

export interface ProfilesStatStripProps {
  current: string | null
  stats: ProfilesStatStripStats
  labels: ProfilesStatStripLabels
}

const countEntries = (counts: Record<string, number>): [string, number][] =>
  Object.entries(counts).sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))

/** 四卡统计条：总数 / 运行中 / 标签 / 认证。 */
export function ProfilesStatStrip({ current, stats, labels }: ProfilesStatStripProps) {
  const t = useShellT()
  const runningClass = current
    ? 'cp-stat cp-stat--running surface-status'
    : 'cp-stat surface-status'

  return (
    <div className="cp-stats" data-testid="profiles-stat-strip">
      <div className="cp-stat surface-status">
        <div className="cp-stat__head">
          <SIcon name="Folder" size="w-3 h-3" />
          {labels.total}
        </div>
        <div className="cp-stat__value cp-stat__value--mono" data-testid="profiles-stat-total">
          {stats.total}
        </div>
        <div className="cp-stat__hint" data-testid="profiles-stat-vendors" data-vendor-count={stats.vendorCount}>
          {labels.vendors}
        </div>
      </div>

      <div className={runningClass} data-testid="profiles-stat-running">
        <div className="cp-stat__head">{labels.running}</div>
        <div className="cp-stat__value cp-stat__value--mono">
          {current || labels.notApplied}
        </div>
        <div className="cp-stat__hint">{current ? labels.runningHint : labels.notApplied}</div>
      </div>

      <div className="cp-stat surface-status">
        <div className="cp-stat__head">{labels.tags}</div>
        <div className="cp-stat__chips" data-testid="profiles-stat-tags">
          {countEntries(stats.tagCounts).map(([tag, count]) => (
            <span key={tag} className="cp-chip">
              #{tag}
              <span className="cp-chip__kbd">{count}</span>
            </span>
          ))}
        </div>
      </div>

      <div className="cp-stat surface-status">
        <div className="cp-stat__head">{labels.auth}</div>
        <div className="cp-stat__chips" data-testid="profiles-stat-auth">
          {countEntries(stats.authCounts).map(([authKey, count]) => (
            <span key={authKey} className="cp-chip">
              {t(`profilePresentation.auth.${authKey}`)}
              <span className="cp-chip__kbd">{count}</span>
            </span>
          ))}
        </div>
      </div>
    </div>
  )
}
