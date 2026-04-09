type TranslateValues = Record<string, string | number | boolean | null | undefined>

type TranslateFn = (key: string, values?: TranslateValues) => string

const PLACEHOLDER_RE = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g

export const hasTemplatePlaceholder = (value: string) => /\{[a-zA-Z_][a-zA-Z0-9_]*\}/.test(value)

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
