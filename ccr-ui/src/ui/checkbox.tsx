import * as CheckboxPrimitive from '@radix-ui/react-checkbox'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Checkbox（shadcn/ui 风格的 Radix 包装）。`src/` 中 20 处原生 type="checkbox"
// （迁移期）由本原语替换。选中态用 accent-primary，边界用 border-strong。

const Checkbox = forwardRef<
  ElementRef<typeof CheckboxPrimitive.Root>,
  ComponentPropsWithoutRef<typeof CheckboxPrimitive.Root>
>(({ className, ...props }, ref) => (
  <CheckboxPrimitive.Root
    ref={ref}
    className={cn(
      'peer h-4 w-4 shrink-0 rounded border border-border-strong bg-bg-surface',
      'transition-colors duration-fast',
      'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base',
      'disabled:cursor-not-allowed disabled:opacity-50',
      'data-[state=checked]:border-accent-primary data-[state=checked]:bg-accent-primary data-[state=checked]:text-text-inverted',
      className,
    )}
    {...props}
  >
    <CheckboxPrimitive.Indicator className="flex items-center justify-center text-current">
      <svg className="h-3 w-3" viewBox="0 0 15 15" fill="none" aria-hidden="true">
        <path
          d="M11.4669 3.72684C11.7558 3.91574 11.8393 4.30308 11.6504 4.59198L7.8259 10.6062C7.62793 10.9089 7.21809 10.9644 6.95058 10.7298L3.79323 7.93758C3.52937 7.70704 3.50207 7.28787 3.73261 7.02401C3.96315 6.76015 4.38232 6.73285 4.64618 6.96339L7.39592 9.36571L10.8004 3.57223C10.9893 3.28333 11.3766 3.19981 11.6655 3.38871L11.4669 3.72684Z"
          fill="currentColor"
        />
      </svg>
    </CheckboxPrimitive.Indicator>
  </CheckboxPrimitive.Root>
))
Checkbox.displayName = CheckboxPrimitive.Root.displayName

export { Checkbox }
