<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in"
    @click.self="handleClose"
  >
    <!-- Background Backdrop -->
    <div
      class="absolute inset-0 bg-black/70 backdrop-blur-md"
      @click="handleClose"
    />

    <!-- Modal Content -->
    <Card 
      ref="modalRef" 
      variant="glass"
      class="relative w-full max-w-4xl max-h-[90vh] overflow-y-auto p-0 shadow-2xl animate-scale-in"
    >
      <!-- Header -->
      <div class="sticky top-0 z-10 px-8 py-6 border-b border-border-subtle bg-bg-elevated flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="p-3 rounded-xl bg-accent-primary/10 text-accent-primary">
            <Settings class="w-6 h-6" />
          </div>
          <div>
            <h2
              :id="titleId"
              class="text-xl font-bold bg-gradient-to-r from-accent-primary to-accent-secondary bg-clip-text text-transparent"
            >
              Edit Configuration
            </h2>
            <p class="text-xs text-text-secondary font-mono flex items-center gap-1">
              <span>ID:</span> {{ configName }}
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

      <!-- Body -->
      <div class="p-8">
        <!-- Loading -->
        <div
          v-if="loading"
          class="flex justify-center py-20"
        >
          <Spinner
            size="lg"
            class="text-accent-primary"
          />
        </div>

        <!-- Form -->
        <form
          v-else
          class="space-y-6"
          @submit.prevent="handleSave"
        >
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Input
              v-model="formData.description"
              label="Description"
              placeholder="Brief description of this config"
              class="md:col-span-2"
            >
              <template #leading>
                <span class="text-lg">📝</span>
              </template>
            </Input>

            <Input
              v-model="formData.base_url"
              label="Base URL"
              placeholder="https://api.anthropic.com"
              class="md:col-span-2"
            >
              <template #leading>
                <span class="text-lg">🌐</span>
              </template>
            </Input>

            <Input
              v-model="formData.auth_token"
              label="Auth Token"
              placeholder="sk-..."
              type="password"
              class="md:col-span-2"
            >
              <template #leading>
                <span class="text-lg">🔑</span>
              </template>
            </Input>

            <Input
              v-model="formData.model"
              label="Default Model"
              placeholder="claude-3-opus-20240229"
            >
              <template #leading>
                <span class="text-lg">🤖</span>
              </template>
            </Input>

            <Input
              v-model="formData.small_fast_model"
              label="Fast Model"
              placeholder="claude-3-haiku-20240307"
            >
              <template #leading>
                <span class="text-lg">⚡</span>
              </template>
            </Input>

            <div class="w-full">
              <label class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-1.5 ml-1">Provider Type</label>
              <div class="relative">
                <select
                  v-model="formData.provider_type"
                  class="w-full bg-bg-surface/50 border border-border-default rounded-xl px-4 py-2.5 text-sm text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20 appearance-none"
                >
                  <option value="">
                    Uncategorized
                  </option>
                  <option value="official_relay">
                    Official Relay
                  </option>
                  <option value="third_party_model">
                    Third Party
                  </option>
                </select>
                <div class="absolute inset-y-0 right-3 flex items-center pointer-events-none text-text-muted">
                  ▼
                </div>
              </div>
            </div>

            <Input
              v-model="formData.provider"
              label="Provider Name"
              placeholder="e.g. anthropic"
              hint="Grouping identifier"
            >
              <template #leading>
                <span class="text-lg">🏢</span>
              </template>
            </Input>

            <Input
              v-model="formData.account"
              label="Account ID"
              placeholder="e.g. personal"
              hint="Account differentiator"
            >
              <template #leading>
                <span class="text-lg">👤</span>
              </template>
            </Input>

            <Input
              v-model="tagsInput"
              label="Tags"
              placeholder="production, backup, test"
              hint="Comma separated"
              class="md:col-span-2"
            >
              <template #leading>
                <span class="text-lg">🏷️</span>
              </template>
            </Input>
          </div>
        </form>
      </div>

      <!-- Footer -->
      <div class="sticky bottom-0 z-10 px-8 py-6 border-t border-border-subtle bg-bg-elevated flex gap-4">
        <Button
          variant="ghost"
          class="flex-1"
          @click="handleClose"
        >
          Cancel
        </Button>
        <Button
          variant="primary"
          class="flex-1"
          :loading="saving"
          @click="handleSave"
        >
          Save Changes
        </Button>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Settings, X } from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { getConfig, updateConfig } from '@/api'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Spinner from '@/components/ui/Spinner.vue'

interface Props {
  isOpen: boolean
  configName: string
}

const props = defineProps<Props>()
const emit = defineEmits(['close', 'saved'])

const titleId = useUniqueId('edit-config-title')
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = ref(props.isOpen)

watch(() => props.isOpen, (val) => isOpenRef.value = val)

const handleClose = () => emit('close')
const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)
useEscapeKey(handleClose, isOpenRef)

watch(isOpenRef, (open) => { if(open) setTimeout(() => focusFirstElement(), 100) })

const loading = ref(false)
const saving = ref(false)
const tagsInput = ref('')
// typed
import type { UpdateConfigRequest, ConfigItem } from '@/types'

const formData = ref<Partial<ConfigItem>>({})

const loadConfig = async () => {
  if (!props.configName) return
  loading.value = true
  try {
    const data = await getConfig(props.configName)
    formData.value = { ...data }
    tagsInput.value = Array.isArray(data.tags) ? data.tags.join(', ') : ''
  } catch (e: any) { alert(e.message) }
  finally { loading.value = false }
}

const handleSave = async () => {
  saving.value = true
  try {
    const tags = tagsInput.value.split(',').map(t => t.trim()).filter(Boolean)
    // Construct valid request payload
    const payload: UpdateConfigRequest = {
       name: props.configName,
       description: formData.value.description,
       base_url: formData.value.base_url || '',
       auth_token: formData.value.auth_token || '',
       model: formData.value.model,
       small_fast_model: formData.value.small_fast_model,
       provider: formData.value.provider,
       provider_type: formData.value.provider_type,
       account: formData.value.account,
       tags: tags.length ? tags : undefined
    }
    await updateConfig(props.configName, payload)
    emit('saved')
    handleClose()
  } catch (e: any) { alert(e.message) }
  finally { saving.value = false }
}

watch(() => props.isOpen, (val) => { if(val) loadConfig() })
</script>
