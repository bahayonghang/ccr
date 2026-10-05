import type { KeyboardEvent } from 'react'
import type { ProviderTemplateOption } from '@/types/providerTemplates'

interface SelectorKeyHandlers {
  visibleCount: number
  activeIndex: number
  results: ProviderTemplateOption[]
  selectManual: () => void
  selectOption: (option: ProviderTemplateOption) => void
  setActiveIndex: (updater: (index: number) => number) => void
  close: () => void
}

export function handleSelectorKeyDown(
  event: KeyboardEvent<HTMLInputElement>,
  handlers: SelectorKeyHandlers,
): void {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    handlers.setActiveIndex((index) => Math.min(handlers.visibleCount - 1, index + 1))
    return
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    handlers.setActiveIndex((index) => Math.max(0, index - 1))
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    applyActiveOption(handlers)
    return
  }
  if (event.key === 'Escape') {
    event.preventDefault()
    handlers.close()
  }
}

function applyActiveOption(handlers: SelectorKeyHandlers): void {
  if (handlers.activeIndex === 0) {
    handlers.selectManual()
    return
  }
  const option = handlers.results[handlers.activeIndex - 1]
  if (option) handlers.selectOption(option)
}
