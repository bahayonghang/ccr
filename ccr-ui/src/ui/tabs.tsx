import * as TabsPrimitive from '@radix-ui/react-tabs'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Tabs（shadcn/ui 风格的 Radix 包装）。`src/` 中 11 处手写 tablist（adhoc-primitives.md
// T1–T11）由本原语替换。键盘导航 / 激活态由 Radix 提供。

const Tabs = TabsPrimitive.Root

type TabsListProps = ComponentPropsWithoutRef<typeof TabsPrimitive.List>

const TabsList = forwardRef<ElementRef<typeof TabsPrimitive.List>, TabsListProps>(
  ({ className, ...props }, ref) => (
    <TabsPrimitive.List
      ref={ref}
      className={cn(
        'inline-flex h-9 items-center justify-center gap-1 rounded-lg bg-bg-elevated p-1 text-text-muted',
        className,
      )}
      {...props}
    />
  ),
)
TabsList.displayName = TabsPrimitive.List.displayName

type TabsTriggerProps = ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>

const TabsTrigger = forwardRef<ElementRef<typeof TabsPrimitive.Trigger>, TabsTriggerProps>(
  ({ className, ...props }, ref) => (
    <TabsPrimitive.Trigger
      ref={ref}
      className={cn(
        'inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium',
        'text-text-muted transition-colors',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30',
        'disabled:pointer-events-none disabled:opacity-50',
        'data-[state=active]:bg-bg-surface data-[state=active]:text-text-primary data-[state=active]:shadow-sm',
        className,
      )}
      {...props}
    />
  ),
)
TabsTrigger.displayName = TabsPrimitive.Trigger.displayName

type TabsContentProps = ComponentPropsWithoutRef<typeof TabsPrimitive.Content>

const TabsContent = forwardRef<ElementRef<typeof TabsPrimitive.Content>, TabsContentProps>(
  ({ className, ...props }, ref) => (
    <TabsPrimitive.Content
      ref={ref}
      className={cn(
        'mt-2 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30',
        className,
      )}
      {...props}
    />
  ),
)
TabsContent.displayName = TabsPrimitive.Content.displayName

export { Tabs, TabsContent, TabsList, TabsTrigger }
