import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { redactLogText, redactLogValue } from '@/utils/logRedact'
import {
  Logger,
  MAX_BRIDGE_ATTEMPTS,
  MAX_FIELDS_JSON_BYTES,
  MAX_MESSAGE_CHARS,
  sanitizeLoggerData,
} from '@/utils/logger'

interface VectorCase {
  id: string
  kind: 'text' | 'value'
  input: unknown
  must_not_contain: string[]
  must_contain: string[]
}

const vectors = JSON.parse(
  readFileSync(
    resolve(dirname(fileURLToPath(import.meta.url)), '../../../crates/ccr-core/testdata/log_redaction_vectors.json'),
    'utf8',
  ),
) as VectorCase[]

describe('logger redaction and IPC limits', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('applies shared redaction vectors', () => {
    expect(vectors.length).toBeGreaterThan(0)
    for (const testCase of vectors) {
      const rendered = testCase.kind === 'text'
        ? redactLogText(String(testCase.input))
        : JSON.stringify(redactLogValue(testCase.input))
      for (const fragment of testCase.must_not_contain) {
        expect(rendered, testCase.id).not.toContain(fragment)
      }
      for (const fragment of testCase.must_contain) {
        expect(rendered, testCase.id).toContain(fragment)
      }
    }
  })

  it('redacts history and queued IPC payloads', () => {
    const appendFrontendLogs = vi.fn().mockResolvedValue(undefined)
    const logger = new Logger({
      isTauriRuntime: () => true,
      appendFrontendLogs,
      sessionId: 'session-fixed',
    })

    logger.error('using key sk-ant-1234567890abcdef', {
      apiKey: 'sk-ant-1234567890abcdef',
      cookie: 'session=supersecretcookievalue',
    })

    const history = logger.getHistory()
    expect(history).toHaveLength(1)
    expect(history[0].correlationId).toBe('session-fixed')
    expect(JSON.stringify(history[0])).not.toContain('sk-ant-1234567890abcdef')
    expect(JSON.stringify(history[0])).not.toContain('supersecretcookievalue')
    expect(logger.getBridgeQueueLength()).toBe(1)
  })

  it('drops Error.stack from sanitized data', () => {
    const error = new Error('boom')
    error.stack = 'Error: boom\n    at secret.js:1:1'
    const sanitized = sanitizeLoggerData(error) as { message?: string, stack?: string }
    expect(sanitized.message).toBe('boom')
    expect(sanitized.stack).toBeUndefined()
  })

  it('truncates long messages and oversize fields', () => {
    const logger = new Logger({ isTauriRuntime: () => false })
    logger.error('x'.repeat(MAX_MESSAGE_CHARS + 40), {
      note: 'y'.repeat(MAX_FIELDS_JSON_BYTES + 8),
    })
    const [entry] = logger.getHistory()
    expect(Array.from(entry.message).length).toBe(MAX_MESSAGE_CHARS)
    expect(entry.data).toEqual({ truncated: true })
  })

  it('reuses the same session correlation id', () => {
    const logger = new Logger({
      isTauriRuntime: () => false,
      sessionId: 'same-session',
    })
    logger.error('one')
    logger.error('two')
    const history = logger.getHistory()
    expect(history[0].correlationId).toBe('same-session')
    expect(history[1].correlationId).toBe('same-session')
  })

  it('retries a missing command at most three times without growing the queue', async () => {
    vi.useFakeTimers()
    const appendFrontendLogs = vi.fn().mockRejectedValue(new Error('unknown command append_frontend_logs'))
    const logger = new Logger({
      isTauriRuntime: () => true,
      appendFrontendLogs,
    })

    logger.error('bridge failure')
    expect(logger.getBridgeQueueLength()).toBe(1)

    await logger.flushNativeBridge()
    expect(appendFrontendLogs).toHaveBeenCalledTimes(1)
    expect(logger.getBridgeQueueLength()).toBe(1)

    await vi.advanceTimersByTimeAsync(1000)
    expect(appendFrontendLogs).toHaveBeenCalledTimes(2)

    await vi.advanceTimersByTimeAsync(1000)
    expect(appendFrontendLogs).toHaveBeenCalledTimes(MAX_BRIDGE_ATTEMPTS)
    expect(logger.getBridgeQueueLength()).toBe(0)
  })
})
