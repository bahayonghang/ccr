import { cn } from '@/ui/cn'

/** MCP 共享面板的表单/列表 class 收口（create / import 共用）。 */

export const mcpFieldLabelClass =
  'text-[0.6875rem] font-semibold uppercase tracking-[0.12em] text-text-muted'

export const mcpFieldInputClass = cn(
  'w-full rounded-xl border border-border-default/55 bg-bg-elevated px-3 py-2 text-sm text-text-primary outline-none',
  'placeholder:text-text-ghost focus:border-accent-primary/40 focus:shadow-md',
  'disabled:cursor-not-allowed disabled:opacity-50',
)

export const mcpMonoInputClass = cn(mcpFieldInputClass, 'font-mono text-xs')

export const mcpPanelHeaderClass =
  'flex items-center justify-between border-b border-border-default/45 px-6 py-5'

export const mcpPanelTitleClass = 'text-base font-bold text-text-primary'

export const mcpPanelBodyClass = 'flex flex-1 flex-col gap-4 overflow-y-auto px-6 py-5'

export const mcpPanelFooterClass =
  'flex justify-end gap-2 border-t border-border-default/45 px-6 py-4'

export const mcpGhostBtnClass = cn(
  'inline-flex items-center gap-1.5 rounded-xl border border-border-default/55 bg-bg-elevated px-3 py-1.5 text-sm font-medium text-text-secondary',
  'transition-colors hover:bg-bg-surface hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50',
)

export const mcpPrimaryBtnClass = cn(
  'inline-flex items-center gap-1.5 rounded-xl border border-accent-primary/20 bg-accent-primary/12 px-3 py-1.5 text-sm font-medium text-text-primary',
  'transition-colors hover:bg-accent-primary/20 disabled:cursor-not-allowed disabled:opacity-50',
)

export const mcpIconBtnClass = cn(
  'flex h-8 w-8 items-center justify-center rounded-lg text-text-muted',
  'transition-colors hover:bg-bg-overlay/70 hover:text-text-primary',
)

export const mcpListActionBtnClass = mcpIconBtnClass

export const mcpKvRowClass =
  'flex items-center gap-2 rounded-lg bg-bg-base/42 px-2 py-1.5'

export const CLAUDE_USER_SCOPE_PATH = '~/.claude.json'
