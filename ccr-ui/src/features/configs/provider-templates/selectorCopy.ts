import type { ProviderTemplate } from '@/types/providerTemplates'
import { tt } from '../locale'

export function selectorCopy(input: {
  selectedTemplate: ProviderTemplate | null
  selectedEndpoint: string
  optionCount: number
  label: string
  placeholder: string
}) {
  const sourceLabel =
    input.selectedTemplate?.source === 'custom' ? tt('自定义', 'Custom') : tt('内置', 'Built-in')
  const title = input.selectedTemplate?.name || input.placeholder || tt('选择模板', 'Choose a template')
  const summary = input.selectedTemplate
    ? [sourceLabel, input.selectedEndpoint.trim()].filter(Boolean).join(' · ')
    : tt(`${input.optionCount} 个可复用模板`, `${input.optionCount} reusable templates`)
  return {
    selectedLabel: input.label || tt('Provider template', 'Provider template'),
    sourceLabel,
    title,
    summary,
  }
}
