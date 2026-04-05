import { logger } from '@/utils/logger'
import { showCurrentWindowIfTauri } from '@/utils/tauriWindow'

const STARTUP_ERROR_TYPES = ['error', 'unhandledrejection'] as const

const sanitizeErrorMessage = (value: unknown): string => {
  if (value instanceof Error) {
    return value.message
  }

  if (typeof value === 'string') {
    return value
  }

  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

const createFallbackText = (headline: string, message: string): HTMLElement => {
  const shell = document.createElement('div')
  shell.setAttribute(
    'style',
    [
      'min-height:100vh',
      'display:flex',
      'align-items:center',
      'justify-content:center',
      'padding:24px',
      'background:radial-gradient(circle at top, rgba(29,78,216,0.18), transparent 45%), #0f172a',
      'color:#e2e8f0',
      'font-family:"MapleBright","Microsoft YaHei UI",system-ui,sans-serif',
    ].join(';'),
  )

  const card = document.createElement('div')
  card.setAttribute(
    'style',
    [
      'width:min(560px,100%)',
      'border-radius:20px',
      'border:1px solid rgba(148,163,184,0.24)',
      'background:rgba(15,23,42,0.92)',
      'box-shadow:0 24px 80px rgba(15,23,42,0.45)',
      'padding:28px',
    ].join(';'),
  )

  const title = document.createElement('h1')
  title.textContent = headline
  title.setAttribute(
    'style',
    'margin:0 0 12px;font-size:24px;line-height:1.2;font-weight:600;color:#f8fafc',
  )

  const description = document.createElement('p')
  description.textContent = message
  description.setAttribute(
    'style',
    'margin:0;font-size:14px;line-height:1.7;color:rgba(226,232,240,0.82);white-space:pre-wrap',
  )

  card.append(title, description)
  shell.append(card)
  return shell
}

export const renderFatalStartup = (message: string): void => {
  if (typeof document === 'undefined') {
    return
  }

  const mountNode = document.querySelector('#app')
  if (!(mountNode instanceof HTMLElement)) {
    return
  }

  mountNode.replaceChildren(
    createFallbackText(
      'CCR Desktop failed to finish startup',
      `${message}\n\nPlease check Monitoring or recent frontend logs for details.`,
    ),
  )

  void showCurrentWindowIfTauri().catch((error) => {
    logger.warn('[startup] failed to reveal fatal startup fallback', error)
  })
}

export const reportStartupFailure = (stage: string, error: unknown): void => {
  const message = sanitizeErrorMessage(error)
  logger.error(`[startup] ${stage} failed`, error)
  renderFatalStartup(`${stage}: ${message}`)
}

export const installStartupErrorHandlers = (): (() => void) => {
  if (typeof window === 'undefined') {
    return () => undefined
  }

  const onError = (event: ErrorEvent) => {
    reportStartupFailure('Unhandled window error', event.error ?? event.message)
  }

  const onUnhandledRejection = (event: PromiseRejectionEvent) => {
    reportStartupFailure('Unhandled promise rejection', event.reason)
  }

  window.addEventListener(STARTUP_ERROR_TYPES[0], onError)
  window.addEventListener(STARTUP_ERROR_TYPES[1], onUnhandledRejection)

  return () => {
    window.removeEventListener(STARTUP_ERROR_TYPES[0], onError)
    window.removeEventListener(STARTUP_ERROR_TYPES[1], onUnhandledRejection)
  }
}
