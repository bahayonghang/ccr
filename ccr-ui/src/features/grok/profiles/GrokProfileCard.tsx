import { memo, useCallback, useEffect, useRef, useState, type RefObject } from 'react'
import type { GrokProfileDto } from '@/types'
import { SIcon } from '@/ui'
import { GROK_FIELD_PLACEHOLDER, grokAuthModeLabel, resolveGrokBaseUrl } from '@/utils/grokProfiles'
import { formatBaseUrlDisplay } from '@/utils/text'
import { t } from '../locale'

interface GrokProfileCardProps {
  profile: GrokProfileDto
  isCurrent: boolean
  disabled?: boolean
  busyAction?: 'apply' | 'delete' | null
  onApply: (name: string) => void
  onEdit: (name: string) => void
  onDelete: (name: string) => void
  onToggle: (name: string, enabled: boolean) => void
}

function CardMenu({
  open,
  disabled,
  enabled,
  deleting,
  menuButton,
  menuPanel,
  onToggleMenu,
  onEdit,
  onToggle,
  onDelete,
}: {
  open: boolean
  disabled: boolean
  enabled: boolean
  deleting: boolean
  menuButton: RefObject<HTMLButtonElement | null>
  menuPanel: RefObject<HTMLDivElement | null>
  onToggleMenu: () => void
  onEdit: () => void
  onToggle: () => void
  onDelete: () => void
}) {
  return (
    <>
      <button
        ref={menuButton}
        type="button"
        className="rounded-md p-1 text-text-muted disabled:opacity-50"
        disabled={disabled}
        aria-expanded={open}
        aria-label={t('grok.profiles.overflowMenu')}
        onClick={onToggleMenu}
      >
        <SIcon name="MenuDots" size="w-4 h-4" />
      </button>
      {open ? (
        <div ref={menuPanel} className="absolute z-20 mt-8 rounded-md border border-border-default bg-bg-elevated p-1 shadow-lg" role="menu">
          <button type="button" role="menuitem" className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm" onClick={onEdit}>
            <SIcon name="Edit2" size="w-4 h-4" />
            {t('grok.profiles.actions.edit')}
          </button>
          <button type="button" role="menuitem" className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm" onClick={onToggle}>
            <SIcon name={enabled ? 'Pause' : 'Play'} size="w-4 h-4" />
            {enabled ? t('grok.profiles.actions.disable') : t('grok.profiles.actions.enable')}
          </button>
          <button type="button" role="menuitem" className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-accent-danger" onClick={onDelete}>
            <SIcon name={deleting ? 'RefreshCw' : 'Trash2'} size="w-4 h-4" />
            {t('grok.profiles.actions.delete')}
          </button>
        </div>
      ) : null}
    </>
  )
}

