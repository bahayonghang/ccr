import { useCallback, useEffect, useMemo, useState } from 'react'
import { NavLink, useLocation } from 'react-router'
import { getCurrentEnvironment } from '@/api'
import { getModuleSubnavItems } from '@/config/moduleSubnav'
import { t } from '@/features/claude/locale'
import { SIcon } from '@/ui'

interface ClaudeSubnavProps {
  module?: string
}

interface SubnavItemView {
  href: string
  label: string
  icon: string
  disabled: boolean
}

const SubnavLink = ({ item, active }: { item: SubnavItemView; active: boolean }) => {
  const className = active
    ? 'module-subnav__item module-subnav__item--active'
    : 'module-subnav__item'
  return (
    <NavLink to={item.href} className={className} aria-current={active ? 'page' : undefined}>
      <SIcon name={item.icon} size="w-4 h-4" />
      <span>{item.label}</span>
    </NavLink>
  )
}

/** Claude 模块子导航。不导入 shell，复用全局 `.module-subnav` 样式。 */
export function ClaudeSubnav({ module = 'claude-code' }: ClaudeSubnavProps) {
  const location = useLocation()
  const [isLocal, setIsLocal] = useState(false)

  const loadEnvironment = useCallback(() => {
    const needsLocal = getModuleSubnavItems(module).some((item) => item.localOnly)
    if (!needsLocal) {
      setIsLocal(true)
      return
    }
    void getCurrentEnvironment()
      .then((environment) => {
        setIsLocal(!environment || environment.env_type === 'local')
      })
      .catch(() => setIsLocal(false))
  }, [module])

  useEffect(() => {
    loadEnvironment()
  }, [loadEnvironment])

  const items = useMemo<SubnavItemView[]>(() => {
    return getModuleSubnavItems(module).map((item) => ({
      href: item.href,
      label: item.labelKey ? t(item.labelKey) : item.label,
      icon: item.icon,
      disabled: Boolean(item.localOnly && !isLocal),
    }))
  }, [isLocal, module])

  const activeHref = useMemo(() => {
    return items
      .map((item) => item.href)
      .filter((href) => location.pathname === href || location.pathname.startsWith(`${href}/`))
      .sort((left, right) => right.length - left.length)[0]
  }, [items, location.pathname])

  if (items.length === 0) return null

  return (
    <nav className="module-subnav" aria-label="Module navigation">
      {items.map((item) =>
        item.disabled ? (
          <span
            key={item.href}
            className="module-subnav__item module-subnav__item--disabled"
            title={t('settingsRaw.unsupportedEnvironment')}
            aria-disabled="true"
          >
            <SIcon name={item.icon} size="w-4 h-4" />
            <span>{item.label}</span>
          </span>
        ) : (
          <SubnavLink key={item.href} item={item} active={activeHref === item.href} />
        ),
      )}
    </nav>
  )
}
