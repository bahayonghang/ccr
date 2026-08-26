import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { render, screen } from '@testing-library/react'
import { beforeAll, describe, expect, it } from 'vitest'
import {
  Badge,
  Button,
  ConfirmModal,
  EmptyState,
  FieldLabel,
  UrlText,
  buttonClass,
  type ButtonVariant,
} from '@/ui'

const root = join(import.meta.dirname, '..', '..')
const PRIMITIVES_CSS_PATH = join(root, 'src/ui/primitives.css')
const TOKENS_CSS_PATH = join(root, 'src/styles/tokens.css')

const BUTTON_VARIANTS: ButtonVariant[] = [
  'primary',
  'secondary',
  'ghost',
  'quiet',
  'warning',
  'danger',
  'accent-soft',
]

const UI_RULE_PREFIXES = ['.ui-btn', '.ui-badge', '.ui-field-label', '.ui-url-text']

const injectStyle = (css: string): HTMLStyleElement => {
  const style = document.createElement('style')
  style.textContent = css
  document.head.appendChild(style)
  return style
}

const extractRuleBlocks = (css: string, selectorPrefix: string): string[] => {
  const blocks: string[] = []
  const pattern = new RegExp(`${selectorPrefix.replace('.', '\\.')}[^,{]*\\{[^}]*\\}`, 'g')
  let match: RegExpExecArray | null
  while ((match = pattern.exec(css)) !== null) {
    blocks.push(match[0])
  }
  return blocks
}

beforeAll(() => {
  if (typeof globalThis.ResizeObserver === 'undefined') {
    class ResizeObserverStub {
      observe(): void {}
      unobserve(): void {}
      disconnect(): void {}
    }
    globalThis.ResizeObserver =
      ResizeObserverStub as unknown as typeof ResizeObserver
  }

  const mouseEventCtor = (globalThis.MouseEvent ?? Event) as unknown as typeof MouseEvent
  class PointerEventStub extends mouseEventCtor {
    readonly pointerId: number
    readonly pointerType: string
    readonly isPrimary: boolean

    constructor(type: string, params: PointerEventInit = {}) {
      super(type, {
        bubbles: params.bubbles,
        cancelable: params.cancelable,
        button: params.button ?? 0,
        ctrlKey: params.ctrlKey ?? false,
        clientX: params.clientX ?? 0,
        clientY: params.clientY ?? 0,
      })
      this.pointerId = params.pointerId ?? 0
      this.pointerType = params.pointerType ?? 'mouse'
      this.isPrimary = params.isPrimary ?? true
    }
  }
  if (typeof globalThis.PointerEvent === 'undefined') {
    const stub = PointerEventStub as unknown as typeof PointerEvent
    globalThis.PointerEvent = stub
    window.PointerEvent = stub
  }

  injectStyle(readFileSync(TOKENS_CSS_PATH, 'utf8'))
  injectStyle(readFileSync(PRIMITIVES_CSS_PATH, 'utf8'))
})

