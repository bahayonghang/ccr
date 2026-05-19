<template>
  <BaseModal
    :model-value="modelValue"
    :title="$t('claudeCode.observer.subscription.dialogTitle')"
    size="sm"
    @update:model-value="onUpdateModelValue"
    @close="$emit('update:modelValue', false)"
  >
    <div class="subscription-dialog">
      <!-- 步骤 1：模式（auto / api_key / subscription） -->
      <label class="subscription-dialog__field">
        <span class="subscription-dialog__label">
          {{ $t('claudeCode.observer.subscription.fieldMode') }}
        </span>
        <select
          v-model="form.mode"
          class="subscription-dialog__control"
        >
          <option value="auto">
            {{ $t('claudeCode.observer.subscription.modeAuto') }}
          </option>
          <option value="api_key">
            {{ $t('claudeCode.observer.subscription.modeApiKey') }}
          </option>
          <option value="subscription">
            {{ $t('claudeCode.observer.subscription.modeSubscription') }}
          </option>
        </select>
      </label>

      <!-- 步骤 2：套餐 -->
      <label class="subscription-dialog__field">
        <span class="subscription-dialog__label">
          {{ $t('claudeCode.observer.subscription.fieldPlan') }}
        </span>
        <select
          v-model="form.plan"
          class="subscription-dialog__control"
          :disabled="form.mode !== 'subscription'"
        >
          <option value="free_pro">
            {{ $t('claudeCode.observer.subscription.planFreePro') }}
          </option>
          <option value="max5x">
            {{ $t('claudeCode.observer.subscription.planMax5x') }}
          </option>
          <option value="max20x">
            {{ $t('claudeCode.observer.subscription.planMax20x') }}
          </option>
          <option value="team">
            {{ $t('claudeCode.observer.subscription.planTeam') }}
          </option>
          <option value="enterprise">
            {{ $t('claudeCode.observer.subscription.planEnterprise') }}
          </option>
          <option value="custom">
            {{ $t('claudeCode.observer.subscription.planCustom') }}
          </option>
        </select>
      </label>

      <!-- 步骤 3：月费（美元） -->
      <label class="subscription-dialog__field">
        <span class="subscription-dialog__label">
          {{ $t('claudeCode.observer.subscription.fieldMonthlyUsd') }}
        </span>
        <input
          v-model.number="form.monthly_usd"
          type="number"
          min="0"
          step="1"
          class="subscription-dialog__control"
          :disabled="form.mode !== 'subscription'"
        >
      </label>

      <p
        v-if="error"
        class="subscription-dialog__error"
      >
        {{ error }}
      </p>
    </div>

    <template #footer>
      <div class="subscription-dialog__actions">
        <button
          type="button"
          class="subscription-dialog__btn subscription-dialog__btn--ghost"
          :disabled="saving"
          @click="$emit('update:modelValue', false)"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          type="button"
          class="subscription-dialog__btn subscription-dialog__btn--primary"
          :disabled="saving"
          @click="onSave"
        >
          {{ saving
            ? $t('claudeCode.observer.subscription.saving')
            : $t('common.save') }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useClaudeObserverStore } from '@/stores/claudeObserver'
import type { SubscriptionDto } from '@/types/claudeObserver'

interface Props {
  modelValue: boolean
  current: SubscriptionDto | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'saved': []
}>()

const store = useClaudeObserverStore()

/*
 * ========================================================================
 * 表单状态
 * ========================================================================
 * 1) 打开时由 props.current 初始化
 * 2) 保存时调用 store.updateSubscription()
 */
const form = reactive({
  mode: 'auto',
  plan: 'free_pro',
  monthly_usd: 0,
})

const saving = ref(false)
const error = ref<string | null>(null)

const syncForm = (next: SubscriptionDto | null) => {
  form.mode = next?.mode ?? 'auto'
  form.plan = next?.plan ?? 'free_pro'
  form.monthly_usd = next?.monthly_usd ?? 0
}

watch(
  () => props.current,
  (next) => syncForm(next),
  { immediate: true },
)

watch(
  () => props.modelValue,
  (open) => {
    // 1.1 打开时刷新表单初值，关闭时清错
    if (open) {
      syncForm(props.current)
      error.value = null
    }
  },
)

const onUpdateModelValue = (value: boolean) => {
  emit('update:modelValue', value)
}

const onSave = async () => {
  saving.value = true
  error.value = null
  try {
    await store.updateSubscription({
      mode: form.mode,
      plan: form.plan,
      monthly_usd: Number.isFinite(form.monthly_usd) ? form.monthly_usd : 0,
    })
    emit('saved')
    emit('update:modelValue', false)
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.subscription-dialog {
  display: grid;
  gap: 0.9rem;
}

.subscription-dialog__field {
  display: grid;
  gap: 0.35rem;
}

.subscription-dialog__label {
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.subscription-dialog__control {
  width: 100%;
  border-radius: 0.65rem;
  border: 1px solid var(--color-border-default);
  background: var(--surface-card-bg);
  color: var(--color-text-primary);
  padding: 0.55rem 0.75rem;
  font-size: 0.88rem;
  transition: border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.subscription-dialog__control:focus-visible {
  outline: none;
  border-color: rgb(var(--color-accent-primary-rgb) / 55%);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 12%);
}

.subscription-dialog__control:disabled {
  opacity: 0.55;
}

.subscription-dialog__error {
  margin: 0;
  color: var(--color-danger);
  font-size: 0.78rem;
}

.subscription-dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
}

.subscription-dialog__btn {
  min-width: 5rem;
  border-radius: 0.65rem;
  padding: 0.55rem 0.95rem;
  font-size: 0.84rem;
  font-weight: 600;
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.subscription-dialog__btn--ghost {
  border: 1px solid var(--color-border-default);
  background: transparent;
  color: var(--color-text-secondary);
}

.subscription-dialog__btn--ghost:hover {
  color: var(--color-text-primary);
}

.subscription-dialog__btn--primary {
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 35%);
  background: var(--color-accent-primary);
  color: white;
}

.subscription-dialog__btn--primary:hover {
  background: var(--color-accent-primary-hover);
}

.subscription-dialog__btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
