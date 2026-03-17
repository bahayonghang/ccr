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
      <div class="sticky top-0 z-10 px-6 py-4 border-b border-white/5 glass-surface backdrop-blur-md flex items-center justify-between">
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
            <p class="text-xs text-white/80">
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
        <ProviderPresetSelector
          :presets="claudePresets"
          :selected-id="selectedTemplate"
          @select="applyPreset"
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
                <span class="text-white/50">#</span>
              </template>
            </Input>

            <Input
              v-model="formData.description"
              :label="$t('configs.addConfig.description')"
              :placeholder="$t('configs.addConfig.descriptionPlaceholder')"
            >
              <template #leading>
                <span class="text-lg">📝</span>
              </template>
            </Input>

            <Input
              v-model="formData.base_url"
              label="Base URL"

              placeholder="https://api.example.com"
              class="md:col-span-2"
            >
              <template #leading>
                <span class="text-lg">🌐</span>
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
                <span class="text-lg">🔑</span>
              </template>
            </Input>

            <Input
              v-model="formData.model"
              label="Model"
              :placeholder="$t('configs.addConfig.modelPlaceholder')"
            >
              <template #leading>
                <span class="text-lg">🤖</span>
              </template>
            </Input>

            <Input
              v-model="formData.small_fast_model"
              label="Fast Model"
              :placeholder="$t('configs.addConfig.smallModelPlaceholder')"
            >
              <template #leading>
                <span class="text-lg">⚡</span>
              </template>
            </Input>

            <div class="w-full">
              <label class="block text-xs font-bold uppercase tracking-wider text-white/50 mb-1.5 ml-1">
                {{ $t('configs.addConfig.providerType') }}
              </label>
              <div class="relative">
                <select
                  v-model="formData.provider_type"
                  class="w-full glass-surface border border-white/20 rounded-xl px-4 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-accent-primary/20 appearance-none shadow-sm hover:border-border-strong cursor-pointer"
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
                <div class="absolute inset-y-0 right-3 flex items-center pointer-events-none text-white/50">
                  ▼
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
                <span class="text-lg">🏢</span>
              </template>
            </Input>

            <Input
              v-model="formData.account"
              :label="$t('configs.addConfig.account')"
              :placeholder="$t('configs.addConfig.accountPlaceholder')"
              :hint="$t('configs.addConfig.accountHint')"
            >
              <template #leading>
                <span class="text-lg">👤</span>
              </template>
            </Input>

            <Input
              v-model="tagsInput"
              :label="$t('configs.addConfig.tags')"
              :placeholder="$t('configs.addConfig.tagsPlaceholder')"
              :hint="$t('configs.addConfig.tagsHint')"
            >
              <template #leading>
                <span class="text-lg">🏷️</span>
              </template>
            </Input>
          </div>
        </form>
      </div>

      <!-- Footer -->
      <div class="sticky bottom-0 z-10 px-6 py-4 border-t border-white/5 glass-surface backdrop-blur-md flex gap-3 justify-end">
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
import ProviderPresetSelector from '@/components/configs/ProviderPresetSelector.vue'
import { useUIStore } from '@/stores/ui'
import type { UpdateConfigRequest } from '@/types'
import type { ProviderPreset } from '@/types/providerPresets'
import { claudePresets } from '@/configs/providerPresets'

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
watch(isOpenRef, val => val && setTimeout(() => focusFirstElement(), 100))

// Form Logic
const uiStore = useUIStore()
const saving = ref(false)
const selectedTemplate = ref<string | null>(null)
const tagsInput = ref('')
const formData = ref<UpdateConfigRequest>({
  name: '', description: '', base_url: '', auth_token: '',
  model: '', small_fast_model: '', provider: '',
  provider_type: '', account: '', tags: []
})

const isFormValid = computed(() =>
  formData.value.name.trim() && formData.value.base_url.trim() && formData.value.auth_token.trim()
)

const applyPreset = (preset: ProviderPreset | null) => {
  if (!preset) {
    // 自定义配置 - 清空所有字段
    selectedTemplate.value = null
    formData.value = {
      name: '', description: '', base_url: '', auth_token: '',
      model: '', small_fast_model: '', provider: '',
      provider_type: '', account: '', tags: [],
    }
    return
  }
  selectedTemplate.value = preset.id
  formData.value = {
    ...formData.value,
    base_url: preset.base_url,
    model: preset.model || '',
    small_fast_model: preset.small_fast_model || '',
    provider: preset.provider || preset.name,
    provider_type: preset.provider_type || '',
    description: preset.description || '',
  }
  // 自动填充 name 建议（用户可修改）
  if (!formData.value.name) {
    formData.value.name = preset.id
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
}

watch(() => props.isOpen, val => val && resetForm())
</script>
