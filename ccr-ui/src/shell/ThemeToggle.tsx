import { useShellPreferencesStore } from '@/shell/stores/shellPreferences'
import { SIcon } from '@/ui/s-icon'

export function ThemeToggle() {
  const theme = useShellPreferencesStore((state) => state.effectiveTheme)
  const toggleTheme = useShellPreferencesStore((state) => state.toggleTheme)
  const nextLabel = theme === 'dark' ? '明亮' : '深色'

  return (
    <button
      type="button"
      className="inline-flex min-h-11 min-w-11 flex-shrink-0 items-center justify-center rounded-full border p-0 leading-none text-text-secondary shadow-sm transition-interactive duration-200 hover:text-accent-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30"
      title={`切换到${nextLabel}模式`}
      aria-label={`切换到${nextLabel}模式`}
      style={{
        background: 'var(--surface-status-bg)',
        borderColor: 'var(--surface-status-border)',
      }}
      onClick={(event) => {
        event.stopPropagation()
        toggleTheme()
      }}
    >
      <SIcon name={theme === 'dark' ? 'Moon' : 'Sun'} size="w-4 h-4" className="pointer-events-none block" />
    </button>
  )
}
