// @vitest-environment node

import { EventEmitter } from 'node:events'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { createFilter } from 'vite'
import { describe, expect, it, vi } from 'vitest'
import { terminateProcessTree } from '../scripts/process-tree.mjs'
import { resolveSmokeMaxWorkers } from '../vitest.smoke.config'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

class FakeChild extends EventEmitter {
  pid = 1234
  exitCode: number | null = null
  signalCode: NodeJS.Signals | null = null
  kill = vi.fn((_signal?: NodeJS.Signals | number) => true)

  finish(signal: NodeJS.Signals = 'SIGTERM') {
    this.signalCode = signal
    this.emit('exit', null, signal)
  }
}

describe('Vite development resource contracts', () => {
  it('ignores generated high-churn directories without excluding source or public assets', async () => {
    const { resolveConfig } = await import('vite')
    const config = await resolveConfig(
      { root, configFile: path.join(root, 'vite.config.ts') },
      'serve',
    )
    const ignored = config.server.watch?.ignored
    const ignorePatterns = Array.isArray(ignored)
      ? ignored.filter((pattern): pattern is string => typeof pattern === 'string')
      : []

    expect(ignorePatterns).toEqual(expect.arrayContaining([
      '**/src-tauri/target/**',
      '**/ref/**',
      '**/logs/**',
    ]))

    const filter = createFilter(undefined, ignorePatterns)
    expect(filter(path.join(root, 'src-tauri/target/debug/probe.txt'))).toBe(false)
    expect(filter(path.join(root, 'ref/mirror.ts'))).toBe(false)
    expect(filter(path.join(root, 'logs/frontend.log'))).toBe(false)
    expect(filter(path.join(root, 'src/main.ts'))).toBe(true)
    expect(filter(path.join(root, 'public/fonts/example.css'))).toBe(true)
  })

  it('keeps a unique root-route warm manifest whose files exist', () => {
    const manifest = JSON.parse(
      readFileSync(path.join(root, 'scripts/dev-warm-targets.json'), 'utf8'),
    ) as { healthPath: string; clientFiles: string[] }
    const required = [
      './src/main.ts',
      './src/App.vue',
      './src/router/index.ts',
      './src/components/MainLayout.vue',
      './src/views/DashboardView.vue',
    ]

    expect(manifest.healthPath).toBe('/')
    expect(new Set(manifest.clientFiles).size).toBe(manifest.clientFiles.length)
    expect(manifest.clientFiles).toEqual(expect.arrayContaining(required))
    expect(manifest.clientFiles).not.toContain('./src/views/AppSettingsView.vue')
    for (const target of manifest.clientFiles) {
      expect(existsSync(path.resolve(root, target))).toBe(true)
    }
  })

  it('leaves module warmup to Vite and performs one bounded health fetch', () => {
    const source = readFileSync(path.join(root, 'scripts/dev-web-warm-start.mjs'), 'utf8')

    expect(source).not.toContain('clientFiles')
    expect(source).not.toContain('probeUrls')
    expect(source.match(/await fetch\(/g)).toHaveLength(1)
    expect(source).toContain('CCR_DEV_HEALTH_TIMEOUT_MS')
    expect(source).toContain('[dev:web] ready')
  })

  it('preserves the Vite cache unless the explicit reset switch is set', () => {
    const source = readFileSync(path.join(root, 'scripts/dev-web-windows.ps1'), 'utf8')

    expect(source).toContain("$env:CCR_DEV_RESET_VITE_CACHE -eq '1'")
    expect(source.match(/Remove-Item[^\r\n]*node_modules\/\.vite/g)).toHaveLength(1)
  })
})

describe('smoke worker budget', () => {
  it('uses bounded local and CI defaults', () => {
    expect(resolveSmokeMaxWorkers({}, 24)).toBe(2)
    expect(resolveSmokeMaxWorkers({ CI: 'true' }, 24)).toBe(4)
    expect(resolveSmokeMaxWorkers({ CI: 'true' }, 3)).toBe(3)
  })

  it('caps valid overrides and safely falls back for invalid values', () => {
    expect(resolveSmokeMaxWorkers({ CCR_TEST_WORKERS: '8' }, 6)).toBe(6)
    expect(resolveSmokeMaxWorkers({ CCR_TEST_WORKERS: '1' }, 6)).toBe(1)
    expect(resolveSmokeMaxWorkers({ CCR_TEST_WORKERS: '0' }, 6)).toBe(2)
    expect(resolveSmokeMaxWorkers({ CCR_TEST_WORKERS: '2.5', CI: '1' }, 6)).toBe(4)
    expect(resolveSmokeMaxWorkers({ CCR_TEST_WORKERS: 'nope' }, 1)).toBe(1)
  })
})

describe('process-tree termination', () => {
  it('uses taskkill for the full Windows tree and is idempotent', async () => {
    const child = new FakeChild()
    const runFile = vi.fn(async () => {
      child.finish()
    })

    const first = terminateProcessTree(child, { platform: 'win32', runFile })
    const second = terminateProcessTree(child, { platform: 'win32', runFile })
    expect(second).toBe(first)
    await first

    expect(runFile).toHaveBeenCalledOnce()
    expect(runFile).toHaveBeenCalledWith(
      'taskkill.exe',
      ['/PID', '1234', '/T', '/F'],
      { windowsHide: true },
    )
  })

  it('treats an already-exited process as successful', async () => {
    const child = new FakeChild()
    child.exitCode = 0
    const runFile = vi.fn()

    await expect(terminateProcessTree(child, { platform: 'win32', runFile })).resolves.toBeUndefined()
    expect(runFile).not.toHaveBeenCalled()
  })

  it('escalates to SIGKILL after the Unix grace period', async () => {
    const child = new FakeChild()
    child.kill.mockImplementation((signal?: NodeJS.Signals | number) => {
      if (signal === 'SIGKILL') queueMicrotask(() => child.finish('SIGKILL'))
      return true
    })

    await terminateProcessTree(child, { platform: 'linux', graceMs: 1, forceWaitMs: 50 })

    expect(child.kill.mock.calls.map(([signal]) => signal)).toEqual(['SIGTERM', 'SIGKILL'])
  })
})
