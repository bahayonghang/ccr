import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { APP_NAME } from '@/config/appMeta'
import { GlobalConfirmDialog } from '@/shell/GlobalConfirmDialog'
import { useUIStore } from '@/shell/stores/ui'
import { Titlebar } from '@/shell/Titlebar'
import { logger } from '@/utils/logger'

vi.mock('@/ui/confirm-modal', () => Promise.reject(new Error('confirm chunk unavailable')))
vi.mock('@/ui/base-modal', () => Promise.reject(new Error('about chunk unavailable')))

describe('lazy shell failure containment', () => {
  beforeEach(() => {
    useUIStore.getState().resolveConfirmDialog(false)
    useUIStore.setState(useUIStore.getInitialState())
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('否决加载失败时的当前确认请求，并保留并发替换语义', async () => {
    const logError = vi.spyOn(logger, 'error').mockImplementation(() => undefined)
    render(<GlobalConfirmDialog />)

    let replaced!: Promise<boolean>
    let current!: Promise<boolean>
    act(() => {
      replaced = useUIStore.getState().requestConfirm({ title: '旧请求', message: 'old' })
      current = useUIStore.getState().requestConfirm({ title: '当前请求', message: 'current' })
    })

    await expect(replaced).resolves.toBe(false)
    await expect(current).resolves.toBe(false)
    expect(useUIStore.getState().confirmDialog).toBeNull()
    expect(logError).toHaveBeenCalledOnce()
    const [message, context] = logError.mock.calls[0] ?? []
    expect(message).toBe('[ErrorBoundary] render failed')
    expect(context).not.toBeNull()
    expect(typeof context).toBe('object')
    expect(context && typeof context === 'object' && 'error' in context && context.error).toBeInstanceOf(Error)
    expect(context && typeof context === 'object' && 'componentStack' in context && context.componentStack)
      .toBeTypeOf('string')
  })

  it('局部收敛 About 加载失败，并避免对缓存 rejection 伪重试', async () => {
    const logError = vi.spyOn(logger, 'error').mockImplementation(() => undefined)
    const { container } = render(<Titlebar />)
    const aboutButton = screen.getByRole('button', { name: APP_NAME })

    fireEvent.click(aboutButton)

    await waitFor(() => expect(logError).toHaveBeenCalledOnce())
    const [message, context] = logError.mock.calls[0] ?? []
    expect(message).toBe('[ErrorBoundary] render failed')
    expect(context).not.toBeNull()
    expect(typeof context).toBe('object')
    expect(context && typeof context === 'object' && 'error' in context && context.error).toBeInstanceOf(Error)
    expect(context && typeof context === 'object' && 'componentStack' in context && context.componentStack)
      .toBeTypeOf('string')
    expect(container.querySelector('.titlebar-shell')).toBeTruthy()
    await waitFor(() => expect((aboutButton as HTMLButtonElement).disabled).toBe(true))

    const failureCount = logError.mock.calls.length
    fireEvent.click(aboutButton)
    expect(logError).toHaveBeenCalledTimes(failureCount)
  })
})
