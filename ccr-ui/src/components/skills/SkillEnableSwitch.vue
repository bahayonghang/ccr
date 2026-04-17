<template>
  <label class="enable-switch">
    <input
      type="checkbox"
      class="enable-switch__input"
      :checked="enabled"
      :disabled="loading"
      @change="handleChange(($event.target as HTMLInputElement).checked)"
    >
    <span
      class="enable-switch__track"
      :class="{ 'enable-switch__track--on': enabled }"
    >
      <span
        class="enable-switch__thumb"
        :class="{ 'enable-switch__thumb--on': enabled }"
      />
    </span>
    <span class="enable-switch__label">{{ enabled ? t('skillsExt.toggle.enabled') : t('skillsExt.toggle.disabled') }}</span>
  </label>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  /** 已禁用的 skill name 集合（从 useSkillToggle.disabled 传入） */
  disabledSet: Set<string>
  skillName: string
  loading?: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle', skillName: string, enabled: boolean): void
}>()

const enabled = computed(() => !props.disabledSet.has(props.skillName))

function handleChange(next: boolean) {
  emit('toggle', props.skillName, next)
}
</script>

<style scoped>
.enable-switch {
  @apply inline-flex cursor-pointer items-center gap-2 text-xs text-text-secondary;
}

.enable-switch__input {
  @apply sr-only;
}

.enable-switch__track {
  @apply relative h-5 w-9 rounded-full border border-border-default/55 transition-colors;

  background-color: rgb(var(--color-bg-base-rgb) / 68%);
}

.enable-switch__track--on {
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 70%),
    rgb(var(--color-accent-secondary-rgb) / 50%)
  );
  border-color: rgb(var(--color-accent-primary-rgb) / 60%);
}

.enable-switch__thumb {
  @apply absolute left-0.5 top-0.5 h-3.5 w-3.5 rounded-full bg-white shadow-sm transition-transform;
}

.enable-switch__thumb--on {
  transform: translateX(1rem);
}

.enable-switch:hover .enable-switch__track {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
}

.enable-switch__label {
  @apply select-none;
}

.enable-switch__input:disabled + .enable-switch__track {
  @apply cursor-not-allowed opacity-50;
}
</style>
