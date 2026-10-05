import * as DialogPrimitive from '@radix-ui/react-dialog'
import {
  forwardRef,
  useCallback,
  useEffect,
  useId,
  useImperativeHandle,
  useRef,
  type ReactNode,
  type Ref,
} from 'react'
import { cn } from './cn'
import { DialogDescription, DialogOverlay, DialogTitle } from './dialog'

// BaseModal React 适配器（08-22-design-system 批次 4，弹层收口）。
//
// 定位：design.md §7 —— Dialog（Radix）是唯一弹层底座，四项行为（焦点陷阱、
// Esc 关闭、滚动锁定、层级）只有一处实现。本适配器保留 BaseModal.vue 的 API
// 形态，使 33 个调用点在阶段 5 视图迁移时改动面最小；适配器本身不含任何
// 弹层行为实现，全部委托给 Radix：
//
//   - 焦点陷阱 / 焦点保存与恢复 / 首元素聚焦 → Radix Dialog（modal 模式）
//   - 滚动锁定（body scroll lock）           → Radix（react-remove-scroll）
//   - Esc 关闭                               → DialogPrimitive.Content 的
//     onEscapeKeyDown（closeOnEscape=false 或 persistent 时 preventDefault）
//   - 层级                                   → dialog.tsx 的 token z 类
//
// 唯一在适配器内实现的行为是遮罩点击关闭的 6px 拖拽阈值（BaseModal.vue 原有
// 语义）：Radix Dialog 1.1.x 以 deferPointerDownOutside 把外部判定推迟到 click
// 且 Content 无 onPointerUpOutside prop，故打开期间用 document 级监听记录
// pointerdown/pointerup 坐标，onPointerDownOutside（preventDefault 接管关闭）
// 时按两段位移决定是否关闭。这不属弹层四项行为的重复实现。
//
// Vue → React API 映射（阶段 5 迁移查表）：
//   - `modelValue: boolean`                 → 同名 prop（受控）
//   - `@update:modelValue`                  → `onUpdateModelValue(value)`
//   - `@close`                              → `onClose()`（任一路径关闭时触发）
//   - `@open`（进入动画后）                 → `onOpened()`（Radix 无 enter 结束
//     回调，用打开后的下一帧近似；jsdom/真实浏览器均可用）
//   - `#header` 具名插槽（作用域 { titleId }）→ `header` prop：ReactNode 或
//     渲染函数 `(scope: { titleId: string }) => ReactNode`
//   - 默认插槽（正文）                      → `children`
//   - `#footer` 具名插槽                    → `footer` prop（ReactNode）
//   - `close()` 暴露方法                    → ref 句柄 `BaseModalHandle.close()`
//   - 其余 props（title/description/size/scrollable/showClose/closeOnBackdrop/
//     closeOnEscape/persistent/surface/contentClass）→ 同名同语义 prop

/** 尺寸档位，与 BaseModal.vue 的 sizeClasses 一一对应。 */
export type BaseModalSize =
  | 'sm'
  | 'md'
  | 'lg'
  | 'xl'
  | '2xl'
  | '3xl'
  | '4xl'
  | '5xl'
  | 'full'

/** ref 句柄：对应 BaseModal.vue 暴露的 `close()`。 */
export interface BaseModalHandle {
  close: () => void
}

export interface BaseModalProps {
  /** 受控开关（对应 Vue 的 v-model）。 */
  modelValue: boolean
  /** 关闭请求（含 Esc / 遮罩 / 关闭按钮 / ref.close）。 */
  onUpdateModelValue?: (value: boolean) => void
  /** 关闭时触发（与 onUpdateModelValue(false) 同步）。 */
  onClose?: () => void
  /** 打开进入后的回调（近似 Vue 的 @open / after-enter）。 */
  onOpened?: () => void
  /** 标题文本；与 `header` 二选一或并用。 */
  title?: string
  /** 副标题文本，接线 aria-describedby。 */
  description?: string
  /** 尺寸档位，默认 'md'。 */
  size?: BaseModalSize
  /** 固定头/脚 + 主体滚动的布局，默认 false。 */
  scrollable?: boolean
  /** 右上角关闭按钮，默认 true。 */
  showClose?: boolean
  /** 点击遮罩是否关闭（受 6px 拖拽阈值约束），默认 true。 */
  closeOnBackdrop?: boolean
  /** Esc 是否关闭，默认 true。 */
  closeOnEscape?: boolean
  /** 阻断全部关闭路径，默认 false。 */
  persistent?: boolean
  /** 面板材质，默认 'glass'。 */
  surface?: 'glass' | 'solid'
  /** 透传到面板的追加类名。 */
  contentClass?: string
  /** 头部内容：ReactNode 或渲染函数（作用域提供 titleId 供自行接线 aria）。 */
  header?: ReactNode | ((scope: { titleId: string }) => ReactNode)
  /** 底部内容（对应 #footer 插槽）。 */
  footer?: ReactNode
  /** 正文内容（对应默认插槽）。 */
  children?: ReactNode
}

