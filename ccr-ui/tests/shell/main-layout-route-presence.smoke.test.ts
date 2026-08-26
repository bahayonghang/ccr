import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const source = readFileSync(path.join(root, 'src/shell/MainLayout.tsx'), 'utf8')
const css = readFileSync(path.join(root, 'src/styles/shell-critical.css'), 'utf8')

describe('MainLayout route presence', () => {
  it('does not wrap Outlet in AnimatePresence', () => {
    expect(source).not.toMatch(/<AnimatePresence/)
    expect(source).toContain('<Outlet')
    expect(source).toContain('MotionConfig')
    expect(source).toContain('className="route-page"')
    expect(css).toContain('.route-page')
    expect(css).toContain("[data-reduced-motion='true'] .route-page")
  })
})
