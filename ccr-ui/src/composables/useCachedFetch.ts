import { computed, ref, type Ref } from 'vue'
import { getErrorMessage } from '@/utils/errorHandler'

interface UseCachedFetchOptions<T> {
  ttlMs: number
  initialValue: T
  isEmpty?: (value: T) => boolean
}

interface UseCachedFetchResult<T> {
  data: Ref<T>
  loading: Ref<boolean>
  error: Ref<string | null>
  lastFetchedAt: Ref<number>
  isCacheValid: Ref<boolean>
  fetch: (fetcher: () => Promise<T>, force?: boolean) => Promise<T>
  setData: (value: T) => void
  invalidate: () => void
}

const defaultIsEmpty = <T>(value: T): boolean => {
  if (Array.isArray(value)) return value.length === 0
  return value == null
}

export function useCachedFetch<T>(options: UseCachedFetchOptions<T>): UseCachedFetchResult<T> {
  const { ttlMs, initialValue, isEmpty = defaultIsEmpty } = options

  const data = ref(initialValue) as Ref<T>
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastFetchedAt = ref(0)
  let inFlightPromise: Promise<T> | null = null

  const isCacheValid = computed(() => {
    if (lastFetchedAt.value === 0 || isEmpty(data.value)) return false
    return Date.now() - lastFetchedAt.value < ttlMs
  })

  const setData = (value: T) => {
    data.value = value
    lastFetchedAt.value = Date.now()
    error.value = null
  }

  const invalidate = () => {
    lastFetchedAt.value = 0
  }

  const fetch = async (fetcher: () => Promise<T>, force = false): Promise<T> => {
    if (!force && isCacheValid.value) {
      return data.value
    }

    if (inFlightPromise) {
      return inFlightPromise
    }

    loading.value = true
    error.value = null

    const promise = fetcher()
      .then((result) => {
        setData(result)
        return result
      })
      .catch((err: unknown) => {
        error.value = getErrorMessage(err)
        throw err
      })
      .finally(() => {
        loading.value = false
        inFlightPromise = null
      })

    inFlightPromise = promise
    return promise
  }

  return {
    data,
    loading,
    error,
    lastFetchedAt,
    isCacheValid,
    fetch,
    setData,
    invalidate,
  }
}
