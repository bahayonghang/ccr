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
    <div
      ref="modalRef"
      role="dialog"
      :aria-modal="true"
      :aria-labelledby="titleId"
      class="relative z-10 w-full max-w-4xl max-h-[90vh] flex flex-col rounded-xl bg-bg-elevated/60 backdrop-blur-xl border border-white/5 shadow-2xl animate-scale-in"
    >
      <!-- Header -->
      <div class="shrink-0 px-8 py-6 border-b border-border-subtle bg-bg-elevated rounded-t-xl flex items-center justify-between">
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
      <div class="flex-1 min-h-0 overflow-y-auto">
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
            class="space-y-8"
            @submit.prevent="handleSave"
          >
            <!-- Section: Description -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 mb-1">
                <FileText class="w-4 h-4 text-emerald-400" />
                <h3 class="text-xs font-bold uppercase tracking-wider text-emerald-400">
                  Basic Info
                </h3>
                <div class="h-px flex-1 bg-emerald-400/20" />
              </div>
              <Input
                v-model="formData.description"
                label="Description"
                placeholder="Brief description of this config"
              >
                <template #leading>
                  <FileText class="w-4 h-4 text-emerald-400" />
                </template>
              </Input>
            </div>

            <!-- Section: Connection -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 mb-1">
                <Globe class="w-4 h-4 text-cyan-400" />
                <h3 class="text-xs font-bold uppercase tracking-wider text-cyan-400">
                  Connection
                </h3>
                <div class="h-px flex-1 bg-cyan-400/20" />
              </div>

              <Input
                v-model="formData.base_url"
                label="Base URL"
                placeholder="https://api.anthropic.com"
              >
                <template #leading>
                  <Globe class="w-4 h-4 text-cyan-400" />
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
                    <KeyRound class="w-4 h-4 text-amber-400" />
                  </template>
                </Input>
                <!-- Toggle visibility button -->
                <button
                  type="button"
                  class="absolute right-3 top-[34px] p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-white/10 transition-all duration-200"
                  :title="showToken ? 'Hide token' : 'Show token'"
                  @click="showToken = !showToken"
                >
                  <Eye
                    v-if="!showToken"
                    class="w-4 h-4"
                  />
                  <EyeOff
                    v-else
                    class="w-4 h-4"
                  />
                </button>
              </div>
            </div>

            <!-- Section: Models -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 mb-1">
                <Bot class="w-4 h-4 text-violet-400" />
                <h3 class="text-xs font-bold uppercase tracking-wider text-violet-400">
                  Models
                </h3>
                <div class="h-px flex-1 bg-violet-400/20" />
              </div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  v-model="formData.model"
                  label="Default Model"
                  placeholder="claude-3-opus-20240229"
                >
                  <template #leading>
                    <Bot class="w-4 h-4 text-violet-400" />
                  </template>
                </Input>

                <Input
                  v-model="formData.small_fast_model"
                  label="Fast Model"
                  placeholder="claude-3-haiku-20240307"
                >
                  <template #leading>
                    <Zap class="w-4 h-4 text-yellow-400" />
                  </template>
                </Input>
              </div>
            </div>

            <!-- Section: Provider -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 mb-1">
                <Building2 class="w-4 h-4 text-orange-400" />
                <h3 class="text-xs font-bold uppercase tracking-wider text-orange-400">
                  Provider
                </h3>
                <div class="h-px flex-1 bg-orange-400/20" />
              </div>

              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <!-- Provider Type Select -->
                <div class="w-full">
                  <label class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-1.5 ml-1">Provider Type</label>
                  <div class="relative">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <Tag class="w-4 h-4 text-orange-400" />
                    </div>
                    <select
                      v-model="formData.provider_type"
                      class="w-full bg-bg-surface border border-border-default rounded-lg pl-10 pr-8 py-2.5 text-sm text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/50 focus:border-accent-primary appearance-none transition-all duration-300 hover:border-border-strong shadow-sm"
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
                      <ChevronDown class="w-3.5 h-3.5 text-text-muted" />
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
                    <Building2 class="w-4 h-4 text-orange-400" />
                  </template>
                </Input>

                <Input
                  v-model="formData.account"
                  label="Account ID"
                  placeholder="e.g. personal"
                  hint="Account differentiator"
                >
                  <template #leading>
                    <User class="w-4 h-4 text-pink-400" />
                  </template>
                </Input>
              </div>
            </div>

            <!-- Section: Tags -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 mb-1">
                <Tags class="w-4 h-4 text-sky-400" />
                <h3 class="text-xs font-bold uppercase tracking-wider text-sky-400">
                  Tags
                </h3>
                <div class="h-px flex-1 bg-sky-400/20" />
              </div>

              <Input
                v-model="tagsInput"
                label="Tags"
                placeholder="production, backup, test"
                hint="Comma separated"
              >
                <template #leading>
                  <Tags class="w-4 h-4 text-sky-400" />
                </template>
              </Input>
            </div>
          </form>
        </div>
      </div>

      <!-- Footer -->
      <div class="shrink-0 px-8 py-6 border-t border-border-subtle bg-bg-elevated rounded-b-xl flex gap-4">
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import {
  Settings, X, FileText, Globe, KeyRound, Bot, Zap,
  Building2, User, Tag, Tags, Eye, EyeOff, ChevronDown
} from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { getConfig, updateConfig } from '@/api'
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

const titleId = useUniqueId('edit-config-title')
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = ref(props.isOpen)

// Auth token visibility toggle
const showToken = ref(true)

watch(() => props.isOpen, (val) => {
  isOpenRef.value = val
  // 锁定/恢复 body 滚动，防止弹窗打开时背景可滚动
  document.body.style.overflow = val ? 'hidden' : ''
})

const handleClose = () => emit('close')
const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)
useEscapeKey(handleClose, isOpenRef)

watch(isOpenRef, (open) => { if(open) setTimeout(() => focusFirstElement(), 100) })

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
  } catch (e: any) { 
    uiStore.showError(e.message || 'Failed to load configuration')
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
  } catch (e: any) { 
    uiStore.showError(e.message || 'Failed to save configuration')
  }
  finally { saving.value = false }
}

watch(() => props.isOpen, (val) => { if(val) loadConfig() })

// 组件卸载时恢复 body 滚动
onUnmounted(() => {
  document.body.style.overflow = ''
})
</script>
