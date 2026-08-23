type TranslateValues = Record<string, string | number | boolean | null | undefined>

type TranslateFn = (key: string, values?: TranslateValues) => string

const PLACEHOLDER_RE = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g

/** 词条里的字面量插值写法：`{'{'}` / `{'}'}` / `{'@'}` / `{'{"k":"v"}'}`。 */
const VUE_I18N_LITERAL_RE = /\{'((?:\\'|[^'])*)'\}/g

export const hasTemplatePlaceholder = (value: string) => /\{[a-zA-Z_][a-zA-Z0-9_]*\}/.test(value)

export function unescapeVueI18nLiterals(template: string): string {
  return template.replace(VUE_I18N_LITERAL_RE, (_raw, inner: string) => inner.replace(/\\'/g, "'"))
}

export function cloneUnescapedMessages(input: unknown): Record<string, unknown> {
  const walk = (value: unknown): unknown => {
    if (typeof value === 'string') return unescapeVueI18nLiterals(value)
    if (Array.isArray(value)) return value.map(walk)
    if (value && typeof value === 'object') {
      const next: Record<string, unknown> = {}
      for (const [key, nested] of Object.entries(value as Record<string, unknown>)) {
        next[key] = walk(nested)
      }
      return next
    }
    return value
  }
  return walk(input) as Record<string, unknown>
}

const interpolateTemplate = (template: string, values: TranslateValues = {}) =>
  template.replace(PLACEHOLDER_RE, (_match, key: string) => {
    const value = values[key]
    return value == null ? `{${key}}` : String(value)
  })

export const translateWithFallback = (
  translate: TranslateFn,
  key: string,
  fallback: string,
  values: TranslateValues = {},
) => {
  const resolved = translate(key, values)
  if (resolved !== key && !hasTemplatePlaceholder(resolved)) {
    return resolved
  }

  const interpolatedResolved = resolved === key ? '' : interpolateTemplate(resolved, values)
  if (interpolatedResolved && !hasTemplatePlaceholder(interpolatedResolved)) {
    return interpolatedResolved
  }

  return interpolateTemplate(fallback, values)
}
