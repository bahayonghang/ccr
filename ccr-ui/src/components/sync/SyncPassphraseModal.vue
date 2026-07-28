<template>
  <BaseModal
    :model-value="modelValue"
    size="sm"
    surface="solid"
    :title="$t('sync.passphrase.title')"
    :description="$t('sync.passphrase.description')"
    @update:model-value="handleModelValue"
    @close="close"
  >
    <form
      class="sync-passphrase-form"
      @submit.prevent="submit"
    >
      <p class="sync-passphrase-target">
        <SIcon
          name="ShieldCheck"
          size="w-4 h-4"
        />
        <span>{{ assetName || $t('sync.passphrase.allAssets') }}</span>
      </p>

      <Input
        v-model="passphrase"
        type="password"
        :label="$t('sync.passphrase.label')"
        :placeholder="$t('sync.passphrase.placeholder')"
      />

      <label class="sync-passphrase-migration">
        <input
          v-model="migratePlaintextV1"
          type="checkbox"
        >
        <span>
          <strong>{{ $t('sync.passphrase.migrateTitle') }}</strong>
          <small>{{ $t('sync.passphrase.migrateDescription') }}</small>
        </span>
      </label>
    </form>

    <template #footer>
      <button
        type="button"
        class="sync-passphrase-button sync-passphrase-button--secondary"
        @click="close"
      >
        {{ $t('common.cancel') }}
      </button>
      <button
        type="button"
        class="sync-passphrase-button sync-passphrase-button--primary"
        :disabled="passphrase.length === 0"
        @click="submit"
      >
        <SIcon
          name="KeyRound"
          size="w-4 h-4"
        />
        {{ $t('sync.passphrase.continue') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Input from '@/components/ui/Input.vue'
import SIcon from '@/components/ui/SIcon.vue'

const props = withDefaults(defineProps<{
  modelValue: boolean
  assetName?: string
}>(), {
  assetName: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'submit': [payload: { passphrase: string; migratePlaintextV1: boolean }]
}>()

const passphrase = ref('')
const migratePlaintextV1 = ref(false)

const clear = () => {
  passphrase.value = ''
  migratePlaintextV1.value = false
}

const close = () => {
  clear()
  emit('update:modelValue', false)
}

const handleModelValue = (value: boolean) => {
  if (!value) close()
}

const submit = () => {
  if (!passphrase.value) return
  const payload = {
    passphrase: passphrase.value,
    migratePlaintextV1: migratePlaintextV1.value,
  }
  clear()
  emit('update:modelValue', false)
  emit('submit', payload)
}

watch(() => props.modelValue, (isOpen) => {
  if (!isOpen) clear()
})
</script>

<style scoped>
.sync-passphrase-form {
  display: grid;
  gap: 16px;
}

.sync-passphrase-target {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.sync-passphrase-migration {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: start;
  gap: 10px;
  cursor: pointer;
}

.sync-passphrase-migration input {
  width: 16px;
  height: 16px;
  margin-top: 2px;
  accent-color: var(--color-accent-primary);
}

.sync-passphrase-migration span {
  display: grid;
  gap: 3px;
}

.sync-passphrase-migration strong {
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 600;
}

.sync-passphrase-migration small {
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

.sync-passphrase-button {
  display: inline-flex;
  min-height: 36px;
  align-items: center;
  justify-content: center;
  gap: 7px;
  border-radius: 6px;
  padding: 0 14px;
  font-size: 13px;
  font-weight: 600;
}

.sync-passphrase-button--secondary {
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
}

.sync-passphrase-button--primary {
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
}

.sync-passphrase-button--primary:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
</style>
