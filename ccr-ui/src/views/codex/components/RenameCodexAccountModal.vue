<template>
  <BaseModal
    :model-value="modelValue"
    :title="tf('codex.auth.rename.title', '重命名 Codex 账号')"
    size="md"
    surface="glass"
    content-class="w-full max-w-[min(440px,calc(100vw-2rem))]"
    @update:model-value="(value) => !value && handleClose()"
  >
    <template #header="{ titleId }">
      <div
        class="px-5 py-3.5 border-b border-border-default/10 flex items-center justify-between"
      >
        <h2
          :id="titleId"
          class="text-base font-semibold text-text-primary"
        >
          {{ tf('codex.auth.rename.title', '重命名 Codex 账号') }}
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
              size="w-4 h-4"
            />
          </template>
        </Button>
      </div>
    </template>

    <div class="p-5 space-y-4">
      <div class="space-y-1.5">
        <label class="text-xs font-semibold uppercase tracking-wider text-text-muted">
          {{ tf('codex.auth.rename.currentLabel', '当前名称') }}
        </label>
        <div class="px-3 py-2 rounded-lg bg-bg-elevated border border-border-default/15 font-mono text-sm text-text-secondary">
          {{ renameForm.oldName || '—' }}
        </div>
      </div>

      <div class="space-y-1.5">
        <label
          for="renameNewName"
          class="text-xs font-semibold uppercase tracking-wider text-text-muted"
        >
          {{ tf('codex.auth.rename.newLabel', '新名称') }}
          <span class="text-red-500">*</span>
        </label>
        <input
          id="renameNewName"
          v-model="renameForm.newName"
          type="text"
          class="input"
          :placeholder="tf('codex.auth.rename.placeholder', '输入新名称（字母/数字/_/-）')"
          @keydown.enter.prevent="handleConfirm"
        >
        <p class="text-[11px] text-text-disabled">
          {{ tf('codex.auth.rename.hint', '只能包含字母、数字、下划线和连字符，长度不超过 32 个字符。') }}
        </p>
      </div>

      <label class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer select-none">
        <input
          v-model="renameForm.force"
          type="checkbox"
          class="w-4 h-4 rounded border-border-default/15 text-accent-primary focus:ring-accent-primary/20"
        >
        {{ tf('codex.auth.rename.forceLabel', '覆盖同名账号 (force)') }}
      </label>

      <div
        v-if="renameError"
        class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-xs text-red-400"
      >
        {{ renameError }}
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2 px-5 py-3 border-t border-border-default/10">
        <Button
          variant="ghost"
          surface="status"
          density="compact"
          :disabled="renameSubmitting"
          @click="handleClose"
        >
          {{ $t('common.cancel') }}
        </Button>
        <Button
          variant="primary"
          density="compact"
          :loading="renameSubmitting"
          :disabled="!canSubmitRename"
          @click="handleConfirm"
        >
          {{ tf('codex.auth.rename.confirm', '重命名') }}
        </Button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useTf } from '@/composables/useTf'
import { useUIStore } from '@/stores/ui'
import { renameCodexAuth } from '@/api'
import { canSubmitAccountRename } from '../codexAuthAccounts'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

defineOptions({ name: 'RenameCodexAccountModal' })

const props = defineProps<{
  modelValue: boolean
  accountName: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  renamed: []
}>()

const { t } = useI18n()
const tf = useTf()
const uiStore = useUIStore()

const renameSubmitting = ref(false)
const renameError = ref<string | null>(null)
const renameForm = reactive({
  oldName: '',
  newName: '',
  force: false,
})

const canSubmitRename = computed(() =>
  canSubmitAccountRename(renameForm.oldName, renameForm.newName)
)

// 弹窗打开时以待重命名账号名播种表单
watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      renameForm.oldName = props.accountName
      renameForm.newName = props.accountName
      renameForm.force = false
      renameError.value = null
    }
  }
)

const handleClose = () => {
  if (renameSubmitting.value) return
  emit('update:modelValue', false)
  renameError.value = null
  renameForm.oldName = ''
  renameForm.newName = ''
  renameForm.force = false
}

const handleConfirm = async () => {
  if (!canSubmitRename.value) {
    renameError.value = tf(
      'codex.auth.rename.invalidName',
      '新名称只能包含字母、数字、下划线与连字符，且不能与原名称相同。'
    )
    return
  }

  const oldName = renameForm.oldName
  const newName = renameForm.newName.trim()
  const force = renameForm.force

  renameError.value = null
  renameSubmitting.value = true
  try {
    await renameCodexAuth(oldName, newName, force)
    emit('update:modelValue', false)
    emit('renamed')
    uiStore.showSuccess(
      tf('codex.auth.rename.success', '已将 {old} 重命名为 {new}', {
        old: oldName,
        new: newName,
      })
    )
    renameForm.oldName = ''
    renameForm.newName = ''
    renameForm.force = false
  } catch (error) {
    logger.error('Failed to rename auth:', error)
    const raw = extractErrorMessage(error) || t('codex.states.saveFailed')
    if (!force && raw.includes('已存在')) {
      renameError.value = tf(
        'codex.auth.rename.conflictHint',
        '{msg} · 勾选 "覆盖同名账号" 后再次确认可强制覆盖。',
        { msg: raw }
      )
    } else {
      renameError.value = raw
    }
  } finally {
    renameSubmitting.value = false
  }
}
</script>
