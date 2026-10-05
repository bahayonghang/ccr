import { useCallback, type KeyboardEvent } from 'react'
import type {
  ProviderTemplateDraftContext,
  ProviderTemplatePlatform,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import { PROVIDER_TEMPLATE_PLATFORM_LABELS } from '@/utils/providerTemplates'
import { CustomTemplateEditor } from './CustomTemplateEditor'
import { handleSelectorKeyDown } from './selectorKeyboard'
import { selectorCopy } from './selectorCopy'
import { SelectedSummary } from './SelectedSummary'
import { SelectorTrigger } from './SelectorTrigger'
import { TemplateSelectorModal } from './TemplateSelectorModal'
import { useSelectorController } from './useSelectorController'
import '../styles/provider-templates.css'

interface ProviderTemplateSelectorProps {
  platform: ProviderTemplatePlatform
  selectedTemplateId?: string | null
  selectedEndpoint?: string
  label?: string
  helper?: string
  placeholder?: string
  disabled?: boolean
  draftContext?: ProviderTemplateDraftContext | null
  getDraftContext?: () => ProviderTemplateDraftContext | null
  onSelect: (selection: ProviderTemplateSelection) => void
  onManual: () => void
}

export function ProviderTemplateSelector({
  platform,
  selectedTemplateId = null,
  selectedEndpoint = '',
  label = '',
  helper = '',
  placeholder = '',
  disabled = false,
  draftContext = null,
  getDraftContext,
  onSelect,
  onManual,
}: ProviderTemplateSelectorProps) {
  const ctrl = useSelectorController({
    platform,
    disabled,
    draftContext,
    getDraftContext,
    onSelect,
    onManual,
  })
  const selectedTemplate = ctrl.templates.find((template) => template.id === selectedTemplateId) ?? null
  const copy = selectorCopy({
    selectedTemplate,
    selectedEndpoint,
    optionCount: ctrl.allOptions.length,
    label,
    placeholder,
  })

  const optionId = useCallback(
    (index: number) => `provider-template-option-${platform}-${index}`,
    [platform],
  )
  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      handleSelectorKeyDown(event, {
        visibleCount: ctrl.search.results.length + 1,
        activeIndex: ctrl.activeIndex,
        results: ctrl.search.results,
        selectManual: ctrl.selectManual,
        selectOption: ctrl.selectOption,
        setActiveIndex: ctrl.setActiveIndex,
        close: ctrl.closeSelector,
      })
    },
    [ctrl],
  )

  return (
    <section className="provider-template-selector">
      <SelectorTrigger
        label={copy.selectedLabel}
        helper={helper}
        disabled={disabled}
        title={copy.title}
        subtitle={copy.summary}
        onOpen={ctrl.openSelector}
      />
      {selectedTemplate ? (
        <SelectedSummary sourceLabel={copy.sourceLabel} name={selectedTemplate.name} endpoint={selectedEndpoint} />
      ) : null}
      <TemplateSelectorModal
        open={ctrl.selectorOpen}
        platformLabel={PROVIDER_TEMPLATE_PLATFORM_LABELS[platform]}
        searchInputId={`provider-template-search-${platform}`}
        query={ctrl.search.query}
        results={ctrl.search.results}
        activeIndex={ctrl.activeIndex}
        optionId={optionId}
        onClose={ctrl.closeSelector}
        onQueryChange={ctrl.search.setQuery}
        onHover={ctrl.setActiveIndex}
        onSelectManual={ctrl.selectManual}
        onSelectOption={ctrl.selectOption}
        onEdit={ctrl.openCustom}
        onDelete={ctrl.removeCustomTemplate}
        onNewCustom={ctrl.openBlankCustom}
        onSaveCurrent={getDraftContext || draftContext ? ctrl.openCurrentCustom : undefined}
        onKeyDown={handleKeyDown}
      />
      <CustomTemplateEditor
        open={ctrl.customOpen}
        editing={Boolean(ctrl.editingCustomId)}
        initial={ctrl.customInitial}
        error={ctrl.customError}
        onClose={ctrl.closeCustom}
        onSave={ctrl.saveCustom}
      />
    </section>
  )
}
