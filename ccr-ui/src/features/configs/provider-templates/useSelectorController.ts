import { useCallback, useMemo, useState } from 'react'
import type {
  ProviderTemplate,
  ProviderTemplateDraftContext,
  ProviderTemplateOption,
  ProviderTemplatePlatform,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import { buildProviderTemplateOptions } from '@/utils/providerTemplates'
import { useConfigsProviderTemplates } from '../hooks/useProviderTemplates'
import {
  buildCustomTemplate,
  draftForCustomSave,
  emptyCustomTemplateForm,
  fillCustomForm,
  type CustomTemplateForm,
} from '../lib/templateForm'
import { tt } from '../locale'
import { useTemplateSearch } from './useTemplateSearch'

interface SelectorControllerInput {
  platform: ProviderTemplatePlatform
  disabled: boolean
  draftContext: ProviderTemplateDraftContext | null
  getDraftContext?: () => ProviderTemplateDraftContext | null
  onSelect: (selection: ProviderTemplateSelection) => void
  onManual: () => void
}

export function useSelectorController({
  platform,
  disabled,
  draftContext,
  getDraftContext,
  onSelect,
  onManual,
}: SelectorControllerInput) {
  const { templates, saveCustomTemplate, removeCustomTemplate } = useConfigsProviderTemplates()
  const [selectorOpen, setSelectorOpen] = useState(false)
  const [customOpen, setCustomOpen] = useState(false)
  const [activeIndex, setActiveIndex] = useState(0)
  const [editingCustomId, setEditingCustomId] = useState('')
  const [customError, setCustomError] = useState('')
  const [customInitial, setCustomInitial] = useState(emptyCustomTemplateForm)
  const allOptions = useMemo(
    () => buildProviderTemplateOptions(templates, platform),
    [platform, templates],
  )
  const search = useTemplateSearch(allOptions)
  const setQuery = search.setQuery

  const resolveDraft = useCallback(
    () => getDraftContext?.() ?? draftContext,
    [draftContext, getDraftContext],
  )

  const openSelector = useCallback(() => {
    if (disabled) return
    setQuery('')
    setActiveIndex(0)
    setSelectorOpen(true)
  }, [disabled, setQuery])

  const closeSelector = useCallback(() => setSelectorOpen(false), [])
  const closeCustom = useCallback(() => setCustomOpen(false), [])
  const selectManual = useCallback(() => {
    onManual()
    setSelectorOpen(false)
  }, [onManual])
  const selectOption = useCallback(
    (option: ProviderTemplateOption) => {
      onSelect({ template: option.template, endpoint: option.endpoint })
      setSelectorOpen(false)
    },
    [onSelect],
  )

  const openCustom = useCallback(
    (template?: ProviderTemplate, fromCurrent = false) => {
      setCustomError('')
      setEditingCustomId(template?.source === 'custom' ? template.id : '')
      setCustomInitial(fillCustomForm({ currentPlatform: platform, draft: resolveDraft(), template, fromCurrent }))
      setCustomOpen(true)
    },
    [platform, resolveDraft],
  )
  const openBlankCustom = useCallback(() => openCustom(), [openCustom])
  const openCurrentCustom = useCallback(() => openCustom(undefined, true), [openCustom])

  const saveCustom = useCallback(
    (values: CustomTemplateForm) => {
      const existing = editingCustomId ? templates.find((item) => item.id === editingCustomId) : undefined
      const draft = draftForCustomSave(platform, existing, resolveDraft())
      if (!draft) {
        setCustomError(tt('请先打开一个 provider 表单，再保存模板。', 'Open a provider form before saving a template.'))
        return
      }
      const built = buildCustomTemplate({ values, draft, existing })
      if (built.error || !built.template) {
        setCustomError(built.error || tt('模板名称不能为空。', 'Template name is required.'))
        return
      }
      saveCustomTemplate(built.template)
      setCustomOpen(false)
    },
    [editingCustomId, platform, resolveDraft, saveCustomTemplate, templates],
  )

  return {
    templates,
    allOptions,
    search,
    selectorOpen,
    customOpen,
    activeIndex,
    setActiveIndex,
    customError,
    customInitial,
    editingCustomId,
    removeCustomTemplate,
    openSelector,
    closeSelector,
    selectManual,
    selectOption,
    openCustom,
    openBlankCustom,
    openCurrentCustom,
    closeCustom,
    saveCustom,
  }
}
