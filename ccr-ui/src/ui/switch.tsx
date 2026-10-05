import * as SwitchPrimitive from '@radix-ui/react-switch'
import { forwardRef, type ElementRef } from 'react'
import { cn } from './cn'

// Switch（shadcn/ui 风格的 Radix 包装）。轨道/拨钮语义对齐
// `src/views/AppSettingsView.vue` 的手写 role="switch"（.app-settings-switch）：
// 未选中 = bg-bg-overlay 轨道，选中 = accent-primary/30 轨道，拨钮 = bg-bg-surface。

const Switch = forwardRef<ElementRef<typeof SwitchPrimitive.Root>, SwitchPrimitive.SwitchProps>(
  ({ className, ...props }, ref) => (
    <SwitchPrimitive.Root
      ref={ref}
      className={cn(
        'peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent',
        'transition-colors duration-fast',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'data-[state=checked]:bg-accent-primary/30 data-[state=unchecked]:bg-bg-overlay',
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          'pointer-events-none block h-4 w-4 rounded-full border border-border-strong bg-bg-surface shadow-sm ring-0',
          'transition-transform duration-fast',
          'data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0',
        )}
      />
    </SwitchPrimitive.Root>
  ),
)
Switch.displayName = SwitchPrimitive.Root.displayName

export { Switch }
