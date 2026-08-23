import { Command as CommandPrimitive } from 'cmdk'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Combobox（shadcn/ui 风格的 cmdk 包装）。`src/` 中 2 处手写
// role="listbox"/"option" + 过滤 + 键盘导航（adhoc-primitives.md C1/C2）由本原语
// 替换。筛选/高亮/键盘导航由 cmdk 提供；样式消费 CCR token 命名空间。

const Combobox = CommandPrimitive

type ComboboxInputProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Input>

const ComboboxInput = forwardRef<ElementRef<typeof CommandPrimitive.Input>, ComboboxInputProps>(
  ({ className, ...props }, ref) => (
    <CommandPrimitive.Input
      ref={ref}
      className={cn(
        'flex h-10 w-full rounded-lg border border-border-default/70 bg-bg-surface px-3 py-2 text-sm text-text-primary',
        'placeholder:text-text-ghost',
        'focus:outline-none focus:ring-2 focus:ring-accent-primary/24 focus:border-accent-primary/50',
        'disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...props}
    />
  ),
)
ComboboxInput.displayName = CommandPrimitive.Input.displayName

type ComboboxListProps = ComponentPropsWithoutRef<typeof CommandPrimitive.List>

const ComboboxList = forwardRef<ElementRef<typeof CommandPrimitive.List>, ComboboxListProps>(
  ({ className, ...props }, ref) => (
    <CommandPrimitive.List
      ref={ref}
      className={cn('max-h-60 overflow-y-auto overflow-x-hidden p-1', className)}
      {...props}
    />
  ),
)
ComboboxList.displayName = CommandPrimitive.List.displayName

type ComboboxEmptyProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Empty>

const ComboboxEmpty = forwardRef<ElementRef<typeof CommandPrimitive.Empty>, ComboboxEmptyProps>(
  ({ className, ...props }, ref) => (
    <CommandPrimitive.Empty
      ref={ref}
      className={cn('py-4 text-center text-sm text-text-muted', className)}
      {...props}
    />
  ),
)
ComboboxEmpty.displayName = CommandPrimitive.Empty.displayName

type ComboboxGroupProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Group>

const ComboboxGroup = forwardRef<ElementRef<typeof CommandPrimitive.Group>, ComboboxGroupProps>(
  ({ className, ...props }, ref) => (
    <CommandPrimitive.Group
      ref={ref}
      className={cn('overflow-hidden text-text-secondary', className)}
      {...props}
    />
  ),
)
ComboboxGroup.displayName = CommandPrimitive.Group.displayName

type ComboboxItemProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Item>

const ComboboxItem = forwardRef<ElementRef<typeof CommandPrimitive.Item>, ComboboxItemProps>(
  ({ className, ...props }, ref) => (
    <CommandPrimitive.Item
      ref={ref}
      className={cn(
        'relative flex cursor-default select-none items-center gap-2 rounded-lg px-2 py-1.5 text-sm outline-none',
        'text-text-secondary transition-colors',
        'data-[selected=true]:bg-bg-overlay/70 data-[selected=true]:text-text-primary',
        'data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50',
        className,
      )}
      {...props}
    />
  ),
)
ComboboxItem.displayName = CommandPrimitive.Item.displayName

type ComboboxSeparatorProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Separator>

const ComboboxSeparator = forwardRef<
  ElementRef<typeof CommandPrimitive.Separator>,
  ComboboxSeparatorProps
>(({ className, ...props }, ref) => (
  <CommandPrimitive.Separator
    ref={ref}
    className={cn('-mx-1 my-1 h-px bg-border-default/40', className)}
    {...props}
  />
))
ComboboxSeparator.displayName = CommandPrimitive.Separator.displayName

type ComboboxDialogProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Dialog>

const ComboboxDialog = forwardRef<ElementRef<typeof CommandPrimitive.Dialog>, ComboboxDialogProps>(
  ({ className, ...props }, ref) => (
    <CommandPrimitive.Dialog
      ref={ref}
      className={cn('text-text-primary', className)}
      {...props}
    />
  ),
)
ComboboxDialog.displayName = CommandPrimitive.Dialog.displayName

type ComboboxLoadingProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Loading>

const ComboboxLoading = forwardRef<
  ElementRef<typeof CommandPrimitive.Loading>,
  ComboboxLoadingProps
>(({ className, ...props }, ref) => (
  <CommandPrimitive.Loading
    ref={ref}
    className={cn('py-4 text-center text-sm text-text-muted', className)}
    {...props}
  />
))
ComboboxLoading.displayName = CommandPrimitive.Loading.displayName

export {
  Combobox,
  ComboboxDialog,
  ComboboxEmpty,
  ComboboxGroup,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
  ComboboxLoading,
  ComboboxSeparator,
}
