import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { randomUUID } from 'node:crypto'
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import type { ProfileEditorAdapter, ProfileWriteOutcome } from '@/configs/profileEditorAdapter'
import {
  claudeProfilePresentation,
  codexProfilePresentation,
  grokProfilePresentation,
  type ProfilePresentationView,
} from '@/configs/profilePresentation'
import { ProfileEditorModal } from '@/components/profiles/ProfileEditorModal'
import { claudeProfileEditorAdapter } from '@/features/claude/profiles/claudeProfileEditorAdapter'
import { codexProfileEditorAdapter } from '@/features/codex/profiles/codexProfileEditorAdapter'
import { grokProfileEditorAdapter } from '@/features/grok/profiles/grokProfileEditorAdapter'

beforeAll(() => {
  if (typeof globalThis.ResizeObserver === 'undefined') {
    class ResizeObserverStub {
      observe(): void {}
      unobserve(): void {}
      disconnect(): void {}
    }
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }
})

interface StubForm {
  name: string
  url: string
  model: string
  secret: string
  tags: string
  enabled: boolean
  timeout: string
  hidden: string
}

const presentation: ProfilePresentationView = claudeProfilePresentation

const emptyForm = (): StubForm => ({
  name: '',
  url: '',
  model: '',
  secret: '',
  tags: '',
  enabled: true,
  timeout: '',
  hidden: 'nope',
})

const collectDataValues = (root: HTMLElement): string[] => {
  const values: string[] = []
  for (const node of root.querySelectorAll('*')) {
    for (const attr of node.attributes) {
      if (attr.name.startsWith('data-')) values.push(attr.value)
    }
  }
  return values
}

const makeAdapter = (
  submit: () => Promise<ProfileWriteOutcome>,
  extras?: { advanced?: boolean },
): ProfileEditorAdapter<StubForm, StubForm> => ({
  createEmpty: emptyForm,
  fromRecord: (record) => ({ ...record, secret: '' }),
  sections: [
    {
      id: 'identity',
      titleKey: 'profileEditor.createTitle',
      layout: 'grid',
      fields: [
        { key: 'name', labelKey: 'profileEditor.save', kind: 'text', required: () => true },
        { key: 'url', labelKey: 'profileEditor.save', kind: 'mono-text' },
      ],
    },
    {
      id: 'auth',
      titleKey: 'profileEditor.save',
      layout: 'group',
      fields: [
        { key: 'secret', labelKey: 'profileEditor.save', kind: 'secret', required: () => true },
        { key: 'hidden', labelKey: 'profileEditor.save', kind: 'text', visible: () => false },
      ],
    },
    {
      id: 'runtime',
      titleKey: 'profileEditor.save',
      layout: 'row',
      fields: [
        { key: 'model', labelKey: 'profileEditor.save', kind: 'choice', options: ['alpha', 'beta'] },
        {
          key: 'tags',
          labelKey: 'profileEditor.save',
          kind: 'multi-value',
          options: ['work'],
        },
        { key: 'enabled', labelKey: 'profileEditor.save', kind: 'boolean' },
        { key: 'timeout', labelKey: 'profileEditor.save', kind: 'number' },
      ],
    },
    ...(extras?.advanced
      ? [
          {
            id: 'advanced',
            titleKey: 'profileEditor.advanced',
            layout: 'grid' as const,
            advanced: true,
            fields: [{ key: 'url', labelKey: 'profileEditor.save', kind: 'text' as const }],
          },
        ]
      : []),
  ],
  validate: (form) =>
    form.name.trim()
      ? []
      : [{ section: 'identity', field: 'name', message: 'name-required' }],
  submit: async (form) => {
    lastSubmitted = form
    return submit()
  },
})

let lastSubmitted: StubForm | null = null

const renderEditor = (
  adapter: ProfileEditorAdapter<StubForm, StubForm>,
  opts?: {
    originalName?: string | null
    target?: StubForm | null
    onClose?: () => void
    onApply?: (name: string) => Promise<void>
    onDone?: (outcome: ProfileWriteOutcome, applied: boolean) => void
  },
) => {
  lastSubmitted = null
  const onClose = opts?.onClose ?? vi.fn()
  const onApply = opts?.onApply ?? vi.fn(async () => undefined)
  const onDone = opts?.onDone ?? vi.fn()
  const result = render(
    <ProfileEditorModal
      open
      adapter={adapter}
      presentation={presentation}
      target={opts?.target ?? null}
      originalName={opts?.originalName ?? null}
      existingNames={['taken']}
      onClose={onClose}
      onApply={onApply}
      onDone={onDone}
    />,
  )
  return { ...result, onClose, onApply, onDone }
}

