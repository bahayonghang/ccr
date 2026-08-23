/** OpenCode 主题渲染变量。独立于 CCR @theme，前缀固定 `--oc-`。 */
export const OC_THEME_VAR_PREFIX = '--oc-'

export const OC_THEME_VARS = [
  '--oc-theme-id',
  '--oc-theme-name',
  '--oc-theme-type',
  '--oc-theme-swatch',
] as const

export type OcThemeVar = (typeof OC_THEME_VARS)[number]
