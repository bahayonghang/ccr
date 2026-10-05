import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it } from 'vitest'
import { appRoutes } from '@/shell/router'
import { useUIStore } from '@/shell/stores/ui'

const renderShell = (path = '/') => {
  const router = createMemoryRouter(appRoutes, { initialEntries: [path] })
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('confirm interaction（AC10）', () => {
  beforeEach(() => {
    useUIStore.getState().resolveConfirmDialog(false)
    useUIStore.setState(useUIStore.getInitialState())
  })

  it('requestConfirm 走 GlobalConfirmDialog，取消不执行', async () => {
    renderShell('/')
    const pending = useUIStore.getState().requestConfirm({
      title: '删除配置',
      message: '此操作不可逆',
      type: 'danger',
    })
    expect(await screen.findByRole('heading', { name: '删除配置' })).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: '取消' }))
    await expect(pending).resolves.toBe(false)
    expect(useUIStore.getState().confirmDialog).toBeNull()
  })

  it('确认按钮让 promise 以 true 收敛', async () => {
    renderShell('/')
    const pending = useUIStore.getState().requestConfirm({
      title: '切换账号',
      message: '确认切换',
      type: 'warning',
      confirmText: '继续',
    })
    fireEvent.click(await screen.findByRole('button', { name: '继续' }))
    await expect(pending).resolves.toBe(true)
  })
})
