type JsonRecord = Record<string, unknown>

export const settingsRecord = (value: unknown): JsonRecord =>
  value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as JsonRecord
    : {}

function readPath(value: unknown, path: readonly string[]): unknown {
  return path.reduce<unknown>((current, key) => settingsRecord(current)[key], value)
}

function setPath(target: JsonRecord, path: readonly string[], value: unknown): void {
  const [head, ...tail] = path
  if (!head) return
  if (tail.length === 0) {
    target[head] = value
    return
  }
  const child = { ...settingsRecord(target[head]) }
  setPath(child, tail, value)
  target[head] = child
}

/** Use source siblings only for APIs that replace submitted top-level objects. */
export function dirtySettingsPatch(
  fields: Readonly<Record<string, string>>,
  payload: unknown,
  input: { dirtyKeys: readonly string[]; source: unknown },
): JsonRecord {
  const patch: JsonRecord = {}
  for (const id of input.dirtyKeys) {
    const key = fields[id]
    if (!key) continue
    const path = key.split('.')
    const root = path[0]!
    if (path.length > 1 && !(root in patch)) patch[root] = settingsRecord(input.source)[root]
    setPath(patch, path, readPath(payload, path))
  }
  return patch
}
