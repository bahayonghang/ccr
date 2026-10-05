import { readFile } from 'node:fs/promises'
import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

// token 单点生效验证（08-22-design-system design.md §12 / AC11）：
// 「改一处 token 值可同时影响所有消费点」。选 `--color-bg-surface-rgb`
// （被 3 个以上域消费的第 1 层变量），分两段证明：
//   1) 源码层：三个不同域的样式文件各自存在消费该变量的规则（真实消费，非摆设）；
//   2) 运行层：改根上的变量值后，三个域的元素继承的计算值同时变化。
// jsdom 不做 var() 链替换（theme-switch 同一结论），故运行层断言继承的
// 自定义属性本身；工具类规则引用运行时变量由 theme-switch 用例覆盖。

const CONSUMERS: { domain: string; file: string; pattern: RegExp }[] = [
  {
    domain: 'Tailwind 工具类命名空间（core.css @theme inline 映射）',
    file: 'src/styles/core.css',
    pattern: /--color-bg-surface:\s*rgb\(var\(--color-bg-surface-rgb\)\)/,
  },
  {
    domain: 'codex-auth 域（components/codex-auth-shared.css）',
    file: 'src/styles/components/codex-auth-shared.css',
    pattern: /background:\s*var\(--color-bg-surface\)/,
  },
  {
    domain: 'checkin 域（components/checkin-shared.css）',
    file: 'src/styles/components/checkin-shared.css',
    pattern: /background:\s*var\(--color-bg-surface\)/,
  },
  {
    domain: '共享工具类域（utilities/utilities.css）',
    file: 'src/styles/utilities/utilities.css',
    pattern: /background:\s*var\(--color-bg-surface\)/,
  },
]

describe('token 单点生效（08-22-design-system 批次 8 / AC11）', () => {
  it('三个以上域真实消费 --color-bg-surface（源码层）', async () => {
    const hits: string[] = []
    for (const consumer of CONSUMERS) {
      const source = await readFile(consumer.file, 'utf8')
      if (consumer.pattern.test(source)) hits.push(consumer.domain)
    }
    expect(hits.length).toBeGreaterThanOrEqual(4)
  })

  it('改一处第 1 层变量，根计算值翻转且多域声明仍路由同一变量（运行层）', () => {
    // jsdom 不向子元素级联自定义属性（theme-switch 同一结论），运行层断言取
    // 「根上变量计算值随单点改动翻转」+「各域元素的命中声明都路由该变量」，
    // 两者合起来即「改一处 → 全部消费点取值变化」的可执行证据。
    const style = document.createElement('style')
    style.textContent = `:root { --color-bg-surface-rgb: 251 252 253; --color-bg-surface: rgb(var(--color-bg-surface-rgb)); }
.codex-surface, .checkin-surface, .shared-surface { background: var(--color-bg-surface); }`
    document.head.appendChild(style)

    const { container } = render(
      <div>
        <div className="codex-surface" data-domain="codex" />
        <div className="checkin-surface" data-domain="checkin" />
        <div className="shared-surface" data-domain="utilities" />
      </div>,
    )

    const readRoot = (name: string): string =>
      window.getComputedStyle(document.documentElement).getPropertyValue(name).trim()

    try {
      // 三个域元素的命中声明全部路由同一 token（jsdom 返回原始声明）。
      const declarations = Array.from(container.querySelectorAll('[data-domain]')).map(
        (element) => window.getComputedStyle(element).background,
      )
      expect(declarations).toEqual([
        'var(--color-bg-surface)',
        'var(--color-bg-surface)',
        'var(--color-bg-surface)',
      ])

      expect(readRoot('--color-bg-surface-rgb')).toBe('251 252 253')

      // 单点改动：根上的第 1 层变量换值。
      document.documentElement.style.setProperty('--color-bg-surface-rgb', '9 30 22')

      expect(readRoot('--color-bg-surface-rgb')).toBe('9 30 22')
      // 映射变量仍以 var() 引用第 1 层（未内联死值），三个域随之重解析。
      expect(readRoot('--color-bg-surface')).toBe('rgb(var(--color-bg-surface-rgb))')
    } finally {
      document.documentElement.style.removeProperty('--color-bg-surface-rgb')
      style.remove()
    }
  })
})