export const GrokProfileCard = memo(function GrokProfileCard({
  profile,
  isCurrent,
  disabled = false,
  busyAction = null,
  onApply,
  onEdit,
  onDelete,
  onToggle,
}: GrokProfileCardProps) {
  const [menuOpen, setMenuOpen] = useState(false)
  const menuButton = useRef<HTMLButtonElement | null>(null)
  const menuPanel = useRef<HTMLDivElement | null>(null)
  const fields = [
    {
      label: t('grok.profiles.fields.baseUrl'),
      value: formatBaseUrlDisplay(resolveGrokBaseUrl(profile, t)),
    },
    { label: t('grok.profiles.fields.model'), value: profile.model || GROK_FIELD_PLACEHOLDER },
    { label: t('grok.profiles.fields.authMode'), value: grokAuthModeLabel(t, profile.auth_mode) },
    { label: t('grok.profiles.fields.apiBackend'), value: profile.api_backend || GROK_FIELD_PLACEHOLDER },
    { label: t('grok.profiles.fields.reasoningEffort'), value: profile.reasoning_effort || GROK_FIELD_PLACEHOLDER },
  ]

  const closeMenu = useCallback((restoreFocus = false) => {
    setMenuOpen(false)
    if (restoreFocus) menuButton.current?.focus()
  }, [])

  const toggleMenu = useCallback(() => {
    setMenuOpen((open) => !open)
  }, [])

  useEffect(() => {
    if (!menuOpen) return
    const items = Array.from(menuPanel.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])
    items[0]?.focus()
    const onPointer = (event: MouseEvent) => {
      const target = event.target as Node
      if (menuPanel.current?.contains(target) || menuButton.current?.contains(target)) return
      closeMenu()
    }
    document.addEventListener('mousedown', onPointer)
    return () => document.removeEventListener('mousedown', onPointer)
  }, [closeMenu, menuOpen])

  const handleApply = useCallback(() => {
    onApply(profile.name)
  }, [onApply, profile.name])
  const handleEdit = useCallback(() => {
    onEdit(profile.name)
    closeMenu(true)
  }, [closeMenu, onEdit, profile.name])
  const handleToggle = useCallback(() => {
    onToggle(profile.name, !profile.enabled)
    closeMenu(true)
  }, [closeMenu, onToggle, profile.enabled, profile.name])
  const handleDelete = useCallback(() => {
    onDelete(profile.name)
    closeMenu(true)
  }, [closeMenu, onDelete, profile.name])

  return (
    <article
      className={[
        'relative flex flex-col gap-2.5 rounded-lg border border-border-default bg-bg-surface p-3.5',
        isCurrent ? 'border-l-2 border-l-accent-primary' : '',
        profile.enabled ? '' : 'opacity-60',
      ].join(' ')}
      data-profile-name={profile.name}
    >
      <div className="flex min-w-0 items-center gap-2">
        <span className={isCurrent ? 'h-2 w-2 shrink-0 rounded-full bg-accent-success' : 'h-2 w-2 shrink-0 rounded-full bg-text-ghost'} />
        <h3 className="min-w-0 flex-1 truncate font-mono text-sm font-semibold text-text-primary" title={profile.name}>
          {profile.name}
        </h3>
        <span className="text-xs text-text-secondary">{t(`grok.profiles.profileKinds.${profile.profile_kind}`)}</span>
        <div className="flex items-center gap-2">
          {isCurrent ? (
            <span className="text-xs text-text-secondary">{t('grok.profiles.currentActive')}</span>
          ) : (
            <button
              type="button"
              className="inline-flex items-center gap-1 rounded-md border border-border-default px-2 py-1 text-xs disabled:opacity-50"
              disabled={disabled || !profile.enabled}
              onClick={handleApply}
            >
              <SIcon name={busyAction === 'apply' ? 'RefreshCw' : 'Play'} size="w-3 h-3" />
              {t('grok.profiles.actions.apply')}
            </button>
          )}
          <CardMenu
            open={menuOpen}
            disabled={disabled}
            enabled={profile.enabled}
            deleting={busyAction === 'delete'}
            menuButton={menuButton}
            menuPanel={menuPanel}
            onToggleMenu={toggleMenu}
            onEdit={handleEdit}
            onToggle={handleToggle}
            onDelete={handleDelete}
          />
        </div>
      </div>
      {profile.description ? <p className="text-sm text-text-secondary">{profile.description}</p> : null}
      <dl className="grid grid-cols-2 gap-2 text-xs">
        {fields.map((field) => (
          <div key={field.label}>
            <dt className="text-text-muted">{field.label}</dt>
            <dd className="truncate text-text-primary" title={field.value}>{field.value}</dd>
          </div>
        ))}
      </dl>
      {profile.tags.length > 0 ? (
        <div className="flex flex-wrap gap-1 text-xs text-text-secondary">
          {profile.tags.map((tag) => (
            <span key={tag}>#{tag}</span>
          ))}
        </div>
      ) : null}
    </article>
  )
})
