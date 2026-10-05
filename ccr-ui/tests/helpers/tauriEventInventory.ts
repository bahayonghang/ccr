import { readdir, readFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

export interface FrontendEventRow {
  event: string
  owner: string
  lifecycle: string
  rustEmitLocation: string
}

const TESTS_DIR = dirname(fileURLToPath(import.meta.url))
export const FRONTEND_EVENT_INVENTORY_PATH = resolve(
  TESTS_DIR,
  '../fixtures/frontend-event-inventory.md',
)
export const TAURI_SRC_ROOT = resolve(TESTS_DIR, '../../src-tauri/src')

const INVENTORY_ROW_RE =
  /^\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*([^|]+?)\s*\|\s*`([^`]+)`\s*\|$/

const walkRustFiles = async (dir: string): Promise<string[]> => {
  const entries = await readdir(dir, { withFileTypes: true })
  const files: string[] = []
  for (const entry of entries) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) files.push(...(await walkRustFiles(fullPath)))
    else if (entry.name.endsWith('.rs')) files.push(fullPath)
  }
  return files
}

export const parseFrontendEventInventory = (markdown: string): FrontendEventRow[] => {
  const rows: FrontendEventRow[] = []
  for (const line of markdown.split(/\r?\n/)) {
    const match = INVENTORY_ROW_RE.exec(line)
    if (!match) continue
    rows.push({
      event: match[1],
      owner: match[2],
      lifecycle: match[3].trim(),
      rustEmitLocation: match[4],
    })
  }
  return rows
}

export const collectRustEmitNames = async (root = TAURI_SRC_ROOT): Promise<Set<string>> => {
  const names = new Set<string>()
  const channelConstRe = /pub const [A-Z0-9_]+: &str = "([^"]+)"/g
  const eventConstRe = /const EVENT_[A-Z0-9_]+: &str = "([^"]+)"/g
  const emitLiteralRe = /\.emit(?:_to)?\s*\((?:[^,]+,\s*)?"([^"]+)"/g
  // 传给 emit helper 的事件名（如 usage.rs 的 emit_usage_import_job_snapshot(..., "usage:job-failed")）。
  const colonEventStringRe = /"([a-z][a-z0-9_.-]*:[a-z0-9_.:-]+)"/g

  for (const file of await walkRustFiles(root)) {
    const source = await readFile(file, 'utf8')
    for (const match of source.matchAll(channelConstRe)) names.add(match[1])
    for (const match of source.matchAll(eventConstRe)) names.add(match[1])
    for (const match of source.matchAll(emitLiteralRe)) names.add(match[1])
    for (const match of source.matchAll(colonEventStringRe)) names.add(match[1])
    channelConstRe.lastIndex = 0
    eventConstRe.lastIndex = 0
    emitLiteralRe.lastIndex = 0
    colonEventStringRe.lastIndex = 0
  }
  return names
}
