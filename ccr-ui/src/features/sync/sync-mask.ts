export const maskSecrets = (value: string): string => {
  return value
    .replace(/((?:api[_-]?key|token|password|secret)["']?\s*[=:]\s*["']?)([^"',\s;}]+)(["']?)/gi, '$1••••••$3')
    .replace(/\b(Bearer\s+)([A-Za-z0-9._~+/-]+=*)/gi, '$1••••••')
    .replace(/(sk-[A-Za-z0-9_-]{8,})/g, 'sk-••••••')
}

export const isAncestorNotFound = (message: string): boolean => {
  return /AncestorNotFound|ancestor\s+not\s+found|ancestor.*not.*found/i.test(message)
}

export const normalizeRemoteParentPath = (remotePath: string): string => {
  const trimmed = remotePath.trim().replace(/\/+$/u, '')
  if (!trimmed) return '/ccr/'
  const segments = trimmed.split('/').filter(Boolean)
  if (segments.length <= 1) return '/'
  return `/${segments[0]}/`
}

export const extractRemotePathFromMessage = (message: string): string | undefined => {
  const match = message.match(/(?:remote\s+path|for)\s+(\/[^\s,;}]+)/i) ?? message.match(/(\/ccr\/[^\s,;}]+)/i)
  return match?.[1]?.replace(/[.)'"]+$/u, '')
}

export const toErrorMessage = (error: unknown, fallback = 'unknown error'): string => {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return fallback
}
