import * as DialogPrimitive from '@radix-ui/react-dialog'
import { forwardRef, type ComponentPropsWithoutRef, type ElementRef } from 'react'
import { cn } from './cn'

// Dialog（shadcn/ui 风格的 Radix 包装，08-22-design-system 批次 3）。
// 样式全部消费 CCR token 命名空间：scrim 遮罩 / surface-modal 悬浮材质 /
// radius / layer-z 层级。四项弹层行为（焦点陷阱、Esc、滚动锁定、层级）由 Radix
// 提供，batch 4（弹层收口）在其上包 BaseModal 适配器。

const Dialog = DialogPrimitive.Root
const DialogTrigger = DialogPrimitive.Trigger
const DialogPortal = DialogPrimitive.Portal
const DialogClose = DialogPrimitive.Close

type DialogOverlayProps = ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>

const DialogOverlay = forwardRef<ElementRef<typeof DialogPrimitive.Overlay>, DialogOverlayProps>(
  ({ className, ...props }, ref) => (
    <DialogPrimitive.Overlay
      ref={ref}
      className={cn(
        'fixed inset-0 z-[var(--layer-modal-backdrop)] bg-scrim backdrop-blur-md',
        'transition-opacity duration-normal',
        'data-[state=open]:opacity-100 data-[state=closed]:opacity-0',
        className,
      )}
      {...props}
    />
  ),
)
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName

type DialogContentProps = ComponentPropsWithoutRef<typeof DialogPrimitive.Content>

const DialogContent = forwardRef<ElementRef<typeof DialogPrimitive.Content>, DialogContentProps>(
  ({ className, children, ...props }, ref) => (
    <DialogPortal>
      <DialogOverlay />
      <DialogPrimitive.Content
        ref={ref}
        className={cn(
          'fixed left-1/2 top-1/2 z-[var(--layer-modal)] grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 p-6',
          'surface-modal overflow-hidden rounded-2xl',
          'transition-interactive duration-normal',
          'data-[state=open]:opacity-100 data-[state=closed]:opacity-0',
          'data-[state=open]:scale-100 data-[state=closed]:scale-95',
          className,
        )}
        {...props}
      >
        {children}
        <DialogPrimitive.Close
          aria-label="关闭"
          className={cn(
            'absolute right-4 top-4 inline-flex h-8 w-8 items-center justify-center rounded-md',
            'text-text-muted transition-colors hover:bg-bg-overlay/70 hover:text-text-primary',
            'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30',
          )}
        >
          <svg
            className="h-4 w-4"
            viewBox="0 0 15 15"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M11.7816 4.03157C12.0062 3.80702 12.0062 3.44295 11.7816 3.2184C11.557 2.99385 11.1929 2.99385 10.9684 3.2184L7.50005 6.68672L4.03164 3.2184C3.80708 2.99385 3.44301 2.99385 3.21846 3.2184C2.99391 3.44295 2.99391 3.80702 3.21846 4.03157L6.68688 7.49999L3.21846 10.9684C2.99391 11.1929 2.99391 11.557 3.21846 11.7816C3.44301 12.0061 3.80708 12.0061 4.03164 11.7816L7.50005 8.31327L10.9684 11.7816C11.1929 12.0061 11.557 12.0061 11.7816 11.7816C12.0062 11.557 12.0062 11.1929 11.7816 10.9684L8.31322 7.49999L11.7816 4.03157Z"
              fill="currentColor"
            />
          </svg>
        </DialogPrimitive.Close>
      </DialogPrimitive.Content>
    </DialogPortal>
  ),
)
DialogContent.displayName = DialogPrimitive.Content.displayName

function DialogHeader({ className, ...props }: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('flex flex-col gap-1.5', className)} {...props} />
}

function DialogFooter({ className, ...props }: ComponentPropsWithoutRef<'div'>) {
  return (
    <div
      className={cn('flex flex-col-reverse gap-2 sm:flex-row sm:justify-end', className)}
      {...props}
    />
  )
}

function DialogTitle({ className, ...props }: ComponentPropsWithoutRef<'h2'>) {
  return (
    <DialogPrimitive.Title
      className={cn('text-lg font-semibold text-text-primary', className)}
      {...props}
    />
  )
}

function DialogDescription({ className, ...props }: ComponentPropsWithoutRef<'p'>) {
  return (
    <DialogPrimitive.Description
      className={cn('text-sm leading-6 text-text-muted', className)}
      {...props}
    />
  )
}

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
}
