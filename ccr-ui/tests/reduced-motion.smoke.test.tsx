import { readFile } from 'node:fs/promises'
import { describe, expect, it, vi } from 'vitest'
import {
  applyReducedMotionToDocument,
  readPrefersReducedMotion,
  REDUCED_MOTION_ATTRIBUTE,
} from '@/utils/reducedMotion'

// reduced motion 单点收敛（08-22-design-system 批次 7 / AC8 前半 + design.md §9）：
//   1) 唯一读系统偏好的模块把结果同步到根 data-reduced-motion 属性并跟随变化；
//   2) src/styles 的 @media (prefers-reduced-motion) 只剩 shell-critical.css 一处兜底，
//      其余降级规则挂属性选择器；
//   3) motion 在 src 内尚无消费点（无双驱动时点核查，animation-disposition.md §二）。

const mediaQueryListeners: ((event: { matches: boolean }) => void)[] = []

const installMatchMediaStub = (initialMatches: boolean) => {
  mediaQueryListeners.length = 0
  const stub = {
    matches: initialMatches,
    addEventListener: (_: string, listener: (event: { matches: boolean }) => void) => {
      mediaQueryListeners.push(listener)
    },
    removeEventListener: (_: string, listener: (event: { matches: boolean }) => void) => {
      const index = mediaQueryListeners.indexOf(listener)
      if (index >= 0) mediaQueryListeners.splice(index, 1)
    },
  }
  vi.stubGlobal('matchMedia', vi.fn().mockReturnValue(stub))
  return stub
}

describe('reduced motion（08-22-design-system 批次 7 / AC8 前半）', () => {
  it('读系统偏好并同步根属性，跟随系统变化，dispose 后解绑', () => {
    const stub = installMatchMediaStub(true)
    document.documentElement.removeAttribute(REDUCED_MOTION_ATTRIBUTE)

    expect(readPrefersReducedMotion()).toBe(true)

    const subscription = applyReducedMotionToDocument()
    expect(subscription.reduced).toBe(true)
    expect(document.documentElement.getAttribute(REDUCED_MOTION_ATTRIBUTE)).toBe('true')

    // 系统偏好翻转 → 属性同步
    stub.matches = false
    mediaQueryListeners.forEach((listener) => listener({ matches: false }))
    expect(document.documentElement.getAttribute(REDUCED_MOTION_ATTRIBUTE)).toBe('false')

    // dispose 后不再跟随
    subscription.dispose()
    stub.matches = true
    mediaQueryListeners.forEach((listener) => listener({ matches: true }))
    expect(document.documentElement.getAttribute(REDUCED_MOTION_ATTRIBUTE)).toBe('false')

    vi.unstubAllGlobals()
  })

  it('@media (prefers-reduced-motion) 在 src/styles 只剩 shell-critical 一处兜底', async () => {
    const files: [string, string][] = [
      ['src/styles/shell-critical.css', await readFile('src/styles/shell-critical.css', 'utf8')],
      ['src/styles/base/base.css', await readFile('src/styles/base/base.css', 'utf8')],
      ['src/styles/utilities/utilities.css', await readFile('src/styles/utilities/utilities.css', 'utf8')],
      ['src/styles/components/home.css', await readFile('src/styles/components/home.css', 'utf8')],
      ['src/styles/components/profiles-page.css', await readFile('src/styles/components/profiles-page.css', 'utf8')],
    ]

    const occurrences = files
      .map(([path, source]) => ({
        path,
        count: (source.match(/@media \(prefers-reduced-motion/g) ?? []).length,
      }))
      .filter((entry) => entry.count > 0)

    expect(occurrences).toEqual([{ path: 'src/styles/shell-critical.css', count: 1 }])

    // 五层降级规则全部改为挂属性选择器。
    for (const [path, source] of files) {
      expect(source, `${path} 应含属性门控降级规则`).toContain("[data-reduced-motion='true']")
    }
  })

  it('animations.css 保留集与判定表一致：进出场类已删，装饰/反馈/悬停类保留', async () => {
    const animations = await readFile('src/styles/animations.css', 'utf8')

    // 保留集
    for (const retained of [
      '@keyframes pulse-subtle',
      '@keyframes bounce-in',
      '@keyframes shake',
      '@keyframes gradient-shift',
      '@keyframes border-glow',
      '.animate-pulse-subtle',
      '.animate-bounce-in',
      '.animate-shake',
      '.animate-gradient-shift',
      '.gpu-accelerate',
      '.hover-animate',
      '.nav-hover-effect',
    ]) {
      expect(animations).toContain(retained)
    }

    // 删除集（进出场 / Vue 过渡 / spin 重复定义）
    for (const removed of [
      '@keyframes fade-in',
      '@keyframes fade-out',
      '@keyframes slide-up',
      '@keyframes scale-in',
      '@keyframes modal-enter',
      '@keyframes backdrop-enter',
      '@keyframes sidebar-item-enter',
      '@keyframes card-enter',
      '@keyframes spin',
      '.animate-fade-in',
      '.animate-slide-up',
      '.animate-scale-in',
      '.animate-spin',
      '.animate-modal-enter',
      '.page-enter-active',
      '.page-slide-lateral-enter-from',
      '.animate-delay-',
      '.animate-fill-',
      'will-change',
    ]) {
      expect(animations, `已删除段不应再出现：${removed}`).not.toContain(removed)
    }
  })

  it('reduced-motion 偏好的读取点在 src 内只有 reducedMotion.ts；main.tsx 已接线', async () => {
    const mainTsx = await readFile('src/main.tsx', 'utf8')
    expect(mainTsx).toContain('applyReducedMotionToDocument')

    const { readdir } = await import('node:fs/promises')
    const { join } = await import('node:path')
    const walk = async (dir: string): Promise<string[]> => {
      const entries = await readdir(dir, { withFileTypes: true })
      const files: string[] = []
      for (const entry of entries) {
        const fullPath = join(dir, entry.name)
        if (entry.isDirectory()) files.push(...(await walk(fullPath)))
        else if (/\.(ts|tsx)$/.test(entry.name)) files.push(fullPath)
      }
      return files
    }

    const readers: string[] = []
    for (const file of await walk('src')) {
      if (file === join('src', 'utils', 'reducedMotion.ts')) continue
      const source = await readFile(file, 'utf8')
      if (source.includes("matchMedia('(prefers-reduced-motion")) {
        readers.push(file)
      }
    }
    // .vue 内的存量读取属阶段 5 迁移范围；ts/tsx 侧唯一读取点是本模块。
    expect(readers).toEqual([])
  })
})
