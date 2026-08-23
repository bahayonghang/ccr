import { useMemo, useRef, useState } from 'react'
import Fuse, { type IFuseOptions } from 'fuse.js'

export interface UseFuzzySearchResult<T> {
  query: string
  setQuery: (query: string) => void
  results: T[]
  fuse: Fuse<T>
}

type FuseKeys<T> = IFuseOptions<T>['keys']
type FuseRest<T> = Omit<IFuseOptions<T>, 'keys'>

export function useFuzzySearch<T>(items: T[], keys: FuseKeys<T>, options?: FuseRest<T>): UseFuzzySearchResult<T> {
  const resolvedOptions = options ?? {}
  const [query, setQuery] = useState('')
  const configRef = useRef({ keys, options: resolvedOptions })
  configRef.current = { keys, options: resolvedOptions }

  const fuse = useMemo(() => {
    return new Fuse(items, { keys: configRef.current.keys, ...configRef.current.options })
  }, [items])

  const results = useMemo(() => {
    const text = query.trim()
    if (!text) return items
    return fuse.search(text).map((hit) => hit.item)
  }, [items, fuse, query])

  return { query, setQuery, results, fuse }
}
