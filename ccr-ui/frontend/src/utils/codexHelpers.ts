// Codex UI Helper Functions
// 用于 Codex 所有子页面的共享工具函数和样式常量

/**
 * 脱敏显示 Token
 * @param token - 完整的 token 字符串
 * @returns 脱敏后的 token
 */
export function maskToken(token: string): string {
  if (!token) return ''
  if (token.length <= 8) return '****'
  return token.slice(0, 4) + '****' + token.slice(-4)
}

/**
 * 根据 Provider 获取对应的颜色
 * @param provider - Provider 名称
 * @returns 对应的颜色值
 */
export function getProviderColor(provider: string): string {
  const colors: Record<string, string> = {
    'GitHub': '#6366f1',
    'Azure': '#0078d4',
    'OpenAI': '#10a37f',
    'Anthropic': '#d97706',
    'Custom': '#ec4899',
    'Google': '#4285f4'
  }
  return colors[provider] || '#8b5cf6'
}

/**
 * 卡片悬停效果处理
 * @param el - HTML 元素
 * @param hover - 是否悬停
 */
export function handleCardHover(el: HTMLElement, hover: boolean): void {
  if (hover) {
    el.style.background = 'rgba(255, 255, 255, 0.95)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.3)'
    el.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)'
    el.style.transform = 'translateY(-4px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}

/**
 * 格式化时间戳
 * @param timestamp - 时间戳或日期字符串
 * @returns 格式化后的时间字符串
 */
export function formatTimestamp(timestamp: string | number | Date): string {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '-'
  
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}

/**
 * 验证 URL 格式
 * @param url - URL 字符串
 * @returns 是否为有效 URL
 */
export function isValidUrl(url: string): boolean {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

/**
 * 验证 GitHub Token 格式
 * @param token - Token 字符串
 * @returns 是否为有效的 GitHub Token
 */
export function isValidGitHubToken(token: string): boolean {
  // GitHub Personal Access Token 格式: ghp_xxxx (40 chars total)
  // GitHub OAuth Token: gho_xxxx
  return /^(ghp|gho)_[a-zA-Z0-9]{36,}$/.test(token)
}

/**
 * 获取模型显示名称
 * @param modelId - 模型 ID
 * @returns 模型显示名称
 */
export function getModelDisplayName(modelId: string): string {
  const modelNames: Record<string, string> = {
    'gpt-4': 'GPT-4',
    'gpt-4-turbo': 'GPT-4 Turbo',
    'gpt-3.5-turbo': 'GPT-3.5 Turbo',
    'claude-sonnet-4-5-20250929': 'Claude Sonnet 4.5',
    'claude-opus-4-20250514': 'Claude Opus 4',
    'claude-3-5-sonnet-20241022': 'Claude 3.5 Sonnet',
    'claude-3-5-haiku-20241022': 'Claude 3.5 Haiku',
    'gemini-2.0-flash-exp': 'Gemini 2.0 Flash',
    'gemini-1.5-flash': 'Gemini 1.5 Flash',
    'gemini-1.5-pro': 'Gemini 1.5 Pro'
  }
  return modelNames[modelId] || modelId
}

/**
 * Codex 卡片样式常量
 */
export const CodexCardStyles = {
  base: {
    background: 'rgba(255, 255, 255, 0.7)',
    border: '1px solid rgba(99, 102, 241, 0.12)',
    borderRadius: '12px',
    padding: '20px',
    transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)'
  },
  hover: {
    background: 'rgba(255, 255, 255, 0.95)',
    borderColor: 'rgba(99, 102, 241, 0.3)',
    boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)',
    transform: 'translateY(-4px)'
  }
}

/**
 * Codex 颜色主题
 */
export const CodexTheme = {
  primary: '#6366f1',
  secondary: '#ec4899',
  success: '#10b981',
  warning: '#f59e0b',
  danger: '#ef4444',
  info: '#3b82f6',
  muted: '#9ca3af',
  
  // Provider colors
  providers: {
    github: '#6366f1',
    azure: '#0078d4',
    openai: '#10a37f',
    anthropic: '#d97706',
    google: '#4285f4',
    custom: '#ec4899'
  },
  
  // Status colors
  status: {
    enabled: '#10b981',
    disabled: '#f59e0b',
    error: '#ef4444'
  }
}

/**
 * 复制文本到剪贴板
 * @param text - 要复制的文本
 * @returns Promise<boolean> - 是否成功
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    // Fallback for older browsers
    try {
      const textarea = document.createElement('textarea')
      textarea.value = text
      textarea.style.position = 'fixed'
      textarea.style.opacity = '0'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      return true
    } catch {
      return false
    }
  }
}

/**
 * 生成随机 ID
 * @param prefix - 前缀
 * @returns 随机 ID
 */
export function generateId(prefix = 'codex'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

/**
 * 防抖函数
 * @param fn - 要防抖的函数
 * @param delay - 延迟时间（毫秒）
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null
  
  return function (this: any, ...args: Parameters<T>) {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
    
    timeoutId = setTimeout(() => {
      fn.apply(this, args)
    }, delay)
  }
}

/**
 * 节流函数
 * @param fn - 要节流的函数
 * @param delay - 延迟时间（毫秒）
 * @returns 节流后的函数
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let lastCall = 0
  
  return function (this: any, ...args: Parameters<T>) {
    const now = Date.now()
    
    if (now - lastCall >= delay) {
      lastCall = now
      fn.apply(this, args)
    }
  }
}

