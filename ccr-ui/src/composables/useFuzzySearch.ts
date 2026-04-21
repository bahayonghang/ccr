/**
 * useFuzzySearch —— 统一封装 fuse.js 的模糊搜索 composable
 *
 * 抽走 useSkillsManager / useMcpManager 中重复的 Fuse 构造与过滤逻辑，
 * 同时让两个页面共享同一份 fuse.js 打包入口（命中 search-vendor chunk）。
 *
 * 用法：
 *   const { query, results } = useFuzzySearch(
 *     groupedItems,
 *     [{ name: 'name', weight: 2 }, { name: 'description', weight: 1 }],
 *     { threshold: 0.4 },
 *   )
 */

import Fuse, { type IFuseOptions } from 'fuse.js'
import { computed, ref, type ComputedRef, type Ref } from 'vue'

export interface UseFuzzySearchResult<T> {
  /** 绑定到输入框的查询词；空串时 results 返回原始 items */
  query: Ref<string>
  /** 命中结果（保持原始顺序以满足调用方后续分组/排序预期） */
  results: ComputedRef<T[]>
  /** 用于高级场景（如高亮）直接取 Fuse 实例 */
  fuse: ComputedRef<Fuse<T>>
}

type ItemsInput<T> = Ref<T[]> | ComputedRef<T[]>

/**
 * 基于 Fuse.js 的响应式模糊搜索。
 * - items 变化时 Fuse 实例自动重建
 * - query 为空或仅空白时直接返回 items 原始数组
 */
export function useFuzzySearch<T>(
  items: ItemsInput<T>,
  keys: IFuseOptions<T>['keys'],
  options: Omit<IFuseOptions<T>, 'keys'> = {},
): UseFuzzySearchResult<T> {
  const query = ref('')

  const fuse = computed(() => new Fuse<T>(items.value, {
    keys,
    ...options,
  }))

  const results = computed<T[]>(() => {
    const text = query.value.trim()
    if (!text) return items.value
    return fuse.value.search(text).map((hit) => hit.item)
  })

  return { query, results, fuse }
}
