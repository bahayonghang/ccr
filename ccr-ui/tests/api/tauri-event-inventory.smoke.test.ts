import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'
import { TAURI_GLOBAL_EVENTS } from '@/shell/eventBridge'
import {
  collectRustEmitNames,
  FRONTEND_EVENT_INVENTORY_PATH,
  parseFrontendEventInventory,
} from '../helpers/tauriEventInventory'

const HIGH_FREQUENCY_EVENTS = ['app-log', 'token-stats', 'app:monitoring'] as const

describe('Tauri event name inventory', () => {
  it('registers new local events in the inventory document', async () => {
    const markdown = await readFile(FRONTEND_EVENT_INVENTORY_PATH, 'utf8')
    expect(markdown).toContain('新增局部事件须同时登记')
  })

  it('keeps the global set equal to TAURI_GLOBAL_EVENTS and a subset of Rust emit names', async () => {
    const markdown = await readFile(FRONTEND_EVENT_INVENTORY_PATH, 'utf8')
    const inventory = parseFrontendEventInventory(markdown)
    const rustNames = await collectRustEmitNames()
    const frontendNames = inventory.map((row) => row.event)
    const globalNames = inventory
      .filter((row) => row.owner === 'eventBridge' && row.lifecycle === '常驻')
      .map((row) => row.event)
    const highFrequency = inventory
      .filter((row) => row.lifecycle === '常驻（批量）')
      .map((row) => row.event)
    const missingOnRust = frontendNames.filter((name) => !rustNames.has(name))

    expect(inventory.length).toBeGreaterThan(0)
    expect(globalNames.sort()).toEqual([...TAURI_GLOBAL_EVENTS].sort())
    expect(highFrequency.sort()).toEqual([...HIGH_FREQUENCY_EVENTS].sort())
    expect(missingOnRust).toEqual([])
    expect(new Set(frontendNames).size).toBe(frontendNames.length)
  })
})
