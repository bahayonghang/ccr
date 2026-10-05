import { cn } from './cn'

export interface PillToggleOption<TValue extends string | number = string> {
  value: TValue
  label: string
  disabled?: boolean
}

interface PillToggleGroupProps<TValue extends string | number> {
  options: PillToggleOption<TValue>[]
  value?: TValue
  onValueChange?: (value: TValue) => void
  ariaLabel?: string
  className?: string
}

export function PillToggleGroup<TValue extends string | number>({
  options,
  value,
  onValueChange,
  ariaLabel,
  className,
}: PillToggleGroupProps<TValue>) {
  return (
    <div className={cn('pill-toggle-group', className)} role="radiogroup" aria-label={ariaLabel}>
      {options.map((option) => {
        const active = value === option.value
        return (
          <button
            key={String(option.value)}
            type="button"
            className={cn('pill-toggle-group__item', active && 'pill-toggle-group__item--active')}
            role="radio"
            aria-checked={active}
            disabled={option.disabled}
            onClick={() => {
              if (option.value === value) return
              onValueChange?.(option.value)
            }}
          >
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