// 尺寸 → 工具类映射（沿用 BaseModal.vue 的档位，全部为 token 化工具类）。
const SIZE_CLASSES: Record<BaseModalSize, string> = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  '2xl': 'max-w-2xl',
  '3xl': 'max-w-3xl',
  '4xl': 'max-w-4xl',
  '5xl': 'max-w-5xl',
  full: 'max-w-[calc(100vw-2rem)] max-h-[calc(100vh-2rem)]',
}

// 材质：glass 走 surfaces.css 的 .surface-modal（--surface-modal-* 四件套）；
// solid 用不透明面板覆盖（与 BaseModal.vue 的 surfaceClasses 等价）。
const SURFACE_CLASSES: Record<NonNullable<BaseModalProps['surface']>, string> = {
  glass: 'surface-modal',
  solid:
    '!bg-bg-elevated !backdrop-blur-none !border-border-default shadow-2xl shadow-black/10 dark:shadow-black/40',
}

// 遮罩点击的拖拽阈值：pointerdown/up 位移超过该值视为拖拽，不关闭。
const DRAG_THRESHOLD = 6

type PointerPoint = { x: number; y: number }

// 遮罩点击判定：两段坐标均已记录且位移 ≤ 阈值视为点击；无坐标记录（如
// 非 pointer 路径触发的 dismiss）按点击处理。
function isBackdropClick(down: PointerPoint | null, up: PointerPoint | null): boolean {
  if (down && up) {
    return Math.hypot(up.x - down.x, up.y - down.y) <= DRAG_THRESHOLD
  }
  return true
}

interface ModalHeaderProps {
  title?: string
  description?: string
  header?: ReactNode | ((scope: { titleId: string }) => ReactNode)
  titleId: string
  descriptionId: string
  showClose: boolean
  scrollable: boolean
}

function ModalHeader({
  title,
  description,
  header,
  titleId,
  descriptionId,
  showClose,
  scrollable,
}: ModalHeaderProps) {
  return (
    <div
      className={cn(
        'relative px-6 pt-6 pb-4',
        showClose && 'pr-12',
        scrollable && 'shrink-0',
      )}
    >
      {typeof header === 'function' ? header({ titleId }) : header}
      {title !== undefined && <DialogTitle id={titleId}>{title}</DialogTitle>}
      {description !== undefined && (
        <DialogDescription id={descriptionId}>{description}</DialogDescription>
      )}
    </div>
  )
}

interface ModalBodyProps {
  scrollable: boolean
  hasFooter: boolean
  children?: ReactNode
}

function ModalBody({ scrollable, hasFooter, children }: ModalBodyProps) {
  return (
    <div
      className={cn(
        'px-6 py-2',
        hasFooter ? 'pb-4' : 'pb-6',
        scrollable && 'min-h-0 flex-1 overflow-y-auto',
      )}
    >
      {children}
    </div>
  )
}

interface ModalFooterBarProps {
  scrollable: boolean
  children?: ReactNode
}

function ModalFooterBar({ scrollable, children }: ModalFooterBarProps) {
  return (
    <div
      className={cn(
        'flex items-center justify-end gap-3 border-t border-border-default px-6 py-4',
        scrollable && 'shrink-0',
      )}
    >
      {children}
    </div>
  )
}

function ModalCloseButton() {
  return (
    <DialogPrimitive.Close
      aria-label="关闭"
      className={cn(
        'absolute right-4 top-4 inline-flex h-8 w-8 items-center justify-center rounded-md',
        'text-text-muted transition-colors hover:bg-bg-overlay/70 hover:text-text-primary',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30',
      )}
    />
  )
}

/**
 * BaseModal 适配器。四项弹层行为的唯一实现点在 Radix / dialog.tsx，
 * 本组件只做 API 翻译与拖拽阈值判定。
 */
