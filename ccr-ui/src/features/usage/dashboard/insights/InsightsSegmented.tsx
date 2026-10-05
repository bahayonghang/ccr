import { useCallback, useRef, type KeyboardEvent, type MouseEvent } from 'react'

export interface SegmentOption<T extends string> {
  value: T
  label: string
  disabled?: boolean
  title?: string
}

interface InsightsSegmentedProps<T extends string> {
  options: SegmentOption<T>[]
  value: T
  onChange: (value: T) => void
  ariaLabel: string
}

const NEXT_KEYS: Record<string, number> = {
  ArrowRight: 1,
  ArrowDown: 1,
  ArrowLeft: -1,
  ArrowUp: -1,
}

// 分段控件（role=radiogroup）：方向键移动选中，roving tabindex，禁用项带 title 说明。
export function InsightsSegmented<T extends string>({
  options,
  value,
  onChange,
  ariaLabel,
}: InsightsSegmentedProps<T>) {
  const groupRef = useRef<HTMLDivElement>(null)

  const handleClick = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      const option = options.find((item) => item.value === event.currentTarget.dataset.value)
      if (option && !option.disabled && option.value !== value) onChange(option.value)
    },
    [onChange, options, value]
  )

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLDivElement>) => {
      const step = NEXT_KEYS[event.key]
      if (!step) return
      event.preventDefault()
      const enabled = options.filter((item) => !item.disabled)
      const current = enabled.findIndex((item) => item.value === value)
      const next = enabled[(current + step + enabled.length) % enabled.length]
      if (!next || next.value === value) return
      onChange(next.value)
      const button = groupRef.current?.querySelector<HTMLButtonElement>(
        `[data-value="${next.value}"]`
      )
      button?.focus()
    },
    [onChange, options, value]
  )

  return (
    <div
      ref={groupRef}
      className="insights-segmented"
      role="radiogroup"
      aria-label={ariaLabel}
      onKeyDown={handleKeyDown}
    >
      {options.map((option) => {
        const active = option.value === value
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={active}
            tabIndex={active ? 0 : -1}
            disabled={option.disabled}
            title={option.title}
            data-value={option.value}
            className="insights-segmented__item"
            onClick={handleClick}
          >
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
