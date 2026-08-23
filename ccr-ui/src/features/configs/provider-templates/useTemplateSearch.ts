import { useMemo, useState } from 'react'
import Fuse from 'fuse.js'
import type { ProviderTemplateOption } from '@/types/providerTemplates'

export function useTemplateSearch(options: ProviderTemplateOption[]) {
  const [query, setQuery] = useState('')
  const fuse = useMemo(
    () =>
      new Fuse(options, {
        keys: [
          { name: 'label', weight: 4 },
          { name: 'subtitle', weight: 2 },
          { name: 'searchText', weight: 3 },
        ],
        threshold: 0.36,
        ignoreLocation: true,
      }),
    [options],
  )
  const results = useMemo(() => {
    const text = query.trim()
    if (!text) return options
    return fuse.search(text).map((hit) => hit.item)
  }, [fuse, options, query])
  return { query, setQuery, results }
}
