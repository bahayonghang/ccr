import type { ButtonHTMLAttributes } from 'react'
import { cn } from './cn'

export type ButtonVariant =
  | 'primary'
  | 'secondary'
  | 'ghost'
  | 'quiet'
  | 'warning'
  | 'danger'
  | 'accent-soft'

export type ButtonSize = 'sm' | 'md'

interface ButtonClassOptions {
  variant?: ButtonVariant
  size?: ButtonSize
  className?: string
}

export function buttonClass({
  variant = 'secondary',
  size = 'md',
  className,
}: ButtonClassOptions = {}): string {
  return cn('ui-btn', `ui-btn--${variant}`, `ui-btn--${size}`, className)
}

export function Button({
  variant = 'secondary',
  size = 'md',
  className,
  type = 'button',
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant
  size?: ButtonSize
}) {
  return (
    <button
      type={type}
      className={buttonClass({ variant, size, className })}
      {...props}
    />
  )
}
