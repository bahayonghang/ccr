// 字体偏好：界面字体 / 代码字体。复刻 themeBootstrap 的 read/persist/apply 模式，
// 纯 localStorage、不触碰 Tauri 后端。用户选中的字体被 prepend 到内置回退栈头部
// （见 styles/tokens.css 的 --font-*-base），缺失或缺字形时浏览器自动沿栈回退。

const FONT_UI_STORAGE_KEY = 'ccr-font-ui'
const FONT_CODE_STORAGE_KEY = 'ccr-font-code'

// 字体族名长度上限，防御性截断超长输入。
export const MAX_FONT_NAME_LEN = 64

export interface FontPreference {
  ui: string
  code: string
}

// 三条字体轨的 CSS 变量与其内置回退基座。界面字体覆盖 sans+brand，代码字体覆盖 mono。
const UI_FONT_CHANNELS: readonly (readonly [cssVar: string, baseVar: string])[] = [
  ['--font-sans', '--font-sans-base'],
  ['--font-brand', '--font-brand-base'],
] as const
const CODE_FONT_CHANNELS: readonly (readonly [cssVar: string, baseVar: string])[] = [
  ['--font-mono', '--font-mono-base'],
] as const

/* ========== 字体预设清单 ========== */
// 下拉预设的字体“名称”字面量——它们是用户可选项（数据），不是 CSS 字体栈。集中
// 于此受控块内，作为 apple-glass-surface-contract 合同测试的例外，避免字体名散落进
// 组件样式而与真等宽栈禁令误撞。自定义输入不受此清单限制。
export const UI_FONT_PRESETS: readonly string[] = [
  'MapleBright',
  'PingFang SC',
  'Microsoft YaHei',
  'Noto Sans SC',
  'Source Han Sans SC',
  'Inter',
  'SF Pro',
  'Segoe UI',
]

export const CODE_FONT_PRESETS: readonly string[] = [
  'Cascadia Code',
  'JetBrains Mono',
  'Fira Code',
  'SF Mono',
  'Consolas',
  'Menlo',
  'Source Code Pro',
]
/* ========== 字体预设清单结束 ========== */

// 净化用户输入后再进 CSS 变量：去除会破坏引号串或构造注入的字符、控制字符、逗号
// （逗号会破坏“单字体族 + 我们补引号”的不变式），折叠空白并截断。
export const sanitizeFontFamily = (input: string): string => {
  if (typeof input !== 'string') return ''

  return (
    input
      .replace(/["'`\\;{}<>(),]/g, '')
      // eslint-disable-next-line no-control-regex
      .replace(/[\u0000-\u001f\u007f]/g, '')
      .replace(/\s+/g, ' ')
      .trim()
      .slice(0, MAX_FONT_NAME_LEN)
      .trim()
  )
}

const readStoredFont = (key: string): string => {
  if (typeof window === 'undefined') return ''

  try {
    return sanitizeFontFamily(localStorage.getItem(key) ?? '')
  } catch {
    return ''
  }
}

const persistFont = (key: string, value: string): void => {
  if (typeof window === 'undefined') return

  const sanitized = sanitizeFontFamily(value)
  try {
    if (sanitized) {
      localStorage.setItem(key, sanitized)
    } else {
      localStorage.removeItem(key)
    }
  } catch {
    // 忽略存储异常，保留当前 UI 状态即可。
  }
}

export const readStoredUiFont = (): string => readStoredFont(FONT_UI_STORAGE_KEY)
export const readStoredCodeFont = (): string => readStoredFont(FONT_CODE_STORAGE_KEY)
export const persistUiFont = (value: string): void => persistFont(FONT_UI_STORAGE_KEY, value)
export const persistCodeFont = (value: string): void => persistFont(FONT_CODE_STORAGE_KEY, value)

export const readStoredFontPreference = (): FontPreference => ({
  ui: readStoredUiFont(),
  code: readStoredCodeFont(),
})

// 组合值：`"用户字体", var(--font-*-base)`。净化后必不含双引号，引号串安全。
export const composeFontStack = (font: string, baseVar: string): string =>
  `"${font}", var(${baseVar})`

const applyFontChannels = (
  channels: readonly (readonly [string, string])[],
  rawFont: string
): void => {
  if (typeof document === 'undefined') return

  const root = document.documentElement
  const font = sanitizeFontFamily(rawFont)

  for (const [cssVar, baseVar] of channels) {
    if (font) {
      root.style.setProperty(cssVar, composeFontStack(font, baseVar))
    } else {
      // 移除内联覆盖 → 回落到 stylesheet 的 var(--font-*-base)。
      root.style.removeProperty(cssVar)
    }
  }
}

export const applyUiFontToDocument = (font: string): void =>
  applyFontChannels(UI_FONT_CHANNELS, font)
export const applyCodeFontToDocument = (font: string): void =>
  applyFontChannels(CODE_FONT_CHANNELS, font)

export const applyFontsToDocument = (ui: string, code: string): void => {
  applyUiFontToDocument(ui)
  applyCodeFontToDocument(code)
}

export const applyInitialFonts = (): FontPreference => {
  const preference = readStoredFontPreference()
  applyFontsToDocument(preference.ui, preference.code)
  return preference
}
