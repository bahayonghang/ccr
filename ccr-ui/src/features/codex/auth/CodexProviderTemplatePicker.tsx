import { useCallback, useMemo, type ChangeEvent } from 'react'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { mergeProviderTemplates, readCustomProviderTemplates } from '@/utils/providerTemplates'
import { fieldInputClass } from '../ui-classes'

interface CodexProviderTemplatePickerProps {
  selectedTemplateId: string | null
  label: string
  helper: string
  manualLabel: string
  onSelect: (selection: ProviderTemplateSelection) => void
  onManual: () => void
}

export function CodexProviderTemplatePicker({
  selectedTemplateId,
  label,
  helper,
  manualLabel,
  onSelect,
  onManual,
}: CodexProviderTemplatePickerProps) {
  const templates = useMemo(
    () => mergeProviderTemplates(BUILT_IN_PROVIDER_TEMPLATES, readCustomProviderTemplates()).filter((item) => item.platforms.codex),
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
      onSelect({ template, endpoint: template.platforms.codex?.baseUrl || template.baseUrls?.[0] })
    },
    [onManual, onSelect, templates],
  )

  return (
    <label className="codex-auth-view__input-group">
      <span className="codex-auth-view__input-label">{label}</span>
      <select className={fieldInputClass} value={selectedTemplateId ?? ''} onChange={handleChange}>
        <option value="">{manualLabel}</option>
        {templates.map((template) => (
          <option key={template.id} value={template.id}>
            {template.name}
          </option>
        ))}
      </select>
      <span className="codex-auth-view__composer-helper">{helper}</span>
    </label>
  )
}
