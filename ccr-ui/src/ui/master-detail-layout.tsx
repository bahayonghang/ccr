import type { CSSProperties, ReactNode } from 'react'
import { cn } from './cn'

interface MasterDetailLayoutProps {
  list: ReactNode
  detail: ReactNode
  listWidth?: string
  className?: string
}

/**
 * 主从布局。列表滚动 / 选中 / 空态 / 加载由消费方负责；
 * 本组件只提供分栏壳与默认列表宽度。
 */
export function MasterDetailLayout({
  list,
  detail,
  listWidth = '20rem',
  className,
}: MasterDetailLayoutProps) {
  const listStyle: CSSProperties = { width: listWidth }
  return (
    <div className={cn('master-detail', className)}>
      <div className="master-detail__list" style={listStyle}>
        {list}
      </div>
      <div className="master-detail__detail">{detail}</div>
    </div>
  )
}
