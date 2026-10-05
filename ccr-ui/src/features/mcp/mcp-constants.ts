import type { PlatformMeta, UnifiedMcpPlatform, UnifiedMcpRequest } from '@/types/unifiedMcp'

export const PLATFORM_META: Record<UnifiedMcpPlatform, PlatformMeta> = {
  claude: { id: 'claude', label: 'Claude Code', color: '#d97706', icon: 'terminal' },
  codex: { id: 'codex', label: 'Codex', color: '#10b981', icon: 'code' },
  gemini: { id: 'gemini', label: 'Antigravity CLI', color: '#8b5cf6', icon: 'sparkles' },
}

export const ALL_PLATFORMS: UnifiedMcpPlatform[] = ['claude', 'codex', 'gemini']

export function createEmptyForm(): UnifiedMcpRequest {
  return {
    platform: 'claude',
    scope: 'user',
    name: '',
    command: null,
    url: null,
    args: null,
    env: null,
  }
}

export function stripUnchangedSecretPreviews(
  patch: Record<string, string> | null,
  current: Record<string, string> | null | undefined,
): void {
  if (!patch) return
  for (const [key, value] of Object.entries(patch)) {
    if (value.includes('•') && value === current?.[key]) delete patch[key]
  }
}

export function toSuccessMessage(raw: unknown, fallback: string): string {
  if (typeof raw === 'string' && raw) return raw
  if (typeof raw === 'object' && raw !== null && 'message' in raw) {
    const message = (raw as { message?: unknown }).message
    if (typeof message === 'string' && message) return message
  }
  return fallback
}
