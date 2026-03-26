import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

describe('Tauri skills arg mapping smoke', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue(null)
  })

  it('uses snake_case keys for marketplace list/detail', async () => {
    const { skillsMarketplaceDetail, skillsMarketplaceList } = await import('@/api/tauri')

    await skillsMarketplaceList('find-skills', 2, 30)
    expect(invokeMock).toHaveBeenLastCalledWith('skills_marketplace_list', {
      query: 'find-skills',
      page: 2,
      page_size: 30,
    })

    await skillsMarketplaceDetail('owner/repo')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_marketplace_detail', {
      package_id: 'owner/repo',
    })
  })

  it('uses snake_case keys for skill detail and content', async () => {
    const { skillsContentGet, skillsContentSave, skillsDetail } = await import('@/api/tauri')

    await skillsDetail('skill-123')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_detail', { skill_id: 'skill-123' })

    await skillsContentGet('skill-123', 'inst-1')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_content_get', {
      skill_id: 'skill-123',
      installation_id: 'inst-1',
    })

    await skillsContentSave('skill-123', 'inst-1', 'raw-content')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_content_save', {
      skill_id: 'skill-123',
      installation_id: 'inst-1',
      raw: 'raw-content',
    })
  })
})

