/**
 * 共享 i18n 叶子 key 与泄漏检测。检测器由实际叶子 key 集合生成（TPR-08），
 * 粗筛正则每段接受 [A-Za-z0-9_]，判定仍以集合命中为准。
 */

/** 递归收集叶子 key。数组与函数不当叶子。 */
export function* leafKeys(obj, prefix = '') {
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      yield* leafKeys(value, path)
    } else {
      yield path
    }
  }
}

/** 粗筛：至少两段、每段 [A-Za-z0-9_]。含下划线的 key 不会被漏掉。 */
export const KEY_SHAPE_RE = /[A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_]+)+/g

export function findLeakedKeys(text, keySet) {
  const hits = []
  const seen = new Set()
  KEY_SHAPE_RE.lastIndex = 0
  let match
  while ((match = KEY_SHAPE_RE.exec(text))) {
    const token = match[0]
    if (keySet.has(token) && !seen.has(token)) {
      seen.add(token)
      hits.push(token)
    }
  }
  return hits
}
