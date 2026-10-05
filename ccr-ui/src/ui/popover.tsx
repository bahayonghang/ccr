import * as PopoverPrimitive from '@radix-ui/react-popover'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Popover（shadcn/ui 风格的 Radix 包装）。浮层材质消费 surface-modal（玻璃悬浮
// 材质），层级 z-popover，圆角/间距走 token 工具类。

const Popover = PopoverPrimitive.Root
const PopoverTrigger = PopoverPrimitive.Trigger
const PopoverAnchor = PopoverPrimitive.Anchor

type PopoverContentProps = ComponentPropsWithoutRef<typeof PopoverPrimitive.Content>

const PopoverContent = forwardRef<
  ElementRef<typeof PopoverPrimitive.Content>,
  PopoverContentProps
>(({ className, align = 'center', sideOffset = 4, ...props }, ref) => (
  <PopoverPrimitive.Portal>
    <PopoverPrimitive.Content
      ref={ref}
      align={align}
      sideOffset={sideOffset}
      className={cn(
        'z-[var(--layer-popover)] w-72 rounded-xl surface-modal p-4',
        'outline-none',
        'transition-interactive duration-normal',
        'data-[state=open]:opacity-100 data-[state=closed]:opacity-0',
        'data-[state=open]:scale-100 data-[state=closed]:scale-95',
        className,
      )}
      {...props}
    />
  </PopoverPrimitive.Portal>
))
PopoverContent.displayName = PopoverPrimitive.Content.displayName

export { Popover, PopoverAnchor, PopoverContent, PopoverTrigger }
