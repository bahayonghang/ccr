import { formatBaseUrlDisplay } from '@/utils/text'
import { cn } from './cn'

interface UrlTextProps {
  value: string
  className?: string
}

export function UrlText({ value, className }: UrlTextProps) {
  return (
    <span className={cn('ui-url-text', className)} title={value}>
      {formatBaseUrlDisplay(value)}
    </span>
  )
}
