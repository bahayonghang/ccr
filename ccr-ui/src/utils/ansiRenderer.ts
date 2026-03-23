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

  const renderLine = (text: string) => {
    if (cache.has(text)) {
      return cache.get(text) ?? ''
    }

    const sanitized = sanitizeTerminal(ansiUp.ansi_to_html(text || ''))
    cache.set(text, sanitized)
    return sanitized
  }

  return {
    clear: () => {
      cache.clear()
    },
    renderLine,
  }
}
