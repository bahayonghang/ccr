/**
 * 通用缓存 Fetch 组合式 API
 * 提供 TTL 缓存、请求去重、错误处理等能力
 */
import { ref, computed, type Ref, type ComputedRef } from 'vue'

export interface CacheOptions {
  /** 缓存有效期（毫秒） */
  ttl: number
  /** 缓存标识 key（用于调试与去重） */
  key: string
}

export interface CachedFetchReturn<T> {
  /** 缓存的数据 */
  data: Ref<T | null>
  /** 是否正在加载 */
  loading: Ref<boolean>
  /** 错误信息 */
  error: Ref<string | null>
  /** 执行 fetch（缓存有效时直接返回，force=true 强制刷新） */
  fetch: (force?: boolean) => Promise<T | null>
  /** 使缓存失效 */
  invalidate: () => void
  /** 缓存是否当前有效 */
  isCacheValid: ComputedRef<boolean>
  /** 最后一次成功 fetch 的时间戳（0 表示从未 fetch） */
  lastFetchedAt: Ref<number>
}

/**
 * 创建一个带 TTL 缓存的 fetch 组合式 API。
 *
 * 特性：
 * - 缓存有效期内重复调用直接返回缓存数据，不发请求
 * - 若已有请求在途，不重复发起（去重）
 * - force=true 可绕过缓存强制刷新
 * - invalidate() 手动清除缓存（下次 fetch 时重新请求）
 *
 * @param fetcher 实际执行网络请求的函数
 * @param options 缓存配置（ttl 毫秒数、key 标识符）
 */
export function useCachedFetch<T>(
  fetcher: () => Promise<T>,
  options: CacheOptions
): CachedFetchReturn<T> {
  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastFetchedAt = ref(0)

  // 当前在途请求的 Promise（去重用）
  let inFlightPromise: Promise<T | null> | null = null

  /** 缓存是否有效（数据存在且未超过 TTL） */
  const isCacheValid: ComputedRef<boolean> = computed(() => {
    if (data.value === null || lastFetchedAt.value === 0) return false
    return Date.now() - lastFetchedAt.value < options.ttl
  })

  /**
   * 执行 fetch 操作。
   * - 缓存有效且 force=false 时直接返回缓存数据
   * - 有请求在途时复用该 Promise，不重复发请求
   */
  async function fetch(force = false): Promise<T | null> {
    // 缓存命中，直接返回
    if (!force && isCacheValid.value) {
      return data.value
    }

    // 去重：复用已有在途请求
    if (inFlightPromise) {
      return inFlightPromise
    }

    loading.value = true
    error.value = null

    inFlightPromise = (async () => {
      try {
        const result = await fetcher()
        data.value = result
        lastFetchedAt.value = Date.now()
        return result
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : String(err)
        error.value = message
        return null
      } finally {
        loading.value = false
        inFlightPromise = null
      }
    })()

    return inFlightPromise
  }

  /** 使缓存失效（下次调用 fetch 时将重新请求） */
  function invalidate() {
    lastFetchedAt.value = 0
    data.value = null
    error.value = null
  }

  return {
    data,
    loading,
    error,
    fetch,
    invalidate,
    isCacheValid,
    lastFetchedAt
  }
}
