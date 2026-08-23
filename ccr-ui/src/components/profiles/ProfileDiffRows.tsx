import type { ProfileDiffRow } from '@/utils/profileDiff'
import './profiles-shared.css'

export interface ProfileDiffRowsProps {
  rows: ProfileDiffRow[]
  /** 缺失值占位符（'—' / '未设置' 由调用方按语境决定） */
  placeholder?: string
}

/** 三行字段 diff（label / 当前值 → 目标值）：Inspector 预览与 Apply 确认框共用。 */
export function ProfileDiffRows({ rows, placeholder = '—' }: ProfileDiffRowsProps) {
  return (
    <ul className="cp-diff-rows">
      {rows.map((row) => (
        <li
          key={row.key}
          className={row.changed ? 'cp-diff-row cp-diff-row--changed' : 'cp-diff-row'}
        >
          <span className="cp-diff-row__label">{row.label}</span>
          <span className="cp-diff-row__values">
            <span
              className={
                row.from === null
                  ? 'cp-diff-row__value cp-diff-row__value--from cp-diff-row__value--empty'
                  : 'cp-diff-row__value cp-diff-row__value--from'
              }
            >
              {row.from ?? placeholder}
            </span>
            <span className="cp-diff-row__arrow" aria-hidden="true">
              →
            </span>
            <span
              className={
                row.to === null
                  ? 'cp-diff-row__value cp-diff-row__value--to cp-diff-row__value--empty'
                  : 'cp-diff-row__value cp-diff-row__value--to'
              }
            >
              {row.to ?? placeholder}
            </span>
          </span>
        </li>
      ))}
    </ul>
  )
}
