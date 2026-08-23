import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { act } from 'react'
import { createRef, forwardRef, useEffect, useState } from 'react'
import { BaseModal, type BaseModalHandle, type BaseModalProps } from '@/ui'
import { beforeAll, describe, expect, it, vi } from 'vitest'

// BaseModal 适配器行为测试（08-22-design-system 批次 4）。
// 断言适配器语义与 BaseModal.vue 一致，且四项弹层行为委托 Radix：
// Esc / 遮罩点击（6px 拖拽阈值）/ 滚动锁定 / aria 接线 / 事件回调 / ref.close()。
//
// jsdom 桩：与 tests/ui-primitives.smoke.test.tsx 相同（ResizeObserver、PointerEvent）。
// 本仓 smoke 套件未引入 jest-dom，断言用原生属性检查。


beforeAll(() => {
  if (typeof globalThis.ResizeObserver === 'undefined') {
    class ResizeObserverStub {
      observe(): void {}
      unobserve(): void {}
      disconnect(): void {}
    }
    globalThis.ResizeObserver =
      ResizeObserverStub as unknown as typeof ResizeObserver
  }

  const mouseEventCtor = (globalThis.MouseEvent ?? Event) as unknown as typeof MouseEvent
  class PointerEventStub extends mouseEventCtor {
    readonly pointerId: number
    readonly pointerType: string
    readonly isPrimary: boolean

    constructor(type: string, params: PointerEventInit = {}) {
      super(type, {
        bubbles: params.bubbles,
        cancelable: params.cancelable,
        button: params.button ?? 0,
        ctrlKey: params.ctrlKey ?? false,
        // 位移坐标必须透传：适配器的拖拽阈值判定依赖 clientX/clientY。
        clientX: params.clientX ?? 0,
        clientY: params.clientY ?? 0,
      })
      this.pointerId = params.pointerId ?? 0
      this.pointerType = params.pointerType ?? 'mouse'
      this.isPrimary = params.isPrimary ?? true
    }
  }
  if (typeof globalThis.PointerEvent === 'undefined') {
    const stub = PointerEventStub as unknown as typeof PointerEvent
    globalThis.PointerEvent = stub
    window.PointerEvent = stub
  }
})

// Radix DismissableLayer 在 setTimeout(0) 之后才注册 document 级 pointerdown
// 监听（@radix-ui/react-dismissable-layer 1.1.x 行为），render 后须让出一个
// 宏任务，外部 pointerdown 才会被探测到。
const settleRadixOutsideDetection = () => act(async () => {
  await new Promise((resolve) => setTimeout(resolve, 0))
})

// Radix Dialog 1.1.x 的 deferPointerDownOutside：外部 pointerdown（button 0）
// 的判定推迟到随后的 click 才派发 onPointerDownOutside。遮罩交互序列 =
// pointerdown → pointerup → click。
function clickBackdrop(overlay: HTMLElement, from: { x: number; y: number }, to: { x: number; y: number }) {
  fireEvent.pointerDown(overlay, { clientX: from.x, clientY: from.y, button: 0 })
  fireEvent.pointerUp(overlay, { clientX: to.x, clientY: to.y, button: 0 })
  fireEvent.click(overlay, { clientX: to.x, clientY: to.y, button: 0 })
}

/** 受控开关壳：open 初始为 true，随 modelValue prop 同步，onUpdateModelValue 改 state。 */
const Controlled = forwardRef<BaseModalHandle, Partial<BaseModalProps>>(
  function Controlled(props, ref) {
    const [open, setOpen] = useState(props.modelValue ?? true)
    useEffect(() => {
      setOpen(props.modelValue ?? true)
    }, [props.modelValue])
    return (
      <BaseModal
        title="默认标题"
        ref={ref}
        {...props}
        modelValue={open}
        onUpdateModelValue={(value) => {
          props.onUpdateModelValue?.(value)
          setOpen(value)
        }}
      >
        <p>正文内容</p>
      </BaseModal>
    )
  },
)

function getOverlay(): HTMLElement {
  // Portal 结构：容器 div 下依次为 Overlay 与 Content（role=dialog）。
  const content = screen.getByRole('dialog')
  const portal = content.parentElement
  expect(portal).not.toBeNull()
  return portal!.firstElementChild as HTMLElement
}

function pressEscape(): void {
  fireEvent.keyDown(document.body, { key: 'Escape' })
}

