import { NavLink } from 'react-router'
import { mainLayoutNavSections } from '@/config/mainLayoutShell'
import { SIcon } from '@/ui/s-icon'
import type { ShellTranslate } from './i18n'

interface MainLayoutNavProps {
  t: ShellTranslate
  onNavigate?: () => void
}

const PLATFORM_NAV_SWATCH: Record<string, string> = {
  '/claude-code': 'bg-platform-claude',
  '/codex': 'bg-platform-codex',
  '/grok': 'bg-platform-grok',
  '/antigravity': 'bg-platform-gemini',
  '/opencode': 'bg-[var(--color-platform-opencode)]',
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
            <div className="sidebar-nav__label">
              {t(section.titleKey)}
              <span className="sidebar-nav__label-rule" aria-hidden="true" />
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
                {PLATFORM_NAV_SWATCH[item.to] ? (
                  <span
                    className={`sidebar-nav__swatch ${PLATFORM_NAV_SWATCH[item.to]}`}
                    aria-hidden="true"
                  />
                ) : null}
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
