/**
 * useFuzzySearch —— 统一封装 fuse.js 的模糊搜索 hook（React）
 *
 * 抽走多处列表页里重复的 Fuse 构造与过滤逻辑，
 * 同时让页面共享同一份 fuse.js 打包入口（命中 search-vendor chunk）。
 *
 * 用法：
 *   const { query, setQuery, results } = useFuzzySearch(
 *     items,
 *     [{ name: 'name', weight: 2 }, { name: 'description', weight: 1 }],
 *     { threshold: 0.4 },
 *   )
 *
 * 08-22-state-logic-port 批次 5c：Vue → React（组件本地瞬态）。
 * 签名变化（消费方为待迁移 .vue 视图与同批 useMcpManager）：
 * - items：Ref<T[]> | ComputedRef<T[]> → 普通 T[]（调用方经 useMemo 提供稳定引用）；
 * - 返回字段 query/results/fuse 由 Ref/computed 改为普通值，新增 setQuery 写入
 *   （原 `query.value = x` 的对应物）；Fuse 实例改为直接值（原 ComputedRef<Fuse>）。
 */

import { useMemo, useRef, useState } from 'react'
import Fuse, { type IFuseOptions } from 'fuse.js'

export interface UseFuzzySearchResult<T> {
  /** 绑定到输入框的查询词；空串时 results 返回原始 items */
  query: string
  /** 查询词写入（原 `query.value = x` 的对应物） */
  setQuery: (query: string) => void
  /** 命中结果（保持原始顺序以满足调用方后续分组/排序预期） */
  results: T[]
  /** 用于高级场景（如高亮）直接取 Fuse 实例 */
  fuse: Fuse<T>
}

/**
 * Fuse.js-backed fuzzy search.
 * - items 变化时 Fuse 实例自动重建（keys/options 为静态配置，经 ref 惰性读取，
 *   不参与重建判定——与原 Vue computed 的追踪面等价）
 * - query 为空或仅空白时直接返回 items 原始数组
 */
export function useFuzzySearch<T>(
  items: T[],
  keys: IFuseOptions<T>['keys'],
  options: Omit<IFuseOptions<T>, 'keys'> = {},
): UseFuzzySearchResult<T> {
  const [query, setQuery] = useState('')

  const configRef = useRef({ keys, options })
  configRef.current = { keys, options }

  // 原 computed(:41)：来源 items（keys/options 为非响应式静态配置）
  const fuse = useMemo(
    () => new Fuse<T>(items, {
      keys: configRef.current.keys,
      ...configRef.current.options,
    }),
    [items],
  )

  const results = useMemo(() => {
    const text = query.trim()
    if (!text) return items
    return fuse.search(text).map((hit) => hit.item)
  }, [items, fuse, query])

  return { query, setQuery, results, fuse }
}
