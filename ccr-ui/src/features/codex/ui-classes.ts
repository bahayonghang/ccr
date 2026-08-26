import { cn } from '@/ui'

export const fieldInputClass = cn(
  'input w-full rounded-xl border border-border-default/55 bg-bg-elevated px-3 py-2 text-sm text-text-primary outline-none',
  'placeholder:text-text-ghost focus:border-accent-primary/40',
  'disabled:cursor-not-allowed disabled:opacity-50',
)

export const panelCardClass =
  'rounded-2xl border border-border-subtle bg-bg-surface p-5'

export const toneIconClass: Record<'success' | 'warning' | 'danger' | 'neutral', string> = {
  success: 'border-accent-success/20 bg-accent-success/10 text-accent-success',
  warning: 'border-accent-warning/20 bg-accent-warning/10 text-accent-warning',
  danger: 'border-accent-danger/20 bg-accent-danger/10 text-accent-danger',
  neutral: 'border-border-default/20 bg-bg-elevated text-text-muted',
}
