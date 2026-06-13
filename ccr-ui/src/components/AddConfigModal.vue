<!-- -->
<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in"
    @click.self="handleClose"
  >
    <!-- Background Backdrop -->
    <div
      class="absolute inset-0 /95 backdrop-blur-md"
      @click="handleClose"
    />

    <!-- Modal Content -->
    <Card 
      ref="modalRef" 
      variant="elevated"
      class="relative w-full max-w-4xl max-h-[90vh] overflow-y-auto p-0 shadow-2xl animate-scale-in glass-surface"
    >
      <!-- Header -->
      <div class="sticky top-0 z-10 px-6 py-4 border-b border-border-default/10 glass-surface backdrop-blur-md flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-accent-success/10 text-accent-success">
            <SIcon
              name="Plus"
              size="w-5 h-5"
            />
          </div>
          <div>
            <h2
              :id="titleId"
              class="text-lg font-bold text-white"
            >
              {{ $t('configs.addConfig.title') }}
            </h2>
            <p class="text-xs text-text-primary">
              {{ $t('configs.addConfig.subtitle') }}
            </p>
          </div>
        </div>
        <Button
          variant="ghost"
          size="icon"
          @click="handleClose"
        >
          <SIcon
            name="X"
            size="w-5 h-5"
          />
        </Button>
      </div>

      <div class="p-6">
        <!-- Preset Provider Selection -->
        <ProviderTemplateSelector
          platform="claude"
          :selected-template-id="selectedTemplate"
          :selected-endpoint="selectedEndpoint"
          :draft-context="claudeLegacyTemplateDraft"
          label="Provider template"
          helper="Search built-in and custom non-secret templates, then fill provider fields in one step."
          @select="applyTemplate"
          @manual="applyManualTemplate"
        />

        <div class="h-px bg-border-subtle mb-8" />

        <!-- Form -->
        <form
          class="space-y-6"
          @submit.prevent="handleSave"
        >
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Input
              v-model="formData.name"
              :label="$t('configs.addConfig.name')"

              :placeholder="$t('configs.addConfig.namePlaceholder')"
            >
              <template #leading>
                <span class="text-text-muted">#</span>
              </template>
            </Input>

            <Input
              v-model="formData.description"
              :label="$t('configs.addConfig.description')"
              :placeholder="$t('configs.addConfig.descriptionPlaceholder')"
            >
              <template #leading>
                <SIcon
                  name="FileText"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <Input
              v-model="formData.base_url"
              label="Base URL"

              placeholder="https://api.example.com"
              class="md:col-span-2"
            >
              <template #leading>
                <SIcon
                  name="Globe"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <Input
              v-model="formData.auth_token"
              label="Auth Token"

              type="password"
              :placeholder="$t('configs.addConfig.tokenPlaceholder')"
              class="md:col-span-2"
            >
              <template #leading>
                <SIcon
                  name="KeyRound"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <Input
              v-model="formData.model"
              label="Model"
              :placeholder="$t('configs.addConfig.modelPlaceholder')"
            >
              <template #leading>
                <SIcon
                  name="Bot"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <Input
              v-model="formData.small_fast_model"
              label="Fast Model"
              :placeholder="$t('configs.addConfig.smallModelPlaceholder')"
            >
              <template #leading>
                <SIcon
                  name="Zap"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <div class="w-full">
              <label class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-1.5 ml-1">
                {{ $t('configs.addConfig.providerType') }}
              </label>
              <div class="relative">
                <select
                  v-model="formData.provider_type"
                  class="w-full glass-surface border border-border-default/25 rounded-xl px-4 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-accent-primary/20 appearance-none shadow-sm hover:border-border-strong cursor-pointer"
                >
                  <option value="">
                    {{ $t('configs.addConfig.providerUncategorized') }}
                  </option>
                  <option value="official_relay">
                    {{ $t('configs.addConfig.providerOfficialRelay') }}
                  </option>
                  <option value="third_party_model">
                    {{ $t('configs.addConfig.providerThirdParty') }}
                  </option>
                </select>
                <div class="absolute inset-y-0 right-3 flex items-center pointer-events-none text-text-muted">
                  <SIcon
                    name="ChevronDown"
                    size="w-4 h-4"
                  />
                </div>
              </div>
            </div>

            <Input
              v-model="formData.provider"
              :label="$t('configs.addConfig.providerName')"
              :placeholder="$t('configs.addConfig.providerNamePlaceholder')"
              :hint="$t('configs.addConfig.providerNameHint')"
            >
              <template #leading>
                <SIcon
                  name="Building2"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <Input
              v-model="formData.account"
              :label="$t('configs.addConfig.account')"
              :placeholder="$t('configs.addConfig.accountPlaceholder')"
              :hint="$t('configs.addConfig.accountHint')"
            >
              <template #leading>
                <SIcon
                  name="User"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>

            <Input
              v-model="tagsInput"
              :label="$t('configs.addConfig.tags')"
              :placeholder="$t('configs.addConfig.tagsPlaceholder')"
              :hint="$t('configs.addConfig.tagsHint')"
            >
              <template #leading>
                <SIcon
                  name="Tags"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </template>
            </Input>
          </div>
        </form>
      </div>

      <!-- Footer -->
      <div class="sticky bottom-0 z-10 px-6 py-4 border-t border-border-default/10 glass-surface backdrop-blur-md flex gap-3 justify-end">
        <Button
          variant="ghost"
          @click="handleClose"
        >
          Cancel
        </Button>
        <Button 
          variant="primary" 
          :loading="saving" 
          :disabled="!isFormValid"
          @click="handleSave"
        >
          {{ saving ? $t('configs.addConfig.saving') : $t('configs.addConfig.save') }}
        </Button>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { addConfig } from '@/api'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import { useUIStore } from '@/stores/ui'
