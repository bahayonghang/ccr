import { useMemo } from 'react'
import { create } from 'zustand'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import type { ProviderTemplate } from '@/types/providerTemplates'
import {
  deleteCustomProviderTemplate,
  mergeProviderTemplates,
  readCustomProviderTemplates,
  upsertCustomProviderTemplate,
  writeCustomProviderTemplates,
} from '@/utils/providerTemplates'

interface ProviderTemplatesState {
  customTemplates: ProviderTemplate[]
  saveCustomTemplate: (template: ProviderTemplate) => void
  removeCustomTemplate: (id: string) => void
}

export const useConfigsProviderTemplatesStore = create<ProviderTemplatesState>()((set, get) => ({
  customTemplates: readCustomProviderTemplates(),
  saveCustomTemplate: (template) => {
    const next = upsertCustomProviderTemplate(get().customTemplates, template)
    writeCustomProviderTemplates(next)
    set({ customTemplates: next })
  },
  removeCustomTemplate: (id) => {
    const next = deleteCustomProviderTemplate(get().customTemplates, id)
    writeCustomProviderTemplates(next)
    set({ customTemplates: next })
  },
}))

export function useConfigsProviderTemplates() {
  const customTemplates = useConfigsProviderTemplatesStore((state) => state.customTemplates)
  const saveCustomTemplate = useConfigsProviderTemplatesStore((state) => state.saveCustomTemplate)
  const removeCustomTemplate = useConfigsProviderTemplatesStore((state) => state.removeCustomTemplate)
  const templates = useMemo(
    () => mergeProviderTemplates(BUILT_IN_PROVIDER_TEMPLATES, customTemplates),
    [customTemplates],
  )
  return { templates, saveCustomTemplate, removeCustomTemplate }
}
