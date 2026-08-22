import { StrictMode } from 'react'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { App } from '@/shell/App'

// mock 底层 Tauri invoke：App → systemApi.getVersion → invokeRuntime → @tauri-apps/api/core
const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
  InvokeArgs: {},
  InvokeOptions: {},
}))

// App 内的 EventSubscriptionCard 会真实调用 listen()；jsdom 无 Tauri 运行时，mock 为已 resolve
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

describe('React 基座最小页面（AC5）', () => {
  it('StrictMode 下点击按钮渲染 check_version 的 IPC 返回值', async () => {
    invokeMock.mockResolvedValue({
      current: '7.2.0',
      latest: null,
      update_available: false,
    })

    render(
      <StrictMode>
        <App />
      </StrictMode>,
    )

    fireEvent.click(screen.getByRole('button', { name: '调用 check_version' }))

    const result = await screen.findByTestId('ipc-result')
    await waitFor(() => {
      expect(result.textContent).toBe('{"current":"7.2.0","latest":null,"update_available":false}')
    })
    // 命令走冻结门面：check_version
    expect(invokeMock).toHaveBeenCalledWith('check_version', undefined)
  })
})
