export interface ObserverChartTheme {
  mode: 'light' | 'dark'
  primary: string
  secondary: string
  tertiary: string
  textMuted: string
  textSecondary: string
  grid: string
  info: string
}

const readCssVar = (name: string, fallback: string): string => {
  if (typeof document === 'undefined') return fallback
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
}

/** 观测图主题。只读 CCR token，不导入 usage 视图。 */
export function readObserverChartTheme(): ObserverChartTheme {
  const dark =
    typeof document !== 'undefined' &&
    (document.documentElement.getAttribute('data-theme') === 'dark' ||
      document.documentElement.classList.contains('dark'))
  return {
    mode: dark ? 'dark' : 'light',
    primary: readCssVar('--color-accent-primary', '#0071E3'),
    secondary: readCssVar('--color-accent-secondary', '#2997FF'),
    tertiary: readCssVar('--color-info', '#5AC8FA'),
    textMuted: readCssVar('--color-text-muted', '#6E6E73'),
    textSecondary: readCssVar('--color-text-secondary', '#6E6E73'),
    grid: readCssVar('--color-border-subtle', '#E5E5EA'),
    info: readCssVar('--color-info', '#7D97B6'),
  }
}

export function infoScaleColor(alpha: string): string {
  return `rgb(var(--color-info-rgb) / ${alpha})`
}
