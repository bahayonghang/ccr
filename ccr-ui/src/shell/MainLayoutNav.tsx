import { NavLink } from 'react-router'
import { mainLayoutNavSections } from '@/config/mainLayoutShell'
import { SIcon } from '@/ui/s-icon'
import type { ShellTranslate } from './i18n'

interface MainLayoutNavProps {
  t: ShellTranslate
  onNavigate?: () => void
}

export function MainLayoutNav({ t, onNavigate }: MainLayoutNavProps) {
  return (
    <nav
      id="primary-navigation"
      className="scrollbar-hide flex-1 space-y-5 overflow-y-auto px-3 pt-3 pb-4"
      aria-label="Primary navigation"
      onClick={onNavigate}
    >
      {mainLayoutNavSections.map((section) => (
        <div key={section.id}>
          {section.titleKey ? (
            <div className="mb-2 flex items-center gap-2 px-3 text-[0.625rem] font-semibold tracking-[0.16em] text-text-muted">
              {t(section.titleKey)}
              <div className="h-px flex-1 bg-border-default/70" />
            </div>
          ) : null}
          <div className={section.titleKey ? 'space-y-0.5' : 'space-y-1'}>
            {section.items.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === '/'}
                className={({ isActive }) =>
                  [
                    'nav-item group',
                    item.to === '/' ? 'nav-item--root' : '',
                    isActive ? 'nav-item--active' : '',
                  ].join(' ')
                }
              >
                <SIcon name={item.icon} size="w-4 h-4" className={item.iconClass} />
                <span>{t(item.labelKey)}</span>
              </NavLink>
            ))}
          </div>
        </div>
      ))}
    </nav>
  )
}
