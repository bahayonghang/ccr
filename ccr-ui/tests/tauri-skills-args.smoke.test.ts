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

  it('uses camelCase keys for marketplace list/detail', async () => {
    const { skillsMarketplaceDetail, skillsMarketplaceList } = await import('@/api/tauri')

    await skillsMarketplaceList('find-skills', 2, 30)
    expect(invokeMock).toHaveBeenLastCalledWith('skills_marketplace_list', {
      query: 'find-skills',
      page: 2,
      pageSize: 30,
    })

    await skillsMarketplaceDetail('owner/repo')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_marketplace_detail', {
      packageId: 'owner/repo',
    })
  })

  it('uses camelCase keys for skill detail and content', async () => {
    const { skillsContentGet, skillsContentSave, skillsDetail } = await import('@/api/tauri')

    await skillsDetail('skill-123')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_detail', { skillId: 'skill-123' })

    await skillsContentGet('skill-123', 'inst-1')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_content_get', {
      skillId: 'skill-123',
      installationId: 'inst-1',
    })

    await skillsContentSave('skill-123', 'inst-1', 'raw-content')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_content_save', {
      skillId: 'skill-123',
      installationId: 'inst-1',
      raw: 'raw-content',
    })
  })

  it('uses camelCase keys for removal and source commands', async () => {
    const {
      skillsRemoveInstallation,
      skillsRemoveSkill,
      skillsSourceRemove,
      skillsSourceSync,
    } = await import('@/api/tauri')

    await skillsRemoveInstallation('skill-123', 'inst-1')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_remove_installation', {
      skillId: 'skill-123',
      installationId: 'inst-1',
    })

    await skillsRemoveSkill('skill-123')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_remove_skill', {
      skillId: 'skill-123',
    })

    await skillsSourceSync('source-1')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_source_sync', {
      sourceId: 'source-1',
    })

    await skillsSourceRemove('source-1')
    expect(invokeMock).toHaveBeenLastCalledWith('skills_source_remove', {
      sourceId: 'source-1',
    })
  })
})
