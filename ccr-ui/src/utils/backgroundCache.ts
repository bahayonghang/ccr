const DB_NAME = 'ccr-ui-background-cache'
const STORE_NAME = 'backgrounds'
const CACHE_KEY = 'anime-background'
let hasAttemptedRuntimeRefresh = false

export const BACKGROUND_CACHE_TTL_MS = 60 * 60 * 1000

export interface BackgroundCacheRecord {
  sourceUrl: string
  contentType: string
  fetchedAt: number
  blob: Blob
}

let databasePromise: Promise<IDBDatabase> | null = null

const getIndexedDb = (): IDBFactory | null => {
  if (typeof window === 'undefined') {
    return null
  }

  return window.indexedDB ?? null
}

const withRequest = <T>(request: IDBRequest<T>): Promise<T> => {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error ?? new Error('IndexedDB request failed'))
  })
}

const openDatabase = async (): Promise<IDBDatabase> => {
  if (databasePromise) {
    return databasePromise
  }

  const indexedDb = getIndexedDb()
  if (!indexedDb) {
    throw new Error('IndexedDB is unavailable')
  }

  databasePromise = new Promise((resolve, reject) => {
    const request = indexedDb.open(DB_NAME, 1)

    request.onupgradeneeded = () => {
      const database = request.result
      if (!database.objectStoreNames.contains(STORE_NAME)) {
        database.createObjectStore(STORE_NAME)
      }
    }

    request.onsuccess = () => {
      const database = request.result

      database.onversionchange = () => {
        database.close()
        databasePromise = null
      }

      resolve(database)
    }

    request.onerror = () => {
      databasePromise = null
      reject(request.error ?? new Error('Failed to open IndexedDB'))
    }

    request.onblocked = () => {
      databasePromise = null
      reject(new Error('IndexedDB open request was blocked'))
    }
  })

  return databasePromise
}

const runTransaction = async <T>(mode: IDBTransactionMode, action: (store: IDBObjectStore) => Promise<T>): Promise<T> => {
  const database = await openDatabase()

  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, mode)
    const store = transaction.objectStore(STORE_NAME)
    let transactionResult: T
    let settled = false

    transaction.oncomplete = () => {
      if (!settled) {
        settled = true
        resolve(transactionResult)
      }
    }

    transaction.onerror = () => {
      if (!settled) {
        settled = true
        reject(transaction.error ?? new Error('IndexedDB transaction failed'))
      }
    }

    transaction.onabort = () => {
      if (!settled) {
        settled = true
        reject(transaction.error ?? new Error('IndexedDB transaction aborted'))
      }
    }

    action(store)
      .then((result) => {
        transactionResult = result
      })
      .catch((error) => {
        try {
          transaction.abort()
        } catch {
          // noop
        }
        if (!settled) {
          settled = true
          reject(error)
        }
      })
  })
}

const isBackgroundCacheRecord = (value: unknown): value is BackgroundCacheRecord => {
  if (typeof value !== 'object' || value === null) {
    return false
  }

  const candidate = value as Partial<BackgroundCacheRecord>
  return typeof candidate.sourceUrl === 'string'
    && typeof candidate.contentType === 'string'
    && typeof candidate.fetchedAt === 'number'
    && candidate.blob instanceof Blob
}

export const loadBackgroundCache = async (): Promise<BackgroundCacheRecord | null> => {
  const record = await runTransaction('readonly', async (store) => {
    return withRequest(store.get(CACHE_KEY))
  })

  if (record == null) {
    return null
  }

  if (isBackgroundCacheRecord(record)) {
    return record
  }

  await clearBackgroundCache().catch(() => undefined)
  return null
}

export const saveBackgroundCache = async (record: BackgroundCacheRecord): Promise<void> => {
  await runTransaction('readwrite', async (store) => {
    await withRequest(store.put(record, CACHE_KEY))
  })
}

export const clearBackgroundCache = async (): Promise<void> => {
  await runTransaction('readwrite', async (store) => {
    await withRequest(store.delete(CACHE_KEY))
  })
}

export const isBackgroundCacheExpired = (record: BackgroundCacheRecord, ttlMs: number = BACKGROUND_CACHE_TTL_MS): boolean => {
  return Date.now() - record.fetchedAt >= ttlMs
}

export const createBackgroundObjectUrl = (record: Pick<BackgroundCacheRecord, 'blob'>): string => {
  return URL.createObjectURL(record.blob)
}

export const revokeBackgroundObjectUrl = (url: string | null | undefined): void => {
  if (!url || !url.startsWith('blob:')) {
    return
  }

  URL.revokeObjectURL(url)
}

export const consumeBackgroundRefreshAttempt = (): boolean => {
  if (hasAttemptedRuntimeRefresh) {
    return false
  }

  hasAttemptedRuntimeRefresh = true
  return true
}
