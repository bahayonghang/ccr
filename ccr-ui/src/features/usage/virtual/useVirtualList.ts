import { useRef, type RefObject } from 'react'
import { useVirtualizer, type Virtualizer } from '@tanstack/react-virtual'

export interface VirtualListOptions {
  count: number
  estimateSize?: () => number
  overscan?: number
}

export interface VirtualListApi<T extends Element> {
  parentRef: RefObject<T | null>
  virtualizer: Virtualizer<T, Element>
}

/** 可复用虚拟列表接线，供 Usage 日志与 Codex 会话列表共用。 */
export function useVirtualList<T extends Element>({
  count,
  estimateSize = () => 48,
  overscan = 8,
}: VirtualListOptions): VirtualListApi<T> {
  const parentRef = useRef<T | null>(null)
  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => parentRef.current,
    estimateSize,
    overscan,
  })

  return { parentRef, virtualizer }
}