describe('action visual types (08-26-visual-type-primitives)', () => {
  const primitivesCss = readFileSync(PRIMITIVES_CSS_PATH, 'utf8')

  it('exports Button, Badge, FieldLabel, UrlText, and buttonClass from @/ui', () => {
    expect(typeof Button).toBe('function')
    expect(typeof Badge).toBe('function')
    expect(typeof FieldLabel).toBe('function')
    expect(typeof UrlText).toBe('function')
    expect(typeof buttonClass).toBe('function')
    expect(buttonClass({ variant: 'primary' })).toContain('ui-btn--primary')
  })

  it('keeps .ui-* rules free of hex and px literals', () => {
    const uiBlocks = UI_RULE_PREFIXES.flatMap((prefix) => extractRuleBlocks(primitivesCss, prefix))
    expect(uiBlocks.length).toBeGreaterThan(0)

    for (const block of uiBlocks) {
      expect(block).not.toMatch(/#[0-9a-fA-F]{3,8}/)
      expect(block).not.toMatch(/\b\d+px\b/)
    }
  })

  it('declares button variant tokens in primitives.css', () => {
    expect(primitivesCss).toContain('.ui-btn--primary')
    expect(primitivesCss).toMatch(/\.ui-btn--primary[\s\S]*background:\s*var\(--color-accent-primary\)/)
    expect(primitivesCss).toMatch(/\.ui-btn--secondary[\s\S]*background:\s*var\(--color-bg-surface\)/)
    expect(primitivesCss).toMatch(/\.ui-btn--ghost[\s\S]*background:\s*transparent/)
    expect(primitivesCss).toMatch(/\.ui-btn--quiet[\s\S]*border:\s*none/)
    expect(primitivesCss).toMatch(/\.ui-btn--warning[\s\S]*background:\s*var\(--color-warning-tint\)/)
    expect(primitivesCss).toMatch(/\.ui-btn--warning[\s\S]*border-color:\s*var\(--color-warning\)/)
    expect(primitivesCss).toMatch(/\.ui-btn--danger[\s\S]*background:\s*var\(--color-danger\)/)
    expect(primitivesCss).toMatch(
      /\.ui-btn--accent-soft[\s\S]*background:\s*rgb\(var\(--color-accent-primary-rgb\) \/ 14%\)/,
    )
    expect(primitivesCss).toMatch(/\.ui-btn:focus-visible[\s\S]*--color-accent-primary-rgb/)
    expect(primitivesCss).toMatch(/\.ui-btn:active:not\(:disabled\)[\s\S]*transform:\s*scale\(0\.96\)/)
    expect(primitivesCss).toMatch(/\.ui-btn:disabled[\s\S]*opacity:\s*0\.55/)
    expect(primitivesCss).toMatch(/\.ui-btn:disabled[\s\S]*cursor:\s*not-allowed/)
  })

  it('declares FieldLabel typography contract in primitives.css', () => {
    expect(primitivesCss).toMatch(/\.ui-field-label[\s\S]*font-size:\s*0\.75rem/)
    expect(primitivesCss).toMatch(/\.ui-field-label[\s\S]*letter-spacing:\s*0\.08em/)
    expect(primitivesCss).toMatch(/\.ui-field-label[\s\S]*color:\s*var\(--color-text-muted\)/)
  })

  it('cancels .ui-btn transform under prefers-reduced-motion', () => {
    expect(primitivesCss).toMatch(
      /@media \(prefers-reduced-motion: reduce\)[\s\S]*\.ui-btn:active:not\(:disabled\)[\s\S]*transform:\s*none/,
    )
  })

  it('renders equal heights for all seven variants at the same size', () => {
    const { container } = render(
      <div>
        {BUTTON_VARIANTS.map((variant) => (
          <Button key={variant} variant={variant} size="md">
            {variant}
          </Button>
        ))}
      </div>,
    )

    const buttons = Array.from(container.querySelectorAll('.ui-btn')) as HTMLElement[]
    expect(buttons).toHaveLength(BUTTON_VARIANTS.length)
    for (const button of buttons) {
      expect(button.className).toContain('ui-btn--md')
    }

    const heights = buttons.map((button) => button.getBoundingClientRect().height)
    if (heights.some((height) => height > 0)) {
      expect(new Set(heights).size).toBe(1)
      expect(heights[0]).toBeGreaterThan(0)
    }

    expect(primitivesCss).toMatch(/\.ui-btn[\s\S]*box-sizing:\s*border-box/)
    expect(primitivesCss).toMatch(/\.ui-btn--md[\s\S]*min-height:\s*var\(--space-11\)/)
    expect(primitivesCss).toMatch(/\.ui-btn--quiet[\s\S]*border:\s*none/)
  })

  it('uses static vs interactive badge cursor semantics', () => {
    const { container: staticContainer } = render(<Badge mode="static">Static</Badge>)
    const staticBadge = staticContainer.querySelector('.ui-badge--static') as HTMLElement
    expect(window.getComputedStyle(staticBadge).cursor).not.toBe('pointer')

    const { container: interactiveContainer } = render(<Badge mode="interactive">Interactive</Badge>)
    const interactiveBadge = interactiveContainer.querySelector('.ui-badge--interactive') as HTMLElement
    expect(window.getComputedStyle(interactiveBadge).cursor).toBe('pointer')
  })

  it('renders FieldLabel at 0.75 × root font size', () => {
    document.documentElement.style.fontSize = '16px'
    const { container } = render(<FieldLabel>Provider</FieldLabel>)
    const label = container.querySelector('.ui-field-label') as HTMLElement
    const rootPx = parseFloat(window.getComputedStyle(document.documentElement).fontSize)
    const labelSize = window.getComputedStyle(label).fontSize
    const labelPx = labelSize.endsWith('rem')
      ? parseFloat(labelSize) * rootPx
      : parseFloat(labelSize)
    expect(labelPx).toBeCloseTo(rootPx * 0.75, 1)
  })

  it('formats long pathname URLs and preserves illegal input', () => {
    const longUrl = 'https://api.example.com/abcdefghijklmnopqrstuvw'
    const { container: longContainer } = render(<UrlText value={longUrl} />)
    const longNode = longContainer.querySelector('.ui-url-text') as HTMLElement
    expect(longNode.textContent).toContain('api.example.com')
    expect(longNode.textContent).toContain('…')
    expect((longNode.textContent ?? '').length).toBeLessThan(longUrl.length)
    expect(longNode.getAttribute('title')).toBe(longUrl)

    const illegal = 'not-a-url'
    const { container: illegalContainer } = render(<UrlText value={illegal} />)
    const illegalNode = illegalContainer.querySelector('.ui-url-text') as HTMLElement
    expect(illegalNode.textContent).toBe(illegal)
    expect(illegalNode.getAttribute('title')).toBe(illegal)
  })

  it('maps EmptyState action to primary Button', () => {
    render(
      <EmptyState title="No items" actionText="Create" onAction={() => {}} />,
    )
    const action = screen.getByRole('button', { name: 'Create' })
    expect(action.className).toContain('ui-btn--primary')
  })

  it.each([
    ['danger', 'ui-btn--danger'],
    ['warning', 'ui-btn--warning'],
    ['info', 'ui-btn--primary'],
  ] as const)('maps ConfirmModal type=%s confirm to %s and cancel to ghost', (type, confirmClass) => {
    render(
      <ConfirmModal
        isOpen
        title="Confirm"
        message="Are you sure?"
        type={type}
        confirmText="Proceed"
        cancelText="Back"
      />,
    )

    const confirm = screen.getByRole('button', { name: 'Proceed' })
    const cancel = screen.getByRole('button', { name: 'Back' })
    expect(confirm.className).toContain(confirmClass)
    expect(cancel.className).toContain('ui-btn--ghost')
  })
})
