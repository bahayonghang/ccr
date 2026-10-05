import { useMemo } from 'react'

interface SparklineProps {
  values: number[]
  width?: number
  height?: number
  strokeWidth?: number
  fill?: string
  label?: string
  className?: string
}

/** SVG sparkline；配色走 currentColor / chart-colors token。 */
export function Sparkline({
  values,
  width = 120,
  height = 38,
  strokeWidth = 2.4,
  fill = 'currentColor',
  label,
  className,
}: SparklineProps) {
  const path = useMemo(() => {
    if (values.length === 0) return ''
    const max = Math.max(...values, 0)
    const min = Math.min(...values, 0)
    const span = Math.max(max - min, 1)
    const step = values.length > 1 ? width / (values.length - 1) : width

    return values
      .map((value, index) => {
        const x = index * step
        const y = height - ((value - min) / span) * height
        return `${index === 0 ? 'M' : 'L'}${x.toFixed(2)} ${y.toFixed(2)}`
      })
      .join(' ')
  }, [height, values, width])

  return (
    <svg
      className={className}
      viewBox={`0 0 ${width} ${height}`}
      width="100%"
      height="100%"
      role="img"
      aria-label={label}
      preserveAspectRatio="none"
    >
      <path
        d={path}
        fill="none"
        stroke={fill}
        strokeWidth={strokeWidth}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  )
}