describe('base-modal 适配器（08-22-design-system 批次 4）', () => {
  it('Esc 关闭：触发 onUpdateModelValue(false) 与 onClose', async () => {
    const onUpdateModelValue = vi.fn()
    const onClose = vi.fn()
    render(
      <Controlled
        onUpdateModelValue={onUpdateModelValue}
        onClose={onClose}
      />,
    )
    expect(screen.queryByRole('dialog')).not.toBeNull()
    pressEscape()
    await waitFor(() => {
      expect(onUpdateModelValue).toHaveBeenCalledWith(false)
    })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('closeOnEscape=false 时 Esc 不关闭', () => {
    const onUpdateModelValue = vi.fn()
    render(<Controlled closeOnEscape={false} onUpdateModelValue={onUpdateModelValue} />)
    pressEscape()
    expect(onUpdateModelValue).not.toHaveBeenCalled()
    expect(screen.queryByRole('dialog')).not.toBeNull()
  })

  it('persistent 时 Esc 不关闭', () => {
    const onUpdateModelValue = vi.fn()
    render(<Controlled persistent onUpdateModelValue={onUpdateModelValue} />)
    pressEscape()
    expect(onUpdateModelValue).not.toHaveBeenCalled()
    expect(screen.queryByRole('dialog')).not.toBeNull()
  })

  it('遮罩点击（位移 ≤6px）关闭', async () => {
    const onUpdateModelValue = vi.fn()
    const onClose = vi.fn()
    render(<Controlled onUpdateModelValue={onUpdateModelValue} onClose={onClose} />)
    await settleRadixOutsideDetection()
    clickBackdrop(getOverlay(), { x: 10, y: 10 }, { x: 12, y: 12 })
    await waitFor(() => {
      expect(onUpdateModelValue).toHaveBeenCalledWith(false)
    })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('遮罩拖拽位移 >6px 不关闭（拖拽阈值）', async () => {
    const onUpdateModelValue = vi.fn()
    render(<Controlled onUpdateModelValue={onUpdateModelValue} />)
    await settleRadixOutsideDetection()
    clickBackdrop(getOverlay(), { x: 10, y: 10 }, { x: 60, y: 60 })
    await settleRadixOutsideDetection()
    expect(onUpdateModelValue).not.toHaveBeenCalled()
    expect(screen.queryByRole('dialog')).not.toBeNull()
  })

  it('closeOnBackdrop=false 时遮罩点击不关闭', async () => {
    const onUpdateModelValue = vi.fn()
    render(<Controlled closeOnBackdrop={false} onUpdateModelValue={onUpdateModelValue} />)
    await settleRadixOutsideDetection()
    clickBackdrop(getOverlay(), { x: 10, y: 10 }, { x: 10, y: 10 })
    await settleRadixOutsideDetection()
    expect(onUpdateModelValue).not.toHaveBeenCalled()
    expect(screen.queryByRole('dialog')).not.toBeNull()
  })

  it('persistent 时遮罩点击不关闭', async () => {
    const onUpdateModelValue = vi.fn()
    render(<Controlled persistent onUpdateModelValue={onUpdateModelValue} />)
    await settleRadixOutsideDetection()
    clickBackdrop(getOverlay(), { x: 10, y: 10 }, { x: 10, y: 10 })
    await settleRadixOutsideDetection()
    expect(onUpdateModelValue).not.toHaveBeenCalled()
    expect(screen.queryByRole('dialog')).not.toBeNull()
  })

  it('打开时锁定 body 滚动，关闭后解除（Radix 委托）', async () => {
    const { rerender } = render(<Controlled />)
    const locked =
      document.body.style.overflow === 'hidden' ||
      document.body.getAttribute('data-scroll-locked') !== null
    expect(locked).toBe(true)

    rerender(<Controlled modelValue={false} />)
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull()
    })
    const unlocked =
      document.body.style.overflow !== 'hidden' &&
      document.body.getAttribute('data-scroll-locked') === null
    expect(unlocked).toBe(true)
  })

  it('aria：role=dialog、labelledby/describedby 接线', () => {
    // 核实结论（@radix-ui/react-dialog 1.1.23）：Content 不再输出 aria-modal
    // 属性（dist 内无该字符串；modal 语义由 role="dialog" + Radix 焦点管理承载），
    // 故不断言 aria-modal。
    render(<Controlled title="确认操作" description="确认说明文本" />)
    const dialog = screen.getByRole('dialog')
    expect(dialog.getAttribute('role')).toBe('dialog')
    const labelledby = dialog.getAttribute('aria-labelledby')
    const describedby = dialog.getAttribute('aria-describedby')
    expect(labelledby).toBeTruthy()
    expect(describedby).toBeTruthy()
    expect(document.getElementById(labelledby!)?.textContent).toContain('确认操作')
    expect(document.getElementById(describedby!)?.textContent).toContain('确认说明文本')
  })

  it('onOpened 在打开后触发；关闭按钮触发关闭回调', async () => {
    const onOpened = vi.fn()
    const onUpdateModelValue = vi.fn()
    render(<Controlled onOpened={onOpened} onUpdateModelValue={onUpdateModelValue} />)
    await waitFor(() => {
      expect(onOpened).toHaveBeenCalledTimes(1)
    })
    fireEvent.click(screen.getByRole('button', { name: '关闭' }))
    await waitFor(() => {
      expect(onUpdateModelValue).toHaveBeenCalledWith(false)
    })
  })

  it('ref.close() 关闭弹窗', async () => {
    const handle = createRef<BaseModalHandle>()
    const onUpdateModelValue = vi.fn()
    render(<Controlled ref={handle} onUpdateModelValue={onUpdateModelValue} />)
    handle.current?.close()
    await waitFor(() => {
      expect(onUpdateModelValue).toHaveBeenCalledWith(false)
    })
  })

  it('header 渲染函数收到 titleId；footer 渲染在底部', () => {
    render(
      <BaseModal
        modelValue
        header={({ titleId }) => (
          <h2 id={titleId}>自定义头部</h2>
        )}
        footer={<button type="button">确定</button>}
      >
        <p>正文内容</p>
      </BaseModal>,
    )
    const dialog = screen.getByRole('dialog')
    const titleId = dialog.getAttribute('aria-labelledby')
    expect(titleId).toBeTruthy()
    expect(document.getElementById(titleId!)?.textContent).toContain('自定义头部')
    expect(screen.queryByRole('button', { name: '确定' })).not.toBeNull()
  })

  it('showClose=false 不渲染关闭按钮', () => {
    render(<Controlled showClose={false} />)
    expect(screen.queryByRole('button', { name: '关闭' })).toBeNull()
  })
})
