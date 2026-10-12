import { render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SyncAssetInfo, SyncStatusView } from '@/types/syncSelection'
import { SyncView } from '@/features/sync/SyncView'
import { ensureLocaleLoaded, setLocale } from '@/i18n'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => ({})),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

const fixture = vi.hoisted(() => ({
  connectedStatus: {
    configured: true,
    enabled: true,
    webdav_url: 'https://dav.example.com/dav/',
    username: 'ada',
    remote_path: '/ccr/',
    auto_sync: false,
    has_password: true,
    remote_accessible: true,
    remote_exists: true,
  },
  fiveAssets: [
  {
    id: 'ccr-platforms',
    group: 'ccr',
    name: 'CCR Platforms',
    description: 'All CCR platform configuration, including auth, URL, key, and provider routing fields.',
    kind: 'directory',
    sensitive: true,
    encryptionState: 'v2_required',
    localPath: '~/.ccr/platforms/',
    resolvedLocalPath: '/home/user/.ccr/platforms/',
    remotePath: 'platforms/',
    localExists: true,
    remoteExists: true,
    canonicalName: null,
  },
  {
    id: 'claude-settings',
    group: 'claude',
    name: 'settings.json',
    description: 'Claude Code user settings only; the rest of ~/.claude is intentionally excluded.',
    kind: 'file',
    sensitive: true,
    encryptionState: 'v2_required',
    localPath: '~/.claude/settings.json',
    resolvedLocalPath: '/home/user/.claude/settings.json',
    remotePath: 'claude/settings.json',
    localExists: true,
    remoteExists: null,
    canonicalName: 'settings.json',
  },
  {
    id: 'claude-memory',
    group: 'claude',
    name: 'CLAUDE.md',
    description: 'Claude Code global memory and instructions file.',
    kind: 'file',
    sensitive: false,
    encryptionState: 'not_applicable',
    localPath: '~/.claude/CLAUDE.md',
    resolvedLocalPath: '/home/user/.claude/CLAUDE.md',
    remotePath: 'claude/CLAUDE.md',
    localExists: false,
    remoteExists: false,
    canonicalName: 'CLAUDE.md',
  },
  {
    id: 'codex-config',
    group: 'codex',
    name: 'config.toml',
    description: 'Codex user configuration only; the rest of ~/.codex is intentionally excluded.',
    kind: 'file',
    sensitive: true,
    encryptionState: 'v2_required',
    localPath: '~/.codex/config.toml',
    resolvedLocalPath: '/home/user/.codex/config.toml',
    remotePath: 'codex/config.toml',
    localExists: true,
    remoteExists: true,
    canonicalName: 'config.toml',
  },
  {
    id: 'codex-agents',
    group: 'codex',
    name: 'AGENTS.md',
    description: 'Codex global instructions file.',
    kind: 'file',
    sensitive: false,
    encryptionState: 'not_applicable',
    localPath: '~/.codex/AGENTS.md',
    resolvedLocalPath: '/home/user/.codex/AGENTS.md',
    remotePath: 'codex/AGENTS.md',
    localExists: true,
    remoteExists: true,
    canonicalName: 'AGENTS.md',
  },
  ],
}))

vi.mock('@/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api')>()
  return {
    ...actual,
    listSyncAssets: vi.fn(async () => fixture.fiveAssets as SyncAssetInfo[]),
    getSyncStatus: vi.fn(async () => fixture.connectedStatus as SyncStatusView),
  }
})

describe('sync page structure', () => {
  beforeEach(async () => {
    fixture.connectedStatus.configured = true
    fixture.connectedStatus.remote_accessible = true
    await ensureLocaleLoaded('zh-CN')
    await setLocale('zh-CN')
  })

  it('shows one catalog for the five assets', async () => {
    render(<SyncView />)

    expect(await screen.findByRole('heading', { name: 'CCR Platforms' })).toBeTruthy()
    expect(screen.getByRole('heading', { level: 1, name: '配置资产同步' })).toBeTruthy()
    expect(screen.queryByText('配置资产控制台')).toBeNull()
    expect(screen.queryByText('固定同步清单')).toBeNull()
    expect(document.querySelector('.sync-scope-strip')).toBeNull()
    expect(document.querySelector('.sync-console-intro')).toBeNull()
    expect(screen.queryByText('所有 platform 配置')).toBeNull()
    expect(screen.queryByText('settings.json + CLAUDE.md')).toBeNull()
    expect(screen.queryByText('config.toml + AGENTS.md')).toBeNull()
    expect(screen.queryByText('All CCR platform configuration, including auth, URL, key, and provider routing fields.')).toBeNull()
    expect(screen.queryByText('Claude Code user settings only; the rest of ~/.claude is intentionally excluded.')).toBeNull()

    const ccrTitles = screen.getAllByRole('heading', { name: 'CCR Platforms' })
    expect(ccrTitles).toHaveLength(1)
    expect(ccrTitles[0]?.tagName).toBe('H3')

    const settings = screen.getByRole('heading', { name: 'settings.json' })
    const claude = screen.getByRole('heading', { name: 'CLAUDE.md' })
    expect(settings.tagName).toBe('H4')
    expect(claude.tagName).toBe('H4')
    expect(settings.closest('.sync-asset-card')?.className).not.toContain('sync-asset-card--missing')
    expect(claude.closest('.sync-asset-card')?.className).toContain('sync-asset-card--missing')
    expect(screen.queryByText('规范名 settings.json')).toBeNull()
    expect(screen.queryByText('规范名 CLAUDE.md')).toBeNull()

    expect(document.querySelectorAll('.sync-asset-group')).toHaveLength(3)
    expect(document.querySelectorAll('.sync-connection-chip')).toHaveLength(1)
    expect(screen.getByText('5 项资产')).toBeTruthy()
    expect(screen.getByText('敏感值已遮罩')).toBeTruthy()
    expect(screen.queryByText('Assets')).toBeNull()
  })

  it('keeps the only enabled primary action on the gating banner when unconfigured', async () => {
    fixture.connectedStatus.configured = false
    render(<SyncView />)

    const configure = await screen.findByRole('button', { name: '配置 WebDAV 账号' })
    expect(configure.className).toContain('ui-btn--primary')
    expect(configure).toHaveProperty('disabled', false)

    const sidebarAdd = screen.getByRole('button', { name: '添加云同步账号' })
    expect(sidebarAdd.className).toContain('ui-btn--secondary')
    expect(sidebarAdd.className).not.toContain('ui-btn--primary')

    expect(screen.getByRole('button', { name: '全部同步一次' })).toHaveProperty('disabled', true)
    for (const syncButton of screen.getAllByRole('button', { name: 'Sync' })) {
      expect(syncButton).toHaveProperty('disabled', true)
    }
  })
})
