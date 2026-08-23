import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { Link } from 'react-router'
import { PageHeader, SIcon } from '@/ui'
import './profiles-shared.css'

export interface ProfilesHeaderLabels {
  title: string
  subtitle: string
  back: string
  reload: string
  export: string
  add: string
  source?: string
  /** ··· 溢出菜单文案 */
  overflow?: string
}

/** 命令面板按钮（Codex 用，Claude 省略 → 不渲染） */
export interface ProfilesHeaderPalette {
  label: string
  shortcut: string
  title: string
}

export interface ProfilesHeaderProps {
  icon: string
  backTo: string
  labels: ProfilesHeaderLabels
  loading?: boolean
  exporting?: boolean
  palette?: ProfilesHeaderPalette | null
  paletteOpen?: boolean
  sourceDisabled?: boolean
  sourceTitle?: string
  onAdd: () => void
  onExport: () => void
  onReload: () => void
  onOpenPalette: () => void
  onEditSource: () => void
}

const menuItemsOf = (root: HTMLElement | null): HTMLElement[] =>
  Array.from(root?.querySelectorAll<HTMLElement>('[role="menuitem"]:not(:disabled)') ?? [])

const focusCycled = (items: HTMLElement[], event: KeyboardEvent<HTMLElement>) => {
  if (items.length === 0) return
  const idx = items.indexOf(document.activeElement as HTMLElement)
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    items[(idx + 1) % items.length]?.focus()
    return
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    items[(idx - 1 + items.length) % items.length]?.focus()
    return
  }
  if (event.key !== 'Tab') return
  const first = items[0]
  const last = items[items.length - 1]
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last?.focus()
    return
  }
  if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first?.focus()
  }
}

/** Profiles 页面顶部标题区：标题/副标题 + （可选命令面板）/重载/导出/添加。 */
export function ProfilesHeader({
  icon,
  backTo,
  labels,
  loading = false,
  exporting = false,
  palette = null,
  paletteOpen = false,
  sourceDisabled = false,
  sourceTitle,
  onAdd,
  onExport,
  onReload,
  onOpenPalette,
  onEditSource,
}: ProfilesHeaderProps) {
  const [menuOpen, setMenuOpen] = useState(false)
  const menuBtnRef = useRef<HTMLButtonElement | null>(null)
  const menuPopRef = useRef<HTMLDivElement | null>(null)

  const closeMenu = (restoreFocus: boolean) => {
    setMenuOpen(false)
    if (restoreFocus) menuBtnRef.current?.focus()
  }

  useEffect(() => {
    if (!menuOpen) return
    menuItemsOf(menuPopRef.current)[0]?.focus()
    const onDocumentPointerDown = (event: MouseEvent) => {
      const target = event.target as Node
      if (menuPopRef.current?.contains(target)) return
      if (menuBtnRef.current?.contains(target)) return
      setMenuOpen(false)
    }
    document.addEventListener('mousedown', onDocumentPointerDown)
    return () => document.removeEventListener('mousedown', onDocumentPointerDown)
  }, [menuOpen])

  const onMenuKeydown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key === 'Escape') {
      event.preventDefault()
      event.stopPropagation()
      closeMenu(true)
      return
    }
    focusCycled(menuItemsOf(menuPopRef.current), event)
  }

  const onMenuItem = (action: 'editSource' | 'export' | 'reload') => {
    if (action === 'reload') onReload()
    else if (action === 'export') onExport()
    else onEditSource()
    closeMenu(true)
  }

  const paletteClass = paletteOpen ? 'cp-btn cp-btn--ghost cp-btn--palette-open' : 'cp-btn cp-btn--ghost'

  return (
    <PageHeader
      className="cp-header"
      title={labels.title}
      description={labels.subtitle}
      leading={
        <div className="cp-header__icon">
          <SIcon name={icon} size="w-5 h-5" />
        </div>
      }
      actions={
        <>
          <Link to={backTo} className="cp-header__back">
            <button type="button" className="cp-btn cp-btn--ghost">
              <SIcon name="ArrowLeft" size="w-3.5 h-3.5" />
              <span>{labels.back}</span>
            </button>
          </Link>

          {palette ? (
            <button
              type="button"
              className={paletteClass}
              disabled={loading}
              aria-pressed={paletteOpen}
              aria-haspopup="dialog"
              title={palette.title}
              onClick={onOpenPalette}
            >
              <SIcon name="Command" size="w-3.5 h-3.5" />
              <span>{palette.label}</span>
              <kbd className="cp-btn__kbd">{palette.shortcut}</kbd>
            </button>
          ) : null}

          <div className="cp-menu">
            <button
              ref={menuBtnRef}
              type="button"
              className="cp-btn cp-btn--ghost"
              disabled={loading}
              aria-expanded={menuOpen}
              aria-haspopup="menu"
              aria-label={labels.overflow ?? '···'}
              title={labels.overflow}
              onClick={() => setMenuOpen((open) => !open)}
            >
              <SIcon name="MenuDots" size="w-3.5 h-3.5" />
            </button>

            {menuOpen ? (
              <div
                ref={menuPopRef}
                className="cp-menu__pop"
                role="menu"
                aria-label={labels.overflow}
                onKeyDown={onMenuKeydown}
              >
                <button
                  type="button"
                  role="menuitem"
                  className="cp-menu__item"
                  disabled={loading}
                  onClick={() => onMenuItem('reload')}
                >
                  <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={loading ? 'cp-spin' : undefined} />
                  <span>{labels.reload}</span>
                </button>
                <button
                  type="button"
                  role="menuitem"
                  className="cp-menu__item"
                  disabled={exporting || loading}
                  onClick={() => onMenuItem('export')}
                >
                  <SIcon name="Download" size="w-3.5 h-3.5" />
                  <span>{labels.export}</span>
                </button>
                {labels.source ? (
                  <button
                    type="button"
                    role="menuitem"
                    className="cp-menu__item"
                    disabled={loading || sourceDisabled}
                    title={sourceTitle}
                    onClick={() => onMenuItem('editSource')}
                  >
                    <SIcon name="FileCode2" size="w-3.5 h-3.5" />
                    <span>{labels.source}</span>
                  </button>
                ) : null}
              </div>
            ) : null}
          </div>

          <button type="button" className="cp-btn cp-btn--primary" disabled={loading} onClick={onAdd}>
            <SIcon name="Plus" size="w-3.5 h-3.5" />
            <span>{labels.add}</span>
          </button>
        </>
      }
    />
  )
}
