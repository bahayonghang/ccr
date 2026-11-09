import { open, readFile, unlink, writeFile } from 'node:fs/promises'
import { randomUUID } from 'node:crypto'
import { fileURLToPath } from 'node:url'

const runtime = process.versions.bun ? 'bun' : 'node'
const samples = [
  'export type Value = { label: string, };    \n',
  '/** 标签说明 */\nexport type Value = { model: string, };    \n',
  '/** 配置与用量 */\r\nexport type Value = { generated_at: string, };      \r\n',
]
const results = []
for (const source of samples) {
  const path = fileURLToPath(new URL('.write-probe-' + randomUUID() + '.txt', import.meta.url))
  const handle = await open(path, 'wx')
  await handle.close()
  const expected = source.replace(/[ \t]+$/gm, '').trimEnd() + '\n'
  const result = { iterations: 0, mismatch: false, expectedBytes: Buffer.byteLength(expected) }
  try {
    for (let index = 0; index < 50; index++) {
      await writeFile(path, source, 'utf8')
      await writeFile(path, expected, 'utf8')
      const actual = await readFile(path, 'utf8')
      result.iterations++
      if (actual !== expected) {
        result.mismatch = true
        result.actualBytes = Buffer.byteLength(actual)
        result.actualTail = actual.slice(-35)
        break
      }
    }
  } finally {
    await unlink(path)
  }
  results.push(result)
}
const record = { runtime, version: process.versions.bun || process.version, platform: process.platform, results }
await writeFile(new URL('normalizer-write-probe-' + runtime + '.json', import.meta.url), JSON.stringify(record, null, 2) + '\n', 'utf8')
process.stdout.write(JSON.stringify(record) + '\n')
