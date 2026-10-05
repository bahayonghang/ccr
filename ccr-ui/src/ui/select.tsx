import * as SelectPrimitive from '@radix-ui/react-select'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Select（shadcn/ui 风格的 Radix 包装）。`src/` 中 20 处原生 <select>（迁移期）由
// 本原语替换。层级 z-popover（Radix Select 是 popper 定位的列表弹层）。

const Select = SelectPrimitive.Root
const SelectGroup = SelectPrimitive.Group
const SelectValue = SelectPrimitive.Value

type SelectTriggerProps = ComponentPropsWithoutRef<typeof SelectPrimitive.Trigger>

const SelectTrigger = forwardRef<ElementRef<typeof SelectPrimitive.Trigger>, SelectTriggerProps>(
  ({ className, children, ...props }, ref) => (
    <SelectPrimitive.Trigger
      ref={ref}
      className={cn(
        'flex h-9 w-full items-center justify-between whitespace-nowrap rounded-lg border border-border-default/70 bg-bg-surface px-3 py-2 text-sm text-text-primary',
        'placeholder:text-text-ghost',
        'focus:outline-none focus:ring-2 focus:ring-accent-primary/24 focus:border-accent-primary/50',
        'disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...props}
    >
      {children}
      <SelectPrimitive.Icon asChild>
        <svg className="ml-2 h-3.5 w-3.5 text-text-muted" viewBox="0 0 15 15" fill="none" aria-hidden="true">
          <path
            d="M3.13523 6.15803C3.3241 5.95657 3.64052 5.94637 3.84198 6.13523L7.5 9.56464L11.158 6.13523C11.3595 5.94637 11.6759 5.95657 11.8648 6.15803C12.0536 6.35949 12.0434 6.67591 11.842 6.86477L7.84198 10.6148C7.64964 10.7951 7.35036 10.7951 7.15803 10.6148L3.15803 6.86477C2.95657 6.67591 2.94637 6.35949 3.13523 6.15803Z"
            fill="currentColor"
          />
        </svg>
      </SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
  ),
)
SelectTrigger.displayName = SelectPrimitive.Trigger.displayName

type SelectContentProps = ComponentPropsWithoutRef<typeof SelectPrimitive.Content>

const SelectContent = forwardRef<ElementRef<typeof SelectPrimitive.Content>, SelectContentProps>(
  ({ className, children, position = 'popper', ...props }, ref) => (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Content
        ref={ref}
        position={position}
        className={cn(
          'relative z-[var(--layer-popover)] max-h-64 min-w-32 overflow-hidden rounded-xl surface-modal p-1',
          'transition-interactive duration-normal',
          'data-[state=open]:opacity-100 data-[state=closed]:opacity-0',
          'data-[state=open]:scale-100 data-[state=closed]:scale-95',
          position === 'popper' && 'data-[side=bottom]:translate-y-1',
          className,
        )}
        {...props}
      >
        <SelectPrimitive.ScrollUpButton className="flex cursor-default items-center justify-center py-1 text-text-muted">
          <svg className="h-3.5 w-3.5" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <path
              d="M7.84182 3.13508C7.64964 2.94621 7.35036 2.94621 7.15803 3.13508L3.15803 7.13508C2.95657 7.32394 2.94637 7.64036 3.13523 7.84182C3.3241 8.04328 3.64052 8.05348 3.84198 7.86462L7.5 4.20921L11.158 7.86462C11.3595 8.05348 11.6759 8.04328 11.8648 7.84182C12.0536 7.64036 12.0434 7.32394 11.842 7.13508L7.84182 3.13508Z"
              fill="currentColor"
            />
          </svg>
        </SelectPrimitive.ScrollUpButton>
        <SelectPrimitive.Viewport
          className={cn('p-1', position === 'popper' && 'h-[var(--radix-select-trigger-height)] w-full min-w-[var(--radix-select-trigger-width)]')}
        >
          {children}
        </SelectPrimitive.Viewport>
        <SelectPrimitive.ScrollDownButton className="flex cursor-default items-center justify-center py-1 text-text-muted">
          <svg className="h-3.5 w-3.5" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <path
              d="M3.13523 6.15803C3.3241 5.95657 3.64052 5.94637 3.84198 6.13523L7.5 9.56464L11.158 6.13523C11.3595 5.94637 11.6759 5.95657 11.8648 6.15803C12.0536 6.35949 12.0434 6.67591 11.842 6.86477L7.84198 10.6148C7.64964 10.7951 7.35036 10.7951 7.15803 10.6148L3.15803 6.86477C2.95657 6.67591 2.94637 6.35949 3.13523 6.15803Z"
              fill="currentColor"
            />
          </svg>
        </SelectPrimitive.ScrollDownButton>
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  ),
)
SelectContent.displayName = SelectPrimitive.Content.displayName

type SelectLabelProps = ComponentPropsWithoutRef<typeof SelectPrimitive.Label>

function SelectLabel({ className, ...props }: SelectLabelProps) {
  return (
    <SelectPrimitive.Label
      className={cn('px-2 py-1.5 text-xs font-medium text-text-muted', className)}
      {...props}
    />
  )
}

type SelectItemProps = ComponentPropsWithoutRef<typeof SelectPrimitive.Item>

const SelectItem = forwardRef<ElementRef<typeof SelectPrimitive.Item>, SelectItemProps>(
  ({ className, children, ...props }, ref) => (
    <SelectPrimitive.Item
      ref={ref}
      className={cn(
        'relative flex w-full cursor-default select-none items-center rounded-lg py-1.5 pl-8 pr-2 text-sm outline-none',
        'text-text-secondary transition-colors',
        'focus:bg-bg-overlay/70 focus:text-text-primary',
        'data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        className,
      )}
      {...props}
    >
      <span className="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
        <SelectPrimitive.ItemIndicator>
          <svg className="h-3.5 w-3.5" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <path
              d="M11.4669 3.72684C11.7558 3.91574 11.8393 4.30308 11.6504 4.59198L7.8259 10.6062C7.62793 10.9089 7.21809 10.9644 6.95058 10.7298L3.79323 7.93758C3.52937 7.70704 3.50207 7.28787 3.73261 7.02401C3.96315 6.76015 4.38232 6.73285 4.64618 6.96339L7.39592 9.36571L10.8004 3.57223C10.9893 3.28333 11.3766 3.19981 11.6655 3.38871L11.4669 3.72684Z"
              fill="currentColor"
            />
          </svg>
        </SelectPrimitive.ItemIndicator>
      </span>
      <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
    </SelectPrimitive.Item>
  ),
)
SelectItem.displayName = SelectPrimitive.Item.displayName

type SelectSeparatorProps = ComponentPropsWithoutRef<typeof SelectPrimitive.Separator>

function SelectSeparator({ className, ...props }: SelectSeparatorProps) {
  return (
    <SelectPrimitive.Separator
      className={cn('-mx-1 my-1 h-px bg-border-default/40', className)}
      {...props}
    />
  )
}

export {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
}
