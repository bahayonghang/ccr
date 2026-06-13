<template>
  <BaseModal
    :model-value="isOpen"
    size="4xl"
    scrollable
    surface="solid"
    title="Edit Configuration"
    @update:model-value="(value: boolean) => { if (!value) handleClose() }"
    @close="handleClose"
  >
    <!-- Header -->
    <template #header="{ titleId }">
      <div class="flex items-center gap-4">
        <div class="p-3 rounded-xl bg-accent-primary/10 text-accent-primary">
          <SIcon
            name="Settings"
            size="w-6 h-6"
          />
        </div>
        <div>
          <h2
            :id="titleId"
            class="text-xl font-bold text-text-primary"
          >
            Edit Configuration
          </h2>
          <p class="text-xs text-text-secondary font-mono flex items-center gap-1">
            <span>ID:</span> {{ configName }}
          </p>
        </div>
      </div>
    </template>

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
      class="space-y-8"
      @submit.prevent="handleSave"
    >
      <!-- Section: Description -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-1">
          <SIcon
            name="FileText"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <h3 class="text-xs font-bold uppercase tracking-wider text-text-muted">
            Basic Info
          </h3>
          <div class="h-px flex-1 bg-border-default" />
        </div>
        <Input
          v-model="formData.description"
          label="Description"
          placeholder="Brief description of this config"
        >
          <template #leading>
            <SIcon
              name="FileText"
              size="w-4 h-4"
              class="text-text-muted"
            />
          </template>
        </Input>
      </div>

      <!-- Section: Connection -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-1">
          <SIcon
            name="Globe"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <h3 class="text-xs font-bold uppercase tracking-wider text-text-muted">
            Connection
          </h3>
          <div class="h-px flex-1 bg-border-default" />
        </div>

        <Input
          v-model="formData.base_url"
          label="Base URL"
          placeholder="https://api.anthropic.com"
        >
          <template #leading>
            <SIcon
              name="Globe"
              size="w-4 h-4"
              class="text-text-muted"
            />
          </template>
        </Input>

        <div class="relative group">
          <Input
            v-model="formData.auth_token"
            label="Auth Token"
            placeholder="sk-..."
            :type="showToken ? 'text' : 'password'"
          >
            <template #leading>
              <SIcon
                name="KeyRound"
                size="w-4 h-4"
                class="text-text-muted"
              />
            </template>
          </Input>
          <!-- Toggle visibility button -->
          <button
            type="button"
            class="absolute right-3 top-[34px] p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-bg-overlay transition-colors duration-200"
            :title="showToken ? 'Hide token' : 'Show token'"
            @click="showToken = !showToken"
          >
            <SIcon
              v-if="!showToken"
              name="Eye"
              size="w-4 h-4"
            />
            <SIcon
              v-else
              name="EyeOff"
              size="w-4 h-4"
            />
          </button>
        </div>
      </div>

      <!-- Section: Models -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-1">
          <SIcon
            name="Bot"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <h3 class="text-xs font-bold uppercase tracking-wider text-text-muted">
            Models
          </h3>
          <div class="h-px flex-1 bg-border-default" />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Input
            v-model="formData.model"
            label="Default Model"
            placeholder="claude-3-opus-20240229"
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
            placeholder="claude-3-haiku-20240307"
          >
            <template #leading>
              <SIcon
                name="Zap"
                size="w-4 h-4"
                class="text-text-muted"
              />
            </template>
          </Input>
        </div>
      </div>

      <!-- Section: Provider -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-1">
          <SIcon
            name="Building2"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <h3 class="text-xs font-bold uppercase tracking-wider text-text-muted">
            Provider
          </h3>
          <div class="h-px flex-1 bg-border-default" />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <!-- Provider Type Select -->
          <div class="w-full">
            <label class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-1.5 ml-1">Provider Type</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <SIcon
                  name="Tag"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
              </div>
              <select
                v-model="formData.provider_type"
                class="w-full bg-bg-elevated border border-border-default rounded-lg pl-10 pr-8 py-2.5 text-sm text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/50 focus:border-accent-primary appearance-none transition-[border-color,box-shadow] duration-300 hover:border-border-strong"
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
              <div class="absolute inset-y-0 right-3 flex items-center pointer-events-none">
                <SIcon
                  name="ChevronDown"
                  size="w-3.5 h-3.5"
                  class="text-text-muted"
                />
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
              <SIcon
                name="Building2"
                size="w-4 h-4"
                class="text-text-muted"
              />
            </template>
          </Input>

          <Input
            v-model="formData.account"
            label="Account ID"
            placeholder="e.g. personal"
            hint="Account differentiator"
          >
            <template #leading>
              <SIcon
                name="User"
                size="w-4 h-4"
                class="text-text-muted"
              />
            </template>
          </Input>
        </div>
      </div>

      <!-- Section: Tags -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-1">
          <SIcon
            name="Tags"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <h3 class="text-xs font-bold uppercase tracking-wider text-text-muted">
            Tags
          </h3>
          <div class="h-px flex-1 bg-border-default" />
        </div>

        <Input
          v-model="tagsInput"
          label="Tags"
          placeholder="production, backup, test"
          hint="Comma separated"
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

    <!-- Footer -->
    <template #footer>
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
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, watch } from 'vue'
import { getConfig, updateConfig } from '@/api'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Spinner from '@/components/ui/Spinner.vue'
import { useUIStore } from '@/stores/ui'

// typed
import type { UpdateConfigRequest, ConfigItem } from '@/types'

interface Props {
  isOpen: boolean
  configName: string
}

const props = defineProps<Props>()
const emit = defineEmits(['close', 'saved'])

// Auth token visibility toggle
const showToken = ref(true)

const handleClose = () => emit('close')

const uiStore = useUIStore()

const loading = ref(false)
const saving = ref(false)
const tagsInput = ref('')

const formData = ref<Partial<ConfigItem>>({})

const loadConfig = async () => {
  if (!props.configName) return
  loading.value = true
  try {
    const data = await getConfig(props.configName)
    formData.value = { ...data }
    tagsInput.value = Array.isArray(data.tags) ? data.tags.join(', ') : ''
  } catch (e) {
    uiStore.showError((e instanceof Error ? e.message : "Error") || 'Failed to load configuration')
  }
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
    uiStore.showSuccess('Configuration saved successfully')
    emit('saved')
    handleClose()
  } catch (e) {
    uiStore.showError((e instanceof Error ? e.message : "Error") || 'Failed to save configuration')
  }
  finally { saving.value = false }
}

watch(() => props.isOpen, (val) => { if(val) loadConfig() })
</script>

