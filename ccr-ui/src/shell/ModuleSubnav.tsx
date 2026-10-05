import { useEffect, useState } from 'react'
import { NavLink, useLocation } from 'react-router'
import { getCurrentEnvironment } from '@/api'
import { getModuleSubnavItems } from '@/config/moduleSubnav'
import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui/s-icon'

interface ModuleSubnavProps {
  module: string
}

export function ModuleSubnav({ module }: ModuleSubnavProps) {
  const t = useShellT()
  const location = useLocation()
  const items = getModuleSubnavItems(module)
  const [isLocalEnvironment, setIsLocalEnvironment] = useState(false)

  useEffect(() => {
    const needsLocal = getModuleSubnavItems(module).some((item) => item.localOnly)
    if (!needsLocal) {
      setIsLocalEnvironment(true)
      return
    }
    void getCurrentEnvironment()
      .then((environment) => {
        setIsLocalEnvironment(!environment || environment.env_type === 'local')
      })
      .catch(() => setIsLocalEnvironment(false))
  }, [module])

  const activeHref = items
    .map((item) => item.href)
    .filter((href) => location.pathname === href || location.pathname.startsWith(`${href}/`))
    .sort((left, right) => right.length - left.length)[0]

  if (items.length === 0) return null

  return (
    <nav className="module-subnav" aria-label="Module navigation">
      {items.map((item) => {
        const label = item.labelKey ? t(item.labelKey) : item.label
        if (item.localOnly && !isLocalEnvironment) {
          return (
            <span
              key={item.href}
              className="module-subnav__item module-subnav__item--disabled"
              title={t('settingsRaw.unsupportedEnvironment')}
              aria-disabled="true"
            >
              <SIcon name={item.icon} size="w-4 h-4" />
              <span>{label}</span>
            </span>
          )
        }
        return (
          <NavLink
            key={item.href}
            to={item.href}
            className={
              activeHref === item.href
                ? 'module-subnav__item module-subnav__item--active'
                : 'module-subnav__item'
            }
            aria-current={activeHref === item.href ? 'page' : undefined}
          >
            <SIcon name={item.icon} size="w-4 h-4" />
            <span>{label}</span>
          </NavLink>
        )
      })}
    </nav>
  )
}
