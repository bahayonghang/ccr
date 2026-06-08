<template>
  <ProviderTemplateSelector
    platform="claude"
    :selected-template-id="selectedId"
    label="Provider template"
    helper="Search built-in and custom provider templates."
    @select="handleSelect"
    @manual="emit('select', null)"
  />
</template>

<script setup lang="ts">
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import type { PlatformPresets, ProviderPreset } from '@/types/providerPresets'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { mapTemplateToClaudeLegacyConfigPatch } from '@/utils/providerTemplates'

defineProps<{
  presets: PlatformPresets
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [preset: ProviderPreset | null]
}>()

function handleSelect(selection: ProviderTemplateSelection) {
  const patch = mapTemplateToClaudeLegacyConfigPatch(selection.template, selection.endpoint)

  emit('select', {
    id: selection.template.id,
    name: selection.template.name,
    category: selection.template.category === 'local' ? 'third_party' : selection.template.category,
    websiteUrl: selection.template.websiteUrl,
    apiKeyUrl: selection.template.apiKeyUrl,
    isPartner: selection.template.isPartner,
    base_url: patch.base_url || '',
    model: patch.model,
    small_fast_model: patch.small_fast_model,
    provider: patch.provider,
    provider_type: patch.provider_type as ProviderPreset['provider_type'],
    description: patch.description,
  })
}
</script>
