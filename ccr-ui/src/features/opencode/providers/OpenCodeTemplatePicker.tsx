import { useCallback, useMemo, type ChangeEvent } from 'react'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { mergeProviderTemplates, readCustomProviderTemplates } from '@/utils/providerTemplates'
import { fieldInputClass } from '../ui-classes'

interface OpenCodeTemplatePickerProps {
  selectedTemplateId: string | null
  label: string
  helper: string
  manualLabel: string
  onSelect: (selection: ProviderTemplateSelection) => void
  onManual: () => void
}

export function OpenCodeTemplatePicker({
  selectedTemplateId,
  label,
  helper,
  manualLabel,
  onSelect,
  onManual,
}: OpenCodeTemplatePickerProps) {
  const templates = useMemo(
    () =>
      mergeProviderTemplates(BUILT_IN_PROVIDER_TEMPLATES, readCustomProviderTemplates()).filter(
        (item) => item.platforms.opencode,
      ),
    [],
  )

  const handleChange = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      const id = event.target.value
      if (!id) {
        onManual()
        return
      }
      const template = templates.find((item) => item.id === id)
      if (!template) return
      onSelect({
        template,
        endpoint: template.platforms.opencode?.baseURL || template.baseUrls?.[0],
      })
    },
    [onManual, onSelect, templates],
  )

  return (
    <label className="block text-xs font-semibold text-text-muted">
      {label}
      <select className={`${fieldInputClass} mt-2`} value={selectedTemplateId ?? ''} onChange={handleChange}>
        <option value="">{manualLabel}</option>
        {templates.map((template) => (
          <option key={template.id} value={template.id}>
            {template.name}
          </option>
        ))}
      </select>
      <span className="mt-1 block text-xs font-normal text-text-ghost">{helper}</span>
    </label>
  )
}