describe('profile editor shell', () => {
  beforeEach(() => {
    lastSubmitted = null
  })

  it('opens the same modal for create and edit with matching footer hints', () => {
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => ({ status: 'ok' }))
    const adapter = makeAdapter(submit)
    const created = renderEditor(adapter)
    expect(screen.getByTestId('profile-editor-shell').getAttribute('data-mode')).toBe('create')
    expect(screen.getByTestId('profile-editor-hint').textContent).toMatch(/appendHint|追加/)
    created.unmount()

    renderEditor(adapter, {
      originalName: 'work',
      target: { ...emptyForm(), name: 'work' },
    })
    expect(screen.getByTestId('profile-editor-shell').getAttribute('data-mode')).toBe('edit')
    expect(screen.getByTestId('profile-editor-hint').textContent).toMatch(/overwriteHint|覆盖/)
  })

  it('keeps advanced sections collapsed and omits the control when none exist', () => {
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => ({ status: 'ok' }))
    const plain = renderEditor(makeAdapter(submit))
    expect(screen.queryByTestId('profile-editor-advanced')).toBeNull()
    plain.unmount()

    renderEditor(makeAdapter(submit, { advanced: true }))
    const toggle = screen.getByTestId('profile-editor-advanced-toggle')
    expect(toggle.getAttribute('aria-expanded')).toBe('false')
    expect(screen.queryByTestId('profile-editor-section-advanced')).toBeNull()
  })

  it('renders seven field kinds, hides invisible fields, and marks required ones', () => {
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => ({ status: 'ok' }))
    renderEditor(makeAdapter(submit))
    const root = screen.getByTestId('profile-editor-shell')
    const kinds = [...root.querySelectorAll('[data-kind]')].map((node) => node.getAttribute('data-kind'))
    expect(kinds.sort()).toEqual(
      ['boolean', 'choice', 'mono-text', 'multi-value', 'number', 'secret', 'text'].sort(),
    )
    expect(root.querySelector('[data-field="hidden"]')).toBeNull()
    expect(root.querySelector('[data-field="name"]')?.closest('[data-required="true"]')).toBeTruthy()
    expect(root.querySelector('input[type="password"]')).toBeTruthy()
  })

  it('lists validation issues and jumps to the failing section', () => {
    const scrollIntoView = vi.fn()
    Element.prototype.scrollIntoView = scrollIntoView
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => ({ status: 'ok' }))
    renderEditor(makeAdapter(submit))
    fireEvent.click(screen.getByTestId('profile-editor-save'))
    expect(screen.getByTestId('profile-editor-summary').textContent).toContain('name-required')
    fireEvent.click(screen.getByTestId('profile-editor-jump-identity'))
    expect(scrollIntoView).toHaveBeenCalled()
  })

  it('accepts model and tag values outside the option list', async () => {
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => ({ status: 'ok' }))
    renderEditor(makeAdapter(submit))
    const root = screen.getByTestId('profile-editor-shell')
    fireEvent.change(root.querySelector('[data-field="name"] input') as HTMLInputElement, {
      target: { value: 'custom-profile' },
    })
    fireEvent.change(root.querySelector('[data-kind="choice"] input') as HTMLInputElement, {
      target: { value: 'custom-model' },
    })
    const tagInput = root.querySelector('[data-kind="multi-value"] input') as HTMLInputElement
    fireEvent.change(tagInput, { target: { value: 'custom-tag' } })
    fireEvent.keyDown(tagInput, { key: 'Enter' })
    fireEvent.click(screen.getByTestId('profile-editor-save'))
    await waitFor(() => {
      expect(lastSubmitted?.model).toBe('custom-model')
      expect(lastSubmitted?.tags).toContain('custom-tag')
    })
  })

  it('closes only on ok and calls apply only after a successful save-and-apply', async () => {
    const outcomes: ProfileWriteOutcome[] = [
      { status: 'recovery', kind: 'rename_apply_failed', message: 'recover' },
      { status: 'blocked', message: 'blocked', forceAllowed: true },
      { status: 'error', message: 'failed' },
      { status: 'ok' },
    ]
    let index = 0
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => outcomes[index++] ?? { status: 'ok' })
    const adapter = makeAdapter(submit)
    const fillName = () => {
      fireEvent.change(
        screen.getByTestId('profile-editor-shell').querySelector('[data-field="name"] input') as HTMLInputElement,
        { target: { value: 'ok-name' } },
      )
    }

    const recovery = renderEditor(adapter)
    fillName()
    fireEvent.click(screen.getByTestId('profile-editor-save-apply'))
    await waitFor(() => expect(screen.getByTestId('profile-editor-error').textContent).toContain('recover'))
    expect(recovery.onClose).not.toHaveBeenCalled()
    expect(recovery.onApply).not.toHaveBeenCalled()
    recovery.unmount()

    const blocked = renderEditor(adapter)
    fillName()
    fireEvent.click(screen.getByTestId('profile-editor-save'))
    await waitFor(() => expect(screen.getByTestId('profile-editor-error').textContent).toContain('blocked'))
    expect(blocked.onClose).not.toHaveBeenCalled()
    expect(blocked.onApply).not.toHaveBeenCalled()
    blocked.unmount()

    const errored = renderEditor(adapter)
    fillName()
    fireEvent.click(screen.getByTestId('profile-editor-save-apply'))
    await waitFor(() => expect(screen.getByTestId('profile-editor-error').textContent).toContain('failed'))
    expect(errored.onClose).not.toHaveBeenCalled()
    expect(errored.onApply).not.toHaveBeenCalled()
    errored.unmount()

    const ok = renderEditor(adapter)
    fillName()
    fireEvent.click(screen.getByTestId('profile-editor-save-apply'))
    await waitFor(() => expect(ok.onClose).toHaveBeenCalled())
    expect(ok.onApply).toHaveBeenCalledWith('ok-name')
  })

  it('does not leak a secret sentinel through text, console, errors, or data attributes', async () => {
    const sentinel = randomUUID()
    const logs: unknown[] = []
    const spy = (method: 'log' | 'info' | 'warn' | 'error' | 'debug') =>
      vi.spyOn(console, method).mockImplementation((...args) => {
        logs.push(...args)
      })
    const spies = [spy('log'), spy('info'), spy('warn'), spy('error'), spy('debug')]
    const submit = vi.fn(async (): Promise<ProfileWriteOutcome> => ({ status: 'error', message: 'save-failed' }))
    renderEditor(makeAdapter(submit))
    const root = screen.getByTestId('profile-editor-shell')
    fireEvent.change(root.querySelector('[data-field="name"] input') as HTMLInputElement, {
      target: { value: 'named' },
    })
    fireEvent.change(root.querySelector('input[type="password"]') as HTMLInputElement, {
      target: { value: sentinel },
    })
    fireEvent.click(screen.getByTestId('profile-editor-save'))
    await waitFor(() => expect(screen.getByTestId('profile-editor-error').textContent).toBe('save-failed'))
    const dialog = screen.getByRole('dialog')
    expect(dialog.textContent ?? '').not.toContain(sentinel)
    expect(screen.getByTestId('profile-editor-error').textContent).not.toContain(sentinel)
    expect(JSON.stringify(logs)).not.toContain(sentinel)
    expect(collectDataValues(dialog).join('|')).not.toContain(sentinel)
    spies.forEach((item) => item.mockRestore())
  })

  it('opens under claude, codex, and grok adapters', () => {
    const shared = {
      open: true,
      target: null,
      originalName: null,
      existingNames: [] as string[],
      onClose: vi.fn(),
    }
    const claude = render(
      <ProfileEditorModal
        {...shared}
        adapter={claudeProfileEditorAdapter}
        presentation={claudeProfilePresentation}
      />,
    )
    expect(screen.getByTestId('profile-editor-shell')).toBeTruthy()
    claude.unmount()

    const codex = render(
      <ProfileEditorModal
        {...shared}
        adapter={codexProfileEditorAdapter}
        presentation={codexProfilePresentation}
      />,
    )
    expect(screen.getByTestId('profile-editor-shell')).toBeTruthy()
    codex.unmount()

    render(
      <ProfileEditorModal
        {...shared}
        adapter={grokProfileEditorAdapter}
        presentation={grokProfilePresentation}
      />,
    )
    expect(screen.getByTestId('profile-editor-shell')).toBeTruthy()
  })
})
