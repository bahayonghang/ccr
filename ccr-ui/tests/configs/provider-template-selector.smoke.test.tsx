import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { ProviderTemplateSelector } from '@/features/configs/provider-templates/ProviderTemplateSelector'

describe('ProviderTemplateSelector', () => {
  it('opens the selector and exposes a manual row', async () => {
    const onSelect = vi.fn()
    const onManual = vi.fn()
    render(
      <ProviderTemplateSelector
        platform="claude"
        selectedTemplateId={null}
        onSelect={onSelect}
        onManual={onManual}
      />,
    )
    fireEvent.click(screen.getByTestId('provider-template-trigger'))
    expect(await screen.findByTestId('provider-template-manual-row')).toBeTruthy()
    fireEvent.click(screen.getByTestId('provider-template-manual-row'))
    expect(onManual).toHaveBeenCalledTimes(1)
  })
})
