import { computed, ref } from 'vue'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import type { ProviderTemplate } from '@/types/providerTemplates'
import {
  deleteCustomProviderTemplate,
  mergeProviderTemplates,
  readCustomProviderTemplates,
  upsertCustomProviderTemplate,
  writeCustomProviderTemplates,
} from '@/utils/providerTemplates'

const customTemplates = ref<ProviderTemplate[]>(readCustomProviderTemplates())

const persist = () => {
  writeCustomProviderTemplates(customTemplates.value)
}

export function useProviderTemplates() {
  const templates = computed(() => mergeProviderTemplates(
    BUILT_IN_PROVIDER_TEMPLATES,
    customTemplates.value,
  ))

  const saveCustomTemplate = (template: ProviderTemplate) => {
    customTemplates.value = upsertCustomProviderTemplate(customTemplates.value, template)
    persist()
  }

  const removeCustomTemplate = (id: string) => {
    customTemplates.value = deleteCustomProviderTemplate(customTemplates.value, id)
    persist()
  }

  const reloadCustomTemplates = () => {
    customTemplates.value = readCustomProviderTemplates()
  }

  return {
    builtInTemplates: BUILT_IN_PROVIDER_TEMPLATES,
    customTemplates,
    templates,
    saveCustomTemplate,
    removeCustomTemplate,
    reloadCustomTemplates,
  }
}