import type { UpdateConfigRequest } from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { mapTemplateToClaudeLegacyConfigPatch } from '@/utils/providerTemplates'
import { MODAL_FOCUS_DELAY_MS } from '@/config/constants'

const props = defineProps<{ isOpen: boolean }>()
const emit = defineEmits(['close', 'saved'])
useI18n()

// Accessibility
const titleId = useUniqueId('add-config-title')
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = ref(props.isOpen)
watch(() => props.isOpen, val => isOpenRef.value = val)

const handleClose = () => emit('close')
const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)
useEscapeKey(handleClose, isOpenRef)
watch(isOpenRef, val => val && setTimeout(() => focusFirstElement(), MODAL_FOCUS_DELAY_MS))

// Form Logic
const uiStore = useUIStore()
const saving = ref(false)
const selectedTemplate = ref<string | null>(null)
const selectedEndpoint = ref('')
const tagsInput = ref('')
const formData = ref<UpdateConfigRequest>({
  name: '', description: '', base_url: '', auth_token: '',
  model: '', small_fast_model: '', provider: '',
  provider_type: '', account: '', tags: []
})

const isFormValid = computed(() =>
  formData.value.name.trim() && formData.value.base_url.trim() && formData.value.auth_token.trim()
)

const claudeLegacyTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'claude',
  defaultName: formData.value.provider || formData.value.name || 'Claude provider',
  name: formData.value.provider || formData.value.name,
  category: 'third_party',
  baseUrls: formData.value.base_url.trim() ? [formData.value.base_url.trim()] : [],
  modelCatalog: [formData.value.model, formData.value.small_fast_model].filter(Boolean) as string[],
  platformOverride: {
    baseUrl: formData.value.base_url,
    provider: formData.value.provider,
    providerType: formData.value.provider_type,
    model: formData.value.model,
    smallFastModel: formData.value.small_fast_model,
    description: formData.value.description,
  },
}))

const applyManualTemplate = () => {
  selectedTemplate.value = null
  selectedEndpoint.value = ''
  formData.value = {
    name: '', description: '', base_url: '', auth_token: '',
    model: '', small_fast_model: '', provider: '',
    provider_type: '', account: '', tags: [],
  }
}

const applyTemplate = (selection: ProviderTemplateSelection) => {
  const patch = mapTemplateToClaudeLegacyConfigPatch(selection.template, selection.endpoint)

  selectedTemplate.value = selection.template.id
  selectedEndpoint.value = selection.endpoint || ''
  formData.value = {
    ...formData.value,
    base_url: patch.base_url || '',
    model: patch.model || '',
    small_fast_model: patch.small_fast_model || '',
    provider: patch.provider || selection.template.name,
    provider_type: patch.provider_type || '',
    description: patch.description || formData.value.description,
  }

  if (!formData.value.name) {
    formData.value.name = patch.suggestedName || selection.template.id
  }
}

const handleSave = async () => {
  if (!isFormValid.value) return
  saving.value = true
  try {
    const tags = tagsInput.value.split(',').map(t => t.trim()).filter(Boolean)
    await addConfig({ ...formData.value, tags: tags.length ? tags : undefined })
    uiStore.showSuccess('Configuration added successfully')
    emit('saved')
    handleClose()
  } catch (e) { 
    uiStore.showError((e instanceof Error ? e.message : "Error") || 'Failed to add configuration')
  }
  finally { saving.value = false }
}

const resetForm = () => {
  formData.value = { name: '', description: '', base_url: '', auth_token: '', model: '', small_fast_model: '', provider: '', provider_type: '', account: '', tags: [] }
  tagsInput.value = ''
  selectedTemplate.value = null
  selectedEndpoint.value = ''
}

watch(() => props.isOpen, val => val && resetForm())
</script>

