import { AnsiUp } from 'ansi_up'
import { sanitizeTerminal } from '@/utils/sanitize'

export interface AnsiRenderer {
  clear: () => void
  renderLine: (text: string) => string
}

export const createAnsiRenderer = (): AnsiRenderer => {
  const ansiUp = new AnsiUp()
  ansiUp.use_classes = true

  const cache = new Map<string, string>()
  const MAX_CACHE_ENTRIES = 4000

  const renderLine = (text: string) => {
    const cached = cache.get(text)
    if (typeof cached !== 'undefined') {
      // Refresh LRU order (Map preserves insertion order).
      cache.delete(text)
      cache.set(text, cached)
      return cached
    }

    const sanitized = sanitizeTerminal(ansiUp.ansi_to_html(text || ''))
    cache.set(text, sanitized)
    if (cache.size > MAX_CACHE_ENTRIES) {
      const oldestKey = cache.keys().next().value
      if (typeof oldestKey !== 'undefined') {
        cache.delete(oldestKey)
      }
    }
    return sanitized
  }

  return {
    clear: () => {
      cache.clear()
    },
    renderLine,
  }
}
