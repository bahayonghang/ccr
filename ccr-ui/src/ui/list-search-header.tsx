import type { ReactNode } from 'react'
import { cn } from './cn'
import { SIcon } from './s-icon'

interface ListSearchHeaderProps {
  searchValue: string
  onSearchValueChange: (value: string) => void
  placeholder?: string
  label?: string
  children?: ReactNode
  className?: string
}

export function ListSearchHeader({
  searchValue,
  onSearchValueChange,
  placeholder = 'Search...',
  label,
  children,
  className,
}: ListSearchHeaderProps) {
  return (
    <div className={cn('list-search-header', className)}>
      <div className="list-search-header__search">
        <SIcon name="Search" size="w-4 h-4" className="text-text-muted" />
        <input
          value={searchValue}
          type="text"
          className="list-search-header__input"
          placeholder={placeholder}
          aria-label={label || placeholder}
          onChange={(event) => onSearchValueChange(event.target.value)}
        />
      </div>
      <div className="list-search-header__actions">{children}</div>
    </div>
  )
}
