import * as TooltipPrimitive from '@radix-ui/react-tooltip'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Tooltip（shadcn/ui 风格的 Radix 包装）。层级 z-tooltip，浮层材质 surface-modal
// （玻璃悬浮材质）。`src/` 中 2 处图表自绘浮层 + 117 处原生 :title 的交互场景
// （adhoc-primitives.md F1/F2）由本原语承载。

const TooltipProvider = TooltipPrimitive.Provider
const Tooltip = TooltipPrimitive.Root
const TooltipTrigger = TooltipPrimitive.Trigger

type TooltipContentProps = ComponentPropsWithoutRef<typeof TooltipPrimitive.Content>

const TooltipContent = forwardRef<
  ElementRef<typeof TooltipPrimitive.Content>,
  TooltipContentProps
>(({ className, sideOffset = 6, ...props }, ref) => (
  <TooltipPrimitive.Portal>
    <TooltipPrimitive.Content
      ref={ref}
      sideOffset={sideOffset}
      className={cn(
        'z-[var(--layer-tooltip)] rounded-md surface-modal px-2.5 py-1.5 text-xs text-text-primary',
        'transition-opacity duration-fast',
        'data-[state=delayed-open]:opacity-100 data-[state=closed]:opacity-0',
        className,
      )}
      {...props}
    />
  </TooltipPrimitive.Portal>
))
TooltipContent.displayName = TooltipPrimitive.Content.displayName

export { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger }
