import { describe, expect, it } from 'vitest'
import router from '@/router'

describe('router smoke', () => {
  it('keeps critical named routes registered', () => {
    const requiredRoutes = [
      'home',
      'codex',
      'usage',
      'monitoring',
      'mcp',
      'skills',
      'sessions',
      'wsl-management',
      'ssh-management',
      'opencode'
    ]

    for (const routeName of requiredRoutes) {
      expect(router.hasRoute(routeName)).toBe(true)
    }
  })

  it('keeps /stats redirected to /usage', () => {
    const statsRoute = router.getRoutes().find(route => route.path === '/stats')

    expect(statsRoute?.redirect).toBe('/usage')
  })
})
