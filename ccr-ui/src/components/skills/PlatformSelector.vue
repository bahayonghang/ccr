<template>
  <section class="panel">
    <div class="panel__header">
      <h2 class="panel__title">
        Targets
      </h2>
      <button
        class="panel__link"
        @click="$emit('select-detected')"
      >
        Detected
      </button>
    </div>
    <div class="platform-grid">
      <button
        v-for="platform in platforms"
        :key="platform.id"
        class="platform-chip"
        :class="{
          'platform-chip--active': modelValue.includes(platform.id),
          'platform-chip--disabled': !platform.detected,
        }"
        :disabled="!platform.detected"
        @click="toggle(platform.id)"
      >
        <SIcon
          :name="platformIcon(platform.id)"
          size="w-4 h-4"
        />
        <span>{{ platform.displayName }}</span>
        <span class="platform-chip__count">{{ platform.installedCount }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { PLATFORM_CONFIG } from '@/types/skills'
import type { Platform, SkillPlatformSummary } from '@/types/skills'

const props = defineProps<{
  platforms: SkillPlatformSummary[]
  modelValue: Platform[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: Platform[]]
  'select-detected': []
}>()

function platformIcon(id: Platform) {
  return PLATFORM_CONFIG[id]?.icon ?? 'Code2'
}

function toggle(id: Platform) {
  emit('update:modelValue',
    props.modelValue.includes(id)
      ? props.modelValue.filter(p => p !== id)
      : [...props.modelValue, id],
  )
}
</script>

<style scoped>
.platform-grid {
  @apply flex flex-col gap-2;
}

.platform-chip {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-3 py-2 text-sm text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  box-shadow: var(--elevation-1);
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.platform-chip--active {
  @apply text-text-primary;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  box-shadow: var(--elevation-2);
}

.platform-chip--disabled {
  @apply opacity-40;
}

.platform-chip__count {
  @apply ml-auto rounded-full px-2 py-0.5 text-xs text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 68%);
}
</style>
