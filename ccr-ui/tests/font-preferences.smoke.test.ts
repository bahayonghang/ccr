import { beforeEach, describe, expect, it } from 'vitest'

beforeEach(() => {
  localStorage.clear()
  document.documentElement.removeAttribute('style')
})

describe('fontPreferences smoke', () => {
  it('sanitizes injection-prone characters, collapses whitespace, and caps length', async () => {
    const { sanitizeFontFamily, MAX_FONT_NAME_LEN } = await import('@/utils/fontPreferences')

    expect(sanitizeFontFamily('  JetBrains   Mono  ')).toBe('JetBrains Mono')
    // 引号/分号/花括号/尖括号/括号/反斜杠/逗号一律剥离，防止破坏引号串或注入额外声明。
    expect(sanitizeFontFamily('Evil";}<>()\\,Font')).toBe('EvilFont')
    // 控制字符（含 DEL）剥离——用 fromCharCode 构造，避免源码内出现真实控制字节。
    const withControls = 'Fira' + String.fromCharCode(0, 31, 127) + 'Code'
    expect(sanitizeFontFamily(withControls)).toBe('FiraCode')
    expect(sanitizeFontFamily('a'.repeat(100)).length).toBe(MAX_FONT_NAME_LEN)
    expect(sanitizeFontFamily('')).toBe('')
  })

  it('persists and rehydrates ui/code fonts, empty value clears the key', async () => {
    const m = await import('@/utils/fontPreferences')

    m.persistUiFont('Inter')
    m.persistCodeFont('JetBrains Mono')
    expect(localStorage.getItem('ccr-font-ui')).toBe('Inter')
    expect(m.readStoredUiFont()).toBe('Inter')
    expect(m.readStoredCodeFont()).toBe('JetBrains Mono')

    m.persistUiFont('')
    expect(localStorage.getItem('ccr-font-ui')).toBeNull()
    expect(m.readStoredUiFont()).toBe('')
  })

  it('applies overrides as user-font prepended to the base fallback stack', async () => {
    const m = await import('@/utils/fontPreferences')
    const root = document.documentElement

    m.applyFontsToDocument('Inter', 'JetBrains Mono')
    expect(root.style.getPropertyValue('--font-sans')).toContain('"Inter"')
    expect(root.style.getPropertyValue('--font-sans')).toContain('var(--font-sans-base)')
    expect(root.style.getPropertyValue('--font-brand')).toContain('"Inter"')
    expect(root.style.getPropertyValue('--font-brand')).toContain('var(--font-brand-base)')
    expect(root.style.getPropertyValue('--font-mono')).toContain('"JetBrains Mono"')
    expect(root.style.getPropertyValue('--font-mono')).toContain('var(--font-mono-base)')
  })

  it('removes the inline override when the font is cleared (back to built-in stack)', async () => {
    const m = await import('@/utils/fontPreferences')
    const root = document.documentElement

    m.applyFontsToDocument('Inter', 'JetBrains Mono')
    m.applyFontsToDocument('', '')
    expect(root.style.getPropertyValue('--font-sans')).toBe('')
    expect(root.style.getPropertyValue('--font-brand')).toBe('')
    expect(root.style.getPropertyValue('--font-mono')).toBe('')
  })

  it('keeps ui and code channels independent', async () => {
    const m = await import('@/utils/fontPreferences')
    const root = document.documentElement

    m.applyFontsToDocument('Inter', '')
    expect(root.style.getPropertyValue('--font-sans')).toContain('"Inter"')
    expect(root.style.getPropertyValue('--font-mono')).toBe('')

    m.applyFontsToDocument('', 'Fira Code')
    expect(root.style.getPropertyValue('--font-sans')).toBe('')
    expect(root.style.getPropertyValue('--font-mono')).toContain('"Fira Code"')
  })

  it('applyInitialFonts hydrates from storage and applies overrides', async () => {
    localStorage.setItem('ccr-font-ui', 'Inter')
    const m = await import('@/utils/fontPreferences')

    const preference = m.applyInitialFonts()
    expect(preference.ui).toBe('Inter')
    expect(document.documentElement.style.getPropertyValue('--font-sans')).toContain('"Inter"')
  })
})
