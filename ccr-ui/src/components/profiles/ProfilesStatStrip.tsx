import { SIcon } from '@/ui'
import './profiles-shared.css'

export interface ProfilesStatStripLabels {
  current: string
  notSet: string
  currentHint: string
  total: string
  totalHint: string
}

/** 第三列（平台特定）：Claude=认证分布；Codex=配置模式 */
export interface ProfilesStatStripSecondary {
  icon: string
  title: string
  value: string
  hint: string
  /** value 是否用等宽字体（Claude 计数用 mono，Codex 文案不用） */
  mono?: boolean
}

/** 第四槽：Health（问题数） */
export interface ProfilesStatStripHealth {
  title: string
  value: string
  hint: string
  /** 缺省 'ShieldCheck' */
  icon?: string
  /** 有问题时 warn 高亮 */
  warn?: boolean
}

export interface ProfilesStatStripProps {
  current: string | null
  total: number
  labels: ProfilesStatStripLabels
  secondary: ProfilesStatStripSecondary
  health: ProfilesStatStripHealth
  onHealthClick: () => void
}

/** 4 列统计条：当前 profile / 配置总数 / 平台特定列 / Health。 */
export function ProfilesStatStrip({
  current,
  total,
  labels,
  secondary,
  health,
  onHealthClick,
}: ProfilesStatStripProps) {
  const secondaryValueClass = secondary.mono
    ? 'cp-stat__value cp-stat__value--mono'
    : 'cp-stat__value'
  const healthClass = health.warn
    ? 'cp-stat cp-stat--clickable cp-stat--warn surface-status'
    : 'cp-stat cp-stat--clickable surface-status'

  return (
    <div className="cp-stats">
      <div className="cp-stat surface-status">
        <div className="cp-stat__head">
          <span className={current ? 'cp-stat__dot cp-stat__dot--good' : 'cp-stat__dot'} />
          {labels.current}
        </div>
        <div className="cp-stat__value cp-stat__value--mono">{current || labels.notSet}</div>
        <div className="cp-stat__hint">{labels.currentHint}</div>
      </div>

      <div className="cp-stat surface-status">
        <div className="cp-stat__head">
          <SIcon name="Folder" size="w-3 h-3" />
          {labels.total}
        </div>
        <div className="cp-stat__value cp-stat__value--mono">{total}</div>
        <div className="cp-stat__hint">{labels.totalHint}</div>
      </div>

      <div className="cp-stat surface-status">
        <div className="cp-stat__head">
          <SIcon name={secondary.icon} size="w-3 h-3" />
          {secondary.title}
        </div>
        <div className={secondaryValueClass}>{secondary.value}</div>
        <div className="cp-stat__hint">{secondary.hint}</div>
      </div>

      <button type="button" className={healthClass} onClick={onHealthClick}>
        <div className="cp-stat__head">
          <SIcon name={health.icon ?? 'ShieldCheck'} size="w-3 h-3" />
          {health.title}
        </div>
        <div className="cp-stat__value cp-stat__value--mono">{health.value}</div>
        <div className="cp-stat__hint">{health.hint}</div>
      </button>
    </div>
  )
}
