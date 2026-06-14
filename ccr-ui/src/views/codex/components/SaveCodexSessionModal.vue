<template>
  <BaseModal
    :model-value="modelValue"
    :title="tf('codex.auth.actions.saveCurrent', 'Save current session')"
    :description="$t('codex.auth.subtitle')"
    size="full"
    surface="glass"
    content-class="w-full max-w-[min(780px,calc(100vw-2rem))] max-h-[90vh] overflow-y-auto"
    @update:model-value="(value) => !value && handleClose()"
  >
    <template #header="{ titleId }">
      <div
        class="px-6 py-4 border-b border-border-default/10 flex items-center justify-between sticky top-0 bg-bg-elevated/95 backdrop-blur z-10"
      >
        <h2
          :id="titleId"
          class="text-xl font-bold text-text-primary"
        >
          {{ tf('codex.auth.actions.saveCurrent', 'Save current session') }}
        </h2>
        <Button
          variant="ghost"
          surface="status"
          density="compact"
          motion="subtle"
          @click="handleClose"
        >
          <template #leading>
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </template>
        </Button>
      </div>
    </template>

    <div class="codex-auth-view__save-shell">
      <div class="codex-auth-view__save-intro">
        <div class="codex-auth-view__save-kicker">
          <span class="codex-auth-view__save-kicker-dot" />
          {{ tf('codex.auth.saveModal.kicker', 'Capture the live runtime') }}
        </div>
        <p class="codex-auth-view__save-lede">
          {{
            tf(
              'codex.auth.saveModal.lede',
              'Store the current Codex login as a reusable CCR account entry with a clearer label, optional notes, and an expiration reminder.'
            )
          }}
        </p>
        <div class="codex-auth-view__save-meta">
          <span class="codex-auth-view__meta-pill">
            {{
              currentInfo?.email ||
                tf('codex.auth.saveModal.meta.runtimeOnly', 'Current runtime session')
            }}
          </span>
          <span class="codex-auth-view__meta-pill codex-auth-view__meta-pill--muted">
            {{ formatAuthMethod(currentInfo?.auth_method) }}
          </span>
        </div>
      </div>

      <div
        v-if="processWarning"
        class="p-4 rounded-lg bg-yellow-500/10 border border-yellow-500/30 text-yellow-600 dark:text-yellow-400"
      >
        <div class="flex items-start gap-3">
          <SIcon
            name="AlertTriangle"
            size="w-5 h-5"
            class="flex-shrink-0 mt-0.5"
          />
          <div>
            <p class="font-medium">
              {{ $t('codex.auth.processWarning') }}
            </p>
            <p class="text-sm mt-1 opacity-80">
              {{ processWarning }}
            </p>
          </div>
        </div>
      </div>

      <div class="codex-auth-view__save-grid">
        <div class="space-y-1.5">
          <label class="text-sm font-semibold text-text-primary">
            {{ $t('codex.auth.fields.accountName') }} <span class="text-red-500">*</span>
          </label>
          <input
            v-model="saveForm.name"
            type="text"
            class="input"
            :placeholder="$t('codex.auth.placeholders.accountName')"
          >
        </div>
        <div class="space-y-1.5">
          <label class="text-sm font-semibold text-text-primary">
            {{ $t('codex.auth.fields.description') }}
          </label>
          <input
            v-model="saveForm.description"
            type="text"
            class="input"
            :placeholder="$t('codex.auth.placeholders.description')"
          >
        </div>
        <div class="codex-auth-view__save-toggle">
          <input
            id="forceOverwrite"
            v-model="saveForm.force"
            type="checkbox"
            class="w-5 h-5 rounded border-border-default/15 text-accent-primary focus:ring-accent-primary/20"
          >
          <label
            for="forceOverwrite"
            class="text-sm font-medium text-text-primary cursor-pointer select-none"
          >
            {{ $t('codex.auth.forceOverwrite') }}
          </label>
        </div>
      </div>
    </div>

    <template #footer>
      <div
        class="px-6 py-4 border-t border-border-default/10 flex justify-end gap-3 bg-bg-surface/70"
      >
        <Button
          variant="secondary"
          surface="status"
          density="compact"
          motion="subtle"
          @click="handleClose"
        >
          {{ $t('codex.actions.cancel') }}
        </Button>
        <Button
          variant="primary"
          surface="card"
          density="compact"
          motion="standard"
          :disabled="saving || !saveForm.name.trim()"
          @click="handleConfirmSave"
        >
          <template #leading>
            <span
              v-if="saving"
              class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
            />
          </template>
          {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
        </Button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useTf } from '@/composables/useTf'
import { useUIStore } from '@/stores/ui'
import { detectCodexProcess, saveCodexAuth } from '@/api'
import type { CodexAuthCurrentInfo, CodexAuthProcessResponse, CodexAuthSaveRequest } from '@/types'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

defineOptions({ name: 'SaveCodexSessionModal' })

const props = defineProps<{
  modelValue: boolean
  currentInfo: CodexAuthCurrentInfo | null
  formatAuthMethod: (method?: string | null) => string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: []
}>()

const { t } = useI18n()
const tf = useTf()
const uiStore = useUIStore()

const saving = ref(false)
const processWarning = ref<string | null>(null)
const saveForm = reactive({
  name: '',
  description: '',
  force: false,
})

// 弹窗打开时探测运行中进程并以当前会话邮箱前缀播种名称
watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return
    saveForm.name = props.currentInfo?.email?.split('@')[0] || ''
    saveForm.description = ''
    saveForm.force = false
    try {
      const processInfo = await detectCodexProcess<CodexAuthProcessResponse>()
      processWarning.value = processInfo.has_running_process
        ? processInfo.warning ||
          t('codex.auth.processDetected', { pids: processInfo.pids.join(', ') })
        : null
    } catch {
      processWarning.value = null
    }
  }
)

const handleClose = () => {
  emit('update:modelValue', false)
  processWarning.value = null
}

const handleConfirmSave = async () => {
  if (!saveForm.name.trim()) {
    uiStore.showError(t('codex.auth.validation.nameRequired'))
    return
  }

  try {
    saving.value = true
    const payload: CodexAuthSaveRequest = {
      name: saveForm.name.trim(),
      description: saveForm.description.trim() || undefined,
      force: saveForm.force,
    }
    await saveCodexAuth(payload)
    emit('update:modelValue', false)
    processWarning.value = null
    emit('saved')
    uiStore.showSuccess(
      tf('codex.auth.feedback.saveCurrentSuccess', 'Current session saved as an account.')
    )
  } catch (error) {
    logger.error('Failed to save auth:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.saveFailed'))
  } finally {
    saving.value = false
  }
}
</script>