export const BaseModal = forwardRef<BaseModalHandle, BaseModalProps>(
  function BaseModal(props, ref) {
    const {
      modelValue,
      onUpdateModelValue,
      onClose,
      onOpened,
      title,
      description,
      size = 'md',
      scrollable = false,
      showClose = true,
      closeOnBackdrop = true,
      closeOnEscape = true,
      persistent = false,
      surface = 'glass',
      contentClass,
      header,
      footer,
      children,
    } = props

    const titleId = useId()
    const descriptionId = useId()
    // 最近一次 pointerdown / pointerup 坐标（打开期间由 document 级监听记录）。
    // Radix Dialog 1.1.x 的 deferPointerDownOutside 把外部判定推迟到随后的 click，
    // 且 custom event 只携带原始 pointerdown；pointerup 位置须适配器自行记录。
    const lastPointerDownRef = useRef<{ x: number; y: number } | null>(null)
    const lastPointerUpRef = useRef<{ x: number; y: number } | null>(null)

    // 统一关闭漏斗：persistent 时静默忽略，其余路径走同一出口，
    // 保证 onUpdateModelValue / onClose 的触发语义与 BaseModal.vue 一致。
    const requestClose = useCallback(() => {
      if (persistent) return
      onUpdateModelValue?.(false)
      onClose?.()
    }, [persistent, onUpdateModelValue, onClose])

    // ref.close()：与 BaseModal.vue 的 defineExpose({ close }) 对应。
    useImperativeHandle(ref, () => ({ close: requestClose }), [requestClose])

    // onOpened：Vue 的 @open 在 transition after-enter 触发；Radix 无对应回调，
    // 用打开后的下一帧近似（见文件头映射说明）。
    useEffect(() => {
      if (!modelValue) return
      let cancelled = false
      const raf = requestAnimationFrame(() => {
        if (!cancelled) onOpened?.()
      })
      return () => {
        cancelled = true
        cancelAnimationFrame(raf)
      }
    }, [modelValue, onOpened])

    // 打开期间记录 pointer 坐标，关闭/卸载时随监听一并清空，不累积。
    useEffect(() => {
      if (!modelValue) return
      const onDown = (event: PointerEvent) => {
        lastPointerDownRef.current = { x: event.clientX, y: event.clientY }
      }
      const onUp = (event: PointerEvent) => {
        lastPointerUpRef.current = { x: event.clientX, y: event.clientY }
      }
      document.addEventListener('pointerdown', onDown)
      document.addEventListener('pointerup', onUp)
      return () => {
        document.removeEventListener('pointerdown', onDown)
        document.removeEventListener('pointerup', onUp)
        lastPointerDownRef.current = null
        lastPointerUpRef.current = null
      }
    }, [modelValue])

    const hasHeader = title !== undefined || header !== undefined
    const hasDescription = description !== undefined

    return (
      <DialogPrimitive.Root
        open={modelValue}
        onOpenChange={(open) => {
          if (!open) requestClose()
        }}
      >
        <DialogPrimitive.Portal>
          <DialogOverlay />
          <DialogPrimitive.Content
            aria-labelledby={hasHeader ? titleId : undefined}
            aria-describedby={hasDescription ? descriptionId : undefined}
            onEscapeKeyDown={(event) => {
              if (persistent || !closeOnEscape) event.preventDefault()
            }}
            onPointerDownOutside={(event) => {
              // 遮罩关闭判定归适配器：preventDefault 阻断 Radix 默认关闭，
              // 按记录到的 pointerdown→pointerup 位移（≤6px 视为点击）决定，
              // 保持 BaseModal.vue 的拖拽阈值语义。deferPointerDownOutside 下
              // 本回调在随后的 click 时才触发，两段坐标均已记录。
              event.preventDefault()
              if (
                closeOnBackdrop &&
                isBackdropClick(lastPointerDownRef.current, lastPointerUpRef.current)
              ) {
                requestClose()
              }
            }}
            // 焦点移出不作为关闭路径：遮罩关闭只认 pointerup（含阈值判定）
            // 与 Esc，避免键盘焦点抖动引发误关。
            onFocusOutside={(event) => event.preventDefault()}
            className={cn(
              'fixed left-1/2 top-1/2 z-[var(--layer-modal)] w-full -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-2xl',
              'transition-interactive duration-normal',
              'data-[state=open]:opacity-100 data-[state=closed]:opacity-0',
              'data-[state=open]:scale-100 data-[state=closed]:scale-95',
              scrollable && 'flex max-h-[90vh] flex-col',
              SURFACE_CLASSES[surface],
              SIZE_CLASSES[size],
              contentClass,
            )}
          >
            {hasHeader && (
              <ModalHeader
                title={title}
                description={description}
                header={header}
                titleId={titleId}
                descriptionId={descriptionId}
                showClose={showClose}
                scrollable={scrollable}
              />
            )}
            <ModalBody scrollable={scrollable} hasFooter={footer !== undefined}>
              {children}
            </ModalBody>
            {footer !== undefined && (
              <ModalFooterBar scrollable={scrollable}>{footer}</ModalFooterBar>
            )}
            {showClose && <ModalCloseButton />}
          </DialogPrimitive.Content>
        </DialogPrimitive.Portal>
      </DialogPrimitive.Root>
    )
  },
)

// 供调用点以 `ref={... as Ref<BaseModalHandle>}` 之外的方式直接引用句柄类型。
export type BaseModalRef = Ref<BaseModalHandle>
