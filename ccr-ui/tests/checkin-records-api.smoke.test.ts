import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

// 回归：失败历史面板的 status/provider_id/keyword/page/page_size 必须真实透传到
// get_checkin_records（曾在 api 层被丢弃，导致「失败历史」混入成功记录、过滤翻页失效）
describe('checkin records API parameter passthrough', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue({ records: [], total: 0 })
  })

  it('passes advanced filters and pagination through to get_checkin_records', async () => {
    const { listCheckinRecords } = await import('@/api/domains/checkin')

    await listCheckinRecords({
      status: 'failed',
      provider_id: 'provider-1',
      keyword: 'stumail',
      page: 2,
      page_size: 5,
    })

    expect(invokeMock).toHaveBeenCalledWith('get_checkin_records', {
      accountId: null,
      limit: null,
      status: 'failed',
      providerId: 'provider-1',
      keyword: 'stumail',
      page: 2,
      pageSize: 5,
    })
  })

  it('defaults pagination without dropping unset filters', async () => {
    const { listCheckinRecords } = await import('@/api/domains/checkin')

    await listCheckinRecords({ page: 1, page_size: 100 })

    expect(invokeMock).toHaveBeenCalledWith('get_checkin_records', {
      accountId: null,
      limit: null,
      status: null,
      providerId: null,
      keyword: null,
      page: 1,
      pageSize: 100,
    })
  })

  it('keeps the legacy number form as a plain limit query', async () => {
    const { listCheckinRecords } = await import('@/api/domains/checkin')

    await listCheckinRecords(50)

    expect(invokeMock).toHaveBeenCalledWith('get_checkin_records', {
      accountId: null,
      limit: 50,
    })
  })
})
