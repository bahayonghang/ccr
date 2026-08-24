import { Icon } from '@iconify/react/offline'
import { iconMap } from '@/config/icons'
import { cn } from './cn'

interface SIconProps {
  name: string
  size?: string
  className?: string
}

/** 全站图标入口。语义名走 iconMap，否则当作 Iconify id。 */
export function SIcon({ name, size = 'w-4 h-4', className }: SIconProps) {
  const iconId = (iconMap as Record<string, string>)[name] ?? name
  return <Icon icon={iconId} className={cn(size, className)} />
}
