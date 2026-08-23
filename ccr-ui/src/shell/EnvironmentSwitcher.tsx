import { useEffect, useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import {
  listEnvironments,
  refreshEnvironments,
  switchEnvironment,
  type EnvironmentInfo,
} from '@/api/runtime/environment'
import { useShellT } from '@/shell/i18n'
import { logger } from '@/utils/logger'
import { SIcon } from '@/ui/s-icon'

const envIcon = (envType: string): string => {
  if (envType === 'wsl') return 'Terminal'
  if (envType === 'ssh') return 'Server'
  return 'Monitor'
}

const envColor = (envType: string): string => {
  if (envType === 'local') return 'text-emerald-400'
  if (envType === 'wsl') return 'text-orange-400'
  if (envType === 'ssh') return 'text-sky-400'
  return 'text-text-muted'
}

export function EnvironmentSwitcher() {
  const t = useShellT()
  const [environments, setEnvironments] = useState<EnvironmentInfo[]>([])
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const currentEnv = environments.find((item) => item.is_active)

  const fetchList = async () => {
    try {
      setEnvironments(await listEnvironments())
    } catch (error) {
      logger.error('[EnvironmentSwitcher] Failed to list environments:', error)
    }
  }

  useEffect(() => {
    void fetchList()
    const onClick = (event: MouseEvent) => {
      const el = (event.target as HTMLElement).closest('.env-switcher')
      if (!el) setIsOpen(false)
    }
    document.addEventListener('click', onClick)
    return () => document.removeEventListener('click', onClick)
  }, [])

  const switchEnv = async (envId: string) => {
    if (currentEnv?.id === envId) {
      setIsOpen(false)
      return
    }
    setIsLoading(true)
    try {
      await switchEnvironment(envId)
      await fetchList()
    } catch (error) {
      logger.error('[EnvironmentSwitcher] Failed to switch:', error)
    } finally {
      setIsLoading(false)
      setIsOpen(false)
    }
  }

  const refreshEnvs = async () => {
    setIsRefreshing(true)
    try {
      setEnvironments(await refreshEnvironments())
    } catch (error) {
      logger.error('[EnvironmentSwitcher] Failed to refresh:', error)
    } finally {
      setIsRefreshing(false)
    }
  }

  return (
    <div className="env-switcher relative shrink-0">
      <button
        type="button"
        className="glass-surface flex items-center gap-2 rounded-lg border border-border-default/60 px-3 py-1.5 text-xs font-medium text-text-secondary"
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        onClick={(event) => {
          event.stopPropagation()
          setIsOpen((open) => !open)
        }}
      >
        <SIcon
          name={envIcon(currentEnv?.env_type || 'local')}
          size="w-3.5 h-3.5"
          className={envColor(currentEnv?.env_type || 'local')}
        />
        <span className="max-w-[120px] truncate">
          {currentEnv?.name || t('common.environment.local')}
        </span>
        <SIcon name="ChevronDown" size="w-3 h-3" className={isOpen ? 'rotate-180' : undefined} />
      </button>
      <AnimatePresence>
        {isOpen ? (
          <motion.div
            className="env-switcher__menu glass-surface absolute top-full right-0 mt-1 w-64 overflow-hidden rounded-xl border border-border-default/70 shadow-xl"
            role="listbox"
            initial={{ opacity: 0, y: -4, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.95 }}
            transition={{ duration: 0.15 }}
          >
            <div className="flex items-center justify-between border-b border-border-default/50 px-3 py-2">
              <span className="text-[10px] font-bold tracking-wider text-text-muted uppercase">
                {t('common.environment.title')}
              </span>
              <button
                type="button"
                className="rounded-md p-1 text-text-muted hover:bg-bg-surface/70"
                disabled={isRefreshing}
                title={t('common.environment.refresh')}
                onClick={(event) => {
                  event.stopPropagation()
                  void refreshEnvs()
                }}
              >
                <SIcon name="RefreshCw" size="w-3 h-3" className={isRefreshing ? 'animate-spin' : undefined} />
              </button>
            </div>
            <div className="max-h-60 overflow-y-auto py-1">
              {environments.map((env) => (
                <button
                  key={env.id}
                  type="button"
                  className={`flex w-full items-center gap-3 px-3 py-2 text-left text-sm ${
                    env.is_active ? 'bg-accent-primary/10 text-accent-primary' : 'text-text-secondary'
                  }`}
                  disabled={isLoading}
                  role="option"
                  aria-selected={env.is_active}
                  onClick={(event) => {
                    event.stopPropagation()
                    void switchEnv(env.id)
                  }}
                >
                  <SIcon
                    name={envIcon(env.env_type)}
                    size="w-4 h-4"
                    className={env.is_active ? 'text-accent-primary' : envColor(env.env_type)}
                  />
                  <div className="min-w-0 flex-1">
                    <div className="truncate font-medium">{env.name}</div>
                    <div className="truncate text-[10px] text-text-muted">{env.description}</div>
                  </div>
                </button>
              ))}
              {environments.length === 0 ? (
                <div className="px-3 py-4 text-center text-xs text-text-muted">
                  {t('common.environment.empty')}
                </div>
              ) : null}
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  )
}
