import { readFile } from 'node:fs/promises'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import {
  getBudgetStatus,
  listOutputStyles,
  listPlugins,
  setBudget,
} from '@/api/domains/claude'

beforeEach(() => {
  invokeMock.mockReset()
})

describe('typed Claude client facade', () => {
  it('keeps migrated commands behind generated clients without generic result escapes', async () => {
    const source = await readFile('src/api/domains/claude.ts', 'utf8')

    expect(source).toContain("import * as claudeGenerated from '../generated/claude'")
    expect(source).not.toMatch(/export const \w+ = async <T/)
    expect(source).not.toMatch(/\bas T\b/)
  })

  it('projects generated response envelopes into existing view types', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'claude_list_plugins') {
        return {
          plugins: [{ id: 'demo', name: 'Demo', version: '1.0.0', enabled: true }],
        }
      }
      if (command === 'claude_get_output_styles') {
        return { styles: [{ name: 'compact', content: '# Compact' }] }
      }
      if (command === 'claude_get_budgets') {
        return {
          enabled: true,
          dailyLimit: 5,
          weeklyLimit: null,
          monthlyLimit: 50,
          warnAtPercent: 80,
          currentCosts: { today: 1, thisWeek: 2, thisMonth: 3 },
          warnings: [{
            period: 'daily',
            currentCost: 1,
            limit: 5,
            usagePercent: 20,
          }],
        }
      }
      throw new Error(`Unexpected command: ${command}`)
    })

    await expect(listPlugins()).resolves.toEqual([
      { id: 'demo', name: 'Demo', version: '1.0.0', enabled: true },
    ])
    await expect(listOutputStyles()).resolves.toEqual([
      { name: 'compact', content: '# Compact' },
    ])
    await expect(getBudgetStatus()).resolves.toMatchObject({
      daily_limit: 5,
      weekly_limit: null,
      monthly_limit: 50,
      warn_threshold: 80,
      current_costs: { today: 1, this_week: 2, this_month: 3 },
      warnings: [{ current_cost: 1, usage_percent: 20 }],
    })
  })

  it('maps the legacy budget request view onto the generated wire shape', async () => {
    invokeMock.mockResolvedValue({ enabled: true })

    await setBudget({
      enabled: true,
      daily_limit: 5,
      weekly_limit: null,
      monthly_limit: 50,
      warn_threshold: 80,
    })

    expect(invokeMock).toHaveBeenCalledWith('claude_update_budgets', {
      budgets: {
        enabled: true,
        dailyLimit: 5,
        weeklyLimit: null,
        monthlyLimit: 50,
        warnAtPercent: 80,
      },
    })
  })
})
