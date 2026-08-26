import { useCallback, useState, type MouseEvent } from 'react'
import { SIcon } from '@/ui'
import { useShellT } from '@/shell/i18n'

export interface ProfileOverflowMenuProps {
  enabled: boolean
  onEdit: () => void
  onToggle?: (enabled: boolean) => void
  onDelete?: () => void
}

/** 卡片/表格溢出：编辑、启停、可选删除。不含平台名。 */
export function ProfileOverflowMenu({
  enabled,
  onEdit,
  onToggle,
  onDelete,
}: ProfileOverflowMenuProps) {
  const t = useShellT()
  const [open, setOpen] = useState(false)
  const stop = (event: MouseEvent) => {
    event.stopPropagation()
  }
  const closeAnd = (run: () => void) => (event: MouseEvent) => {
    event.stopPropagation()
    setOpen(false)
    run()
  }
  const onMenuClick = useCallback((event: MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation()
    setOpen((current) => !current)
  }, [])
  if (!onToggle && !onDelete) return null

  return (
    <div className="cp-menu" onClick={stop}>
      <button
        type="button"
        className="cp-btn cp-btn--ghost"
        data-testid="profile-overflow"
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label={t('profilesSurface.overflow')}
        title={t('profilesSurface.overflow')}
        onClick={onMenuClick}
      >
        <SIcon name="MenuDots" size="w-3.5 h-3.5" />
      </button>
      {open ? (
        <div className="cp-menu__pop" role="menu" aria-label={t('profilesSurface.overflow')}>
          <button type="button" role="menuitem" className="cp-menu__item" onClick={closeAnd(onEdit)}>
            {t('profilesSurface.edit')}
          </button>
          {onToggle ? (
            <button
              type="button"
              role="menuitem"
              className="cp-menu__item"
              onClick={closeAnd(() => onToggle(!enabled))}
            >
              {enabled ? t('profilesSurface.stop') : t('profilesSurface.enable')}
            </button>
          ) : null}
          {onDelete ? (
            <button
              type="button"
              role="menuitem"
              className="cp-menu__item"
              data-testid="profile-overflow-delete"
              onClick={closeAnd(onDelete)}
            >
              {t('profilesSurface.delete')}
            </button>
          ) : null}
        </div>
      ) : null}
    </div>
  )
}
