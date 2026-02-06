<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in"
    @click.self="handleClose"
  >
    <!-- Background Backdrop -->
    <div
      class="absolute inset-0 bg-bg-base/95 backdrop-blur-md"
      @click="handleClose"
    />

    <!-- Modal Content -->
    <Card 
      ref="modalRef" 
      variant="elevated"
      class="relative w-full max-w-4xl max-h-[90vh] overflow-y-auto p-0 shadow-2xl animate-scale-in bg-bg-elevated"
    >
      <!-- Header -->
      <div class="sticky top-0 z-10 px-6 py-4 border-b border-border-subtle bg-bg-elevated backdrop-blur-xl flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-accent-success/10 text-accent-success">
            <Plus class="w-5 h-5" />
          </div>
          <div>
            <h2
              :id="titleId"
              class="text-lg font-bold text-text-primary"
            >
              {{ $t('configs.addConfig.title') }}
            </h2>
            <p class="text-xs text-text-secondary">
              {{ $t('configs.addConfig.subtitle') }}
            </p>
          </div>
        </div>
        <Button
          variant="ghost"
          size="icon"
          @click="handleClose"
        >
          <X class="w-5 h-5" />
        </Button>
      </div>

      <div class="p-6">
        <!-- Template Selection -->
        <section class="mb-8">
          <label class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-4">
            🚀 {{ $t('configs.addConfig.selectTemplate') }}
          </label>
          <div class="grid grid-cols-2 md:grid-cols-5 gap-3">
            <button
              v-for="template in templates"
              :key="template.id"
              class="group relative p-3 rounded-xl border transition-all duration-300 text-left hover:shadow-lg hover:-translate-y-1"
              :class="selectedTemplate === template.id 
                ? 'bg-accent-primary/10 border-accent-primary ring-1 ring-accent-primary/50' 
                : 'bg-bg-surface/50 border-border-default hover:border-accent-primary/50'"
              @click="applyTemplate(template)"
            >
              <div class="text-2xl mb-2 grayscale group-hover:grayscale-0 transition-all">
                {{ template.icon }}
              </div>
              <h3
                class="font-bold text-sm text-text-primary"
                :class="selectedTemplate === template.id ? 'text-accent-primary' : ''"
              >
                {{ template.label }}
              </h3>
              <p class="text-[10px] text-text-secondary line-clamp-2 mt-1 opacity-80">
                {{ template.description }}
              </p>
            </button>
          </div>
        </section>

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
              <label class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-1.5 ml-1">
                {{ $t('configs.addConfig.providerType') }}
              </label>
              <div class="relative">
                <select
                  v-model="formData.provider_type"
                  class="w-full bg-bg-surface border border-border-default rounded-xl px-4 py-2.5 text-sm text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20 appearance-none shadow-sm hover:border-border-strong cursor-pointer"
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
      <div class="sticky bottom-0 z-10 px-6 py-4 border-t border-border-subtle bg-bg-elevated backdrop-blur-xl flex gap-3 justify-end">
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
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus, X } from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { addConfig } from '@/api'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import type { UpdateConfigRequest } from '@/types'

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
const saving = ref(false)
const selectedTemplate = ref<string | null>(null)
const tagsInput = ref('')
const formData = ref<UpdateConfigRequest>({
  name: '', description: '', base_url: '', auth_token: '',
  model: '', small_fast_model: '', provider: '',
  provider_type: '', account: '', tags: []
})

const templates = [
  { id: 'cc_relay', label: 'CC Relay', description: 'Official Relay', icon: '🔄', base_url: 'https://api.claudecc.com', model: 'claude-sonnet-4-20250514', provider_type: 'official_relay' },
  { id: 'kimi', label: 'Moonshot', description: 'Kimi AI', icon: '🌙', base_url: 'https://api.moonshot.cn/v1', model: 'moonshot-v1-128k', provider_type: 'third_party_model' },
  { id: 'zhipu', label: 'Zhipu GLM', description: 'ChatGLM', icon: '🧠', base_url: 'https://open.bigmodel.cn/api/paas/v4', model: 'glm-4.6', provider_type: 'third_party_model' },
  { id: 'deepseek', label: 'DeepSeek', description: 'DeepSeek Chat', icon: '🔍', base_url: 'https://api.deepseek.com/v1', model: 'deepseek-chat', provider_type: 'third_party_model' },
  { id: 'qwen', label: 'Qwen', description: 'Tongyi Qianwen', icon: '☁️', base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1', model: 'qwen-max', provider_type: 'third_party_model' }
]

const isFormValid = computed(() => 
  formData.value.name.trim() && formData.value.base_url.trim() && formData.value.auth_token.trim()
)

const applyTemplate = (tpl: any) => {
  selectedTemplate.value = tpl.id
  formData.value = { ...formData.value, ...tpl }
  if(!formData.value.name) formData.value.name = tpl.id.replace(/_/g, '-')
}

const handleSave = async () => {
  if (!isFormValid.value) return
  saving.value = true
  try {
    const tags = tagsInput.value.split(',').map(t => t.trim()).filter(Boolean)
    await addConfig({ ...formData.value, tags: tags.length ? tags : undefined })
    emit('saved')
    handleClose()
  } catch (e: any) { alert(e.message) }
  finally { saving.value = false }
}

const resetForm = () => {
  formData.value = { name: '', description: '', base_url: '', auth_token: '', model: '', small_fast_model: '', provider: '', provider_type: '', account: '', tags: [] }
  tagsInput.value = ''
  selectedTemplate.value = null
}

watch(() => props.isOpen, val => val && resetForm())
</script>
